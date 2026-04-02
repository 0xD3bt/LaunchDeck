"use strict";

require("dotenv").config({ quiet: true });

const fs = require("fs");
const path = require("path");
const readline = require("readline");
const bs58 = require("bs58");
const BN = require("bn.js");
const {
  BagsSDK,
  BAGS_FEE_SHARE_V2_PROGRAM_ID,
  METEORA_DAMM_V2_PROGRAM_ID,
  METEORA_DBC_PROGRAM_ID,
  WRAPPED_SOL_MINT,
} = require("@bagsfm/bags-sdk");
const {
  BaseFeeMode,
  CollectFeeMode,
  swapQuote,
  swapQuoteExactOut,
} = require("@meteora-ag/dynamic-bonding-curve-sdk");
const {
  Connection,
  Keypair,
  PublicKey,
  SystemInstruction,
  SystemProgram,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
} = require("@solana/web3.js");
const {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} = require("@solana/spl-token");

const DEFAULT_BAGS_WALLET = new PublicKey("3muhBpbVeoDy4fBrC1SWnfkUooy2Pn6woV1GxDUhESfC");
const DEFAULT_BAGS_CONFIG = new PublicKey("AxpMibQQBqVbQF7EzBUeCbpxRkuk6yfTWRLGVLh5qrce");
const DEFAULT_TOTAL_SUPPLY = 1_000_000_000n;
const BAGS_TOTAL_SUPPLY = 1_000_000_000n * 10n ** 9n;
const BAGS_INITIAL_SQRT_PRICE = new BN("3141367320245630");
const BAGS_MIGRATION_QUOTE_THRESHOLD = new BN("85000000000");
const BAGS_CURVE = [
  {
    sqrtPrice: new BN("6401204812200420"),
    liquidity: new BN("3929368168768468756200000000000000"),
  },
  {
    sqrtPrice: new BN("13043817825332782"),
    liquidity: new BN("2425988008058820449100000000000000"),
  },
];
const APP_DATA_DIR = path.join(process.cwd(), ".local", "launchdeck");
const BAGS_CREDENTIALS_PATH = path.join(APP_DATA_DIR, "bags-credentials.json");
const BAGS_SESSION_PATH = path.join(APP_DATA_DIR, "bags-session.json");

function readJsonFile(filePath) {
  try {
    if (!fs.existsSync(filePath)) return {};
    const raw = fs.readFileSync(filePath, "utf8").trim();
    return raw ? JSON.parse(raw) : {};
  } catch (_error) {
    return {};
  }
}

function readStoredBagsCredentials() {
  const persisted = readJsonFile(BAGS_CREDENTIALS_PATH);
  const session = readJsonFile(BAGS_SESSION_PATH);
  return {
    apiKey: String(session.apiKey || persisted.apiKey || process.env.BAGS_API_KEY || "").trim(),
    authToken: String(session.authToken || persisted.authToken || "").trim(),
    agentUsername: String(session.agentUsername || persisted.agentUsername || "").trim(),
    verifiedWallet: String(session.verifiedWallet || persisted.verifiedWallet || "").trim(),
  };
}

function requireApiKey(request) {
  const stored = readStoredBagsCredentials();
  const apiKey = String(request.apiKey || stored.apiKey || "").trim();
  if (!apiKey) {
    throw new Error("BAGS_API_KEY is required for Bagsapp integration.");
  }
  return apiKey;
}

function parseSecretBytes(secret) {
  const value = String(secret || "").trim();
  if (!value) throw new Error("Wallet secret was empty.");
  if (value.startsWith("[")) {
    const parsed = JSON.parse(value);
    if (!Array.isArray(parsed)) {
      throw new Error("Wallet secret JSON must be an array of bytes.");
    }
    return Uint8Array.from(parsed);
  }
  try {
    return Uint8Array.from(bs58.decode(value));
  } catch (_error) {
    return Uint8Array.from(Buffer.from(value, "base64"));
  }
}

function parseKeypair(secret) {
  return Keypair.fromSecretKey(parseSecretBytes(secret));
}

function readTransactionBlockhash(transaction) {
  if (transaction instanceof VersionedTransaction) {
    return transaction.message.recentBlockhash;
  }
  return transaction.recentBlockhash || "";
}

function serializeTransaction(transaction) {
  return Buffer.from(transaction.serialize()).toString("base64");
}

function signTransaction(transaction, signer) {
  if (transaction instanceof VersionedTransaction) {
    transaction.sign([signer]);
    return transaction;
  }
  if (transaction instanceof Transaction) {
    transaction.sign(signer);
    return transaction;
  }
  if (typeof transaction.sign === "function") {
    transaction.sign([signer]);
    return transaction;
  }
  throw new Error("Unsupported Bags transaction type for signing.");
}

