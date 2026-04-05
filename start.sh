#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
engine_manifest_path="$(cd "$project_root/rust/launchdeck-engine" && pwd)/Cargo.toml"
launchdeck_log_dir="$project_root/.local/launchdeck"

get_configured_numeric_setting() {
  local default_value="$1"
  shift
  local variable_names=("$@")
  local file_name file_path variable_name value

  for file_name in ".env" ".env.local" ".env.example"; do
    file_path="$project_root/$file_name"
    [[ -f "$file_path" ]] || continue
    for variable_name in "${variable_names[@]}"; do
      value="$(awk -F= -v key="$variable_name" '
        $1 ~ "^[[:space:]]*" key "[[:space:]]*$" {
          gsub(/^[[:space:]]+|[[:space:]]+$/, "", $2)
          if ($2 ~ /^[0-9]+$/) {
            print $2
            exit
          }
        }
      ' "$file_path")"
      if [[ -n "$value" ]]; then
        printf '%s\n' "$value"
        return
      fi
    done
  done

  printf '%s\n' "$default_value"
}

get_configured_engine_port() {
  get_configured_numeric_setting 8789 "LAUNCHDECK_PORT"
}

get_configured_follow_daemon_port() {
  get_configured_numeric_setting 8790 "LAUNCHDECK_FOLLOW_DAEMON_PORT"
}

stop_launchdeck_process() {
  local pid="$1"
  local reason="$2"

  [[ "$pid" =~ ^[0-9]+$ ]] || return 0
  [[ "$pid" != "$$" ]] || return 0
  if kill -0 "$pid" 2>/dev/null; then
    if kill -TERM "$pid" 2>/dev/null; then
      sleep 0.2
    fi
    if kill -0 "$pid" 2>/dev/null; then
      kill -KILL "$pid" 2>/dev/null || true
    fi
    echo "Stopped process $pid ($reason)." >&2
  fi
}

list_listening_pids_for_port() {
  local port="$1"

  if command -v ss >/dev/null 2>&1; then
    ss -ltnp "( sport = :$port )" 2>/dev/null \
      | awk 'match($0, /pid=([0-9]+)/, m) { print m[1] }'
    return
  fi

  if command -v lsof >/dev/null 2>&1; then
    lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true
    return
  fi
}

stop_processes_listening_on_port() {
  local port="$1"
  local label="$2"
  local pid

  while IFS= read -r pid; do
    [[ -n "$pid" ]] || continue
    stop_launchdeck_process "$pid" "$label listener on port $port"
  done < <(list_listening_pids_for_port "$port" | awk '!seen[$0]++')
}

stop_old_launchdeck_runtime() {
  local engine_port follow_daemon_port pid

  if command -v pgrep >/dev/null 2>&1; then
    while IFS= read -r pid; do
      [[ -n "$pid" ]] || continue
      stop_launchdeck_process "$pid" "existing LaunchDeck runtime"
    done < <(pgrep -af "launchdeck-engine|launchdeck-follow-daemon|$engine_manifest_path" \
      | awk -v self="$$" '$1 != self { print $1 }' \
      | awk '!seen[$0]++')
  fi

  engine_port="$(get_configured_engine_port)"
  follow_daemon_port="$(get_configured_follow_daemon_port)"
  stop_processes_listening_on_port "$engine_port" "LaunchDeck engine"
  stop_processes_listening_on_port "$follow_daemon_port" "LaunchDeck follow daemon"

  printf '%s\n%s\n' "$engine_port" "$follow_daemon_port"
}

wait_for_health_endpoint() {
  local url="$1"
  local name="$2"
  local pid="${3:-}"
  local log_path="${4:-}"
  local max_attempts="${5:-20}"
  local delay_seconds="${6:-0.5}"
  local slow_notice_attempt="${7:-10}"
  local attempt body slow_notice_shown=0

  for ((attempt = 1; attempt <= max_attempts; attempt++)); do
    body="$(curl -fsS --max-time 2 "$url" 2>/dev/null || true)"
    if [[ "$body" == *'"ok":true'* || "$body" == *'"running":true'* ]]; then
      return 0
    fi

    if [[ -n "$pid" ]] && ! kill -0 "$pid" 2>/dev/null; then
      echo "Error: $name process exited before reporting healthy startup at $url." >&2
      if [[ -n "$log_path" ]]; then
        echo "Error: Check $log_path for details." >&2
      fi
      return 1
    fi

    if (( slow_notice_shown == 0 && attempt >= slow_notice_attempt )); then
      echo "$name is still compiling or starting..." >&2
      slow_notice_shown=1
    fi

    sleep "$delay_seconds"
  done

  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    echo "$name is still compiling or starting in the background at $url." >&2
    if [[ -n "$log_path" ]]; then
      echo "Info: Check $log_path only if it does not become healthy soon." >&2
    fi
    return 2
  fi

  echo "Error: $name did not become healthy at $url." >&2
  if [[ -n "$log_path" ]]; then
    echo "Error: Check $log_path for details." >&2
  fi
  return 1
}

start_launchdeck_processes() {
  local ports engine_port follow_daemon_port daemon_stdout_path daemon_stderr_path stdout_path stderr_path
  local daemon_pid engine_pid daemon_wait_pid engine_wait_pid
  mapfile -t ports < <(stop_old_launchdeck_runtime)
  engine_port="${ports[0]}"
  follow_daemon_port="${ports[1]}"

  mkdir -p "$launchdeck_log_dir"
  cd "$project_root"

  daemon_stdout_path="$launchdeck_log_dir/follow-daemon.log"
  daemon_stderr_path="$launchdeck_log_dir/follow-daemon-error.log"
  nohup cargo run --manifest-path "rust/launchdeck-engine/Cargo.toml" --bin launchdeck-follow-daemon \
    >"$daemon_stdout_path" 2>"$daemon_stderr_path" </dev/null &
  daemon_pid=$!

  stdout_path="$launchdeck_log_dir/engine.log"
  stderr_path="$launchdeck_log_dir/engine-error.log"
  nohup cargo run --manifest-path "rust/launchdeck-engine/Cargo.toml" --bin launchdeck-engine \
    >"$stdout_path" 2>"$stderr_path" </dev/null &
  engine_pid=$!

  (
    if wait_for_health_endpoint "http://127.0.0.1:$follow_daemon_port/health" "LaunchDeck follow daemon" "$daemon_pid" "$daemon_stderr_path" 40 0.5; then
      echo "LaunchDeck follow daemon ready on port $follow_daemon_port."
    else
      case $? in
        2) echo "LaunchDeck follow daemon is still starting in the background on port $follow_daemon_port." >&2 ;;
        *) return 1 ;;
      esac
    fi
  ) &
  daemon_wait_pid=$!

  (
    if wait_for_health_endpoint "http://127.0.0.1:$engine_port/health" "LaunchDeck Rust host" "$engine_pid" "$stderr_path" 60 0.5; then
      echo "LaunchDeck Rust host ready on port $engine_port."
      if command -v xdg-open >/dev/null 2>&1; then
        xdg-open "http://127.0.0.1:$engine_port" >/dev/null 2>&1 || true
      fi
    else
      case $? in
        2) echo "LaunchDeck Rust host is still starting in the background on port $engine_port." >&2 ;;
        *) return 1 ;;
      esac
    fi
  ) &
  engine_wait_pid=$!

  wait "$daemon_wait_pid"
  wait "$engine_wait_pid"
}

cd "$project_root"
start_launchdeck_processes
