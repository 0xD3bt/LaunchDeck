#!/usr/bin/env node
"use strict";

require("dotenv").config({ quiet: true });

const fs = require("fs");
const path = require("path");
const {
  Connection,
  PublicKey,
  VersionedTransaction,
} = require("@solana/web3.js");

const BONK_SUPER_ALT_CONST = "BONK_USD1_SUPER_LOOKUP_TABLE";
const CURRENT_WORKSPACE = process.cwd();
const DEFAULT_LOCAL_DATA_DIR = String(process.env.LAUNCHDECK_LOCAL_DATA_DIR || "").trim()
  || path.join(CURRENT_WORKSPACE, ".local", "launchdeck");
const DEFAULT_BONK_HELPER_CACHE_PATH = String(process.env.LAUNCHDECK_BONK_HELPER_CACHE_PATH || "").trim()
  || path.join(DEFAULT_LOCAL_DATA_DIR, "bonk-helper-cache.json");
const DEFAULT_FOLLOW_DAEMON_STATE_PATH = path.join(DEFAULT_LOCAL_DATA_DIR, "follow-daemon-state.json");

function parseArgs(argv) {
  const options = {
    rpcUrl: String(process.env.SOLANA_RPC_URL || process.env.RPC_URL || process.env.HELIUS_RPC_URL || "").trim(),
    statePath: DEFAULT_FOLLOW_DAEMON_STATE_PATH,
    bonkHelperCachePath: DEFAULT_BONK_HELPER_CACHE_PATH,
    bonkTraceIds: [],
    bagsInputPaths: [],
    reportJson: "",
    reportMd: "",
    extensionJson: "",
    validationMd: "",
    superAltAddress: "",
  };
  for (let index = 0; index < argv.length; index += 1) {
    const entry = argv[index];
    switch (entry) {
      case "--rpc-url":
        options.rpcUrl = String(argv[++index] || "").trim();
        break;
      case "--state-path":
        options.statePath = String(argv[++index] || "").trim() || DEFAULT_FOLLOW_DAEMON_STATE_PATH;
        break;
      case "--bonk-cache-path":
        options.bonkHelperCachePath = String(argv[++index] || "").trim() || DEFAULT_BONK_HELPER_CACHE_PATH;
        break;
      case "--bonk-trace-id":
        options.bonkTraceIds.push(String(argv[++index] || "").trim());
        break;
      case "--bags-input":
        options.bagsInputPaths.push(String(argv[++index] || "").trim());
        break;
      case "--report-json":
        options.reportJson = String(argv[++index] || "").trim();
        break;
      case "--report-md":
        options.reportMd = String(argv[++index] || "").trim();
        break;
      case "--extension-json":
        options.extensionJson = String(argv[++index] || "").trim();
        break;
      case "--validation-md":
        options.validationMd = String(argv[++index] || "").trim();
        break;
      case "--super-alt":
        options.superAltAddress = String(argv[++index] || "").trim();
        break;
      default:
        throw new Error(`Unknown argument: ${entry}`);
    }
  }
  options.bonkTraceIds = options.bonkTraceIds.filter(Boolean);
  options.bagsInputPaths = options.bagsInputPaths.filter(Boolean);
  return options;
}