function signTransactions(transactions, signer) {
  return (Array.isArray(transactions) ? transactions : []).map((transaction) =>
    signTransaction(transaction, signer)
  );
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function normalizeTransactions(transactions, {
  labelPrefix,
  computeUnitLimit = null,
  computeUnitPriceMicroLamports = null,
  inlineTipLamports = null,
  inlineTipAccount = null,
  lastValidBlockHeight,
}) {
  return transactions.map((transaction, index) => ({
    label: transactions.length === 1 ? labelPrefix : `${labelPrefix}-${index + 1}`,
    format: transaction instanceof VersionedTransaction ? "v0" : "legacy",
    blockhash: readTransactionBlockhash(transaction),
    lastValidBlockHeight,
    serializedBase64: serializeTransaction(transaction),
    lookupTablesUsed: [],
    computeUnitLimit,
    computeUnitPriceMicroLamports,
    inlineTipLamports,
    inlineTipAccount: inlineTipLamports && inlineTipAccount ? inlineTipAccount : null,
  }));
}

async function resolveLookupTableAccounts(connection, transaction) {
  if (!(transaction instanceof VersionedTransaction)) {
    return [];
  }
  const lookups = transaction.message.addressTableLookups || [];
  const resolved = await Promise.all(lookups.map(async (lookup) => {
    const response = await connection.getAddressLookupTable(lookup.accountKey);
    if (!response || !response.value) {
      throw new Error(`Address lookup table not found: ${lookup.accountKey.toBase58()}`);
    }
    return response.value;
  }));
  return resolved;
}

async function decompileTransactionInstructions(connection, transaction) {
  if (transaction instanceof VersionedTransaction) {
    const addressLookupTableAccounts = await resolveLookupTableAccounts(connection, transaction);
    const message = TransactionMessage.decompile(transaction.message, { addressLookupTableAccounts });
    return {
      instructions: message.instructions,
      addressLookupTableAccounts,
    };
  }
  return {
    instructions: transaction.instructions || [],
    addressLookupTableAccounts: [],
  };
}

function isInlineTipInstruction(instruction, ownerPubkey, tipAccount, tipLamports) {
  if (!tipAccount || !tipLamports) return false;
  if (!instruction.programId || !instruction.programId.equals(SystemProgram.programId)) {
    return false;
  }
  try {
    if (SystemInstruction.decodeInstructionType(instruction) !== "Transfer") {
      return false;
    }
    const decoded = SystemInstruction.decodeTransfer(instruction);
    return decoded.fromPubkey.equals(ownerPubkey)
      && decoded.toPubkey.equals(new PublicKey(tipAccount))
      && Number(decoded.lamports) === Number(tipLamports);
  } catch (_error) {
    return false;
  }
}

function buildInlineTipInstruction(ownerPubkey, tipAccount, tipLamports) {
  if (!tipAccount || !tipLamports) return null;
  return SystemProgram.transfer({
    fromPubkey: ownerPubkey,
    toPubkey: new PublicKey(tipAccount),
    lamports: Number(tipLamports),
  });
}

async function ensureInlineTipOnTransaction(connection, owner, transaction, txConfig) {
  const tipInstruction = buildInlineTipInstruction(
    owner.publicKey,
    txConfig && txConfig.tipAccount,
    txConfig && txConfig.tipLamports,
  );
  if (!tipInstruction) {
    return signTransaction(transaction, owner);
  }
  if (transaction instanceof VersionedTransaction) {
    const { instructions, addressLookupTableAccounts } = await decompileTransactionInstructions(connection, transaction);
    if (instructions.some((instruction) => (
      isInlineTipInstruction(
        instruction,
        owner.publicKey,
        txConfig && txConfig.tipAccount,
        txConfig && txConfig.tipLamports,
      )
    ))) {
      return signTransaction(transaction, owner);
    }
    const rebuilt = new VersionedTransaction(
      new TransactionMessage({
        payerKey: owner.publicKey,
        recentBlockhash: readTransactionBlockhash(transaction),
        instructions: [...instructions, tipInstruction],
      }).compileToV0Message(addressLookupTableAccounts),
    );
    rebuilt.sign([owner]);
    return rebuilt;
  }
  const instructions = transaction.instructions || [];
  if (instructions.some((instruction) => (
    isInlineTipInstruction(
      instruction,
      owner.publicKey,
      txConfig && txConfig.tipAccount,
      txConfig && txConfig.tipLamports,
    )
  ))) {
    return signTransaction(transaction, owner);
  }
  const rebuilt = new Transaction();
  rebuilt.feePayer = owner.publicKey;
  rebuilt.recentBlockhash = readTransactionBlockhash(transaction);
  instructions.forEach((instruction) => rebuilt.add(instruction));
  rebuilt.add(tipInstruction);
  rebuilt.sign(owner);
  return rebuilt;
}

function parseDecimalToBigInt(raw, decimals, label) {
  const value = String(raw || "").trim();
  if (!value) throw new Error(`${label} is required.`);
  if (!/^\d+(\.\d+)?$/.test(value)) {
    throw new Error(`Invalid ${label}: ${value}`);
  }
  const [wholePart, fractionPart = ""] = value.split(".");
  const paddedFraction = `${fractionPart}${"0".repeat(decimals)}`.slice(0, decimals);
  return BigInt(wholePart) * (10n ** BigInt(decimals)) + BigInt(paddedFraction || "0");
}

function formatDecimal(value, decimals, precision = 6) {
  const divisor = 10n ** BigInt(decimals);
  const whole = value / divisor;
  let fraction = (value % divisor).toString().padStart(decimals, "0").slice(0, precision);
  fraction = fraction.replace(/0+$/, "");
  return fraction ? `${whole}.${fraction}` : whole.toString();
}

function formatSupplyPercent(valueBaseUnits) {
  const raw = BigInt(String(valueBaseUnits || 0));
  if (raw <= 0n) return "0";
  const scaled = (raw * 1_000_000n) / BAGS_TOTAL_SUPPLY;
  const whole = scaled / 10_000n;
  const fraction = scaled % 10_000n;
  if (fraction === 0n) return whole.toString();
  return `${whole}.${fraction.toString().padStart(4, "0").replace(/0+$/, "")}`;
}

function slippageModeFromRequest(request) {
  const slippageBps = Number(request.slippageBps || 0);
  if (Number.isFinite(slippageBps) && slippageBps > 0) {
    return { slippageMode: "manual", slippageBps };
  }
  return { slippageMode: "auto" };
}

function normalizeLamportsValue(raw) {
  const numeric = Number(raw);
  if (!Number.isFinite(numeric) || numeric <= 0) return 0;
  return numeric < 1 ? Math.round(numeric * 1_000_000_000) : Math.round(numeric);
}

function normalizePercentileKey(raw) {
  const value = String(raw || "p75").trim().toLowerCase();
  switch (value) {
    case "p25":
    case "25":
    case "25th":
      return "p25";
    case "p50":
    case "50":
    case "median":
      return "p50";
    case "p75":
    case "75":
    case "75th":
      return "p75";
    case "p95":
    case "95":
    case "95th":
      return "p95";
    case "p99":
    case "99":
    case "99th":
      return "p99";
    default:
      return "p75";
  }
}

function firstFiniteNumber(...values) {
  for (const value of values) {
    const numeric = Number(value);
    if (Number.isFinite(numeric)) return numeric;
  }
  return null;
}

function extractJitoPercentiles(rawPayload) {
  const sample = Array.isArray(rawPayload)
    ? rawPayload[0]
    : Array.isArray(rawPayload && rawPayload.value)
      ? rawPayload.value[0]
      : rawPayload;
  if (!sample || typeof sample !== "object") {
    return { p25: 0, p50: 0, p75: 0, p95: 0, p99: 0 };
  }
  return {
    p25: normalizeLamportsValue(firstFiniteNumber(
      sample.p25,
      sample.percentile25,
      sample.tipFloor25,
      sample.landed_tips_25th_percentile,
    )),
    p50: normalizeLamportsValue(firstFiniteNumber(
      sample.p50,
      sample.percentile50,
      sample.median,
      sample.landed_tips_50th_percentile,
    )),
    p75: normalizeLamportsValue(firstFiniteNumber(
      sample.p75,
      sample.percentile75,
      sample.tipFloor75,
      sample.landed_tips_75th_percentile,
    )),
    p95: normalizeLamportsValue(firstFiniteNumber(
      sample.p95,
      sample.percentile95,
      sample.tipFloor95,
      sample.landed_tips_95th_percentile,
    )),
    p99: normalizeLamportsValue(firstFiniteNumber(
      sample.p99,
      sample.percentile99,
      sample.tipFloor99,
      sample.landed_tips_99th_percentile,
    )),
  };
}

function extractHeliusPriorityEstimate(rawPayload) {
  const result = rawPayload && typeof rawPayload === "object" && rawPayload.result
    ? rawPayload.result
    : rawPayload;
  const levels = result && typeof result === "object" && result.priorityFeeLevels
    ? result.priorityFeeLevels
    : {};
  return {
    recommended: normalizeLamportsValue(
      firstFiniteNumber(result && result.priorityFeeEstimate, result && result.recommended)
    ),
    levels: {
      none: normalizeLamportsValue(firstFiniteNumber(levels.none, levels.min)),
      low: normalizeLamportsValue(firstFiniteNumber(levels.low)),
      medium: normalizeLamportsValue(firstFiniteNumber(levels.medium)),
      high: normalizeLamportsValue(firstFiniteNumber(levels.high)),
      veryHigh: normalizeLamportsValue(firstFiniteNumber(levels.veryHigh)),
      unsafeMax: normalizeLamportsValue(firstFiniteNumber(levels.unsafeMax, levels.max)),
    },
  };
}

async function fetchHeliusPriorityEstimate(rpcUrl) {
  const heliusPriorityLevel = String(process.env.LAUNCHDECK_AUTO_FEE_HELIUS_PRIORITY_LEVEL || "veryHigh")
    .trim()
    .toLowerCase();
  const options = heliusPriorityLevel === "recommended"
    ? { recommended: true }
    : { includeAllPriorityFeeLevels: true };
  const response = await fetch(String(rpcUrl || "").trim(), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: "launchdeck-helius-priority-estimate",
      method: "getPriorityFeeEstimate",
      params: [
        {
          options,
        },
      ],
    }),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(`Helius priority estimate request failed with status ${response.status}.`);
  }
  if (payload && payload.error) {
    throw new Error(
      `Helius priority estimate failed: ${payload.error.message || JSON.stringify(payload.error)}`
    );
  }
  return {
    raw: payload,
    normalized: extractHeliusPriorityEstimate(payload),
  };
}

