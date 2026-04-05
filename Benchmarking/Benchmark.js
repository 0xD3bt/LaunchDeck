#!/usr/bin/env node
/**
 * RPC + WebSocket benchmarks (ms).
 *
 * HTTP JSON-RPC: `getMultipleAccounts` with a fixed 3-account basket — cold
 * (new TCP/TLS each request via agent: false) vs warm (single keep-alive agent after warmup).
 *
 * WebSocket: `slotSubscribe` ack, `accountSubscribe` ack, and `slotSubscribe`
 * first-notification timing; Helius unified hosts also measure `transactionSubscribe`
 * ack + first-notification timing.
 *
 * Writes JSON under ./.local/launchdeck/rpc-ws-bench/ at the repo root.
 *
 * Usage:
 *   npm run ws-bench -- "wss://a?..." "wss://b?..."
 *   npm run ws-bench -- --rpc-only "https://..."
 *   npm run ws-bench -- --ws-only "wss://..."
 *
 * Optional env URLs: --from-env → SOLANA_WS_URL, HELIUS_WS_URL (same order).
 * Longer runs: --preset quick|standard|long|extended (warmup + samples per metric).
 */

const fs = require("fs");
const http = require("http");
const https = require("https");
const os = require("os");
const path = require("path");
const WebSocket = require("ws");

const PROJECT_ROOT = path.resolve(__dirname, "..");
const RESULT_SUBDIR = path.join(".local", "launchdeck", "rpc-ws-bench");
const DEFAULT_HTTP_TIMEOUT_MS = 30000;
const HELIUS_STANDARD_HOST = "mainnet.helius-rpc.com";
const HELIUS_GATEKEEPER_HOST = "beta.helius-rpc.com";
const SHYFT_FREE_TIER_HOST = "rpc.fra.shyft.to";
const DEFAULT_TRANSACTION_SUBSCRIBE_ACCOUNT =
  "11111111111111111111111111111111";

/** Longer runs: each metric uses `samples` iterations (HTTP cold = that many new connections). */
const PRESETS = {
  quick: { warmup: 10, samples: 80 },
  standard: { warmup: 25, samples: 200 },
  long: { warmup: 50, samples: 400 },
  extended: { warmup: 75, samples: 800 },
};

const HTTP_BENCH_METHOD = "getMultipleAccounts";
const HTTP_BENCH_COMMITMENT = "processed";
const HTTP_BENCH_ACCOUNT_SOURCE = "recent LaunchDeck launch-history mints";
const DEFAULT_HTTP_ACCOUNT_BASKET = [
  "8yvYMFQQfayE4Suzj432PQKcKjvN8LsRRp7ZnbKK3RrP",
  "4XVxLCBjUyrLtxxWpFyS1ABAn82ymmKmEcBufJ82km86",
  "Efi6avQnLNMYwzQ1DjwGnqagT2a42PFtDpzGCrQNwko4",
];
const DEFAULT_HTTP_BENCH_BODY = (id) =>
  JSON.stringify({
    jsonrpc: "2.0",
    id,
    method: HTTP_BENCH_METHOD,
    params: [
      DEFAULT_HTTP_ACCOUNT_BASKET,
      {
        encoding: "base64",
        commitment: HTTP_BENCH_COMMITMENT,
      },
    ],
  });

function parseIntegerFlag(flag, rawValue, { min = Number.MIN_SAFE_INTEGER } = {}) {
  if (rawValue == null) {
    throw new Error(`missing value for ${flag}`);
  }
  const value = Number(rawValue);
  if (!Number.isInteger(value) || value < min) {
    throw new Error(`invalid value for ${flag}: ${rawValue}`);
  }
  return value;
}

function normalizedHostname(urlStr) {
  try {
    return new URL(urlStr).hostname.trim().toLowerCase();
  } catch {
    return "";
  }
}

function isHeliusUnifiedHost(urlStr) {
  const host = normalizedHostname(urlStr);
  return host === HELIUS_STANDARD_HOST || host === HELIUS_GATEKEEPER_HOST;
}

function rewriteHeliusHost(urlStr, targetHostname) {
  const parsed = new URL(urlStr);
  parsed.hostname = targetHostname;
  return parsed.toString();
}

