#![allow(non_snake_case)]

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    task::JoinHandle,
    time::{Duration, timeout},
};

#[derive(Clone)]
pub struct HelperWorkerConfig {
    pub helper_name: &'static str,
    pub project_root: PathBuf,
    pub script_path: PathBuf,
    pub timeout_ms: u64,
}

#[derive(Debug)]
pub enum HelperWorkerError {
    Transport(String),
    Request(String),
}

#[derive(Serialize)]
struct WorkerRequestEnvelope<'a, T> {
    requestId: u64,
    request: &'a T,
}

#[derive(Deserialize)]
struct WorkerResponseEnvelope {
    requestId: u64,
    ok: bool,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<String>,
}

struct WorkerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    stderr_tail: Arc<Mutex<String>>,
    stderr_task: JoinHandle<()>,
}

impl WorkerProcess {
    async fn spawn(config: &HelperWorkerConfig) -> Result<Self, HelperWorkerError> {
        let mut child = Command::new("node")
            .arg(&config.script_path)
            .arg("--worker")
            .current_dir(&config.project_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                HelperWorkerError::Transport(format!(
                    "Failed to start {} worker: {error}",
                    config.helper_name
                ))
            })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            HelperWorkerError::Transport(format!(
                "{} worker stdin was unavailable.",
                config.helper_name
            ))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            HelperWorkerError::Transport(format!(
                "{} worker stdout was unavailable.",
                config.helper_name
            ))
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            HelperWorkerError::Transport(format!(
                "{} worker stderr was unavailable.",
                config.helper_name
            ))
        })?;
        let stderr_tail = Arc::new(Mutex::new(String::new()));
        let stderr_task = spawn_stderr_task(config.helper_name, stderr, Arc::clone(&stderr_tail));
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
            stderr_tail,
            stderr_task,
        })
    }

    async fn kill(&mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
        self.stderr_task.abort();
    }

    async fn stderr_suffix(&self) -> String {
        let tail = self.stderr_tail.lock().await.trim().to_string();
        if tail.is_empty() {
            String::new()
        } else {
            format!(" stderr: {}", tail.replace('\n', " | "))
        }
    }
}

fn spawn_stderr_task(
    helper_name: &'static str,
    stderr: ChildStderr,
    tail: Arc<Mutex<String>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("[{} worker] {}", helper_name, line);
            let mut guard = tail.lock().await;
            if !guard.is_empty() {
                guard.push('\n');
            }
            guard.push_str(&line);
            const MAX_TAIL_LEN: usize = 4096;
            if guard.len() > MAX_TAIL_LEN {
                let start = guard.len().saturating_sub(MAX_TAIL_LEN);
                *guard = guard[start..].to_string();
            }
        }
    })
}