async function estimateFees(request) {
  const apiKey = requireApiKey(request);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const requestedTipLamports = Math.max(0, Number(request.requestedTipLamports || 0));
  const tipPolicy = request.tipPolicy || {};
  const setupJitoTipCapLamports = Math.max(0, Number(tipPolicy.setupJitoTipCapLamports || 0));
  const setupJitoTipMinLamports = Math.max(0, Number(tipPolicy.setupJitoTipMinLamports || 0));
  const setupJitoTipPercentile = normalizePercentileKey(tipPolicy.setupJitoTipPercentile);
  const warnings = [];
  const [heliusResult, jitoResult] = await Promise.allSettled([
    fetchHeliusPriorityEstimate(request.rpcUrl),
    sdk.solana.getJitoRecentFees(),
  ]);

  let helius = {
    raw: null,
    normalized: {
      recommended: 0,
      levels: { none: 0, low: 0, medium: 0, high: 0, veryHigh: 0, unsafeMax: 0 },
    },
    error: null,
  };
  if (heliusResult.status === "fulfilled") {
    helius = {
      raw: heliusResult.value.raw,
      normalized: heliusResult.value.normalized,
      error: null,
    };
  } else {
    helius.error = String(heliusResult.reason && heliusResult.reason.message || heliusResult.reason || "");
    if (helius.error) warnings.push(`Helius priority estimate unavailable: ${helius.error}`);
  }

  let jito = {
    raw: null,
    normalized: { p25: 0, p50: 0, p75: 0, p95: 0, p99: 0 },
    error: null,
  };
  if (jitoResult.status === "fulfilled") {
    jito = {
      raw: jitoResult.value,
      normalized: extractJitoPercentiles(jitoResult.value),
      error: null,
    };
  } else {
    jito.error = String(jitoResult.reason && jitoResult.reason.message || jitoResult.reason || "");
    if (jito.error) warnings.push(`Jito recent fee estimate unavailable: ${jito.error}`);
  }

  const estimatedJitoTipLamports = jito.normalized[setupJitoTipPercentile] || 0;
  let setupJitoTipLamports = estimatedJitoTipLamports;
  let setupJitoTipSource = "jito-recent-fees";
  if (setupJitoTipLamports <= 0 && requestedTipLamports > 0) {
    setupJitoTipLamports = requestedTipLamports;
    setupJitoTipSource = "user-requested-fallback";
  }
  if (setupJitoTipLamports > 0) {
    setupJitoTipLamports = Math.max(setupJitoTipLamports, setupJitoTipMinLamports);
  }
  if (setupJitoTipCapLamports > 0) {
    setupJitoTipLamports = Math.min(setupJitoTipLamports, setupJitoTipCapLamports);
  }
  if (setupJitoTipLamports <= 0) {
    setupJitoTipSource = "none";
  }

  return {
    helius,
    jito,
    setupJitoTipLamports,
    setupJitoTipSource,
    setupJitoTipPercentile,
    setupJitoTipCapLamports,
    setupJitoTipMinLamports,
    warnings,
  };
}