function expandHeliusBothUrls(urls) {
  const expanded = [];
  const seen = new Set();
  for (const url of urls) {
    const variants = isHeliusUnifiedHost(url)
      ? [
          rewriteHeliusHost(url, HELIUS_STANDARD_HOST),
          rewriteHeliusHost(url, HELIUS_GATEKEEPER_HOST),
        ]
      : [url];
    for (const variant of variants) {
      if (seen.has(variant)) continue;
      seen.add(variant);
      expanded.push(variant);
    }
  }
  return expanded;
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function maybeSleep(ms) {
  if (ms > 0) {
    await sleep(ms);
  }
}

function pct(sorted, p) {
  const k = Math.min(
    sorted.length - 1,
    Math.max(0, Math.round(p * (sorted.length - 1)))
  );
  return sorted[k];
}

function stdev(times, mean) {
  if (times.length < 2) return NaN;
  const v =
    times.reduce((s, x) => s + (x - mean) ** 2, 0) / (times.length - 1);
  return Math.sqrt(v);
}

/** Origin + path + `?…` if query present (no secrets in saved JSON). */
function redactUrl(u) {
  try {
    const x = new URL(u);
    const q = x.search ? "?…" : "";
    return `${x.protocol}//${x.host}${x.pathname}${q}`;
  } catch {
    return "(invalid-url)";
  }
}

function endpointLabel(urlStr, index) {
  const host = normalizedHostname(urlStr);
  if (host === HELIUS_STANDARD_HOST) {
    return `helius_standard_${index + 1}`;
  }
  if (host === HELIUS_GATEKEEPER_HOST) {
    return `helius_gatekeeper_${index + 1}`;
  }
  if (host === SHYFT_FREE_TIER_HOST) {
    return `shyft_free_tier_${index + 1}`;
  }
  return `endpoint_${index + 1}`;
}

function summarizeLatencyMs(times) {
  if (!times.length) return null;
  const sorted = [...times].sort((a, b) => a - b);
  const mean = times.reduce((a, b) => a + b, 0) / times.length;
  return {
    n: times.length,
    min: Number(Math.min(...times).toFixed(2)),
    max: Number(Math.max(...times).toFixed(2)),
    mean: Number(mean.toFixed(2)),
    stdev:
      times.length > 1 ? Number(stdev(times, mean).toFixed(2)) : null,
    p50: Number(pct(sorted, 0.5).toFixed(2)),
    p95: Number(pct(sorted, 0.95).toFixed(2)),
    samplesMs: times.map((x) => Number(x.toFixed(2))),
  };
}

function printLatencyStats(title, times) {
  const s = summarizeLatencyMs(times);
  if (!s) return;
  console.log(title);
  console.log(`  min:    ${s.min.toFixed(2)}`);
  console.log(`  max:    ${s.max.toFixed(2)}`);
  console.log(`  mean:   ${s.mean.toFixed(2)}`);
  console.log(
    `  stdev:  ${s.stdev != null ? s.stdev.toFixed(2) : "n/a"}`
  );
  console.log(`  p50:    ${s.p50.toFixed(2)}`);
  console.log(`  p95:    ${s.p95.toFixed(2)}`);
}

/** wss://host/path?q → https://host/path?q */
function wsUrlToHttpRpcUrl(wsUrl) {
  if (wsUrl.startsWith("wss://")) {
    return "https://" + wsUrl.slice("wss://".length);
  }
  if (wsUrl.startsWith("ws://")) {
    return "http://" + wsUrl.slice("ws://".length);
  }
  return wsUrl;
}

/**
 * POST JSON-RPC; measures full request until response body end.
 * @param {string} urlStr - https:// or http://
 * @param {string} body
 * @param {http.Agent | https.Agent | false | undefined} agent - false = new connection (cold)
 * @param {number} timeoutMs
 */
function httpJsonRpcRoundTrip(
  urlStr,
  body,
  agent,
  timeoutMs = DEFAULT_HTTP_TIMEOUT_MS
) {
  return new Promise((resolve, reject) => {
    const t0 = performance.now();
    const u = new URL(urlStr);
    const isHttps = u.protocol === "https:";
    const lib = isHttps ? https : http;
    const defaultPort = isHttps ? 443 : 80;
    const opts = {
      hostname: u.hostname,
      port: u.port || defaultPort,
      path: u.pathname + u.search,
      method: "POST",
      agent: agent === undefined ? false : agent,
      headers: {
        "Content-Type": "application/json",
        "Content-Length": Buffer.byteLength(body),
      },
    };
    const req = lib.request(opts, (res) => {
      let buf = "";
      res.setEncoding("utf8");
      res.on("data", (c) => {
        buf += c;
      });
      res.on("end", () => {
        const ms = performance.now() - t0;
        if (res.statusCode && res.statusCode >= 400) {
          reject(
            new Error(`HTTP ${res.statusCode}: ${buf.slice(0, 240)}`)
          );
          return;
        }
        try {
          const msg = JSON.parse(buf);
          if (msg.error) reject(new Error(JSON.stringify(msg.error)));
          else resolve(ms);
        } catch (e) {
          reject(e);
        }
      });
    });
    req.setTimeout(timeoutMs, () => {
      req.destroy(new Error(`HTTP request timed out after ${timeoutMs}ms`));
    });
    req.on("error", reject);
    req.write(body);
    req.end();
  });
}

// --- HTTP RPC: cold (each request opts out of keep-alive) ---

async function benchHttpRpcCold(httpUrl, label, samples, requestGapMs) {
  console.log(`\n--- HTTP JSON-RPC cold (${label}) ---`);
  console.log(httpUrl.replace(/\?.*$/, "?…"));
  const times = [];
  for (let i = 0; i < samples; i++) {
    if (i > 0) {
      await maybeSleep(requestGapMs);
    }
    const body = DEFAULT_HTTP_BENCH_BODY(i + 1);
    times.push(await httpJsonRpcRoundTrip(httpUrl, body, false));
  }
  printLatencyStats(
    `${HTTP_BENCH_METHOD}_ms (agent:false each request, n=${samples}):`,
    times
  );
  return { stats: summarizeLatencyMs(times) };
}

// --- HTTP RPC: warm (one keep-alive agent) ---

async function benchHttpRpcWarm(httpUrl, label, warmup, samples, requestGapMs) {
  console.log(`\n--- HTTP JSON-RPC warm (${label}) ---`);
  console.log(httpUrl.replace(/\?.*$/, "?…"));
  const u = new URL(httpUrl);
  const Agent = u.protocol === "https:" ? https.Agent : http.Agent;
  const agent = new Agent({ keepAlive: true, maxSockets: 1 });
  try {
    for (let i = 0; i < warmup; i++) {
      if (i > 0) {
        await maybeSleep(requestGapMs);
      }
      const body = DEFAULT_HTTP_BENCH_BODY(500_000 + i);
      await httpJsonRpcRoundTrip(httpUrl, body, agent);
    }
    const times = [];
    for (let i = 0; i < samples; i++) {
      if (i > 0 || warmup > 0) {
        await maybeSleep(requestGapMs);
      }
      const body = DEFAULT_HTTP_BENCH_BODY(i + 1);
      times.push(await httpJsonRpcRoundTrip(httpUrl, body, agent));
    }
    printLatencyStats(
      `${HTTP_BENCH_METHOD}_ms (keep-alive agent, after ${warmup} warmup, n=${samples}):`,
      times
    );
    return { stats: summarizeLatencyMs(times), warmup };
  } finally {
    agent.destroy();
  }
}

// --- WebSocket slotSubscribe ---

function waitJsonRpcResponse(ws, expectId, timeoutMs) {
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => {
      ws.removeListener("message", onMsg);
      reject(new Error(`timeout waiting for json-rpc id=${expectId}`));
    }, timeoutMs);
    function onMsg(data) {
      let msg;
      try {
        msg = JSON.parse(data.toString());
      } catch {
        return;
      }
      if (msg.method) return;
      if (msg.id !== expectId) return;
      clearTimeout(t);
      ws.removeListener("message", onMsg);
      if (msg.error) reject(new Error(JSON.stringify(msg.error)));
      else resolve(msg.result);
    }
    ws.on("message", onMsg);
  });
}

async function subscribeAckCycle(
  ws,
  reqId,
  { subscribeMethod, subscribeParams, unsubscribeMethod }
) {
  const t0 = performance.now();
  ws.send(
    JSON.stringify({
      jsonrpc: "2.0",
      id: reqId,
      method: subscribeMethod,
      params: subscribeParams,
    })
  );
  const subId = await waitJsonRpcResponse(ws, reqId, 20000);
  const ackMs = performance.now() - t0;

  const unsubId = reqId + 1_000_000;
  ws.send(
    JSON.stringify({
      jsonrpc: "2.0",
      id: unsubId,
      method: unsubscribeMethod,
      params: [subId],
    })
  );
  await waitJsonRpcResponse(ws, unsubId, 10000).catch(() => {});

  return ackMs;
}

const SLOT_SUBSCRIBE_BENCH = {
  subscribeMethod: "slotSubscribe",
  subscribeParams: [],
  unsubscribeMethod: "slotUnsubscribe",
  notificationMethod: "slotNotification",
};