pub fn helper_worker_enabled(env_name: &str) -> bool {
    matches!(
        std::env::var(env_name)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub struct HelperWorkerClient {
    config: HelperWorkerConfig,
    state: Mutex<Option<WorkerProcess>>,
    next_request_id: AtomicU64,
}

impl HelperWorkerClient {
    pub fn new(config: HelperWorkerConfig) -> Self {
        Self {
            config,
            state: Mutex::new(None),
            next_request_id: AtomicU64::new(1),
        }
    }

    pub async fn request<T: Serialize, R: DeserializeOwned>(
        &self,
        request: &T,
    ) -> Result<R, HelperWorkerError> {
        let value = self.request_value(request).await?;
        serde_json::from_value(value).map_err(|error| {
            HelperWorkerError::Transport(format!(
                "Failed to decode {} worker response: {error}",
                self.config.helper_name
            ))
        })
    }

    async fn request_value<T: Serialize>(&self, request: &T) -> Result<Value, HelperWorkerError> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let payload = serde_json::to_string(&WorkerRequestEnvelope {
            requestId: request_id,
            request,
        })
        .map_err(|error| {
            HelperWorkerError::Transport(format!(
                "Failed to serialize {} worker request: {error}",
                self.config.helper_name
            ))
        })?;
        for attempt in 0..2 {
            let mut guard = self.state.lock().await;
            if guard.is_none() {
                *guard = Some(WorkerProcess::spawn(&self.config).await?);
            }
            let exchange = timeout(
                Duration::from_millis(self.config.timeout_ms),
                self.exchange_locked(
                    guard.as_mut().expect("worker initialized"),
                    request_id,
                    &payload,
                ),
            )
            .await;
            match exchange {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(HelperWorkerError::Request(error))) => {
                    return Err(HelperWorkerError::Request(error));
                }
                Ok(Err(HelperWorkerError::Transport(error))) => {
                    if let Some(worker) = guard.as_mut() {
                        worker.kill().await;
                    }
                    *guard = None;
                    if attempt == 0 {
                        continue;
                    }
                    return Err(HelperWorkerError::Transport(error));
                }
                Err(_) => {
                    let stderr_suffix = if let Some(worker) = guard.as_ref() {
                        worker.stderr_suffix().await
                    } else {
                        String::new()
                    };
                    if let Some(worker) = guard.as_mut() {
                        worker.kill().await;
                    }
                    *guard = None;
                    if attempt == 0 {
                        continue;
                    }
                    return Err(HelperWorkerError::Transport(format!(
                        "{} worker timed out after {}ms.{}",
                        self.config.helper_name, self.config.timeout_ms, stderr_suffix
                    )));
                }
            }
        }
        Err(HelperWorkerError::Transport(format!(
            "{} worker request failed without a specific error.",
            self.config.helper_name
        )))
    }

    async fn exchange_locked(
        &self,
        worker: &mut WorkerProcess,
        request_id: u64,
        payload: &str,
    ) -> Result<Value, HelperWorkerError> {
        worker
            .stdin
            .write_all(payload.as_bytes())
            .await
            .map_err(|error| {
                HelperWorkerError::Transport(format!(
                    "Failed to send {} worker request: {error}",
                    self.config.helper_name
                ))
            })?;
        worker.stdin.write_all(b"\n").await.map_err(|error| {
            HelperWorkerError::Transport(format!(
                "Failed to terminate {} worker request: {error}",
                self.config.helper_name
            ))
        })?;
        worker.stdin.flush().await.map_err(|error| {
            HelperWorkerError::Transport(format!(
                "Failed to flush {} worker request: {error}",
                self.config.helper_name
            ))
        })?;
        let mut ignored_lines = Vec::new();
        loop {
            let line = worker.stdout.next_line().await.map_err(|error| {
                HelperWorkerError::Transport(format!(
                    "Failed to read {} worker response: {error}",
                    self.config.helper_name
                ))
            })?;
            let line = match line {
                Some(line) => line,
                None => {
                    let stderr_suffix = worker.stderr_suffix().await;
                    let ignored_suffix = if ignored_lines.is_empty() {
                        String::new()
                    } else {
                        format!(" ignored stdout: {}", ignored_lines.join(" | "))
                    };
                    return Err(HelperWorkerError::Transport(format!(
                        "{} worker closed stdout unexpectedly.{}{}",
                        self.config.helper_name, stderr_suffix, ignored_suffix
                    )));
                }
            };
            let envelope = match serde_json::from_str::<WorkerResponseEnvelope>(&line) {
                Ok(envelope) => envelope,
                Err(_) => {
                    ignored_lines.push(line);
                    if ignored_lines.len() > 8 {
                        ignored_lines.remove(0);
                    }
                    continue;
                }
            };
            if envelope.requestId != request_id {
                ignored_lines.push(format!("mismatched-request-id:{}", envelope.requestId));
                if ignored_lines.len() > 8 {
                    ignored_lines.remove(0);
                }
                continue;
            }
            if envelope.ok {
                return envelope.result.ok_or_else(|| {
                    HelperWorkerError::Transport(format!(
                        "{} worker response was missing a result payload.",
                        self.config.helper_name
                    ))
                });
            }
            return Err(HelperWorkerError::Request(format!(
                "{} worker error: {}",
                self.config.helper_name,
                envelope
                    .error
                    .unwrap_or_else(|| "unknown worker error".to_string())
            )));
        }
    }
}