function bagsConfigTypeForMode(mode) {
  switch (String(mode || "").trim().toLowerCase()) {
    case "bags-025-1":
      return "d16d3585-6488-4a6c-9a6f-e6c39ca0fda3";
    case "bags-1-025":
      return "a7c8e1f2-3d4b-5a6c-9e0f-1b2c3d4e5f6a";
    default:
      return "fa29606e-5e48-4c37-827f-4b03d58ee23d";
  }
}

function bagsModeForPostMigrationFeeBps(feeBps) {
  switch (Number(feeBps || 0)) {
    case 200:
      return "bags-2-2";
    case 100:
      return "bags-025-1";
    case 25:
      return "bags-1-025";
    default:
      return "";
  }
}

function bagsModeForPrePostFees(preFeePercent, postFeePercent) {
  const pre = Number(preFeePercent || 0);
  const post = Number(postFeePercent || 0);
  if (pre === 2 && post === 2) return "bags-2-2";
  if (pre === 0.25 && post === 1) return "bags-025-1";
  if (pre === 1 && post === 0.25) return "bags-1-025";
  return "";
}

function bagsPreMigrationFeeBpsForMode(mode) {
  switch (String(mode || "").trim().toLowerCase()) {
    case "bags-025-1":
      return 25;
    case "bags-1-025":
      return 100;
    case "bags-2-2":
    case "":
      return 200;
    default:
      return 200;
  }
}

function bagsCliffFeeNumeratorForMode(mode) {
  return Math.round((bagsPreMigrationFeeBpsForMode(mode) / 10000) * 1_000_000_000);
}

function buildBagsInitialBuyVirtualPool() {
  return {
    quoteReserve: new BN(0),
    sqrtPrice: BAGS_INITIAL_SQRT_PRICE,
    activationPoint: new BN(0),
    volatilityTracker: {
      volatilityAccumulator: new BN(0),
    },
  };
}

function buildBagsInitialBuyConfig(mode) {
  return {
    collectFeeMode: CollectFeeMode.QuoteToken,
    migrationQuoteThreshold: BAGS_MIGRATION_QUOTE_THRESHOLD,
    poolFees: {
      baseFee: {
        cliffFeeNumerator: new BN(String(bagsCliffFeeNumeratorForMode(mode))),
        firstFactor: 0,
        secondFactor: new BN(0),
        thirdFactor: new BN(0),
        baseFeeMode: BaseFeeMode.FeeSchedulerLinear,
      },
      // DBC SDK checks this flag; Bags initial buy math uses dynamic fee disabled.
      dynamicFee: {
        initialized: new BN(0),
      },
    },
    curve: BAGS_CURVE,
  };
}

