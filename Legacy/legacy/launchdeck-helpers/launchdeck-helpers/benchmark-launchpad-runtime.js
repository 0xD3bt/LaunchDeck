"use strict";

const fs = require("fs");
const path = require("path");
const { performance } = require("perf_hooks");

const DEFAULT_BASE_URL = process.env.LAUNCHDECK_BASE_URL || "http://127.0.0.1:8789";
const DEFAULT_AUTH_TOKEN = process.env.LAUNCHDECK_ENGINE_AUTH_TOKEN || "4815927603149027";
const DEFAULT_ITERATIONS = 5;

function parseArgs(argv) {
  const options = {
    baseUrl: DEFAULT_BASE_URL,
    authToken: DEFAULT_AUTH_TOKEN,
    iterations: DEFAULT_ITERATIONS,
    launchpad: "pump",
    launchMode: "regular",
    mode: "sol",
    amount: "0.1",
    quoteAsset: "sol",
    scenarios: ["startup-warm", "runtime-status", "quote"],
    output: "",
    config: "",
    runAction: "build",
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];
    switch (arg) {
      case "--base-url":
        options.baseUrl = String(next || "").trim() || options.baseUrl;
        index += 1;
        break;
      case "--auth-token":
        options.authToken = String(next || "").trim();
        index += 1;
        break;
      case "--iterations":
        options.iterations = Math.max(1, Number.parseInt(next || "", 10) || DEFAULT_ITERATIONS);
        index += 1;
        break;
      case "--launchpad":
        options.launchpad = String(next || "").trim() || options.launchpad;
        index += 1;
        break;
      case "--launch-mode":
        options.launchMode = String(next || "").trim() || options.launchMode;
        index += 1;
        break;
      case "--mode":
        options.mode = String(next || "").trim() || options.mode;
        index += 1;
        break;
      case "--amount":
        options.amount = String(next || "").trim() || options.amount;
        index += 1;
        break;
      case "--quote-asset":
        options.quoteAsset = String(next || "").trim() || options.quoteAsset;
        index += 1;
        break;
      case "--config":
        options.config = String(next || "").trim();
        index += 1;
        break;
      case "--run-action":
        options.runAction = String(next || "").trim() || options.runAction;
        index += 1;
        break;
      case "--output":
        options.output = String(next || "").trim();
        index += 1;
        break;
      case "--scenarios":
        options.scenarios = String(next || "")
          .split(",")
          .map((value) => String(value || "").trim())
          .filter(Boolean);
        index += 1;
        break;
      default:
        break;
    }
  }
  if (options.config && !options.scenarios.includes("api-run")) {
    options.scenarios.push("api-run");
  }
  return options;
}

function percentile(sortedValues, fraction) {
  if (!sortedValues.length) return 0;
  const index = Math.min(sortedValues.length - 1, Math.max(0, Math.ceil(sortedValues.length * fraction) - 1));
  return sortedValues[index];
}

function summarizeScenario(name, samples) {
  const durations = samples.map((sample) => sample.elapsedMs).sort((left, right) => left - right);
  const total = durations.reduce((sum, value) => sum + value, 0);
  return {
    name,
    iterations: samples.length,
    minMs: durations[0] || 0,
    avgMs: durations.length ? Number((total / durations.length).toFixed(2)) : 0,
    p50Ms: percentile(durations, 0.50),
    p95Ms: percentile(durations, 0.95),
    maxMs: durations[durations.length - 1] || 0,
    samples,
  };
}

async function requestJson(method, url, { authToken = "", body } = {}) {
  const headers = { "content-type": "application/json" };
  if (authToken) headers["x-launchdeck-engine-auth"] = authToken;
  const startedAt = performance.now();
  const response = await fetch(url, {
    method,
    headers,
    body: body == null ? undefined : JSON.stringify(body),
  });
  const elapsedMs = Number((performance.now() - startedAt).toFixed(2));
  const text = await response.text();
  let payload = null;
  try {
    payload = text ? JSON.parse(text) : null;
  } catch (_error) {
    payload = { raw: text };
  }
  if (!response.ok) {
    const error = new Error(`Request failed: ${response.status} ${response.statusText}`);
    error.payload = payload;
    throw error;
  }
  return { elapsedMs, payload };
}

