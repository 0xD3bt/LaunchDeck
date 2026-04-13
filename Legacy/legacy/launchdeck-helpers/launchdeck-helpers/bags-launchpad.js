"use strict";

require("dotenv").config({ quiet: true });

const fs = require("fs");
const path = require("path");
const readline = require("readline");
const bs58Module = require("bs58");
const bs58 = bs58Module.default || bs58Module;
const BN = require("bn.js");
const {
  BagsSDK,
  BAGS_FEE_SHARE_V2_PROGRAM_ID,
  WRAPPED_SOL_MINT,
} = require("@bagsfm/bags-sdk");
const {
  CpAmm,
  deriveCustomizablePoolAddress,
  derivePoolAddress,
} = require("@meteora-ag/cp-amm-sdk");
const {
  BaseFeeMode,
  CollectFeeMode,
  DAMM_V2_MIGRATION_FEE_ADDRESS,
  DynamicBondingCurveClient,
  deriveDbcPoolAddress,
  swapQuote,
  swapQuoteExactOut,
} = require("@meteora-ag/dynamic-bonding-curve-sdk");
const {
  ComputeBudgetInstruction,
  ComputeBudgetProgram,
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
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} = require("@solana/spl-token");

const DEFAULT_BAGS_WALLET = new PublicKey("3muhBpbVeoDy4fBrC1SWnfkUooy2Pn6woV1GxDUhESfC");
const DEFAULT_BAGS_CONFIG = new PublicKey("AxpMibQQBqVbQF7EzBUeCbpxRkuk6yfTWRLGVLh5qrce");
const DEFAULT_TOTAL_SUPPLY = 1_000_000_000n;
const BAGS_TOTAL_SUPPLY = 1_000_000_000n * 10n ** 9n;
const BAGS_INITIAL_SQRT_PRICE = new BN("3141367320245630");
const BAGS_MIGRATION_QUOTE_THRESHOLD = new BN("85000000000");
const JITODONTFRONT_ACCOUNT = new PublicKey("jitodontfront111111111111111111111111111111");
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
const STRICT_BASE64_PATTERN = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;

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
  if (value.startsWith("base64:")) {
    const decoded = Buffer.from(value.slice("base64:".length), "base64");
    if (!decoded.length) {
      throw new Error("Wallet secret base64 payload was empty.");
    }
    return Uint8Array.from(decoded);
  }
  if (value.startsWith("base58:")) {
    return Uint8Array.from(bs58.decode(value.slice("base58:".length)));
  }
  if (value.startsWith("[")) {
    const parsed = JSON.parse(value);
    if (!Array.isArray(parsed)) {
      throw new Error("Wallet secret JSON must be an array of bytes.");
    }
    return Uint8Array.from(parsed);
  }
  try {
    return Uint8Array.from(bs58.decode(value));
  } catch (base58Error) {
    if (!STRICT_BASE64_PATTERN.test(value)) {
      throw new Error(`Wallet secret was not valid base58 or base64: ${base58Error.message}`);
    }
    const decoded = Buffer.from(value, "base64");
    if (!decoded.length) {
      throw new Error("Wallet secret base64 payload was empty.");
    }
    return Uint8Array.from(decoded);
  }
}

function parseKeypair(secret) {
  const secretBytes = parseSecretBytes(secret);
  if (secretBytes.length !== 64) {
    throw new Error(`Wallet secret must decode to 64 bytes, got ${secretBytes.length}.`);
  }
  return Keypair.fromSecretKey(secretBytes);
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

function lookupTablesUsedOnTransaction(transaction) {
  if (!(transaction instanceof VersionedTransaction)) {
    return [];
  }
  return (transaction.message.addressTableLookups || []).map((lookup) => (
    lookup.accountKey.toBase58()
  ));
}

function signTransaction(transaction, signer) {
  if (transaction instanceof VersionedTransaction) {
    transaction.sign([signer]);
    return transaction;
  }
  if (transaction instanceof Transaction) {
    transaction.partialSign(signer);
    return transaction;
  }
  if (typeof transaction.partialSign === "function") {
    transaction.partialSign(signer);
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
    lookupTablesUsed: lookupTablesUsedOnTransaction(transaction),
    computeUnitLimit,
    computeUnitPriceMicroLamports,
    inlineTipLamports,
    inlineTipAccount: inlineTipLamports && inlineTipAccount ? inlineTipAccount : null,
  }));
}

function txConfigWithoutInlineTip(txConfig) {
  if (!txConfig) {
    return txConfig;
  }
  return {
    ...txConfig,
    tipLamports: 0,
    tipAccount: "",
  };
}

function isComputeBudgetInstruction(instruction) {
  return Boolean(
    instruction
    && instruction.programId
    && instruction.programId.equals(ComputeBudgetProgram.programId)
  );
}

function buildComputeBudgetInstructions(txConfig) {
  const instructions = [];
  const computeUnitLimit = Number(txConfig && txConfig.computeUnitLimit || 0);
  const computeUnitPriceMicroLamports = Number(
    txConfig && txConfig.computeUnitPriceMicroLamports || 0
  );
  if (Number.isFinite(computeUnitLimit) && computeUnitLimit > 0) {
    instructions.push(ComputeBudgetProgram.setComputeUnitLimit({
      units: Math.floor(computeUnitLimit),
    }));
  }
  if (Number.isFinite(computeUnitPriceMicroLamports) && computeUnitPriceMicroLamports > 0) {
    instructions.push(ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: Math.floor(computeUnitPriceMicroLamports),
    }));
  }
  return instructions;
}

function splitComputeBudgetInstructions(instructions) {
  const nonComputeBudgetInstructions = [];
  const preservedComputeBudgetInstructions = [];
  let computeUnitLimit = null;
  let computeUnitPriceMicroLamports = null;
  for (const instruction of instructions || []) {
    if (!isComputeBudgetInstruction(instruction)) {
      nonComputeBudgetInstructions.push(instruction);
      continue;
    }
    try {
      const type = ComputeBudgetInstruction.decodeInstructionType(instruction);
      if (type === "SetComputeUnitLimit") {
        const decoded = ComputeBudgetInstruction.decodeSetComputeUnitLimit(instruction);
        const units = Number(decoded && decoded.units || 0);
        if (Number.isFinite(units) && units > 0) {
          computeUnitLimit = units;
          continue;
        }
      }
      if (type === "SetComputeUnitPrice") {
        const decoded = ComputeBudgetInstruction.decodeSetComputeUnitPrice(instruction);
        const microLamports = Number(decoded && decoded.microLamports || 0);
        if (Number.isFinite(microLamports) && microLamports > 0) {
          computeUnitPriceMicroLamports = microLamports;
          continue;
        }
      }
    } catch (_error) {
      // Preserve any compute-budget instructions we don't explicitly normalize.
    }
    preservedComputeBudgetInstructions.push(instruction);
  }
  return {
    nonComputeBudgetInstructions,
    preservedComputeBudgetInstructions,
    computeUnitLimit,
    computeUnitPriceMicroLamports,
  };
}