async function getPartnerLaunchParams(sdk) {
  try {
    const partnerConfigState = await sdk.partner.getPartnerConfig(DEFAULT_BAGS_WALLET);
    if (partnerConfigState.partner.toBase58() !== DEFAULT_BAGS_WALLET.toBase58()) {
      throw new Error("Bags partner config resolved to an unexpected partner wallet.");
    }
    return {
      partner: DEFAULT_BAGS_WALLET,
      partnerConfig: DEFAULT_BAGS_CONFIG,
    };
  } catch (error) {
    const message = String(error && error.message || "").toLowerCase();
    if (message.includes("not found")) {
      return {};
    }
    throw error;
  }
}

function imageInputFromPath(filePath) {
  const absolutePath = path.resolve(String(filePath || "").trim());
  if (!absolutePath || !fs.existsSync(absolutePath)) {
    throw new Error("Bags launch requires a readable local image file.");
  }
  if (!fs.statSync(absolutePath).isFile()) {
    throw new Error("Bags launch requires a readable local image file.");
  }
  const buffer = fs.readFileSync(absolutePath);
  const extension = path.extname(absolutePath).toLowerCase();
  const contentType = extension === ".jpg" || extension === ".jpeg"
    ? "image/jpeg"
    : extension === ".gif"
      ? "image/gif"
      : extension === ".svg"
        ? "image/svg+xml"
      : extension === ".webp"
        ? "image/webp"
        : "image/png";
  return {
    value: buffer,
    options: {
      filename: path.basename(absolutePath) || "token-image.png",
      contentType,
    },
  };
}

async function resolveFeeClaimers(sdk, ownerPublicKey, request) {
  const rows = Array.isArray(request.feeSharing) ? request.feeSharing : [];
  const ownerBase58 = ownerPublicKey.toBase58();
  const mergedClaimers = new Map();
  let allocatedNonOwnerBps = 0;
  for (const row of rows) {
    const type = String(row && row.type || "wallet").trim().toLowerCase();
    const shareBps = Number(row && row.shareBps);
    if (!Number.isFinite(shareBps) || shareBps <= 0) continue;
    let wallet;
    if (type === "wallet") {
      wallet = new PublicKey(String(row.address || "").trim());
    } else if (type === "github") {
      const githubUsername = String(row.githubUsername || "").trim().replace(/^@+/, "");
      if (!githubUsername) {
        throw new Error("Bags GitHub fee-share rows require a GitHub username.");
      }
      const result = await sdk.state.getLaunchWalletV2(githubUsername, "github");
      wallet = result.wallet;
    } else {
      throw new Error(`Unsupported Bags fee-share recipient type: ${type}`);
    }
    const walletBase58 = wallet.toBase58();
    if (walletBase58 === ownerBase58) {
      continue;
    }
    allocatedNonOwnerBps += shareBps;
    mergedClaimers.set(walletBase58, (mergedClaimers.get(walletBase58) || 0) + shareBps);
  }
  if (allocatedNonOwnerBps > 10000) {
    throw new Error("Bags fee-share rows exceed 10000 total bps.");
  }
  const resolved = Array.from(mergedClaimers.entries()).map(([address, userBps]) => ({
    user: new PublicKey(address),
    userBps,
  }));
  const creatorBps = 10000 - allocatedNonOwnerBps;
  if (creatorBps > 0 || resolved.length === 0) {
    resolved.unshift({
      user: ownerPublicKey,
      userBps: creatorBps > 0 ? creatorBps : 10000,
    });
  }
  return resolved;
}

async function quoteLaunch(request) {
  const amount = String(request.amount || "").trim();
  if (!amount) return null;
  const buyMode = String(request.mode || "").trim().toLowerCase();
  if (buyMode !== "sol" && buyMode !== "tokens") {
    throw new Error(`Unsupported Bags dev buy quote mode: ${buyMode || "(empty)"}. Expected sol or tokens.`);
  }
  const virtualPool = buildBagsInitialBuyVirtualPool();
  const config = buildBagsInitialBuyConfig(request.launchMode || "bags-2-2");
  if (buyMode === "sol") {
    const buyAmountLamports = parseDecimalToBigInt(amount, 9, "buy amount");
    if (buyAmountLamports <= 0n) return null;
    const quote = await swapQuote(
      virtualPool,
      config,
      false,
      new BN(buyAmountLamports.toString()),
      0,
      false,
      new BN(0)
    );
    return {
      mode: buyMode,
      input: amount,
      estimatedTokens: formatDecimal(BigInt(quote.amountOut.toString()), 9, 6),
      estimatedSol: formatDecimal(buyAmountLamports, 9, 6),
      estimatedQuoteAmount: formatDecimal(buyAmountLamports, 9, 6),
      quoteAsset: "sol",
      quoteAssetLabel: "SOL",
      estimatedSupplyPercent: formatSupplyPercent(quote.amountOut.toString()),
    };
  }

  const desiredTokens = parseDecimalToBigInt(amount, 9, "buy amount");
  if (desiredTokens <= 0n) return null;
  const quote = swapQuoteExactOut(
    virtualPool,
    config,
    false,
    new BN(desiredTokens.toString()),
    0,
    false,
    new BN(0)
  );
  const requiredLamports = BigInt(quote.amountOut.toString());
  return {
    mode: buyMode,
    input: amount,
    estimatedTokens: formatDecimal(desiredTokens, 9, 6),
    estimatedSol: formatDecimal(requiredLamports, 9, 6),
    estimatedQuoteAmount: formatDecimal(requiredLamports, 9, 6),
    quoteAsset: "sol",
    quoteAssetLabel: "SOL",
    estimatedSupplyPercent: formatSupplyPercent(desiredTokens),
  };
}