function buildScenarioRunner(options, scenario) {
  switch (scenario) {
    case "startup-warm":
      return () =>
        requestJson("POST", `${options.baseUrl}/api/startup-warm`, {
          authToken: options.authToken,
          body: {},
        }).then(({ elapsedMs, payload }) => ({
          elapsedMs,
          backend: payload && payload.launchpadBackends ? payload.launchpadBackends : null,
          summary: payload && payload.startupWarm ? payload.startupWarm : null,
        }));
    case "runtime-status":
      return () =>
        requestJson("GET", `${options.baseUrl}/api/runtime-status`, {
          authToken: options.authToken,
        }).then(({ elapsedMs, payload }) => ({
          elapsedMs,
          warmMode: payload && payload.warm ? payload.warm.mode : "",
          warmReason: payload && payload.warm ? payload.warm.reason : "",
        }));
    case "quote": {
      const params = new URLSearchParams({
        launchpad: options.launchpad,
        launchMode: options.launchMode,
        mode: options.mode,
        amount: options.amount,
        quoteAsset: options.quoteAsset,
      });
      return () =>
        requestJson("GET", `${options.baseUrl}/api/quote?${params.toString()}`, {
          authToken: options.authToken,
        }).then(({ elapsedMs, payload }) => ({
          elapsedMs,
          backend: payload && payload.backend ? payload.backend : "",
          rolloutState: payload && payload.rolloutState ? payload.rolloutState : "",
        }));
    }
    case "api-run": {
      if (!options.config) {
        throw new Error("The api-run scenario requires --config <path>.");
      }
      const rawConfig = JSON.parse(fs.readFileSync(path.resolve(options.config), "utf8"));
      return () =>
        requestJson("POST", `${options.baseUrl}/api/run`, {
          authToken: options.authToken,
          body: {
            action: options.runAction,
            rawConfig,
          },
        }).then(({ elapsedMs, payload }) => ({
          elapsedMs,
          action: payload && payload.action ? payload.action : options.runAction,
          backend: payload && payload.report && payload.report.execution
            ? payload.report.execution.launchpadBackend || ""
            : "",
          rolloutState: payload && payload.report && payload.report.execution
            ? payload.report.execution.launchpadRolloutState || ""
            : "",
          backendTotalMs: payload && payload.report && payload.report.execution && payload.report.execution.timings
            ? payload.report.execution.timings.backendTotalElapsedMs || null
            : null,
          compileMs: payload && payload.report && payload.report.execution && payload.report.execution.timings
            ? payload.report.execution.timings.compileTransactionsMs || null
            : null,
          sendMs: payload && payload.report && payload.report.execution && payload.report.execution.timings
            ? payload.report.execution.timings.sendMs || null
            : null,
        }));
    }
    default:
      throw new Error(`Unsupported benchmark scenario: ${scenario}`);
  }
}

async function run() {
  const options = parseArgs(process.argv.slice(2));
  const report = {
    capturedAt: new Date().toISOString(),
    baseUrl: options.baseUrl,
    launchpad: options.launchpad,
    launchMode: options.launchMode,
    mode: options.mode,
    amount: options.amount,
    quoteAsset: options.quoteAsset,
    iterations: options.iterations,
    scenarios: [],
  };
  for (const scenario of options.scenarios) {
    const runner = buildScenarioRunner(options, scenario);
    const samples = [];
    for (let iteration = 0; iteration < options.iterations; iteration += 1) {
      // Keep the raw sample payload slim so baseline files stay readable.
      // Scenario-specific metadata is enough for diffing backend ownership and timing shifts.
      const sample = await runner();
      samples.push({
        iteration: iteration + 1,
        ...sample,
      });
    }
    report.scenarios.push(summarizeScenario(scenario, samples));
  }
  const rendered = JSON.stringify(report, null, 2);
  if (options.output) {
    fs.writeFileSync(path.resolve(options.output), rendered);
    console.log(`Wrote benchmark report to ${path.resolve(options.output)}`);
  } else {
    console.log(rendered);
  }
}

run().catch((error) => {
  console.error(error && error.stack ? error.stack : String(error));
  if (error && error.payload) {
    console.error(JSON.stringify(error.payload, null, 2));
  }
  process.exit(1);
});