const HELIUS_TRANSACTION_SUBSCRIBE_BENCH = {
  subscribeMethod: "transactionSubscribe",
  subscribeParams: [
    {
      accountInclude: [DEFAULT_TRANSACTION_SUBSCRIBE_ACCOUNT],
      failed: false,
      vote: false,
    },
    {
      commitment: "processed",
      encoding: "jsonParsed",
      transactionDetails: "none",
      showRewards: false,
      maxSupportedTransactionVersion: 0,
    },
  ],
  unsubscribeMethod: "transactionUnsubscribe",
  notificationMethod: "transactionNotification",
};

function buildAccountSubscribeBench(account) {
  return {
    subscribeMethod: "accountSubscribe",
    subscribeParams: [
      account,
      {
        encoding: "base64",
        commitment: HTTP_BENCH_COMMITMENT,
      },
    ],
    unsubscribeMethod: "accountUnsubscribe",
    notificationMethod: "accountNotification",
  };
}

function waitJsonRpcNotification(ws, expectedMethod, expectedSubscriptionId, timeoutMs) {
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => {
      ws.removeListener("message", onMsg);
      reject(new Error(`timeout waiting for notification ${expectedMethod}`));
    }, timeoutMs);
    function onMsg(data) {
      let msg;
      try {
        msg = JSON.parse(data.toString());
      } catch {
        return;
      }
      if (!msg.method || !msg.params) return;
      if (expectedMethod && msg.method !== expectedMethod) return;
      if (
        expectedSubscriptionId != null &&
        msg.params.subscription !== expectedSubscriptionId
      ) {
        return;
      }
      clearTimeout(t);
      ws.removeListener("message", onMsg);
      resolve(msg);
    }
    ws.on("message", onMsg);
  });
}

async function subscribeFirstNotificationCycle(
  ws,
  reqId,
  {
    subscribeMethod,
    subscribeParams,
    unsubscribeMethod,
    notificationMethod,
  }
) {
  ws.send(
    JSON.stringify({
      jsonrpc: "2.0",
      id: reqId,
      method: subscribeMethod,
      params: subscribeParams,
    })
  );
  const subId = await waitJsonRpcResponse(ws, reqId, 20000);
  const t0 = performance.now();
  await waitJsonRpcNotification(ws, notificationMethod, subId, 30000);
  const firstNotificationMs = performance.now() - t0;

  const unsubId = reqId + 1_000_000;
  ws.send(
    JSON.stringify({
      jsonrpc: "2.0",
      id: unsubId,
      method: unsubscribeMethod,
      params: [subId],
    })
  );
  await waitJsonRpcResponse(ws, unsubId, 10000).catch(() => {});

  return firstNotificationMs;
}