async function prepareLaunch(request) {
  const apiKey = requireApiKey(request);
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const feeClaimers = await resolveFeeClaimers(sdk, owner.publicKey, request);
  if (feeClaimers.length > 15) {
    throw new Error("LaunchDeck Bags fee sharing currently supports up to 15 total claimers including the creator.");
  }

  const tokenInfo = await sdk.tokenLaunch.createTokenInfoAndMetadata({
    image: imageInputFromPath(request.imageLocalPath),
    name: String(request.token && request.token.name || "").trim(),
    symbol: String(request.token && request.token.symbol || "").trim(),
    description: String(request.token && request.token.description || "").trim(),
    website: String(request.token && request.token.website || "").trim() || undefined,
    twitter: String(request.token && request.token.twitter || "").trim() || undefined,
    telegram: String(request.token && request.token.telegram || "").trim() || undefined,
  });

  const tipLamports = Number(request.txConfig && request.txConfig.tipLamports || 0);
  const tipWallet = request.txConfig && request.txConfig.tipAccount
    ? new PublicKey(String(request.txConfig.tipAccount).trim())
    : null;
  const tokenMint = new PublicKey(tokenInfo.tokenMint);
  const partnerLaunchParams = await getPartnerLaunchParams(sdk);
  const configResult = await sdk.config.createBagsFeeShareConfig({
    payer: owner.publicKey,
    baseMint: tokenMint,
    feeClaimers,
    ...partnerLaunchParams,
    bagsConfigType: bagsConfigTypeForMode(request.mode),
  }, tipLamports > 0 && tipWallet ? {
    tipWallet,
    tipLamports,
  } : undefined);

  const initialBuyLamports = request.devBuy && String(request.devBuy.amount || "").trim()
    ? Number(parseDecimalToBigInt(request.devBuy.amount, 9, "dev buy amount"))
    : 0;
  const { lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  const directSetupTransactions = signTransactions(configResult.transactions, owner);
  const setupTransactions = normalizeTransactions(directSetupTransactions, {
    labelPrefix: "bags-config-direct",
    computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
    computeUnitPriceMicroLamports: Number(
      request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
    ) || null,
    lastValidBlockHeight,
  });
  const setupBundles = [];
  for (const [index, bundle] of (Array.isArray(configResult.bundles) ? configResult.bundles : []).entries()) {
    const signedBundleTransactions = signTransactions(bundle, owner);
    const compiledBundleTransactions = normalizeTransactions(signedBundleTransactions, {
      labelPrefix: `bags-config-bundle-${index + 1}`,
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      lastValidBlockHeight,
    });
    setupBundles.push({
      label: `bags-config-bundle-${index + 1}`,
      compiledTransactions: compiledBundleTransactions,
    });
  }
  const compiledTransactions = [
    ...setupBundles.flatMap((bundle) => bundle.compiledTransactions),
    ...setupTransactions,
  ];

  return {
    mint: tokenMint.toBase58(),
    launchCreator: owner.publicKey.toBase58(),
    configKey: configResult.meteoraConfigKey.toBase58(),
    metadataUri: tokenInfo.tokenMetadata,
    identityLabel: String(request.identityLabel || "").trim(),
    compiledTransactions,
    setupBundles,
    setupTransactions,
    initialBuyLamports,
  };
}

async function buildLaunchTransaction(request) {
  const apiKey = requireApiKey(request);
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const tokenMint = new PublicKey(String(request.mint || "").trim());
  const configKey = new PublicKey(String(request.configKey || "").trim());
  const tipLamports = Number(request.txConfig && request.txConfig.tipLamports || 0);
  const tipWallet = request.txConfig && request.txConfig.tipAccount
    ? new PublicKey(String(request.txConfig.tipAccount).trim())
    : null;
  const initialBuyLamports = request.devBuy && String(request.devBuy.amount || "").trim()
    ? Number(parseDecimalToBigInt(request.devBuy.amount, 9, "dev buy amount"))
    : 0;
  let launchTransaction;
  let launchError = null;
  for (let attempt = 0; attempt < 5; attempt += 1) {
    try {
      launchTransaction = await sdk.tokenLaunch.createLaunchTransaction({
        metadataUrl: String(request.metadataUri || "").trim(),
        tokenMint,
        launchWallet: owner.publicKey,
        initialBuyLamports,
        configKey,
        tipConfig: tipLamports > 0 && tipWallet ? {
          tipWallet,
          tipLamports,
        } : undefined,
      });
      launchError = null;
      break;
    } catch (error) {
      launchError = error;
      if (attempt === 4) break;
      await sleep(1200);
    }
  }
  if (!launchTransaction) {
    throw launchError || new Error("Failed to create Bags launch transaction.");
  }
  signTransaction(launchTransaction, owner);
  const { lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  return {
    compiledTransaction: normalizeTransactions([launchTransaction], {
      labelPrefix: "launch",
      lastValidBlockHeight,
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: tipLamports || null,
      inlineTipAccount: tipWallet ? tipWallet.toBase58() : null,
    })[0],
  };
}

async function compileFollowBuy(request) {
  const apiKey = requireApiKey(request);
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const quote = await sdk.trade.getQuote({
    inputMint: new PublicKey(WRAPPED_SOL_MINT),
    outputMint: new PublicKey(request.mint),
    amount: Number(parseDecimalToBigInt(request.buyAmountSol, 9, "buy amount")),
    ...slippageModeFromRequest(request),
  });
  const swap = await sdk.trade.createSwapTransaction({
    quoteResponse: quote,
    userPublicKey: owner.publicKey,
  });
  const transaction = await ensureInlineTipOnTransaction(connection, owner, swap.transaction, request.txConfig);
  return {
    compiledTransaction: normalizeTransactions([transaction], {
      labelPrefix: request.labelPrefix || "follow-buy",
      computeUnitLimit: swap.computeUnitLimit || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(request.txConfig && request.txConfig.tipLamports || 0) || null,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount
        ? String(request.txConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: swap.lastValidBlockHeight,
    })[0],
    quote,
  };
}

async function compileFollowSell(request) {
  const apiKey = requireApiKey(request);
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const mint = new PublicKey(request.mint);
  const tokenAccount = getAssociatedTokenAddressSync(mint, owner.publicKey, false, TOKEN_PROGRAM_ID);
  let balanceInfo;
  try {
    balanceInfo = await connection.getTokenAccountBalance(tokenAccount, request.commitment || "processed");
  } catch (_error) {
    return { compiledTransaction: null };
  }
  const rawAmount = BigInt(balanceInfo.value.amount || "0");
  if (rawAmount <= 0n) {
    return { compiledTransaction: null };
  }
  const sellAmount = rawAmount * BigInt(Number(request.sellPercent || 0)) / 100n;
  if (sellAmount <= 0n) {
    return { compiledTransaction: null };
  }
  const quote = await sdk.trade.getQuote({
    inputMint: mint,
    outputMint: new PublicKey(WRAPPED_SOL_MINT),
    amount: Number(sellAmount),
    ...slippageModeFromRequest(request),
  });
  const swap = await sdk.trade.createSwapTransaction({
    quoteResponse: quote,
    userPublicKey: owner.publicKey,
  });
  const transaction = await ensureInlineTipOnTransaction(connection, owner, swap.transaction, request.txConfig);
  return {
    compiledTransaction: normalizeTransactions([transaction], {
      labelPrefix: request.labelPrefix || "follow-sell",
      computeUnitLimit: swap.computeUnitLimit || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(request.txConfig && request.txConfig.tipLamports || 0) || null,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount
        ? String(request.txConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: swap.lastValidBlockHeight,
    })[0],
    quote,
  };
}

async function fetchMarketSnapshot(request) {
  const apiKey = requireApiKey(request);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const mint = new PublicKey(request.mint);
  const [supplyInfo, creators] = await Promise.all([
    connection.getTokenSupply(mint, request.commitment || "processed"),
    sdk.state.getTokenCreators(mint).catch(() => []),
  ]);
  const supplyAmount = BigInt(supplyInfo.value.amount || "0");
  const decimals = Number(supplyInfo.value.decimals || 6);
  const priceQuoteAmount = 10n ** BigInt(decimals);
  const quote = await sdk.trade.getQuote({
    inputMint: mint,
    outputMint: new PublicKey(WRAPPED_SOL_MINT),
    amount: Number(priceQuoteAmount),
    slippageMode: "auto",
  });
  const outAmount = BigInt(quote.outAmount || "0");
  const marketCapLamports = priceQuoteAmount > 0n
    ? (supplyAmount * outAmount) / priceQuoteAmount
    : 0n;
  const creator = Array.isArray(creators)
    ? (creators.find((entry) => entry && entry.isCreator)?.wallet || creators[0]?.wallet || "")
    : "";
  return {
    mint: mint.toBase58(),
    creator,
    virtualTokenReserves: "0",
    virtualSolReserves: "0",
    realTokenReserves: "0",
    realSolReserves: "0",
    tokenTotalSupply: supplyAmount.toString(),
    complete: false,
    marketCapLamports: marketCapLamports.toString(),
    marketCapSol: formatDecimal(marketCapLamports, 9, 6),
    quoteAsset: "sol",
    quoteAssetLabel: "SOL",
  };
}

async function detectImportContext(request) {
  const apiKey = requireApiKey(request);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const mint = new PublicKey(request.mint);
  const creators = await sdk.state.getTokenCreators(mint).catch(() => []);
  if (!Array.isArray(creators) || !creators.length) {
    return null;
  }

  const notes = [];
  const feeRecipients = creators
    .filter((entry) => Number(entry && entry.royaltyBps || 0) > 0)
    .map((entry) => {
      const provider = String(entry && entry.provider || "").trim().toLowerCase();
      const providerUsername = String(entry && (entry.providerUsername || entry.githubUsername || entry.twitterUsername) || "").trim().replace(/^@+/, "");
      if (provider && provider !== "github" && provider !== "solana" && provider !== "wallet" && providerUsername) {
        notes.push(`Recovered ${provider} fee route @${providerUsername} as wallet ${entry.wallet}.`);
      }
      return provider === "github" && providerUsername
        ? {
          type: "github",
          githubUsername: providerUsername,
          address: "",
          shareBps: Number(entry.royaltyBps || 0),
          sourceProvider: provider,
          sourceUsername: providerUsername,
        }
        : {
          type: "wallet",
          githubUsername: "",
          address: String(entry.wallet || "").trim(),
          shareBps: Number(entry.royaltyBps || 0),
          sourceProvider: provider,
          sourceUsername: providerUsername,
        };
    });

  let mode = "";
  let detectionSource = "bags-state";
  let marketKey = "";
  let configKey = "";
  let venue = "";
  try {
    const quote = await sdk.trade.getQuote({
      inputMint: new PublicKey(WRAPPED_SOL_MINT),
      outputMint: mint,
      amount: 1_000_000,
      slippageMode: "auto",
    });
    const leg = Array.isArray(quote.routePlan) ? quote.routePlan.find((entry) => entry && entry.marketKey) : null;
    if (leg && leg.marketKey) {
      marketKey = String(leg.marketKey).trim();
      venue = String(leg.venue || "").trim();
      const marketPubkey = new PublicKey(marketKey);
      const marketAccount = await connection.getAccountInfo(marketPubkey, request.commitment || "confirmed");
      const owner = marketAccount && marketAccount.owner ? marketAccount.owner.toBase58() : "";
      if (marketAccount && owner === METEORA_DAMM_V2_PROGRAM_ID) {
        const decodedPool = sdk.state.getDammV2Program().coder.accounts.decode("pool", marketAccount.data);
        const feeNumerator = Number(decodedPool.poolFees.baseFee.cliffFeeNumerator.toString());
        const feeBps = Math.round(feeNumerator / 100000);
        mode = bagsModeForPostMigrationFeeBps(feeBps);
        detectionSource = "bags-state+damm-pool";
      } else if (marketAccount && owner === METEORA_DBC_PROGRAM_ID) {
        const decodedPool = sdk.state.getDbcProgram().coder.accounts.decode("pool", marketAccount.data);
        configKey = decodedPool.config.toBase58();
        const configAccount = await connection.getAccountInfo(decodedPool.config, request.commitment || "confirmed");
        if (configAccount) {
          const decodedConfig = sdk.state.getDbcProgram().coder.accounts.decode("config", configAccount.data);
          const preFeePercent = Number(decodedConfig.creatorTradingFeePercentage || 0);
          const postFeePercent = Number(decodedConfig.creatorMigrationFeePercentage || 0);
          mode = bagsModeForPrePostFees(preFeePercent, postFeePercent);
          detectionSource = "bags-state+dbc-config";
        }
      }
    }
  } catch (_error) {
    // Keep Bags launchpad detection even if live pool/mode recovery is unavailable.
  }
  if (!mode) {
    notes.push("Bags mode could not be recovered confidently from current market state.");
  }

  return {
    launchpad: "bagsapp",
    mode,
    quoteAsset: "sol",
    creator: String(creators.find((entry) => entry && entry.isCreator)?.wallet || creators[0]?.wallet || "").trim(),
    feeRecipients,
    marketKey,
    configKey,
    venue,
    detectionSource,
    notes,
  };
}

async function readRequest() {
  const chunks = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk);
  }
  const raw = Buffer.concat(chunks).toString("utf8").trim();
  return raw ? JSON.parse(raw) : {};
}

async function handleRequest(request) {
  let response;
  switch (request.action) {
    case "ping":
      response = { ok: true };
      break;
    case "quote":
      response = await quoteLaunch(request);
      break;
    case "estimate-fees":
      response = await estimateFees(request);
      break;
    case "prepare-launch":
      response = await prepareLaunch(request);
      break;
    case "build-launch-transaction":
      response = await buildLaunchTransaction(request);
      break;
    case "build-launch":
      response = await prepareLaunch(request);
      break;
    case "compile-follow-buy":
    case "compile-follow-buy-atomic":
      response = await compileFollowBuy(request);
      break;
    case "compile-follow-sell":
      response = await compileFollowSell(request);
      break;
    case "fetch-market-snapshot":
      response = await fetchMarketSnapshot(request);
      break;
    case "detect-import-context":
      response = await detectImportContext(request);
      break;
    default:
      throw new Error(`Unsupported bags helper action: ${request.action || "(missing)"}`);
  }
  return response;
}

async function runWorkerLoop() {
  const reader = readline.createInterface({
    input: process.stdin,
    crlfDelay: Infinity,
  });
  for await (const line of reader) {
    if (!line.trim()) {
      continue;
    }
    let requestId = null;
    try {
      const envelope = JSON.parse(line);
      requestId = envelope && envelope.requestId != null ? envelope.requestId : null;
      const result = await handleRequest(envelope.request || {});
      process.stdout.write(`${JSON.stringify({ requestId, ok: true, result })}\n`);
    } catch (error) {
      process.stdout.write(`${JSON.stringify({
        requestId,
        ok: false,
        error: error && error.message ? error.message : String(error),
      })}\n`);
    }
  }
}

async function main() {
  if (process.argv.includes("--worker")) {
    await runWorkerLoop();
    return;
  }
  const request = await readRequest();
  const response = await handleRequest(request);
  process.stdout.write(JSON.stringify(response));
}

main().catch((error) => {
  process.stderr.write(`${error && error.stack ? error.stack : String(error)}\n`);
  process.exit(1);
});
