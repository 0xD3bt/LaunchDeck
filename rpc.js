"use strict";

const https = require("https");

const DEFAULT_RPC_URL = "https://api.mainnet-beta.solana.com";

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function getRpcUrl(args = {}) {
  if (args["rpc-url"]) return args["rpc-url"];
  if (process.env.SOLANA_RPC_URL) return process.env.SOLANA_RPC_URL;
  return DEFAULT_RPC_URL;
}

function rpcJson(url, method, params) {
  return new Promise((resolve, reject) => {
    const body = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method,
      params,
    });

    const parsed = new URL(url);
    const req = https.request(
      {
        protocol: parsed.protocol,
        hostname: parsed.hostname,
        port: parsed.port || (parsed.protocol === "https:" ? 443 : 80),
        path: `${parsed.pathname}${parsed.search}`,
        method: "POST",
        headers: {
          "content-type": "application/json",
          "content-length": Buffer.byteLength(body),
        },
      },
      (res) => {
        let data = "";
        res.setEncoding("utf8");
        res.on("data", (chunk) => {
          data += chunk;
        });
        res.on("end", () => {
          if (res.statusCode < 200 || res.statusCode >= 300) {
            reject(new Error(`HTTP ${res.statusCode}: ${data.slice(0, 300)}`));
            return;
          }

          try {
            const parsedJson = JSON.parse(data);
            if (parsedJson.error) {
              reject(new Error(`${parsedJson.error.code}: ${parsedJson.error.message}`));
              return;
            }
            resolve(parsedJson.result);
          } catch (error) {
            reject(new Error(`Failed to parse RPC response: ${error.message}`));
          }
        });
      }
    );

    req.on("error", reject);
    req.write(body);
    req.end();
  });
}

async function rpcJsonWithRetry(url, method, params, maxAttempts = 5) {
  let lastError = null;

  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    try {
      return await rpcJson(url, method, params);
    } catch (error) {
      lastError = error;
      const message = String(error.message || "");
      const isRateLimit =
        message.includes("429") ||
        message.includes("-32429") ||
        message.toLowerCase().includes("rate limit") ||
        message.toLowerCase().includes("too many requests");

      if (!isRateLimit || attempt === maxAttempts) {
        break;
      }

      await sleep(Math.min(500 * 2 ** (attempt - 1), 5000));
    }
  }

  throw lastError;
}

async function getTransaction(rpcUrl, signature) {
  return rpcJsonWithRetry(rpcUrl, "getTransaction", [
    signature,
    {
      encoding: "jsonParsed",
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    },
  ]);
}

async function getAccountInfo(rpcUrl, address) {
  return rpcJsonWithRetry(rpcUrl, "getAccountInfo", [
    address,
    {
      encoding: "jsonParsed",
      commitment: "confirmed",
    },
  ]);
}

async function getSignaturesForAddress(rpcUrl, address, limit = 20) {
  return rpcJsonWithRetry(rpcUrl, "getSignaturesForAddress", [
    address,
    {
      limit,
      commitment: "confirmed",
    },
  ]);
}

module.exports = {
  DEFAULT_RPC_URL,
  getRpcUrl,
  getTransaction,
  getAccountInfo,
  getSignaturesForAddress,
  rpcJsonWithRetry,
};