async function benchWebSocketSlotSubscribe(uri, label, warmup, samples, requestGapMs) {
  console.log(`\n--- WebSocket (${label}) ---`);
  console.log(uri.replace(/\?.*$/, "?…"));

  async function openWs() {
    const ws = new WebSocket(uri, { handshakeTimeout: 30000 });
    await new Promise((resolve, reject) => {
      ws.once("open", resolve);
      ws.once("error", reject);
    });
    return ws;
  }

  async function withFreshWs(run) {
    const ws = await openWs();
    try {
      return await run(ws);
    } finally {
      ws.close();
      await new Promise((r) => ws.once("close", r));
    }
  }

  const tConn = performance.now();
  {
    const ws = await openWs();
    ws.close();
    await new Promise((r) => ws.once("close", r));
  }
  const connectOnlyMs = performance.now() - tConn;

  const acks = await withFreshWs(async (ws) => {
    for (let i = 0; i < warmup; i++) {
      if (i > 0) {
        await maybeSleep(requestGapMs);
      }
      await subscribeAckCycle(ws, 100_000 + i, SLOT_SUBSCRIBE_BENCH);
    }

    const samplesMs = [];
    for (let i = 0; i < samples; i++) {
      if (i > 0 || warmup > 0) {
        await maybeSleep(requestGapMs);
      }
      samplesMs.push(await subscribeAckCycle(ws, i, SLOT_SUBSCRIBE_BENCH));
    }
    return samplesMs;
  });

  const accountAcks = await withFreshWs(async (ws) => {
    for (let i = 0; i < warmup; i++) {
      if (i > 0) {
        await maybeSleep(requestGapMs);
      }
      const account =
        DEFAULT_HTTP_ACCOUNT_BASKET[i % DEFAULT_HTTP_ACCOUNT_BASKET.length];
      await subscribeAckCycle(
        ws,
        600_000 + i,
        buildAccountSubscribeBench(account)
      );
    }

    const samplesMs = [];
    for (let i = 0; i < samples; i++) {
      if (i > 0 || warmup > 0) {
        await maybeSleep(requestGapMs);
      }
      const account =
        DEFAULT_HTTP_ACCOUNT_BASKET[i % DEFAULT_HTTP_ACCOUNT_BASKET.length];
      samplesMs.push(
        await subscribeAckCycle(
          ws,
          500_000 + i,
          buildAccountSubscribeBench(account)
        )
      );
    }
    return samplesMs;
  });

  const slotFirstNotifications = await withFreshWs(async (ws) => {
    for (let i = 0; i < warmup; i++) {
      if (i > 0) {
        await maybeSleep(requestGapMs);
      }
      await subscribeFirstNotificationCycle(
        ws,
        1_100_000 + i,
        SLOT_SUBSCRIBE_BENCH
      );
    }

    const samplesMs = [];
    for (let i = 0; i < samples; i++) {
      if (i > 0 || warmup > 0) {
        await maybeSleep(requestGapMs);
      }
      samplesMs.push(
        await subscribeFirstNotificationCycle(
          ws,
          1_000_000 + i,
          SLOT_SUBSCRIBE_BENCH
        )
      );
    }
    return samplesMs;
  });

  let transactionSubscribeAck = null;
  let transactionSubscribeFirstNotification = null;
  if (isHeliusUnifiedHost(uri)) {
    const txAcks = await withFreshWs(async (ws) => {
      for (let i = 0; i < warmup; i++) {
        if (i > 0) {
          await maybeSleep(requestGapMs);
        }
        await subscribeAckCycle(
          ws,
          2_100_000 + i,
          HELIUS_TRANSACTION_SUBSCRIBE_BENCH
        );
      }

      const samplesMs = [];
      for (let i = 0; i < samples; i++) {
        if (i > 0 || warmup > 0) {
          await maybeSleep(requestGapMs);
        }
        samplesMs.push(
          await subscribeAckCycle(
            ws,
            2_000_000 + i,
            HELIUS_TRANSACTION_SUBSCRIBE_BENCH
          )
        );
      }
      return samplesMs;
    });
    transactionSubscribeAck = summarizeLatencyMs(txAcks);

    const txFirstNotifications = await withFreshWs(async (ws) => {
      for (let i = 0; i < warmup; i++) {
        if (i > 0) {
          await maybeSleep(requestGapMs);
        }
        await subscribeFirstNotificationCycle(
          ws,
          3_100_000 + i,
          HELIUS_TRANSACTION_SUBSCRIBE_BENCH
        );
      }

      const samplesMs = [];
      for (let i = 0; i < samples; i++) {
        if (i > 0 || warmup > 0) {
          await maybeSleep(requestGapMs);
        }
        samplesMs.push(
          await subscribeFirstNotificationCycle(
            ws,
            3_000_000 + i,
            HELIUS_TRANSACTION_SUBSCRIBE_BENCH
          )
        );
      }
      return samplesMs;
    });
    transactionSubscribeFirstNotification =
      summarizeLatencyMs(txFirstNotifications);
  }

  const slotFirstNotification = summarizeLatencyMs(slotFirstNotifications);
  const accountSubscribeAck = summarizeLatencyMs(accountAcks);
  const sorted = [...acks].sort((a, b) => a - b);
  const mean = acks.reduce((a, b) => a + b, 0) / acks.length;

  console.log(
    `connect_handshake_ms (open+close, no traffic): ${connectOnlyMs.toFixed(2)}`
  );
  console.log(
    `slotSubscribe_ack_ms (same connection, after ${warmup} warmup, n=${samples}):`
  );
  console.log(`  min:    ${Math.min(...acks).toFixed(2)}`);
  console.log(`  max:    ${Math.max(...acks).toFixed(2)}`);
  console.log(`  mean:   ${mean.toFixed(2)}`);
  console.log(
    `  stdev:  ${acks.length > 1 ? stdev(acks, mean).toFixed(2) : "n/a"}`
  );
  console.log(`  p50:    ${pct(sorted, 0.5).toFixed(2)}`);
  console.log(`  p95:    ${pct(sorted, 0.95).toFixed(2)}`);
  console.log(
    `accountSubscribe_ack_ms (rotating LaunchDeck-history accounts, after ${warmup} warmup, n=${samples}):`
  );
  console.log(`  min:    ${accountSubscribeAck.min.toFixed(2)}`);
  console.log(`  max:    ${accountSubscribeAck.max.toFixed(2)}`);
  console.log(`  mean:   ${accountSubscribeAck.mean.toFixed(2)}`);
  console.log(
    `  stdev:  ${
      accountSubscribeAck.stdev != null
        ? accountSubscribeAck.stdev.toFixed(2)
        : "n/a"
    }`
  );
  console.log(`  p50:    ${accountSubscribeAck.p50.toFixed(2)}`);
  console.log(`  p95:    ${accountSubscribeAck.p95.toFixed(2)}`);
  console.log(
    `slotSubscribe_first_notification_ms (after subscribe ack, same connection, n=${samples}):`
  );
  console.log(`  min:    ${slotFirstNotification.min.toFixed(2)}`);
  console.log(`  max:    ${slotFirstNotification.max.toFixed(2)}`);
  console.log(`  mean:   ${slotFirstNotification.mean.toFixed(2)}`);
  console.log(
    `  stdev:  ${
      slotFirstNotification.stdev != null
        ? slotFirstNotification.stdev.toFixed(2)
        : "n/a"
    }`
  );
  console.log(`  p50:    ${slotFirstNotification.p50.toFixed(2)}`);
  console.log(`  p95:    ${slotFirstNotification.p95.toFixed(2)}`);
  if (transactionSubscribeAck) {
    console.log(
      `transactionSubscribe_ack_ms (same connection, after ${warmup} warmup, n=${samples}):`
    );
    console.log(`  min:    ${transactionSubscribeAck.min.toFixed(2)}`);
    console.log(`  max:    ${transactionSubscribeAck.max.toFixed(2)}`);
    console.log(`  mean:   ${transactionSubscribeAck.mean.toFixed(2)}`);
    console.log(
      `  stdev:  ${
        transactionSubscribeAck.stdev != null
          ? transactionSubscribeAck.stdev.toFixed(2)
          : "n/a"
      }`
    );
    console.log(`  p50:    ${transactionSubscribeAck.p50.toFixed(2)}`);
    console.log(`  p95:    ${transactionSubscribeAck.p95.toFixed(2)}`);
    console.log(
      `transactionSubscribe_first_notification_ms (after subscribe ack, same connection, n=${samples}):`
    );
    console.log(
      `  min:    ${transactionSubscribeFirstNotification.min.toFixed(2)}`
    );
    console.log(
      `  max:    ${transactionSubscribeFirstNotification.max.toFixed(2)}`
    );
    console.log(
      `  mean:   ${transactionSubscribeFirstNotification.mean.toFixed(2)}`
    );
    console.log(
      `  stdev:  ${
        transactionSubscribeFirstNotification.stdev != null
          ? transactionSubscribeFirstNotification.stdev.toFixed(2)
          : "n/a"
      }`
    );
    console.log(
      `  p50:    ${transactionSubscribeFirstNotification.p50.toFixed(2)}`
    );
    console.log(
      `  p95:    ${transactionSubscribeFirstNotification.p95.toFixed(2)}`
    );
  }

  return {
    connectHandshakeMs: Number(connectOnlyMs.toFixed(2)),
    slotSubscribeAck: summarizeLatencyMs(acks),
    accountSubscribeAck,
    slotSubscribeFirstNotification: slotFirstNotification,
    transactionSubscribeAck,
    transactionSubscribeFirstNotification,
    warmup,
  };
}