function buildMergedComputeBudgetInstructions(existingBudgetState, txConfig) {
  const requestedComputeUnitLimit = Number(txConfig && txConfig.computeUnitLimit || 0);
  const requestedComputeUnitPriceMicroLamports = Number(
    txConfig && txConfig.computeUnitPriceMicroLamports || 0
  );
  const effectiveComputeUnitLimit = Math.max(
    Number(existingBudgetState && existingBudgetState.computeUnitLimit || 0),
    Number.isFinite(requestedComputeUnitLimit) ? requestedComputeUnitLimit : 0,
  );
  const effectiveComputeUnitPriceMicroLamports = Math.max(
    Number(existingBudgetState && existingBudgetState.computeUnitPriceMicroLamports || 0),
    Number.isFinite(requestedComputeUnitPriceMicroLamports)
      ? requestedComputeUnitPriceMicroLamports
      : 0,
  );
  return buildComputeBudgetInstructions({
    computeUnitLimit: effectiveComputeUnitLimit,
    computeUnitPriceMicroLamports: effectiveComputeUnitPriceMicroLamports,
  });
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

function hasJitoDontFrontAccount(instruction) {
  return Boolean(
    instruction
    && Array.isArray(instruction.keys)
    && instruction.keys.some((key) => key && key.pubkey && key.pubkey.equals(JITODONTFRONT_ACCOUNT))
  );
}

function ensureJitoDontFrontOnInstruction(instruction) {
  if (!instruction || !Array.isArray(instruction.keys) || hasJitoDontFrontAccount(instruction)) {
    return false;
  }
  instruction.keys.push({
    pubkey: JITODONTFRONT_ACCOUNT,
    isSigner: false,
    isWritable: false,
  });
  return true;
}

function hasAdditionalRequiredSigners(transaction, ownerPubkey) {
  if (!transaction || !ownerPubkey) return false;
  if (transaction instanceof VersionedTransaction) {
    const requiredSignatures = Number(transaction.message?.header?.numRequiredSignatures || 0);
    const staticAccountKeys = Array.isArray(transaction.message?.staticAccountKeys)
      ? transaction.message.staticAccountKeys
      : [];
    return staticAccountKeys
      .slice(0, requiredSignatures)
      .some((pubkey) => pubkey && !pubkey.equals(ownerPubkey));
  }
  const signerKeys = new Set();
  if (transaction.feePayer) {
    signerKeys.add(transaction.feePayer.toBase58());
  }
  for (const instruction of transaction.instructions || []) {
    for (const key of instruction.keys || []) {
      if (key && key.isSigner && key.pubkey) {
        signerKeys.add(key.pubkey.toBase58());
      }
    }
  }
  signerKeys.delete(ownerPubkey.toBase58());
  return signerKeys.size > 0;
}

function parseBlockhashOverride(request) {
  if (!request) return null;
  const blockhash = String(request.recentBlockhash || "").trim();
  const lvbhRaw = request.lastValidBlockHeight;
  if (!blockhash || lvbhRaw == null) return null;
  const lastValidBlockHeight = Number(lvbhRaw);
  if (!Number.isFinite(lastValidBlockHeight)) return null;
  return { blockhash, lastValidBlockHeight };
}

async function ensureTxConfigOnTransaction(connection, owner, transaction, txConfig, commitment, blockhashOverride) {
  const wantDontFront = Boolean(txConfig && txConfig.jitodontfront);
  const requestedComputeBudgetInstructions = buildComputeBudgetInstructions(txConfig);
  const tipInstruction = buildInlineTipInstruction(
    owner.publicKey,
    txConfig && txConfig.tipAccount,
    txConfig && txConfig.tipLamports,
  );
  const hasExtraSigners = hasAdditionalRequiredSigners(transaction, owner.publicKey);
  if (hasExtraSigners) {
    // Bags setup transactions can already carry non-owner signatures from the SDK.
    // Rebuilding them here or refreshing their blockhash would invalidate those signatures
    // because we only control the owner key.
    return signTransaction(transaction, owner);
  }
  const { blockhash: freshBlockhash } = blockhashOverride && blockhashOverride.blockhash
    ? { blockhash: blockhashOverride.blockhash }
    : await connection.getLatestBlockhash(commitment || "confirmed");
  if (transaction instanceof VersionedTransaction) {
    const { instructions, addressLookupTableAccounts } = await decompileTransactionInstructions(connection, transaction);
    const existingBudgetState = splitComputeBudgetInstructions(instructions);
    const filteredInstructions = existingBudgetState.nonComputeBudgetInstructions;
    const computeBudgetInstructions = buildMergedComputeBudgetInstructions(existingBudgetState, txConfig);
    let modified = false;
    if (wantDontFront) {
      for (const instruction of filteredInstructions) {
        modified = ensureJitoDontFrontOnInstruction(instruction) || modified;
      }
    }
    const hasTip = filteredInstructions.some((instruction) => (
      isInlineTipInstruction(
        instruction,
        owner.publicKey,
        txConfig && txConfig.tipAccount,
        txConfig && txConfig.tipLamports,
      )
    ));
    if (existingBudgetState.computeUnitLimit !== null
      || existingBudgetState.computeUnitPriceMicroLamports !== null
      || existingBudgetState.preservedComputeBudgetInstructions.length > 0) {
      modified = true;
    }
    if (tipInstruction && !hasTip) {
      filteredInstructions.push(tipInstruction);
      modified = true;
    }
    if (computeBudgetInstructions.length > 0) {
      modified = true;
    }
    const rebuiltInstructions = [
      ...computeBudgetInstructions,
      ...existingBudgetState.preservedComputeBudgetInstructions,
      ...filteredInstructions,
    ];
    const rebuilt = new VersionedTransaction(
      new TransactionMessage({
        payerKey: owner.publicKey,
        recentBlockhash: freshBlockhash,
        instructions: rebuiltInstructions,
      }).compileToV0Message(addressLookupTableAccounts),
    );
    rebuilt.sign([owner]);
    return rebuilt;
  }
  const instructions = transaction.instructions || [];
  const existingBudgetState = splitComputeBudgetInstructions(instructions);
  const filteredInstructions = existingBudgetState.nonComputeBudgetInstructions;
  const computeBudgetInstructions = buildMergedComputeBudgetInstructions(existingBudgetState, txConfig);
  let modified = false;
  if (wantDontFront) {
    for (const instruction of filteredInstructions) {
      modified = ensureJitoDontFrontOnInstruction(instruction) || modified;
    }
  }
  const hasTip = filteredInstructions.some((instruction) => (
    isInlineTipInstruction(
      instruction,
      owner.publicKey,
      txConfig && txConfig.tipAccount,
      txConfig && txConfig.tipLamports,
    )
  ));
  if (existingBudgetState.computeUnitLimit !== null
    || existingBudgetState.computeUnitPriceMicroLamports !== null
    || existingBudgetState.preservedComputeBudgetInstructions.length > 0) {
    modified = true;
  }
  if (tipInstruction && !hasTip) {
    filteredInstructions.push(tipInstruction);
    modified = true;
  }
  if (computeBudgetInstructions.length > 0) {
    modified = true;
  }
  if (!modified) {
    transaction.recentBlockhash = freshBlockhash;
    return signTransaction(transaction, owner);
  }
  const rebuiltInstructions = [
    ...computeBudgetInstructions,
    ...existingBudgetState.preservedComputeBudgetInstructions,
    ...filteredInstructions,
  ];
  const rebuilt = new Transaction();
  rebuilt.feePayer = owner.publicKey;
  rebuilt.recentBlockhash = freshBlockhash;
  rebuiltInstructions.forEach((instruction) => rebuilt.add(instruction));
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

function toBn(value) {
  if (value instanceof BN) return value;
  if (value && typeof value.toString === "function") {
    return new BN(value.toString());
  }
  return new BN(0);
}

function parseOptionalPublicKey(value) {
  const raw = String(value || "").trim();
  if (!raw) return null;
  try {
    return new PublicKey(raw);
  } catch (_error) {
    return null;
  }
}

function formatErrorDetails(error) {
  if (!error) return "Unknown error";
  const baseMessage = error && error.message ? String(error.message) : String(error);
  const status = error && error.status != null ? `status=${error.status}` : "";
  const method = error && error.method ? String(error.method).toUpperCase() : "";
  const url = error && error.url ? String(error.url) : "";
  let payload = "";
  if (error && error.data !== undefined) {
    try {
      payload = typeof error.data === "string" ? error.data : JSON.stringify(error.data);
    } catch (_error) {
      payload = String(error.data);
    }
  }
  const location = method && url ? `${method} ${url}` : (method || url);
  const suffix = [status, location, payload ? `payload=${payload}` : ""]
    .filter(Boolean)
    .join(" | ");
  return suffix ? `${baseMessage} (${suffix})` : baseMessage;
}

function normalizeCachedBagsLaunch(raw) {
  const source = raw && typeof raw === "object" ? raw : {};
  const rawMigrationFeeOption = source.migrationFeeOption;
  const migrationFeeOption = rawMigrationFeeOption === null
    || rawMigrationFeeOption === undefined
    || String(rawMigrationFeeOption).trim() === ""
    ? null
    : Number(rawMigrationFeeOption);
  return {
    configKey: parseOptionalPublicKey(source.configKey),
    migrationFeeOption: Number.isFinite(migrationFeeOption) ? migrationFeeOption : null,
    expectedMigrationFamily: String(source.expectedMigrationFamily || "").trim(),
    expectedDammConfigKey: parseOptionalPublicKey(source.expectedDammConfigKey),
    expectedDammDerivationMode: String(source.expectedDammDerivationMode || "").trim(),
    preMigrationDbcPoolAddress: parseOptionalPublicKey(source.preMigrationDbcPoolAddress),
  };
}

function buildLocalTradeFailClosedError(code, message, extras = {}) {
  const detail = Object.entries(extras)
    .filter(([, value]) => value !== null && value !== undefined && String(value).trim() !== "")
    .map(([key, value]) => `${key}=${value}`)
    .join(" ");
  return new Error(
    `[bags-local:${code}] ${message}${detail ? ` (${detail})` : ""}`,
  );
}

async function currentPointForDbcConfig(connection, configState, commitment) {
  const currentSlot = await connection.getSlot(commitment || "processed");
  if (Number(configState && configState.activationType || 0) === 0) {
    return new BN(String(currentSlot));
  }
  const currentTime = await connection.getBlockTime(currentSlot);
  return new BN(String(currentTime || Math.floor(Date.now() / 1000)));
}

function isCompletedDbcPool(poolState, configState) {
  if (!poolState || !configState) return false;
  if (Boolean(poolState.isMigrated)) return true;
  if (!configState.migrationQuoteThreshold || !poolState.quoteReserve) return false;
  return toBn(poolState.quoteReserve).gte(toBn(configState.migrationQuoteThreshold));
}

async function loadLocalDbcState(connection, mint, commitment, cachedLaunch) {
  const cached = normalizeCachedBagsLaunch(cachedLaunch);
  const client = new DynamicBondingCurveClient(connection, commitment || "processed");
  const poolAccount = await client.state.getPoolByBaseMint(mint).catch(() => null);
  if (!poolAccount || !poolAccount.account || !poolAccount.account.config) {
    return null;
  }
  if (cached.preMigrationDbcPoolAddress && !poolAccount.publicKey.equals(cached.preMigrationDbcPoolAddress)) {
    return null;
  }
  if (cached.configKey && !poolAccount.account.config.equals(cached.configKey)) {
    return null;
  }
  const configKey = cached.configKey || poolAccount.account.config;
  const configState = await client.state.getPoolConfig(configKey).catch(() => null);
  if (!configState || !configState.quoteMint || !configState.quoteMint.equals(NATIVE_MINT)) {
    return null;
  }
  const derivedPoolAddress = deriveDbcPoolAddress(configState.quoteMint, mint, configKey);
  if (cached.preMigrationDbcPoolAddress && !derivedPoolAddress.equals(cached.preMigrationDbcPoolAddress)) {
    return null;
  }
  return {
    client,
    poolAddress: poolAccount.publicKey,
    poolState: poolAccount.account,
    configKey,
    configState,
    derivedPoolAddress,
    derivedMatches: poolAccount.publicKey.equals(derivedPoolAddress),
    isMigrated: Boolean(poolAccount.account.isMigrated),
    isCompleted: isCompletedDbcPool(poolAccount.account, configState),
  };
}

async function prepareLocalTransactionForSigning(connection, owner, transaction, commitment) {
  if (transaction instanceof VersionedTransaction) {
    const { lastValidBlockHeight } = await connection.getLatestBlockhash(commitment || "confirmed");
    return { transaction, lastValidBlockHeight };
  }
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash(commitment || "confirmed");
  if (!transaction.feePayer) {
    transaction.feePayer = owner.publicKey;
  }
  if (!transaction.recentBlockhash) {
    transaction.recentBlockhash = blockhash;
  }
  return { transaction, lastValidBlockHeight };
}

async function currentTimeForDamm(connection, commitment) {
  const currentSlot = await connection.getSlot(commitment || "processed");
  let currentTime = null;
  try {
    currentTime = await connection.getBlockTime(currentSlot);
  } catch (_error) {
    currentTime = null;
  }
  return {
    currentSlot,
    currentTime: currentTime || Math.floor(Date.now() / 1000),
  };
}

function tokenProgramForFlag(flag) {
  return Number(flag || 0) === 0 ? TOKEN_PROGRAM_ID : TOKEN_2022_PROGRAM_ID;
}

function deriveCanonicalDammPoolAddress(mint, configState) {
  const feeOption = Number(configState && configState.migrationFeeOption);
  if (!Number.isFinite(feeOption) || feeOption < 0) {
    return null;
  }
  if (feeOption === 6) {
    return deriveCustomizablePoolAddress(mint, NATIVE_MINT);
  }
  const dammConfig = DAMM_V2_MIGRATION_FEE_ADDRESS[feeOption];
  if (!dammConfig) {
    return null;
  }
  return derivePoolAddress(dammConfig, mint, NATIVE_MINT);
}

function deriveCachedDammPoolAddress(mint, cachedLaunch) {
  const cached = normalizeCachedBagsLaunch(cachedLaunch);
  if (cached.migrationFeeOption === 6 || cached.expectedMigrationFamily === "customizable") {
    return {
      poolAddress: deriveCustomizablePoolAddress(mint, NATIVE_MINT),
      configAddress: cached.expectedDammConfigKey,
    };
  }
  if (cached.expectedDammConfigKey) {
    return {
      poolAddress: derivePoolAddress(cached.expectedDammConfigKey, mint, NATIVE_MINT),
      configAddress: cached.expectedDammConfigKey,
    };
  }
  if (Number.isFinite(cached.migrationFeeOption) && cached.migrationFeeOption >= 0) {
    const configAddress = DAMM_V2_MIGRATION_FEE_ADDRESS[cached.migrationFeeOption] || null;
    if (!configAddress) {
      return null;
    }
    return {
      poolAddress: derivePoolAddress(configAddress, mint, NATIVE_MINT),
      configAddress,
    };
  }
  return null;
}

function expectedMigrationFamilyFromConfig(configState) {
  const feeOption = Number(configState && configState.migrationFeeOption);
  if (!Number.isFinite(feeOption) || feeOption < 0) {
    return "";
  }
  return feeOption === 6 ? "customizable" : "fixed";
}

function expectedDammConfigKeyFromConfig(configState) {
  const feeOption = Number(configState && configState.migrationFeeOption);
  if (!Number.isFinite(feeOption) || feeOption < 0) {
    return "";
  }
  if (feeOption === 6) {
    return DAMM_V2_MIGRATION_FEE_ADDRESS[6] ? DAMM_V2_MIGRATION_FEE_ADDRESS[6].toBase58() : "";
  }
  return DAMM_V2_MIGRATION_FEE_ADDRESS[feeOption]
    ? DAMM_V2_MIGRATION_FEE_ADDRESS[feeOption].toBase58()
    : "";
}

async function summarizeLaunchMigrationConfig(connection, mint, configKey, commitment) {
  try {
    const client = new DynamicBondingCurveClient(connection, commitment || "processed");
    const configState = await client.state.getPoolConfig(configKey);
    const preMigrationDbcPool = deriveDbcPoolAddress(NATIVE_MINT, mint, configKey);
    const migrationFeeOption = Number(configState && configState.migrationFeeOption);
    return {
      migrationFeeOption: Number.isFinite(migrationFeeOption) ? migrationFeeOption : null,
      expectedMigrationFamily: expectedMigrationFamilyFromConfig(configState),
      expectedDammConfigKey: expectedDammConfigKeyFromConfig(configState),
      expectedDammDerivationMode: migrationFeeOption === 6 ? "customizable" : "config-derived",
      preMigrationDbcPoolAddress: preMigrationDbcPool.toBase58(),
    };
  } catch (_error) {
    return {
      migrationFeeOption: null,
      expectedMigrationFamily: "",
      expectedDammConfigKey: "",
      expectedDammDerivationMode: "",
      preMigrationDbcPoolAddress: "",
    };
  }
}

async function loadLocalDammState(connection, mint, commitment, cachedLaunch) {
  const cached = normalizeCachedBagsLaunch(cachedLaunch);
  const dbcClient = new DynamicBondingCurveClient(connection, commitment || "processed");
  const poolAccount = await dbcClient.state.getPoolByBaseMint(mint).catch(() => null);
  if (!poolAccount || !poolAccount.account || !poolAccount.account.config || !Boolean(poolAccount.account.isMigrated)) {
    return null;
  }
  if (cached.preMigrationDbcPoolAddress && !poolAccount.publicKey.equals(cached.preMigrationDbcPoolAddress)) {
    return null;
  }
  if (cached.configKey && !poolAccount.account.config.equals(cached.configKey)) {
    return null;
  }
  const configKey = cached.configKey || poolAccount.account.config;
  const configState = await dbcClient.state.getPoolConfig(configKey).catch(() => null);
  if (!configState || !configState.quoteMint || !configState.quoteMint.equals(NATIVE_MINT)) {
    return null;
  }
  const derivedDbcPoolAddress = deriveDbcPoolAddress(configState.quoteMint, mint, configKey);
  if (!poolAccount.publicKey.equals(derivedDbcPoolAddress)) {
    return null;
  }
  const localDbc = {
    client: dbcClient,
    poolAddress: poolAccount.publicKey,
    poolState: poolAccount.account,
    configKey,
    configState,
    derivedPoolAddress: derivedDbcPoolAddress,
    derivedMatches: true,
    isMigrated: true,
    isCompleted: true,
  };
  let resolved = deriveCachedDammPoolAddress(mint, cached);
  if (!resolved) {
    resolved = {
      poolAddress: deriveCanonicalDammPoolAddress(mint, localDbc.configState),
      configAddress: Number(localDbc.configState && localDbc.configState.migrationFeeOption) === 6
        ? null
        : DAMM_V2_MIGRATION_FEE_ADDRESS[Number(localDbc.configState.migrationFeeOption)] || null,
    };
  }
  if (!resolved || !resolved.poolAddress) {
    return null;
  }
  const client = new CpAmm(connection);
  const exists = await client.isPoolExist(resolved.poolAddress).catch(() => false);
  if (!exists) {
    return null;
  }
  const poolState = await client.fetchPoolState(resolved.poolAddress).catch(() => null);
  if (!poolState) {
    return null;
  }
  return {
    client,
    localDbc,
    poolAddress: resolved.poolAddress,
    poolState,
    configAddress: resolved.configAddress,
    tokenAProgram: tokenProgramForFlag(poolState.tokenAFlag),
    tokenBProgram: tokenProgramForFlag(poolState.tokenBFlag),
  };
}

async function tryBuildLocalDbcFollowBuy(connection, owner, request) {
  const mint = new PublicKey(request.mint);
  const localDbc = await loadLocalDbcState(
    connection,
    mint,
    request.commitment || "processed",
    request.bagsLaunch,
  );
  if (!localDbc || !localDbc.derivedMatches || localDbc.isCompleted || localDbc.isMigrated) {
    return null;
  }
  const amountIn = new BN(parseDecimalToBigInt(request.buyAmountSol, 9, "buy amount").toString());
  if (amountIn.lte(new BN(0))) {
    return null;
  }
  const currentPoint = await currentPointForDbcConfig(connection, localDbc.configState, request.commitment);
  const quote = await localDbc.client.pool.swapQuote({
    virtualPool: localDbc.poolState,
    config: localDbc.configState,
    swapBaseForQuote: false,
    amountIn,
    slippageBps: Number(request.slippageBps || 0),
    hasReferral: false,
    currentPoint,
  }).catch((error) => {
    if (error && String(error.message || error).includes("Virtual pool is completed")) {
      return null;
    }
    throw error;
  });
  if (!quote) {
    return null;
  }
  const built = await localDbc.client.pool.swap({
    owner: owner.publicKey,
    pool: localDbc.poolAddress,
    amountIn,
    minimumAmountOut: quote.minimumAmountOut,
    swapBaseForQuote: false,
    referralTokenAccount: null,
  });
  const prepared = await prepareLocalTransactionForSigning(connection, owner, built, request.commitment);
  const transaction = await ensureTxConfigOnTransaction(
    connection,
    owner,
    prepared.transaction,
    request.txConfig,
    request.commitment,
  );
  return {
    compiledTransaction: normalizeTransactions([transaction], {
      labelPrefix: request.labelPrefix || "follow-buy",
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(request.txConfig && request.txConfig.tipLamports || 0) || null,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount
        ? String(request.txConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: prepared.lastValidBlockHeight,
    })[0],
    quote: {
      source: "local-dbc",
      pool: localDbc.poolAddress.toBase58(),
      config: localDbc.poolState.config.toBase58(),
      inAmount: amountIn.toString(),
      outAmount: quote.amountOut.toString(),
      minimumAmountOut: quote.minimumAmountOut.toString(),
    },
  };
}

async function tryBuildLocalDbcFollowSell(connection, owner, request) {
  const mint = new PublicKey(request.mint);
  const localDbc = await loadLocalDbcState(
    connection,
    mint,
    request.commitment || "processed",
    request.bagsLaunch,
  );
  if (!localDbc || !localDbc.derivedMatches || localDbc.isCompleted || localDbc.isMigrated) {
    return null;
  }
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
  const amountIn = new BN(sellAmount.toString());
  const currentPoint = await currentPointForDbcConfig(connection, localDbc.configState, request.commitment);
  const quote = await localDbc.client.pool.swapQuote({
    virtualPool: localDbc.poolState,
    config: localDbc.configState,
    swapBaseForQuote: true,
    amountIn,
    slippageBps: Number(request.slippageBps || 0),
    hasReferral: false,
    currentPoint,
  }).catch((error) => {
    if (error && String(error.message || error).includes("Virtual pool is completed")) {
      return null;
    }
    throw error;
  });
  if (!quote) {
    return null;
  }
  const built = await localDbc.client.pool.swap({
    owner: owner.publicKey,
    pool: localDbc.poolAddress,
    amountIn,
    minimumAmountOut: quote.minimumAmountOut,
    swapBaseForQuote: true,
    referralTokenAccount: null,
  });
  const prepared = await prepareLocalTransactionForSigning(connection, owner, built, request.commitment);
  const transaction = await ensureTxConfigOnTransaction(
    connection,
    owner,
    prepared.transaction,
    request.txConfig,
    request.commitment,
  );
  return {
    compiledTransaction: normalizeTransactions([transaction], {
      labelPrefix: request.labelPrefix || "follow-sell",
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(request.txConfig && request.txConfig.tipLamports || 0) || null,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount
        ? String(request.txConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: prepared.lastValidBlockHeight,
    })[0],
    quote: {
      source: "local-dbc",
      pool: localDbc.poolAddress.toBase58(),
      config: localDbc.poolState.config.toBase58(),
      inAmount: amountIn.toString(),
      outAmount: quote.amountOut.toString(),
      minimumAmountOut: quote.minimumAmountOut.toString(),
    },
  };
}

async function tryFetchLocalDbcMarketSnapshot(connection, mint, commitment, cachedLaunch) {
  const localDbc = await loadLocalDbcState(connection, mint, commitment || "processed", cachedLaunch);
  if (!localDbc || !localDbc.derivedMatches || localDbc.isCompleted || localDbc.isMigrated) {
    return null;
  }
  const [supplyInfo, currentPoint] = await Promise.all([
    connection.getTokenSupply(mint, commitment || "processed"),
    currentPointForDbcConfig(connection, localDbc.configState, commitment || "processed"),
  ]);
  const supplyAmount = BigInt(supplyInfo.value.amount || "0");
  const decimals = Number(supplyInfo.value.decimals || 6);
  const priceQuoteAmount = 10n ** BigInt(decimals);
  const quote = await localDbc.client.pool.swapQuote({
    virtualPool: localDbc.poolState,
    config: localDbc.configState,
    swapBaseForQuote: true,
    amountIn: new BN(priceQuoteAmount.toString()),
    slippageBps: 0,
    hasReferral: false,
    currentPoint,
  });
  const outAmount = BigInt(quote.amountOut.toString());
  const marketCapLamports = priceQuoteAmount > 0n
    ? (supplyAmount * outAmount) / priceQuoteAmount
    : 0n;
  return {
    mint: mint.toBase58(),
    creator: localDbc.poolState.creator ? localDbc.poolState.creator.toBase58() : "",
    virtualTokenReserves: localDbc.poolState.baseReserve ? localDbc.poolState.baseReserve.toString() : "0",
    virtualSolReserves: localDbc.poolState.quoteReserve ? localDbc.poolState.quoteReserve.toString() : "0",
    realTokenReserves: localDbc.poolState.baseReserve ? localDbc.poolState.baseReserve.toString() : "0",
    realSolReserves: localDbc.poolState.quoteReserve ? localDbc.poolState.quoteReserve.toString() : "0",
    tokenTotalSupply: supplyAmount.toString(),
    complete: false,
    marketCapLamports: marketCapLamports.toString(),
    marketCapSol: formatDecimal(marketCapLamports, 9, 6),
    quoteAsset: "sol",
    quoteAssetLabel: "SOL",
  };
}

async function tryBuildLocalDammFollowBuy(connection, owner, request) {
  const mint = new PublicKey(request.mint);
  const localDamm = await loadLocalDammState(
    connection,
    mint,
    request.commitment || "processed",
    request.bagsLaunch,
  );
  if (!localDamm) {
    return null;
  }
  const amountIn = new BN(parseDecimalToBigInt(request.buyAmountSol, 9, "buy amount").toString());
  if (amountIn.lte(new BN(0))) {
    return null;
  }
  const timing = await currentTimeForDamm(connection, request.commitment);
  const quote = localDamm.client.getQuote({
    inAmount: amountIn,
    inputTokenMint: NATIVE_MINT,
    slippage: Number(request.slippageBps || 0) / 100,
    poolState: localDamm.poolState,
    currentTime: timing.currentTime,
    currentSlot: timing.currentSlot,
  });
  const built = await localDamm.client.swap({
    payer: owner.publicKey,
    pool: localDamm.poolAddress,
    inputTokenMint: NATIVE_MINT,
    outputTokenMint: mint,
    amountIn,
    minimumAmountOut: quote.minSwapOutAmount,
    tokenAMint: localDamm.poolState.tokenAMint,
    tokenBMint: localDamm.poolState.tokenBMint,
    tokenAVault: localDamm.poolState.tokenAVault,
    tokenBVault: localDamm.poolState.tokenBVault,
    tokenAProgram: localDamm.tokenAProgram,
    tokenBProgram: localDamm.tokenBProgram,
    referralTokenAccount: null,
  });
  const prepared = await prepareLocalTransactionForSigning(connection, owner, built, request.commitment);
  const transaction = await ensureTxConfigOnTransaction(
    connection,
    owner,
    prepared.transaction,
    request.txConfig,
    request.commitment,
  );
  return {
    compiledTransaction: normalizeTransactions([transaction], {
      labelPrefix: request.labelPrefix || "follow-buy",
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(request.txConfig && request.txConfig.tipLamports || 0) || null,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount
        ? String(request.txConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: prepared.lastValidBlockHeight,
    })[0],
    quote: {
      source: "local-damm-v2",
      pool: localDamm.poolAddress.toBase58(),
      config: localDamm.configAddress ? localDamm.configAddress.toBase58() : "customizable",
      inAmount: amountIn.toString(),
      outAmount: quote.swapOutAmount.toString(),
      minimumAmountOut: quote.minSwapOutAmount.toString(),
    },
  };
}

async function tryBuildLocalDammFollowSell(connection, owner, request) {
  const mint = new PublicKey(request.mint);
  const localDamm = await loadLocalDammState(
    connection,
    mint,
    request.commitment || "processed",
    request.bagsLaunch,
  );
  if (!localDamm) {
    return null;
  }
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
  const amountIn = new BN(sellAmount.toString());
  const timing = await currentTimeForDamm(connection, request.commitment);
  const quote = localDamm.client.getQuote({
    inAmount: amountIn,
    inputTokenMint: mint,
    slippage: Number(request.slippageBps || 0) / 100,
    poolState: localDamm.poolState,
    currentTime: timing.currentTime,
    currentSlot: timing.currentSlot,
  });
  const built = await localDamm.client.swap({
    payer: owner.publicKey,
    pool: localDamm.poolAddress,
    inputTokenMint: mint,
    outputTokenMint: NATIVE_MINT,
    amountIn,
    minimumAmountOut: quote.minSwapOutAmount,
    tokenAMint: localDamm.poolState.tokenAMint,
    tokenBMint: localDamm.poolState.tokenBMint,
    tokenAVault: localDamm.poolState.tokenAVault,
    tokenBVault: localDamm.poolState.tokenBVault,
    tokenAProgram: localDamm.tokenAProgram,
    tokenBProgram: localDamm.tokenBProgram,
    referralTokenAccount: null,
  });
  const prepared = await prepareLocalTransactionForSigning(connection, owner, built, request.commitment);
  const transaction = await ensureTxConfigOnTransaction(
    connection,
    owner,
    prepared.transaction,
    request.txConfig,
    request.commitment,
  );
  return {
    compiledTransaction: normalizeTransactions([transaction], {
      labelPrefix: request.labelPrefix || "follow-sell",
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(request.txConfig && request.txConfig.tipLamports || 0) || null,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount
        ? String(request.txConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: prepared.lastValidBlockHeight,
    })[0],
    quote: {
      source: "local-damm-v2",
      pool: localDamm.poolAddress.toBase58(),
      config: localDamm.configAddress ? localDamm.configAddress.toBase58() : "customizable",
      inAmount: amountIn.toString(),
      outAmount: quote.swapOutAmount.toString(),
      minimumAmountOut: quote.minSwapOutAmount.toString(),
    },
  };
}

async function tryFetchLocalDammMarketSnapshot(connection, mint, commitment, cachedLaunch) {
  const localDamm = await loadLocalDammState(connection, mint, commitment || "processed", cachedLaunch);
  if (!localDamm) {
    return null;
  }
  const { currentSlot, currentTime } = await currentTimeForDamm(connection, commitment || "processed");
  const supplyInfo = await connection.getTokenSupply(mint, commitment || "processed");
  const supplyAmount = BigInt(supplyInfo.value.amount || "0");
  const decimals = Number(supplyInfo.value.decimals || 6);
  const priceQuoteAmount = 10n ** BigInt(decimals);
  const quote = localDamm.client.getQuote({
    inAmount: new BN(priceQuoteAmount.toString()),
    inputTokenMint: mint,
    slippage: 0,
    poolState: localDamm.poolState,
    currentTime,
    currentSlot,
  });
  const outAmount = BigInt(quote.swapOutAmount.toString());
  const marketCapLamports = priceQuoteAmount > 0n
    ? (supplyAmount * outAmount) / priceQuoteAmount
    : 0n;
  const [vaultABalance, vaultBBalance] = await Promise.all([
    connection.getTokenAccountBalance(localDamm.poolState.tokenAVault, commitment || "processed").catch(() => null),
    connection.getTokenAccountBalance(localDamm.poolState.tokenBVault, commitment || "processed").catch(() => null),
  ]);
  const tokenAReserve = BigInt(vaultABalance && vaultABalance.value && vaultABalance.value.amount || "0");
  const tokenBReserve = BigInt(vaultBBalance && vaultBBalance.value && vaultBBalance.value.amount || "0");
  const isTokenABase = localDamm.poolState.tokenAMint.equals(mint);
  const realTokenReserves = isTokenABase ? tokenAReserve : tokenBReserve;
  const realSolReserves = isTokenABase ? tokenBReserve : tokenAReserve;
  return {
    mint: mint.toBase58(),
    creator: localDamm.poolState.creator ? localDamm.poolState.creator.toBase58() : "",
    virtualTokenReserves: "0",
    virtualSolReserves: "0",
    realTokenReserves: realTokenReserves.toString(),
    realSolReserves: realSolReserves.toString(),
    tokenTotalSupply: supplyAmount.toString(),
    complete: true,
    marketCapLamports: marketCapLamports.toString(),
    marketCapSol: formatDecimal(marketCapLamports, 9, 6),
    quoteAsset: "sol",
    quoteAssetLabel: "SOL",
  };
}

async function failClosedBagsTradeError(connection, mint, commitment, request, action) {
  const cached = normalizeCachedBagsLaunch(request && request.bagsLaunch);
  const mintKey = mint instanceof PublicKey ? mint : new PublicKey(mint);
  const dbcClient = new DynamicBondingCurveClient(connection, commitment || "processed");
  const poolAccount = await dbcClient.state.getPoolByBaseMint(mintKey).catch(() => null);
  if (!poolAccount || !poolAccount.account || !poolAccount.account.config) {
    throw buildLocalTradeFailClosedError(
      "dbc_pool_not_found",
      `Canonical Bags ${action} requires a local Meteora DBC pool, but none was found.`,
      {
        mint: mintKey.toBase58(),
        configKey: cached.configKey ? cached.configKey.toBase58() : "",
        expectedPool: cached.preMigrationDbcPoolAddress
          ? cached.preMigrationDbcPoolAddress.toBase58()
          : "",
      },
    );
  }
  if (cached.preMigrationDbcPoolAddress && !poolAccount.publicKey.equals(cached.preMigrationDbcPoolAddress)) {
    throw buildLocalTradeFailClosedError(
      "dbc_pool_mismatch",
      `Resolved DBC pool did not match the cached LaunchDeck Bags pool for ${action}.`,
      {
        mint: mintKey.toBase58(),
        resolvedPool: poolAccount.publicKey.toBase58(),
        expectedPool: cached.preMigrationDbcPoolAddress.toBase58(),
      },
    );
  }
  if (cached.configKey && !poolAccount.account.config.equals(cached.configKey)) {
    throw buildLocalTradeFailClosedError(
      "dbc_config_mismatch",
      `Resolved DBC config did not match the cached LaunchDeck Bags config for ${action}.`,
      {
        mint: mintKey.toBase58(),
        resolvedConfig: poolAccount.account.config.toBase58(),
        expectedConfig: cached.configKey.toBase58(),
      },
    );
  }
  const configKey = cached.configKey || poolAccount.account.config;
  const configState = await dbcClient.state.getPoolConfig(configKey).catch(() => null);
  if (!configState || !configState.quoteMint || !configState.quoteMint.equals(NATIVE_MINT)) {
    throw buildLocalTradeFailClosedError(
      "dbc_config_not_found",
      `Canonical Bags ${action} could not load the expected local DBC config.`,
      {
        mint: mintKey.toBase58(),
        configKey: configKey.toBase58(),
      },
    );
  }
  const derivedPoolAddress = deriveDbcPoolAddress(configState.quoteMint, mintKey, configKey);
  if (cached.preMigrationDbcPoolAddress && !derivedPoolAddress.equals(cached.preMigrationDbcPoolAddress)) {
    throw buildLocalTradeFailClosedError(
      "dbc_pool_not_derived",
      `Cached LaunchDeck Bags DBC pool does not match deterministic derivation for ${action}.`,
      {
        mint: mintKey.toBase58(),
        derivedPool: derivedPoolAddress.toBase58(),
        expectedPool: cached.preMigrationDbcPoolAddress.toBase58(),
      },
    );
  }
  if (!poolAccount.publicKey.equals(derivedPoolAddress)) {
    throw buildLocalTradeFailClosedError(
      "dbc_pool_not_derived",
      `Resolved DBC pool did not match deterministic derivation for canonical Bags ${action}.`,
      {
        mint: mintKey.toBase58(),
        resolvedPool: poolAccount.publicKey.toBase58(),
        derivedPool: derivedPoolAddress.toBase58(),
      },
    );
  }
  if (!Boolean(poolAccount.account.isMigrated) && !isCompletedDbcPool(poolAccount.account, configState)) {
    throw buildLocalTradeFailClosedError(
      action === "snapshot" ? "dbc_snapshot_failed" : "dbc_quote_failed_boundary",
      `Canonical Bags ${action} stayed on the local DBC path but returned no usable result.`,
      {
        mint: mintKey.toBase58(),
        pool: poolAccount.publicKey.toBase58(),
        configKey: configKey.toBase58(),
      },
    );
  }
  const dammResolution = deriveCachedDammPoolAddress(mintKey, cached);
  if (!dammResolution || !dammResolution.poolAddress) {
    throw buildLocalTradeFailClosedError(
      "migration_family_unresolved",
      `Canonical Bags ${action} could not resolve the migrated DAMM v2 family from cached launch metadata.`,
      {
        mint: mintKey.toBase58(),
        migrationFeeOption: Number.isFinite(cached.migrationFeeOption) ? cached.migrationFeeOption : "",
        expectedMigrationFamily: cached.expectedMigrationFamily,
        expectedDammConfigKey: cached.expectedDammConfigKey
          ? cached.expectedDammConfigKey.toBase58()
          : "",
      },
    );
  }
  const dammClient = new CpAmm(connection);
  const dammExists = await dammClient.isPoolExist(dammResolution.poolAddress).catch(() => false);
  if (!dammExists) {
    throw buildLocalTradeFailClosedError(
      "canonical_damm_pool_not_found",
      `Canonical Bags ${action} resolved to a migrated DAMM v2 pool that was not found on-chain.`,
      {
        mint: mintKey.toBase58(),
        pool: dammResolution.poolAddress.toBase58(),
        configKey: dammResolution.configAddress ? dammResolution.configAddress.toBase58() : "customizable",
      },
    );
  }
  const dammState = await dammClient.fetchPoolState(dammResolution.poolAddress).catch(() => null);
  if (!dammState) {
    throw buildLocalTradeFailClosedError(
      "canonical_damm_pool_not_found",
      `Canonical Bags ${action} resolved to a migrated DAMM v2 pool that could not be loaded.`,
      {
        mint: mintKey.toBase58(),
        pool: dammResolution.poolAddress.toBase58(),
      },
    );
  }
  throw buildLocalTradeFailClosedError(
    action === "snapshot" ? "damm_snapshot_failed" : "damm_quote_failed",
    `Canonical Bags ${action} resolved to the local DAMM v2 pool but returned no usable result.`,
    {
      mint: mintKey.toBase58(),
      pool: dammResolution.poolAddress.toBase58(),
      configKey: dammResolution.configAddress ? dammResolution.configAddress.toBase58() : "customizable",
    },
  );
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
  const heliusPriorityLevel = String(process.env.LAUNCHDECK_AUTO_FEE_HELIUS_PRIORITY_LEVEL || "high")
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

function bagsModeForPrePostFees(preFeePercent, postFeePercent) {
  const pre = Number(preFeePercent || 0);
  const post = Number(postFeePercent || 0);
  if (pre === 2 && post === 2) return "bags-2-2";
  if (pre === 0.25 && post === 1) return "bags-025-1";
  if (pre === 1 && post === 0.25) return "bags-1-025";
  return "";
}

function bagsModeFromDbcConfig(configState) {
  if (!configState) return "";
  return bagsModeForPrePostFees(
    Number(configState.creatorTradingFeePercentage || 0),
    Number(configState.creatorMigrationFeePercentage || 0),
  );
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
  const resolvedRows = await Promise.all(rows.map(async (row) => {
    const type = String(row && row.type || "wallet").trim().toLowerCase();
    const shareBps = Number(row && row.shareBps);
    if (!Number.isFinite(shareBps) || shareBps <= 0) return null;
    let wallet;
    if (type === "wallet") {
      wallet = new PublicKey(String(row.address || "").trim());
    } else if (["github", "twitter", "x", "kick", "tiktok"].includes(type)) {
      const cachedWallet = parseOptionalPublicKey(row && row.address);
      if (cachedWallet) {
        wallet = cachedWallet;
      } else {
      const normalizedType = type === "x" ? "twitter" : type;
      const socialHandle = String(row.githubUsername || "").trim().replace(/^@+/, "");
      const socialId = String(row.githubUserId || "").trim();
      const lookupTarget = normalizedType === "github"
        ? (socialHandle || socialId)
        : socialHandle;
      if (!lookupTarget) {
        throw new Error(
          normalizedType === "github"
            ? "Bags GitHub fee-share rows require a GitHub username or user id."
            : `Bags ${normalizedType} fee-share rows require a username.`,
        );
      }
      const result = await sdk.state.getLaunchWalletV2(lookupTarget, normalizedType);
      wallet = result.wallet;
      }
    } else {
      throw new Error(`Unsupported Bags fee-share recipient type: ${type}`);
    }
    return {
      walletBase58: wallet.toBase58(),
      shareBps,
    };
  }));
  for (const entry of resolvedRows) {
    if (!entry) continue;
    const { walletBase58, shareBps } = entry;
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

async function lookupFeeRecipient(request) {
  const apiKey = requireApiKey(request);
  const provider = String(request && request.provider || "").trim().toLowerCase();
  const normalizedType = provider === "x" ? "twitter" : provider;
  if (!["github", "twitter", "kick", "tiktok"].includes(normalizedType)) {
    throw new Error(`Unsupported Bags fee-share recipient type: ${provider || "(missing)"}`);
  }
  const socialHandle = String(
    request && (request.username || request.githubUsername) || "",
  ).trim().replace(/^@+/, "");
  const socialId = String(request && request.githubUserId || "").trim();
  const lookupTarget = normalizedType === "github"
    ? (socialHandle || socialId)
    : socialHandle;
  if (!lookupTarget) {
    throw new Error(
      normalizedType === "github"
        ? "Bags GitHub fee-share rows require a GitHub username or user id."
        : `Bags ${normalizedType} fee-share rows require a username.`,
    );
  }
  const rpcUrl = String(request && request.rpcUrl || process.env.SOLANA_RPC_URL || "").trim();
  if (!rpcUrl) {
    throw new Error("SOLANA_RPC_URL is required for Bagsapp integration.");
  }
  const commitment = request && request.commitment || "processed";
  const connection = new Connection(rpcUrl, commitment);
  const sdk = new BagsSDK(apiKey, connection, commitment);
  try {
    const result = await sdk.state.getLaunchWalletV2(lookupTarget, normalizedType);
    const walletValue = result && result.wallet != null ? result.wallet : "";
    const wallet = walletValue && typeof walletValue.toBase58 === "function"
      ? walletValue.toBase58()
      : String(walletValue || "").trim();
    return {
      found: Boolean(wallet),
      provider: normalizedType,
      lookupTarget,
      wallet,
      resolvedUsername: socialHandle,
      githubUserId: normalizedType === "github" ? socialId : "",
      notFound: false,
      error: "",
    };
  } catch (error) {
    const message = formatErrorDetails(error);
    return {
      found: false,
      provider: normalizedType,
      lookupTarget,
      wallet: "",
      resolvedUsername: socialHandle,
      githubUserId: normalizedType === "github" ? socialId : "",
      notFound: /status 404|not found/i.test(message),
      error: message,
    };
  }
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
  const prepareStartedAt = Date.now();
  const apiKey = requireApiKey(request);
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
  const feeRecipientResolveStartedAt = Date.now();
  const feeClaimers = await resolveFeeClaimers(sdk, owner.publicKey, request);
  const feeRecipientResolveMs = Date.now() - feeRecipientResolveStartedAt;
  if (feeClaimers.length > 15) {
    throw new Error("LaunchDeck Bags fee sharing currently supports up to 15 total claimers including the creator.");
  }

  const metadataUploadStartedAt = Date.now();
  const tokenInfo = await sdk.tokenLaunch.createTokenInfoAndMetadata({
    image: imageInputFromPath(request.imageLocalPath),
    name: String(request.token && request.token.name || "").trim(),
    symbol: String(request.token && request.token.symbol || "").trim(),
    description: String(request.token && request.token.description || "").trim(),
    website: String(request.token && request.token.website || "").trim() || undefined,
    twitter: String(request.token && request.token.twitter || "").trim() || undefined,
    telegram: String(request.token && request.token.telegram || "").trim() || undefined,
  });
  const metadataUploadMs = Date.now() - metadataUploadStartedAt;

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
  const configKey = configResult.meteoraConfigKey;
  const launchMigration = await summarizeLaunchMigrationConfig(
    connection,
    tokenMint,
    configKey,
    request.commitment || "processed",
  );

  const initialBuyLamports = request.devBuy && String(request.devBuy.amount || "").trim()
    ? Number(parseDecimalToBigInt(request.devBuy.amount, 9, "dev buy amount"))
    : 0;
  const blockhashOverride = parseBlockhashOverride(request);
  const sharedLastValidBlockHeight = blockhashOverride
    ? blockhashOverride.lastValidBlockHeight
    : (await connection.getLatestBlockhash(request.commitment || "confirmed")).lastValidBlockHeight;
  const directSetupTxConfig = request.txConfig;
  const bundledSetupTxConfig = request.txConfig && request.txConfig.singleBundleTipLastTx
    ? txConfigWithoutInlineTip(request.txConfig)
    : request.txConfig;
  const directSetupTransactions = await Promise.all(
    signTransactions(configResult.transactions, owner).map((transaction) =>
      ensureTxConfigOnTransaction(
        connection,
        owner,
        transaction,
        directSetupTxConfig,
        request.commitment,
        blockhashOverride,
      )
    )
  );
  const setupTransactions = normalizeTransactions(directSetupTransactions, {
    labelPrefix: "bags-config-direct",
    computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
    computeUnitPriceMicroLamports: Number(
      request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
    ) || null,
    inlineTipLamports: Number(directSetupTxConfig && directSetupTxConfig.tipLamports || 0) || null,
    inlineTipAccount: directSetupTxConfig && directSetupTxConfig.tipAccount
      ? String(directSetupTxConfig.tipAccount).trim()
      : null,
    lastValidBlockHeight: sharedLastValidBlockHeight,
  });
  const setupBundles = [];
  for (const [index, bundle] of (Array.isArray(configResult.bundles) ? configResult.bundles : []).entries()) {
    const signedBundleTransactions = await Promise.all(
      signTransactions(bundle, owner).map((transaction) =>
        ensureTxConfigOnTransaction(
          connection,
          owner,
          transaction,
          bundledSetupTxConfig,
          request.commitment,
          blockhashOverride,
        )
      )
    );
    const compiledBundleTransactions = normalizeTransactions(signedBundleTransactions, {
      labelPrefix: `bags-config-bundle-${index + 1}`,
      computeUnitLimit: Number(request.txConfig && request.txConfig.computeUnitLimit || 0) || null,
      computeUnitPriceMicroLamports: Number(
        request.txConfig && request.txConfig.computeUnitPriceMicroLamports || 0
      ) || null,
      inlineTipLamports: Number(bundledSetupTxConfig && bundledSetupTxConfig.tipLamports || 0) || null,
      inlineTipAccount: bundledSetupTxConfig && bundledSetupTxConfig.tipAccount
        ? String(bundledSetupTxConfig.tipAccount).trim()
        : null,
      lastValidBlockHeight: sharedLastValidBlockHeight,
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
    configKey: configKey.toBase58(),
    metadataUri: tokenInfo.tokenMetadata,
    identityLabel: String(request.identityLabel || "").trim(),
    migrationFeeOption: launchMigration.migrationFeeOption,
    expectedMigrationFamily: launchMigration.expectedMigrationFamily,
    expectedDammConfigKey: launchMigration.expectedDammConfigKey,
    expectedDammDerivationMode: launchMigration.expectedDammDerivationMode,
    preMigrationDbcPoolAddress: launchMigration.preMigrationDbcPoolAddress,
    compiledTransactions,
    setupBundles,
    setupTransactions,
    initialBuyLamports,
    timings: {
      prepareLaunchMs: Date.now() - prepareStartedAt,
      feeRecipientResolveMs,
      metadataUploadMs,
    },
  };
}

async function buildLaunchTransaction(request) {
  const launchBuildStartedAt = Date.now();
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
      launchError = new Error(formatErrorDetails(error));
      if (attempt === 4) break;
      await sleep(1200);
    }
  }
  if (!launchTransaction) {
    throw launchError || new Error("Failed to create Bags launch transaction.");
  }
  const blockhashOverride = parseBlockhashOverride(request);
  launchTransaction = await ensureTxConfigOnTransaction(
    connection,
    owner,
    signTransaction(launchTransaction, owner),
    request.txConfig,
    request.commitment,
    blockhashOverride,
  );
  const lastValidBlockHeight = blockhashOverride
    ? blockhashOverride.lastValidBlockHeight
    : (await connection.getLatestBlockhash(request.commitment || "confirmed")).lastValidBlockHeight;
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
    timings: {
      launchBuildMs: Date.now() - launchBuildStartedAt,
    },
  };
}

async function compileFollowBuy(request) {
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const local = await tryBuildLocalDbcFollowBuy(connection, owner, request);
  if (local) {
    return local;
  }
  const localDamm = await tryBuildLocalDammFollowBuy(connection, owner, request);
  if (localDamm) {
    return localDamm;
  }
  await failClosedBagsTradeError(
    connection,
    new PublicKey(request.mint),
    request.commitment || "processed",
    request,
    "buy",
  );
}

async function compileFollowSell(request) {
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const local = await tryBuildLocalDbcFollowSell(connection, owner, request);
  if (local) {
    return local;
  }
  const localDamm = await tryBuildLocalDammFollowSell(connection, owner, request);
  if (localDamm) {
    return localDamm;
  }
  await failClosedBagsTradeError(
    connection,
    new PublicKey(request.mint),
    request.commitment || "processed",
    request,
    "sell",
  );
}

async function fetchMarketSnapshot(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const mint = new PublicKey(request.mint);
  const local = await tryFetchLocalDbcMarketSnapshot(
    connection,
    mint,
    request.commitment || "processed",
    request.bagsLaunch,
  );
  if (local) {
    return local;
  }
  const localDamm = await tryFetchLocalDammMarketSnapshot(
    connection,
    mint,
    request.commitment || "processed",
    request.bagsLaunch,
  );
  if (localDamm) {
    return localDamm;
  }
  await failClosedBagsTradeError(
    connection,
    mint,
    request.commitment || "processed",
    request,
    "snapshot",
  );
}

async function detectLocalCanonicalImportMarket(connection, mint, commitment) {
  const dbcClient = new DynamicBondingCurveClient(connection, commitment || "processed");
  const poolAccount = await dbcClient.state.getPoolByBaseMint(mint).catch(() => null);
  if (!poolAccount || !poolAccount.account || !poolAccount.account.config) {
    return null;
  }
  const configKey = poolAccount.account.config;
  const configState = await dbcClient.state.getPoolConfig(configKey).catch(() => null);
  if (!configState || !configState.quoteMint || !configState.quoteMint.equals(NATIVE_MINT)) {
    return null;
  }
  const derivedPoolAddress = deriveDbcPoolAddress(configState.quoteMint, mint, configKey);
  if (!poolAccount.publicKey.equals(derivedPoolAddress)) {
    return null;
  }
  if (!Boolean(poolAccount.account.isMigrated) && !isCompletedDbcPool(poolAccount.account, configState)) {
    return {
      phase: "dbc",
      mode: bagsModeFromDbcConfig(configState),
      marketKey: poolAccount.publicKey.toBase58(),
      configKey: configKey.toBase58(),
      venue: "Meteora Dynamic Bonding Curve",
      detectionSource: "bags-state+rpc-dbc",
      notes: [
        "Recovered canonical pre-migration DBC market from RPC without Bags trade quotes.",
      ],
    };
  }
  if (Boolean(poolAccount.account.isMigrated)) {
    const poolAddress = deriveCanonicalDammPoolAddress(mint, configState);
    if (!poolAddress) {
      return null;
    }
    const dammClient = new CpAmm(connection);
    const exists = await dammClient.isPoolExist(poolAddress).catch(() => false);
    if (!exists) {
      return null;
    }
    const poolState = await dammClient.fetchPoolState(poolAddress).catch(() => null);
    if (!poolState) {
      return null;
    }
    const notes = [
      "Recovered canonical post-migration DAMM v2 market from RPC without Bags trade quotes.",
    ];
    const family = expectedMigrationFamilyFromConfig(configState);
    if (family) {
      notes.push(`Resolved migration family from DBC config: ${family}.`);
    }
    return {
      phase: "damm-v2",
      mode: bagsModeFromDbcConfig(configState),
      marketKey: poolAddress.toBase58(),
      configKey: configKey.toBase58(),
      venue: "Meteora DAMM v2",
      detectionSource: "bags-state+rpc-damm-v2",
      notes,
    };
  }
  return null;
}

async function detectImportContext(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const mint = new PublicKey(request.mint);
  const stored = readStoredBagsCredentials();
  const apiKey = String(request && request.apiKey || stored.apiKey || "").trim();
  const notes = [];
  let creators = [];
  if (apiKey) {
    const sdk = new BagsSDK(apiKey, connection, request.commitment || "processed");
    creators = await sdk.state.getTokenCreators(mint).catch((error) => {
      notes.push(`Bags creator routes could not be recovered from Bags state: ${formatErrorDetails(error)}.`);
      return [];
    });
  } else {
    notes.push("Bags creator routes were skipped because no Bags API key is configured.");
  }
  const creatorWallet = String(
    creators.find((entry) => entry && entry.isCreator)?.wallet || creators[0]?.wallet || "",
  ).trim();
  const feeRecipients = creators
    .filter((entry) => Number(entry && entry.royaltyBps || 0) > 0)
    .filter((entry) => {
      const wallet = String(entry && entry.wallet || "").trim();
      return !creatorWallet || wallet !== creatorWallet;
    })
    .map((entry) => {
      const provider = String(entry && entry.provider || "").trim().toLowerCase();
      const providerUsername = String(entry && (entry.providerUsername || entry.githubUsername || entry.twitterUsername) || "").trim().replace(/^@+/, "");
      if (provider && provider !== "github" && provider !== "solana" && provider !== "wallet" && providerUsername) {
        notes.push(`Recovered ${provider} fee route @${providerUsername} as wallet ${entry.wallet}.`);
      }
      const isSupportedSocial = ["github", "twitter", "x", "kick", "tiktok"].includes(provider);
      return isSupportedSocial && providerUsername
        ? {
          type: provider,
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
  const localMarket = await detectLocalCanonicalImportMarket(
    connection,
    mint,
    request.commitment || "processed",
  ).catch(() => null);
  if (localMarket) {
    mode = localMarket.mode;
    marketKey = localMarket.marketKey;
    configKey = localMarket.configKey;
    venue = localMarket.venue;
    detectionSource = localMarket.detectionSource;
    notes.push(...localMarket.notes);
  } else {
    notes.push("Canonical Bags market could not be recovered from RPC-only state.");
  }
  if (!localMarket && creators.length === 0) {
    return null;
  }
  if (!mode) {
    notes.push("Bags mode could not be recovered confidently from current market state.");
  }

  return {
    launchpad: "bagsapp",
    mode,
    quoteAsset: "sol",
    creator: creatorWallet,
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
    case "lookup-fee-recipient":
      response = await lookupFeeRecipient(request);
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
        error: formatErrorDetails(error),
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
  process.stderr.write(`${formatErrorDetails(error)}\n`);
  if (error && error.stack) {
    process.stderr.write(`${error.stack}\n`);
  }
  process.exit(1);
});