function readFileSafe(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function projectFile(relativePath) {
  return path.join(CURRENT_WORKSPACE, relativePath);
}

function ensureDirectory(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function writeJson(filePath, value) {
  ensureDirectory(filePath);
  fs.writeFileSync(filePath, JSON.stringify(value, null, 2), "utf8");
}

function writeText(filePath, value) {
  ensureDirectory(filePath);
  fs.writeFileSync(filePath, value, "utf8");
}

function extractRustStringConst(source, constName) {
  const escaped = constName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(new RegExp(`const\\s+${escaped}:\\s+[^=]+?=\\s+"([^"]+)";`));
  return match ? match[1] : "";
}

function extractRustStringArray(source, constName) {
  const escaped = constName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(new RegExp(`const\\s+${escaped}:\\s+[^=]+?=\\s*\\[(.*?)\\];`, "s"));
  if (!match) return [];
  return Array.from(match[1].matchAll(/"([^"]+)"/g)).map((entry) => entry[1]);
}

function extractJsStringConst(source, constName) {
  const escaped = constName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(new RegExp(`const\\s+${escaped}\\s*=\\s*"([^"]+)";`));
  return match ? match[1] : "";
}

function tryPublicKey(value) {
  try {
    return new PublicKey(String(value).trim()).toBase58();
  } catch (_error) {
    return "";
  }
}

function findProgramAddress(seedBuffers, programId) {
  return PublicKey.findProgramAddressSync(seedBuffers, new PublicKey(programId))[0].toBase58();
}

function addCandidate(candidates, address, info = {}) {
  const normalized = tryPublicKey(address);
  if (!normalized) return;
  const existing = candidates.get(normalized) || {
    address: normalized,
    platforms: new Set(),
    categories: new Set(),
    sources: new Set(),
    reasons: new Set(),
    traces: new Set(),
    occurrences: 0,
    providerCompatibility: new Set(),
    notes: new Set(),
    inlineOnly: false,
  };
  if (info.platform) existing.platforms.add(info.platform);
  if (info.category) existing.categories.add(info.category);
  if (info.source) existing.sources.add(info.source);
  if (info.reason) existing.reasons.add(info.reason);
  if (info.traceId) existing.traces.add(info.traceId);
  if (info.providerCompatibility) existing.providerCompatibility.add(info.providerCompatibility);
  if (info.note) existing.notes.add(info.note);
  if (info.inlineOnly) existing.inlineOnly = true;
  existing.occurrences += Number(info.occurrences || 1);
  candidates.set(normalized, existing);
}

function serializeCandidate(entry, currentSuperAltSet) {
  const platforms = Array.from(entry.platforms).sort();
  const categories = Array.from(entry.categories).sort();
  const sources = Array.from(entry.sources).sort();
  const reasons = Array.from(entry.reasons).sort();
  const traces = Array.from(entry.traces).sort();
  const providerCompatibility = Array.from(entry.providerCompatibility).sort();
  const notes = Array.from(entry.notes).sort();
  const sharedAcrossPlatforms = platforms.length > 1;
  const alreadyInSuperAlt = currentSuperAltSet.has(entry.address);
  const candidateScore = (
    (sharedAcrossPlatforms ? 50 : 0)
    + (alreadyInSuperAlt ? 0 : 20)
    + platforms.length * 10
    + Math.min(entry.occurrences, 10)
  );
  return {
    address: entry.address,
    platforms,
    categories,
    sources,
    reasons,
    traces,
    occurrences: entry.occurrences,
    providerCompatibility,
    notes,
    inlineOnly: entry.inlineOnly,
    sharedAcrossPlatforms,
    alreadyInSuperAlt,
    candidateScore,
  };
}

function uniqueStrings(values) {
  return Array.from(new Set((values || []).filter(Boolean)));
}

function loadBonkHelperCache(cachePath) {
  try {
    return JSON.parse(readFileSafe(cachePath));
  } catch (_error) {
    return null;
  }
}

function loadPinnedLookupTableSnapshot(cachePath, address) {
  const cache = loadBonkHelperCache(cachePath);
  const snapshot = cache
    && cache.entries
    && cache.entries["lookup-table-snapshots"]
    && cache.entries["lookup-table-snapshots"][address]
    && cache.entries["lookup-table-snapshots"][address].value;
  if (!snapshot || !snapshot.state || !Array.isArray(snapshot.state.addresses)) {
    return null;
  }
  return snapshot;
}

async function loadLookupTableAccount(connection, cachePath, address) {
  const normalized = tryPublicKey(address);
  if (!normalized) {
    throw new Error(`Invalid lookup table address: ${address}`);
  }
  if (connection) {
    const response = await connection.getAddressLookupTable(new PublicKey(normalized));
    if (response && response.value) {
      return {
        address: normalized,
        source: "rpc",
        addresses: response.value.state.addresses.map((entry) => entry.toBase58()),
      };
    }
  }
  const snapshot = loadPinnedLookupTableSnapshot(cachePath, normalized);
  if (snapshot) {
    return {
      address: normalized,
      source: "cache",
      addresses: snapshot.state.addresses.map((entry) => String(entry)),
    };
  }
  throw new Error(`Lookup table not found via RPC or cache: ${normalized}`);
}

function loadTraceTransactions(statePath, traceId) {
  const payload = JSON.parse(readFileSafe(statePath));
  const jobs = Array.isArray(payload.jobs) ? payload.jobs : [];
  const job = jobs.find((entry) => entry.traceId === traceId);
  if (!job) {
    throw new Error(`Trace id not found in state file: ${traceId}`);
  }
  const transactions = [];
  for (const action of job.actions || []) {
    for (const tx of action.preSignedTransactions || []) {
      if (!tx || !tx.serializedBase64) continue;
      try {
        transactions.push(VersionedTransaction.deserialize(Buffer.from(tx.serializedBase64, "base64")));
      } catch (_error) {
        // Ignore non-versioned transactions for ALT harvesting.
      }
    }
  }
  return transactions;
}

async function collectLookedUpAddressesFromTransactions(connection, cachePath, transactions) {
  const tables = new Map();
  const lookedUpAddresses = new Set();
  const lookupTableAddresses = new Set();
  for (const transaction of transactions) {
    for (const lookup of transaction.message.addressTableLookups || []) {
      const lookupTableAddress = lookup.accountKey.toBase58();
      lookupTableAddresses.add(lookupTableAddress);
      if (!tables.has(lookupTableAddress)) {
        tables.set(
          lookupTableAddress,
          await loadLookupTableAccount(connection, cachePath, lookupTableAddress),
        );
      }
      const table = tables.get(lookupTableAddress);
      const indexes = [
        ...Array.from(lookup.writableIndexes || []),
        ...Array.from(lookup.readonlyIndexes || []),
      ];
      for (const index of indexes) {
        const address = table.addresses[index];
        if (address) {
          lookedUpAddresses.add(address);
        }
      }
    }
  }
  return {
    lookupTableAddresses: Array.from(lookupTableAddresses).sort(),
    lookedUpAddresses: Array.from(lookedUpAddresses).sort(),
  };
}

function collectSerializedTransactions(value, sink = []) {
  if (!value) return sink;
  if (Array.isArray(value)) {
    value.forEach((entry) => collectSerializedTransactions(entry, sink));
    return sink;
  }
  if (typeof value !== "object") {
    return sink;
  }
  if (typeof value.serializedBase64 === "string" && value.serializedBase64.trim()) {
    sink.push(value.serializedBase64.trim());
  }
  Object.values(value).forEach((entry) => collectSerializedTransactions(entry, sink));
  return sink;
}

function collectLookupTablesUsed(value, sink = []) {
  if (!value) return sink;
  if (Array.isArray(value)) {
    value.forEach((entry) => collectLookupTablesUsed(entry, sink));
    return sink;
  }
  if (typeof value !== "object") {
    return sink;
  }
  if (Array.isArray(value.lookupTablesUsed)) {
    for (const entry of value.lookupTablesUsed) {
      if (typeof entry === "string" && entry.trim()) {
        sink.push(entry.trim());
      }
    }
  }
  Object.values(value).forEach((entry) => collectLookupTablesUsed(entry, sink));
  return sink;
}

function deserializeVersionedTransactions(serializedTransactions) {
  return serializedTransactions
    .map((serialized) => {
      try {
        return VersionedTransaction.deserialize(Buffer.from(serialized, "base64"));
      } catch (_error) {
        return null;
      }
    })
    .filter(Boolean);
}

function collectPumpStaticAddresses() {
  const pumpSource = readFileSafe(projectFile("rust/launchdeck-engine/src/pump_native.rs"));
  const pumpProgram = extractRustStringConst(pumpSource, "PUMP_PROGRAM_ID");
  const pumpAmmProgram = extractRustStringConst(pumpSource, "PUMP_AMM_PROGRAM_ID");
  const mayhemProgram = extractRustStringConst(pumpSource, "MAYHEM_PROGRAM_ID");
  const pumpFeeProgram = extractRustStringConst(pumpSource, "PUMP_FEE_PROGRAM_ID");
  const pumpAgentProgram = extractRustStringConst(pumpSource, "PUMP_AGENT_PAYMENTS_PROGRAM_ID");
  const tokenProgram = extractRustStringConst(pumpSource, "TOKEN_PROGRAM_ID");
  const token2022Program = extractRustStringConst(pumpSource, "TOKEN_2022_PROGRAM_ID");
  const computeBudgetProgram = extractRustStringConst(pumpSource, "COMPUTE_BUDGET_PROGRAM_ID");
  const wsolMint = extractRustStringConst(pumpSource, "WSOL_MINT");
  const usdcMint = extractRustStringConst(pumpSource, "USDC_MINT");
  const usdtMint = extractRustStringConst(pumpSource, "USDT_MINT");
  const usd1Mint = extractRustStringConst(pumpSource, "USD1_MINT");
  const jitodontfront = extractRustStringConst(pumpSource, "JITODONTFRONT_ACCOUNT");
  const defaultLookupTables = extractRustStringArray(pumpSource, "DEFAULT_LOOKUP_TABLES");
  const stableAddresses = [
    pumpProgram,
    pumpAmmProgram,
    mayhemProgram,
    pumpFeeProgram,
    pumpAgentProgram,
    tokenProgram,
    token2022Program,
    computeBudgetProgram,
    wsolMint,
    usdcMint,
    usdtMint,
    usd1Mint,
    jitodontfront,
  ].filter(Boolean);
  const derivedAddresses = uniqueStrings([
    findProgramAddress([Buffer.from("__event_authority")], pumpProgram),
    findProgramAddress([Buffer.from("__event_authority")], pumpAmmProgram),
    findProgramAddress([Buffer.from("__event_authority")], mayhemProgram),
    findProgramAddress([Buffer.from("__event_authority")], pumpFeeProgram),
    findProgramAddress([Buffer.from("__event_authority")], pumpAgentProgram),
    findProgramAddress([Buffer.from("mint-authority")], pumpProgram),
    findProgramAddress([Buffer.from("global")], pumpProgram),
    findProgramAddress([Buffer.from("global-params")], mayhemProgram),
    findProgramAddress([Buffer.from("global_volume_accumulator")], pumpProgram),
    findProgramAddress([Buffer.from("fee_config"), new PublicKey(pumpProgram).toBuffer()], pumpFeeProgram),
    findProgramAddress([Buffer.from("fee-program-global")], pumpFeeProgram),
    findProgramAddress([Buffer.from("global-config")], pumpAgentProgram),
    findProgramAddress([Buffer.from("sol-vault")], mayhemProgram),
  ]);
  return {
    defaultLookupTables,
    staticAddresses: stableAddresses,
    derivedAddresses,
  };
}

function collectProviderStaticAddresses() {
  const mainSource = readFileSafe(projectFile("rust/launchdeck-engine/src/main.rs"));
  const uiBridgeSource = readFileSafe(projectFile("rust/launchdeck-engine/src/ui_bridge.rs"));
  const heliusSenderTips = uniqueStrings([
    ...extractRustStringArray(mainSource, "HELIUS_SENDER_TIP_ACCOUNTS"),
    ...extractRustStringArray(uiBridgeSource, "HELIUS_SENDER_TIP_ACCOUNTS"),
  ]);
  const jitoTips = uniqueStrings([
    ...extractRustStringArray(mainSource, "JITO_TIP_ACCOUNTS"),
    ...extractRustStringArray(uiBridgeSource, "JITO_TIP_ACCOUNTS"),
  ]);
  const helloMoonTips = uniqueStrings([
    ...extractRustStringArray(mainSource, "HELLOMOON_TIP_ACCOUNTS"),
    ...extractRustStringArray(uiBridgeSource, "HELLOMOON_TIP_ACCOUNTS"),
  ]);
  return {
    hellomoon: {
      status: "candidate-alt-safe-needs-validation",
      rationale: "Static Hello Moon tip accounts are selected from constant lists and are strong shared ALT candidates.",
      addresses: helloMoonTips,
    },
    jito: {
      status: "candidate-alt-safe-needs-validation",
      rationale: "Static Jito tip accounts are selected from constant lists and are strong shared ALT candidates.",
      addresses: jitoTips,
    },
    helius: {
      status: "keep-inline-until-validated",
      rationale: "Helius Sender tip accounts are static, but transport behavior may depend on inline visibility and should be validated before ALT promotion.",
      addresses: heliusSenderTips,
    },
  };
}

function collectBagsStaticAddresses() {
  const bagsSource = readFileSafe(projectFile("rust/launchdeck-engine/src/bags_native.rs"));
  const defaultBagsWallet = extractRustStringConst(bagsSource, "DEFAULT_BAGS_WALLET");
  const defaultBagsConfig = extractRustStringConst(bagsSource, "DEFAULT_BAGS_CONFIG");
  const wrappedSolMintMatch = extractRustStringConst(bagsSource, "WRAPPED_SOL_MINT");
  return uniqueStrings([
    defaultBagsWallet,
    defaultBagsConfig,
    wrappedSolMintMatch,
  ]);
}

function buildProviderValidationChecklist(providerMatrix) {
  return [
    "# Shared ALT Provider Validation Checklist",
    "",
    "## Goal",
    "Validate whether provider-specific static infrastructure addresses remain compatible when loaded via ALT instead of being included inline.",
    "",
    "## Providers",
    ...Object.entries(providerMatrix).flatMap(([provider, entry]) => [
      `### ${provider}`,
      `- Default policy: \`${entry.status}\``,
      `- Rationale: ${entry.rationale}`,
      `- Static addresses: ${entry.addresses.length}`,
      "- Validation steps:",
      "1. Submit a control transaction with the provider-static address inline.",
      "2. Submit the same transaction shape with the provider-static address supplied via ALT.",
      "3. Compare provider acceptance, returned signature/bundle id, confirmation behavior, and any endpoint-specific errors.",
      "4. If ALT-loaded behavior differs from inline behavior, keep that address inline.",
      "",
    ]),
  ].join("\n");
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const connection = options.rpcUrl ? new Connection(options.rpcUrl, "confirmed") : null;
  const bonkLaunchSource = readFileSafe(projectFile("scripts/bonk-launchpad.js"));
  const superAltAddress = options.superAltAddress
    || extractJsStringConst(bonkLaunchSource, BONK_SUPER_ALT_CONST);
  if (!superAltAddress) {
    throw new Error("Could not resolve the current shared super ALT address.");
  }

  const currentSuperAlt = await loadLookupTableAccount(connection, options.bonkHelperCachePath, superAltAddress);
  const currentSuperAltSet = new Set(currentSuperAlt.addresses);

  const pumpCandidates = collectPumpStaticAddresses();
  const providerMatrix = collectProviderStaticAddresses();
  const bagsStaticAddresses = collectBagsStaticAddresses();
  const candidates = new Map();

  for (const address of currentSuperAlt.addresses) {
    addCandidate(candidates, address, {
      platform: "bonk",
      category: "current-super-alt",
      source: `super-alt:${superAltAddress}`,
      reason: "Already present in shared super ALT",
    });
  }

  for (const lookupTableAddress of pumpCandidates.defaultLookupTables) {
    addCandidate(candidates, lookupTableAddress, {
      platform: "pump",
      category: "pump-default-lookup-table",
      source: "pump-default-lookup-table",
      reason: "Pump default lookup table address",
    });
    try {
      const table = await loadLookupTableAccount(connection, options.bonkHelperCachePath, lookupTableAddress);
      for (const address of table.addresses) {
        addCandidate(candidates, address, {
          platform: "pump",
          category: "pump-default-lut-address",
          source: `lookup-table:${lookupTableAddress}`,
          reason: "Address present in Pump default lookup table",
        });
      }
    } catch (error) {
      addCandidate(candidates, lookupTableAddress, {
        platform: "pump",
        category: "pump-default-lookup-table-unresolved",
        source: "pump-default-lookup-table",
        note: `Failed to expand lookup table via RPC/cache: ${error.message}`,
      });
    }
  }

  for (const address of pumpCandidates.staticAddresses) {
    addCandidate(candidates, address, {
      platform: "pump",
      category: "pump-static-address",
      source: "pump_native.rs constants",
      reason: "Static Pump program or mint address",
    });
  }
  for (const address of pumpCandidates.derivedAddresses) {
    addCandidate(candidates, address, {
      platform: "pump",
      category: "pump-stable-pda",
      source: "pump_native.rs derived PDA",
      reason: "Stable Pump PDA derived from static seeds",
    });
  }

  const bonkTraceSummaries = [];
  for (const traceId of options.bonkTraceIds) {
    const transactions = loadTraceTransactions(options.statePath, traceId);
    const traceData = await collectLookedUpAddressesFromTransactions(
      connection,
      options.bonkHelperCachePath,
      transactions,
    );
    bonkTraceSummaries.push({
      traceId,
      transactionCount: transactions.length,
      lookupTableAddresses: traceData.lookupTableAddresses,
      lookedUpAddressCount: traceData.lookedUpAddresses.length,
    });
    for (const address of traceData.lookedUpAddresses) {
      addCandidate(candidates, address, {
        platform: "bonk",
        category: "bonk-trace-looked-up-address",
        source: `bonk-trace:${traceId}`,
        traceId,
        reason: "Observed in Bonk trace lookup expansion",
      });
    }
  }

  const bagsInputSummaries = [];
  for (const inputPath of options.bagsInputPaths) {
    const raw = JSON.parse(readFileSafe(path.resolve(CURRENT_WORKSPACE, inputPath)));
    const serializedTransactions = uniqueStrings(collectSerializedTransactions(raw));
    const lookupTablesUsed = uniqueStrings(collectLookupTablesUsed(raw));
    const transactions = deserializeVersionedTransactions(serializedTransactions);
    let lookedUpAddresses = [];
    if (transactions.length > 0) {
      try {
        const traceData = await collectLookedUpAddressesFromTransactions(
          connection,
          options.bonkHelperCachePath,
          transactions,
        );
        lookedUpAddresses = traceData.lookedUpAddresses;
        for (const address of traceData.lookedUpAddresses) {
          addCandidate(candidates, address, {
            platform: "bags",
            category: "bags-observed-looked-up-address",
            source: `bags-input:${inputPath}`,
            reason: "Observed in Bags serialized transaction lookup expansion",
          });
        }
      } catch (error) {
        bagsInputSummaries.push({
          inputPath,
          serializedTransactionCount: transactions.length,
          lookupTablesUsed,
          error: error.message,
        });
        continue;
      }
    }
    bagsInputSummaries.push({
      inputPath,
      serializedTransactionCount: transactions.length,
      lookupTablesUsed,
      lookedUpAddressCount: lookedUpAddresses.length,
    });
  }

  for (const address of bagsStaticAddresses) {
    addCandidate(candidates, address, {
      platform: "bags",
      category: "bags-static-address",
      source: "bags_native.rs constants",
      reason: "Static Bags launch configuration address",
    });
  }

  Object.entries(providerMatrix).forEach(([provider, entry]) => {
    entry.addresses.forEach((address) => {
      addCandidate(candidates, address, {
        platform: "provider-static",
        category: "provider-static-address",
        source: `${provider}-tip-accounts`,
        reason: `${provider} static infrastructure address`,
        providerCompatibility: entry.status,
        inlineOnly: entry.status === "keep-inline-until-validated",
        note: entry.rationale,
      });
    });
  });

  const serializedCandidates = Array.from(candidates.values())
    .map((entry) => serializeCandidate(entry, currentSuperAltSet))
    .sort((left, right) => (
      right.candidateScore - left.candidateScore
      || left.address.localeCompare(right.address)
    ));

  const extensionCandidates = serializedCandidates.filter((entry) => !entry.alreadyInSuperAlt && !entry.inlineOnly);
  const extensionSet = extensionCandidates.map((entry) => ({
    address: entry.address,
    platforms: entry.platforms,
    categories: entry.categories,
    reasons: entry.reasons,
    candidateScore: entry.candidateScore,
  }));

  const report = {
    generatedAt: new Date().toISOString(),
    rpcUrl: options.rpcUrl || null,
    currentSuperAlt: {
      address: superAltAddress,
      source: currentSuperAlt.source,
      addressCount: currentSuperAlt.addresses.length,
      addresses: currentSuperAlt.addresses,
    },
    providerCompatibility: providerMatrix,
    pump: {
      defaultLookupTables: pumpCandidates.defaultLookupTables,
      staticAddresses: pumpCandidates.staticAddresses,
      derivedAddresses: pumpCandidates.derivedAddresses,
    },
    bonk: {
      traceSummaries: bonkTraceSummaries,
    },
    bags: {
      staticAddresses: bagsStaticAddresses,
      inputSummaries: bagsInputSummaries,
    },
    merged: {
      candidateCount: serializedCandidates.length,
      currentSharedAddressCount: currentSuperAlt.addresses.length,
      projectedSharedAddressCount: currentSuperAlt.addresses.length + extensionSet.length,
      remainingCapacityAfterExtension: 256 - (currentSuperAlt.addresses.length + extensionSet.length),
      candidates: serializedCandidates,
      extensionSet,
    },
  };

  const reportMarkdown = [
    "# Shared ALT Harvest Report",
    "",
    `- Generated at: ${report.generatedAt}`,
    `- Current shared super ALT: \`${superAltAddress}\``,
    `- Current address count: ${currentSuperAlt.addresses.length}`,
    `- Candidate count: ${serializedCandidates.length}`,
    `- Extension candidates: ${extensionSet.length}`,
    `- Projected address count after extension: ${report.merged.projectedSharedAddressCount}`,
    `- Remaining capacity after extension: ${report.merged.remainingCapacityAfterExtension}`,
    "",
    "## Provider Compatibility",
    ...Object.entries(providerMatrix).flatMap(([provider, entry]) => [
      `### ${provider}`,
      `- Status: \`${entry.status}\``,
      `- Address count: ${entry.addresses.length}`,
      `- Rationale: ${entry.rationale}`,
      "",
    ]),
    "## Top Extension Candidates",
    ...extensionSet.slice(0, 25).map((entry) => (
      `- \`${entry.address}\` (${entry.platforms.join(", ")}) score=${entry.candidateScore} categories=${entry.categories.join(", ")}`
    )),
    "",
  ].join("\n");

  const validationChecklist = buildProviderValidationChecklist(providerMatrix);

  if (options.reportJson) {
    writeJson(path.resolve(CURRENT_WORKSPACE, options.reportJson), report);
  }
  if (options.reportMd) {
    writeText(path.resolve(CURRENT_WORKSPACE, options.reportMd), reportMarkdown);
  }
  if (options.extensionJson) {
    writeJson(path.resolve(CURRENT_WORKSPACE, options.extensionJson), {
      sharedSuperAlt: superAltAddress,
      currentAddressCount: currentSuperAlt.addresses.length,
      extensionCandidates: extensionSet,
    });
  }
  if (options.validationMd) {
    writeText(path.resolve(CURRENT_WORKSPACE, options.validationMd), validationChecklist);
  }

  if (!options.reportJson && !options.reportMd && !options.extensionJson && !options.validationMd) {
    process.stdout.write(`${reportMarkdown}\n`);
  }
}

main().catch((error) => {
  console.error(error && error.stack ? error.stack : String(error));
  process.exitCode = 1;
});