function parseArgs(argv) {
  const urls = [];
  let warmup = PRESETS.quick.warmup;
  let samples = PRESETS.quick.samples;
  let warmupExplicit = false;
  let samplesExplicit = false;
  let pauseMs = 1000;
  let fromEnv = false;
  let mode = "both";
  let noSave = false;
  let preset = null;
  let heliusBoth = false;
  let requestGapMs = 0;

  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--help" || a === "-h") {
      return {
        help: true,
        urls,
        warmup,
        samples,
        pauseMs,
        fromEnv,
        mode,
        noSave,
        preset,
        heliusBoth,
        requestGapMs,
      };
    }
    if (a === "--preset") {
      preset = String(argv[++i] || "").trim().toLowerCase();
      continue;
    }
    if (a === "--warmup") {
      warmup = parseIntegerFlag("--warmup", argv[++i], { min: 0 });
      warmupExplicit = true;
      continue;
    }
    if (a === "--samples") {
      samples = parseIntegerFlag("--samples", argv[++i], { min: 1 });
      samplesExplicit = true;
      continue;
    }
    if (a === "--pause-ms") {
      pauseMs = parseIntegerFlag("--pause-ms", argv[++i], { min: 0 });
      continue;
    }
    if (a === "--request-gap-ms") {
      requestGapMs = parseIntegerFlag("--request-gap-ms", argv[++i], { min: 0 });
      continue;
    }
    if (a === "--max-rps") {
      const maxRps = parseIntegerFlag("--max-rps", argv[++i], { min: 1 });
      requestGapMs = Math.ceil(1000 / maxRps);
      continue;
    }
    if (a === "--from-env") {
      fromEnv = true;
      continue;
    }
    if (a === "--ws-only") {
      mode = "ws";
      continue;
    }
    if (a === "--rpc-only" || a === "--http-only") {
      mode = "rpc";
      continue;
    }
    if (a === "--no-save") {
      noSave = true;
      continue;
    }
    if (a === "--helius-both") {
      heliusBoth = true;
      continue;
    }
    if (a.startsWith("-")) {
      throw new Error(`unknown flag: ${a}`);
    }
    urls.push(a);
  }

  if (fromEnv) {
    const sol = (process.env.SOLANA_WS_URL || "").trim();
    const hel = (process.env.HELIUS_WS_URL || "").trim();
    if (sol) urls.push(sol);
    if (hel) urls.push(hel);
  }

  let presetApplied = null;
  if (preset) {
    const p = PRESETS[preset];
    if (!p) {
      const names = Object.keys(PRESETS).join(", ");
      throw new Error(`unknown --preset "${preset}" (use: ${names})`);
    }
    presetApplied = preset;
    if (!warmupExplicit) warmup = p.warmup;
    if (!samplesExplicit) samples = p.samples;
  }

  const expandedUrls = heliusBoth ? expandHeliusBothUrls(urls) : urls;

  return {
    help: false,
    urls: expandedUrls,
    warmup,
    samples,
    pauseMs,
    requestGapMs,
    fromEnv,
    mode,
    noSave,
    preset: presetApplied,
    heliusBoth,
  };
}

function printHelp() {
  console.log(`launchdeck RPC + WebSocket bench (ms)

Usage:
  node Benchmarking/Benchmark.js [options] <url> [url ...]
  npm run ws-bench -- [options] <url> [url ...]

URLs:
  wss:// or ws://   WebSocket + derived HTTPS/HTTP for JSON-RPC (unless --ws-only)
  https:// or http:// JSON-RPC only (cold + warm); WebSocket skipped

Options:
  --preset <name>  quick | standard | long | extended — sets warmup + samples (see below)
  --warmup <n>     WS + HTTP warm paths: cycles before timing (default: preset or quick=10)
  --samples <n>    timed cycles per metric (default: preset or quick=80)
  --pause-ms <n>   delay between top-level URL entries (default 1000)
  --request-gap-ms <n>  delay between individual timed/warmup requests (default 0)
  --max-rps <n>    shorthand for per-request pacing; e.g. 10 => 100ms gap
  --from-env       append SOLANA_WS_URL then HELIUS_WS_URL if set
  --helius-both    for any Helius unified URL, benchmark both mainnet + beta hosts
  --ws-only        WebSocket benchmark only (slotSubscribe + accountSubscribe; Helius hosts also include transactionSubscribe)
  --rpc-only       HTTP ${HTTP_BENCH_METHOD} only (alias: --http-only)
  --no-save        do not write ./.local/launchdeck/rpc-ws-bench/run-*.{json,md}

Results (default): ./.local/launchdeck/rpc-ws-bench/run-<timestamp>.md (markdown summary) + .json (full report, query strings redacted)

Presets (warmup / samples per metric; HTTP cold runs samples new connections each):
  quick     10 / 80
  standard  25 / 200
  long      50 / 400
  extended  75 / 800
  --warmup / --samples after --preset override that field only.

Default mode runs, per wss URL: HTTP cold, HTTP warm, then WebSocket.
WebSocket runs include slotSubscribe ack, accountSubscribe ack, and slotSubscribe time-to-first-notification.
Helius unified hosts also include transactionSubscribe ack + first-notification timing.
For https URLs: HTTP cold + warm only.

Example:
  npm run ws-bench -- "wss://rpc.example?api_key=..." "wss://mainnet.helius-rpc.com/?api-key=..."
  npm run ws-bench -- --helius-both "wss://mainnet.helius-rpc.com/?api-key=..."
  npm run ws-bench -- --max-rps 10 "wss://rpc.fra.shyft.to/?api_key=..."
`);
}

function ensureResultDir() {
  const dir = path.join(PROJECT_ROOT, RESULT_SUBDIR);
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

/** Markdown table for latency summary (no raw sample list). */
function formatStatsTableMd(stats) {
  if (!stats) {
    return "_No data._\n\n";
  }
  const rows = [
    "| Metric | Value |",
    "|--------|-------|",
    `| Samples (n) | ${stats.n} |`,
    `| Min (ms) | ${stats.min} |`,
    `| Max (ms) | ${stats.max} |`,
    `| Mean (ms) | ${stats.mean} |`,
    `| Stdev (ms) | ${stats.stdev != null ? stats.stdev : "—"} |`,
    `| p50 (ms) | ${stats.p50} |`,
    `| p95 (ms) | ${stats.p95} |`,
  ];
  return `${rows.join("\n")}\n\n`;
}

function formatMs(value) {
  return typeof value === "number" && Number.isFinite(value) ? `${value} ms` : "—";
}

function endpointDisplayName(ep) {
  try {
    const host = new URL(ep.inputUrl).hostname.toLowerCase();
    if (host === HELIUS_STANDARD_HOST) return "Helius standard";
    if (host === HELIUS_GATEKEEPER_HOST) return "Helius Gatekeeper";
    if (host === SHYFT_FREE_TIER_HOST) return "Shyft free tier";
    return host;
  } catch {
    return ep.label.replace(/_/g, " ");
  }
}

function bestEndpointByMetric(endpoints, getter) {
  let best = null;
  for (const ep of endpoints) {
    const value = getter(ep);
    if (typeof value !== "number" || !Number.isFinite(value)) continue;
    if (!best || value < best.value) {
      best = { ep, value };
    }
  }
  return best;
}

function buildReportMarkdown(report) {
  const lines = [];
  lines.push("# LaunchDeck benchmark report");
  lines.push("");
  lines.push(
    "RPC + WebSocket latency (`Benchmark.js`). All times in **milliseconds** unless noted."
  );
  lines.push("");
  lines.push("## Benchmark setup");
  lines.push("");
  lines.push("| Field | Value |");
  lines.push("|-------|-------|");
  lines.push(`| Started | ${report.startedAt} |`);
  lines.push(`| Finished | ${report.finishedAt || "—"} |`);
  lines.push(`| Host | \`${report.hostname}\` |`);
  lines.push(`| Working directory | \`${report.cwd}\` |`);
  lines.push(`| Exit code | ${report.exitCode ?? "—"} |`);
  lines.push(`| Tool | ${report.tool} (\`${report.script}\`) |`);
  if (report.runError) {
    lines.push(`| **Run error** | ${report.runError.replace(/\|/g, "\\|")} |`);
  }
  const o = report.options || {};
  const requestGapMs = o.requestGapMs ?? 0;
  const approxMaxRps =
    requestGapMs > 0 ? Number((1000 / requestGapMs).toFixed(2)) : null;
  lines.push(`| Preset | ${o.preset ?? "(none — quick defaults)"} |`);
  lines.push(`| Warmup cycles | ${o.warmup} |`);
  lines.push(`| Timed samples per metric | ${o.samples} |`);
  lines.push(`| Pause between URLs (ms) | ${o.pauseMs} |`);
  lines.push(`| Per-request gap (ms) | ${requestGapMs} |`);
  lines.push(
    `| Approx per-endpoint max RPS | ${
      approxMaxRps != null ? approxMaxRps : "unlimited"
    } |`
  );
  lines.push(`| Mode | ${o.mode} |`);
  lines.push(`| From env | ${o.fromEnv ? "yes" : "no"} |`);
  lines.push(`| Helius both | ${o.heliusBoth ? "yes" : "no"} |`);
  lines.push(`| HTTP benchmark method | \`${HTTP_BENCH_METHOD}\` |`);
  lines.push(`| HTTP benchmark commitment | \`${HTTP_BENCH_COMMITMENT}\` |`);
  lines.push(`| HTTP benchmark account source | ${HTTP_BENCH_ACCOUNT_SOURCE} |`);
  lines.push(
    `| HTTP benchmark accounts | \`${DEFAULT_HTTP_ACCOUNT_BASKET.join("`, `")}\` |`
  );
  lines.push(
    `| Helius transactionSubscribe filter | \`accountInclude=${DEFAULT_TRANSACTION_SUBSCRIBE_ACCOUNT}\`, \`failed=false\`, \`vote=false\`, \`commitment=processed\`, \`encoding=jsonParsed\`, \`transactionDetails=none\` |`
  );
  lines.push("");
  lines.push("## Input URLs (redacted)");
  lines.push("");
  (report.inputUrlsRedacted || []).forEach((u, i) => {
    lines.push(`${i + 1}. \`${u}\``);
  });
  lines.push("");

  const eps = report.endpoints || [];
  const okEps = eps.filter((e) => e.status === "ok");
  if (okEps.length > 0) {
    const hasHttp = okEps.some((ep) => ep.httpCold || ep.httpWarm);
    const hasWs = okEps.some((ep) => ep.webSocket);
    const bestHttpCold = bestEndpointByMetric(okEps, (ep) => ep.httpCold?.mean);
    const bestHttpWarm = bestEndpointByMetric(okEps, (ep) => ep.httpWarm?.mean);
    const bestWsHandshake = bestEndpointByMetric(
      okEps,
      (ep) => ep.webSocket?.connectHandshakeMs
    );
    const bestWsSlot = bestEndpointByMetric(
      okEps,
      (ep) => ep.webSocket?.slotSubscribeAck?.mean
    );
    const bestWsAccount = bestEndpointByMetric(
      okEps,
      (ep) => ep.webSocket?.accountSubscribeAck?.mean
    );
    const bestWsTx = bestEndpointByMetric(
      okEps,
      (ep) => ep.webSocket?.transactionSubscribeAck?.mean
    );
    const bestWsSlotFirst = bestEndpointByMetric(
      okEps,
      (ep) => ep.webSocket?.slotSubscribeFirstNotification?.mean
    );
    const bestWsTxFirst = bestEndpointByMetric(
      okEps,
      (ep) => ep.webSocket?.transactionSubscribeFirstNotification?.mean
    );

    lines.push("## Shareable summary");
    lines.push("");
    lines.push(
      `**Setup:** ${o.samples} timed samples per metric, ${o.warmup} warmup cycles, ${
        requestGapMs > 0 ? `${requestGapMs} ms gap (~${approxMaxRps} RPS per endpoint)` : "no request gap"
      }, mode=\`${o.mode}\`.`
    );
    lines.push("");
    lines.push("**Lower is better.** `avg` is the mean latency. `p95` shows the slower tail.");
    lines.push("");
    lines.push(
      "| Provider | HTTP cold avg | HTTP warm avg | WS handshake | WS slotSubscribe avg | WS accountSubscribe avg | WS transactionSubscribe avg |"
    );
    lines.push(
      "|----------|----------------|----------------|--------------|-----------------------|--------------------------|------------------------------|"
    );
    for (const ep of okEps) {
      lines.push(
        `| ${endpointDisplayName(ep)} | ${formatMs(ep.httpCold?.mean)} | ${
          formatMs(ep.httpWarm?.mean)
        } | ${formatMs(ep.webSocket?.connectHandshakeMs)} | ${
          formatMs(ep.webSocket?.slotSubscribeAck?.mean)
        } | ${formatMs(ep.webSocket?.accountSubscribeAck?.mean)} | ${
          formatMs(ep.webSocket?.transactionSubscribeAck?.mean)
        } |`
      );
    }
    lines.push("");
    lines.push(
      "| Provider | HTTP cold p95 | HTTP warm p95 | WS slotSubscribe p95 | WS accountSubscribe p95 | WS transactionSubscribe p95 |"
    );
    lines.push(
      "|----------|---------------|---------------|----------------------|-------------------------|-----------------------------|"
    );
    for (const ep of okEps) {
      lines.push(
        `| ${endpointDisplayName(ep)} | ${formatMs(ep.httpCold?.p95)} | ${
          formatMs(ep.httpWarm?.p95)
        } | ${formatMs(ep.webSocket?.slotSubscribeAck?.p95)} | ${
          formatMs(ep.webSocket?.accountSubscribeAck?.p95)
        } | ${formatMs(ep.webSocket?.transactionSubscribeAck?.p95)} |`
      );
    }
    lines.push("");
    lines.push(
      "| Provider | WS slot first notification avg | WS slot first notification p95 | WS transaction first notification avg | WS transaction first notification p95 |"
    );
    lines.push(
      "|----------|--------------------------------|--------------------------------|--------------------------------------|--------------------------------------|"
    );
    for (const ep of okEps) {
      lines.push(
        `| ${endpointDisplayName(ep)} | ${formatMs(
          ep.webSocket?.slotSubscribeFirstNotification?.mean
        )} | ${formatMs(
          ep.webSocket?.slotSubscribeFirstNotification?.p95
        )} | ${formatMs(
          ep.webSocket?.transactionSubscribeFirstNotification?.mean
        )} | ${formatMs(
          ep.webSocket?.transactionSubscribeFirstNotification?.p95
        )} |`
      );
    }
    lines.push("");

    lines.push("## Winners");
    lines.push("");
    if (bestHttpCold) {
      lines.push(
        `- Fastest HTTP cold avg: **${endpointDisplayName(bestHttpCold.ep)}** at **${formatMs(bestHttpCold.value)}**`
      );
    }
    if (bestHttpWarm) {
      lines.push(
        `- Fastest HTTP warm avg: **${endpointDisplayName(bestHttpWarm.ep)}** at **${formatMs(bestHttpWarm.value)}**`
      );
    }
    if (bestWsHandshake) {
      lines.push(
        `- Fastest WS handshake: **${endpointDisplayName(bestWsHandshake.ep)}** at **${formatMs(bestWsHandshake.value)}**`
      );
    }
    if (bestWsSlot) {
      lines.push(
        `- Fastest WS slotSubscribe avg: **${endpointDisplayName(bestWsSlot.ep)}** at **${formatMs(bestWsSlot.value)}**`
      );
    }
    if (bestWsAccount) {
      lines.push(
        `- Fastest WS accountSubscribe avg: **${endpointDisplayName(bestWsAccount.ep)}** at **${formatMs(bestWsAccount.value)}**`
      );
    }
    if (bestWsTx) {
      lines.push(
        `- Fastest WS transactionSubscribe avg: **${endpointDisplayName(bestWsTx.ep)}** at **${formatMs(bestWsTx.value)}**`
      );
    }
    if (bestWsSlotFirst) {
      lines.push(
        `- Fastest WS slot first notification avg: **${endpointDisplayName(bestWsSlotFirst.ep)}** at **${formatMs(bestWsSlotFirst.value)}**`
      );
    }
    if (bestWsTxFirst) {
      lines.push(
        `- Fastest WS transaction first notification avg: **${endpointDisplayName(bestWsTxFirst.ep)}** at **${formatMs(bestWsTxFirst.value)}**`
      );
    }
    lines.push("");

    if (hasHttp) {
      lines.push("## HTTP summary");
      lines.push("");
      lines.push(
        "| Provider | Cold mean | Cold p95 | Warm mean | Warm p95 |"
      );
      lines.push("|----------|-----------|----------|-----------|----------|");
      for (const ep of okEps.filter((ep) => ep.httpCold || ep.httpWarm)) {
        lines.push(
          `| ${endpointDisplayName(ep)} | ${ep.httpCold?.mean ?? "—"} | ${
            ep.httpCold?.p95 ?? "—"
          } | ${ep.httpWarm?.mean ?? "—"} | ${ep.httpWarm?.p95 ?? "—"} |`
        );
      }
      lines.push("");
    }

    if (hasWs) {
      lines.push("## WebSocket summary");
      lines.push("");
      lines.push(
        "| Provider | Handshake | slotSubscribe ack mean | slotSubscribe ack p95 | accountSubscribe ack mean | accountSubscribe ack p95 | slot first notification mean | slot first notification p95 | transactionSubscribe ack mean | transactionSubscribe ack p95 | transaction first notification mean | transaction first notification p95 |"
      );
      lines.push(
        "|----------|-----------|------------------------|-----------------------|---------------------------|--------------------------|------------------------------|-----------------------------|-------------------------------|------------------------------|-------------------------------------|------------------------------------|"
      );
      for (const ep of okEps.filter((ep) => ep.webSocket)) {
        const h = ep.webSocket?.connectHandshakeMs ?? "—";
        const sAckMean = ep.webSocket?.slotSubscribeAck?.mean ?? "—";
        const sAckP95 = ep.webSocket?.slotSubscribeAck?.p95 ?? "—";
        const aAckMean = ep.webSocket?.accountSubscribeAck?.mean ?? "—";
        const aAckP95 = ep.webSocket?.accountSubscribeAck?.p95 ?? "—";
        const sFirstMean =
          ep.webSocket?.slotSubscribeFirstNotification?.mean ?? "—";
        const sFirstP95 =
          ep.webSocket?.slotSubscribeFirstNotification?.p95 ?? "—";
        const tAckMean = ep.webSocket?.transactionSubscribeAck?.mean ?? "—";
        const tAckP95 = ep.webSocket?.transactionSubscribeAck?.p95 ?? "—";
        const tFirstMean =
          ep.webSocket?.transactionSubscribeFirstNotification?.mean ?? "—";
        const tFirstP95 =
          ep.webSocket?.transactionSubscribeFirstNotification?.p95 ?? "—";
        lines.push(
          `| ${endpointDisplayName(ep)} | ${h} | ${sAckMean} | ${sAckP95} | ${aAckMean} | ${aAckP95} | ${sFirstMean} | ${sFirstP95} | ${tAckMean} | ${tAckP95} | ${tFirstMean} | ${tFirstP95} |`
        );
      }
      lines.push("");
    }
  }

  lines.push("---");
  lines.push("");

  for (const ep of eps) {
    lines.push(`## ${endpointDisplayName(ep)}`);
    lines.push("");
    lines.push(`**Status:** \`${ep.status}\``);
    lines.push("");
    if (ep.reason) {
      lines.push(`**Note:** ${ep.reason}`);
      lines.push("");
    }
    if (ep.error) {
      lines.push(`**Error:** ${String(ep.error).replace(/\|/g, "\\|")}`);
      lines.push("");
    }
    lines.push(`- **Input URL:** \`${ep.inputUrl}\``);
    if (ep.derivedHttpRpcUrl) {
      lines.push(`- **Derived HTTP RPC URL:** \`${ep.derivedHttpRpcUrl}\``);
    }
    lines.push("");

    if (ep.httpCold) {
      lines.push(
        `### HTTP JSON-RPC — cold (\`${HTTP_BENCH_METHOD}\`, new TCP/TLS each request)`
      );
      lines.push("");
      lines.push(formatStatsTableMd(ep.httpCold));
    }
    if (ep.httpWarm) {
      lines.push(
        `### HTTP JSON-RPC — warm (\`${HTTP_BENCH_METHOD}\`, keep-alive, warmup cycles: ${ep.httpWarmWarmup ?? "—"})`
      );
      lines.push("");
      lines.push(formatStatsTableMd(ep.httpWarm));
    }
    if (ep.webSocket) {
      lines.push("### WebSocket");
      lines.push("");
      lines.push(
        `**Connect handshake** (open + close, no JSON-RPC): **${ep.webSocket.connectHandshakeMs} ms**`
      );
      lines.push("");
      lines.push(
        `**slotSubscribe ack** (after ${ep.webSocket.warmup ?? "—"} warmup, same connection):`
      );
      lines.push("");
      lines.push(formatStatsTableMd(ep.webSocket.slotSubscribeAck));
      lines.push(
        `**accountSubscribe ack** (rotating the LaunchDeck-history account basket, after ${ep.webSocket.warmup ?? "—"} warmup):`
      );
      lines.push("");
      lines.push(formatStatsTableMd(ep.webSocket.accountSubscribeAck));
      lines.push(
        "**slotSubscribe first notification** (time from subscribe ack to the first stream event on the same live connection):"
      );
      lines.push("");
      lines.push(formatStatsTableMd(ep.webSocket.slotSubscribeFirstNotification));
      if (ep.webSocket.transactionSubscribeAck) {
        lines.push(
          `**transactionSubscribe ack** (after ${ep.webSocket.warmup ?? "—"} warmup, same connection; Helius-style watcher path):`
        );
        lines.push("");
        lines.push(formatStatsTableMd(ep.webSocket.transactionSubscribeAck));
        lines.push(
          "**transactionSubscribe first notification** (time from subscribe ack to the first matching stream event on the same live connection):"
        );
        lines.push("");
        lines.push(
          formatStatsTableMd(ep.webSocket.transactionSubscribeFirstNotification)
        );
      }
    }
    lines.push("---");
    lines.push("");
  }

  lines.push("## Raw per-sample data");
  lines.push("");
  lines.push(
    "Per-request millisecond arrays are stored in the matching **`.json`** file under each metric as `samplesMs` (and the full machine-readable report)."
  );
  lines.push("");
  return lines.join("\n");
}

function writeReport(report, noSave) {
  if (noSave) return null;
  const dir = ensureResultDir();
  const stamp = new Date().toISOString().replace(/:/g, "-");
  const base = `run-${stamp}`;
  const jsonName = `${base}.json`;
  const mdName = `${base}.md`;
  const absJson = path.join(dir, jsonName);
  const absMd = path.join(dir, mdName);
  fs.writeFileSync(absJson, JSON.stringify(report, null, 2), "utf8");
  fs.writeFileSync(absMd, buildReportMarkdown(report), "utf8");
  return {
    absJson,
    absMd,
    relJson: path.join(RESULT_SUBDIR, jsonName),
    relMd: path.join(RESULT_SUBDIR, mdName),
  };
}

async function benchOneEndpoint(rawUrl, index, opts) {
  const u = rawUrl.trim();
  const label = endpointLabel(u, index);
  const isWs = u.startsWith("wss://") || u.startsWith("ws://");
  const isHttp = u.startsWith("https://") || u.startsWith("http://");

  const entry = {
    label,
    inputUrl: redactUrl(u),
    status: "ok",
  };

  if (!isWs && !isHttp) {
    entry.status = "skipped";
    entry.reason = "unrecognized URL scheme";
    return entry;
  }

  if (opts.mode === "ws" && !isWs) {
    entry.status = "skipped";
    entry.reason = "--ws-only requires wss:// or ws://";
    return entry;
  }

  console.log(`\n########## ${label} ##########`);

  const httpUrl = isWs ? wsUrlToHttpRpcUrl(u) : u;
  entry.derivedHttpRpcUrl = isWs ? redactUrl(httpUrl) : undefined;

  try {
    if (opts.mode !== "ws") {
      if (isHttp || isWs) {
        entry.httpCold = (
          await benchHttpRpcCold(httpUrl, label, opts.samples, opts.requestGapMs)
        ).stats;
        await sleep(200);
        const warm = await benchHttpRpcWarm(
          httpUrl,
          label,
          opts.warmup,
          opts.samples,
          opts.requestGapMs
        );
        entry.httpWarm = warm.stats;
        entry.httpWarmWarmup = warm.warmup;
      }
    }

    if (opts.mode !== "rpc" && isWs) {
      await sleep(200);
      entry.webSocket = await benchWebSocketSlotSubscribe(
        u,
        label,
        opts.warmup,
        opts.samples,
        opts.requestGapMs
      );
    }
  } catch (e) {
    entry.status = "error";
    entry.error = e instanceof Error ? e.message : String(e);
    return entry;
  }

  return entry;
}

async function main() {
  const cwd = process.cwd();
  const startedAt = new Date().toISOString();

  let opts;
  try {
    opts = parseArgs(process.argv.slice(2));
  } catch (e) {
    console.error(e.message || e);
    process.exit(1);
  }

  if (opts.help) {
    printHelp();
    process.exit(0);
  }

  if (opts.urls.length === 0) {
    printHelp();
    process.exit(1);
  }

  if (opts.warmup < 0 || opts.samples < 1 || opts.pauseMs < 0 || opts.requestGapMs < 0) {
    console.error("invalid --warmup, --samples, --pause-ms, or --request-gap-ms");
    process.exit(1);
  }

  const report = {
    schemaVersion: 1,
    tool: "Benchmark",
    script: "Benchmarking/Benchmark.js",
    startedAt,
    finishedAt: null,
    hostname: os.hostname(),
    cwd,
    options: {
      preset: opts.preset,
      warmup: opts.warmup,
      samples: opts.samples,
      pauseMs: opts.pauseMs,
      requestGapMs: opts.requestGapMs,
      mode: opts.mode,
      fromEnv: opts.fromEnv,
      noSave: opts.noSave,
      heliusBoth: opts.heliusBoth,
    },
    inputUrlsRedacted: opts.urls.map(redactUrl),
    endpoints: [],
  };

  let exitCode = 0;
  try {
    let idx = 0;
    for (const u of opts.urls) {
      const entry = await benchOneEndpoint(u, idx, opts);
      report.endpoints.push(entry);
      if (entry.status === "error") {
        exitCode = 1;
      }
      idx += 1;
      if (idx < opts.urls.length) await sleep(opts.pauseMs);
    }
  } catch (e) {
    exitCode = 1;
    report.runError = e instanceof Error ? e.message : String(e);
    console.error(e);
  } finally {
    report.finishedAt = new Date().toISOString();
    report.exitCode = exitCode;
    try {
      const out = writeReport(report, opts.noSave);
      if (out) {
        console.log("\nSaved results:");
        console.log(`  ${out.relMd}   (markdown summary)`);
        console.log(`  ${out.relJson} (full report)`);
      }
    } catch (e) {
      console.error("Failed to write results:", e.message || e);
    }
  }

  process.exit(exitCode);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
