"use strict";

require("dotenv").config({ quiet: true });

const bs58 = require("bs58");
const BN = require("bn.js");
const {
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
  createAssociatedTokenAccountIdempotentInstruction,
  getAssociatedTokenAddressSync,
} = require("@solana/spl-token");
const {
  Curve,
  LaunchpadConfig,
  PlatformConfig,
  Raydium,
  Token,
  TokenAmount,
  LAUNCHPAD_PROGRAM,
  TxVersion,
  getPdaLaunchpadConfigId,
  getPdaLaunchpadPoolId,
  getPdaLaunchpadVaultId,
} = require("@raydium-io/raydium-sdk-v2");

const FIXED_COMPUTE_UNIT_LIMIT = 1_000_000;
const TOKEN_DECIMALS = 6;
const PACKET_LIMIT = 1232;
const BASE64_PACKET_LIMIT = Math.ceil(PACKET_LIMIT / 3) * 4;
const BONK_USD1_SUPER_LOOKUP_TABLE = "GHVFasDr4sFtF2fMNBLnaRUKeSxX77DgK5SsThB3Ro7U";
const LETSBONK_PLATFORM = new PublicKey("FfYek5vEz23cMkWsdJwG2oa6EphsvXSHrGpdALN4g6W1");
const BONKERS_PLATFORM = new PublicKey("82NMHVCKwehXgbXMyzL41mvv3sdkypaMCtTxvJ4CtTzm");
const USD1_MINT = new PublicKey("USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB");
const RAYDIUM_ROUTE_PROGRAM = new PublicKey("routeUGWgWzqBWFcrCfv8tritsqukccJPu3q5GPP3xS");
const PINNED_USD1_ROUTE_POOL_ID = "AQAGYQsdU853WAKhXM79CgNdoyhrRwXvYHX6qrDyC1FS";
const PREFERRED_USD1_ROUTE_CONFIG = "E64NGkDLLCdQ2yFNPcavaKptrEgmiQaNykUuLC1Qgwyp";
const USD1_ROUTE_SETUP_CACHE = new Map();
const BONK_LAUNCH_DEFAULTS_CACHE = new Map();
const BONK_LAUNCH_DEFAULTS_IN_FLIGHT = new Map();
const RAYDIUM_LAUNCH_CONFIGS_CACHE = new Map();
const RAYDIUM_LAUNCH_CONFIGS_IN_FLIGHT = new Map();

function resolveQuoteAssetConfig(asset) {
  return String(asset || "").trim().toLowerCase() === "usd1"
    ? { asset: "usd1", label: "USD1", mint: USD1_MINT, decimals: 6 }
    : { asset: "sol", label: "SOL", mint: NATIVE_MINT, decimals: 9 };
}

function envFloat(name, fallback) {
  const value = Number(process.env[name]);
  return Number.isFinite(value) ? value : fallback;
}

function envInt(name, fallback) {
  const value = Number.parseInt(process.env[name] || "", 10);
  return Number.isFinite(value) ? value : fallback;
}

function bonkLaunchDefaultsCacheTtlMs() {
  return envInt("BONK_LAUNCH_DEFAULTS_CACHE_TTL_MS", 5000);
}

function getUsd1TopupPolicy() {
  return {
    maxPriceImpactPct: envFloat("BONK_USD1_MAX_PRICE_IMPACT_PCT", 5),
    minPoolTvlUsd: envFloat("BONK_USD1_MIN_POOL_TVL_USD", 100000),
    minRemainingSol: envFloat("BONK_USD1_MIN_REMAINING_SOL", 0.02),
    maxSearchIterations: envInt("BONK_USD1_MAX_INPUT_SEARCH_ITERATIONS", 10),
    routeSetupCacheTtlMs: envInt("BONK_USD1_ROUTE_SETUP_CACHE_TTL_MS", 5000),
    searchToleranceBps: envInt("BONK_USD1_SEARCH_TOLERANCE_BPS", 50),
    searchMinLamports: envInt("BONK_USD1_SEARCH_MIN_LAMPORTS", 50000),
    searchBufferBps: envInt("BONK_USD1_SEARCH_BUFFER_BPS", 25),
    searchBufferMinLamports: envInt("BONK_USD1_SEARCH_BUFFER_MIN_LAMPORTS", 25000),
  };
}

function createUsd1QuoteMetrics() {
  return {
    quoteCalls: 0,
    quoteTotalMs: 0,
    quoteCacheHits: 0,
    routeSetupLocalHits: 0,
    routeSetupCacheHits: 0,
    routeSetupCacheMisses: 0,
    routeSetupFetchMs: 0,
    expansionQuoteCalls: 0,
    binarySearchQuoteCalls: 0,
    bufferQuoteCalls: 0,
    searchIterations: 0,
  };
}

function createUsd1QuoteRequestContext() {
  return {
    localCache: new Map(),
    metrics: createUsd1QuoteMetrics(),
  };
}

function ensureUsd1QuoteRequestContext(context) {
  return context || createUsd1QuoteRequestContext();
}

function readTimedCache(map, key, ttlMs) {
  const cached = map.get(key);
  if (!cached) {
    return null;
  }
  if (Date.now() - cached.storedAtMs > ttlMs) {
    map.delete(key);
    return null;
  }
  return cached.value;
}

function writeTimedCache(map, key, value) {
  map.set(key, {
    storedAtMs: Date.now(),
    value,
  });
}

function readUsd1RouteSetupCache(key, ttlMs) {
  return readTimedCache(USD1_ROUTE_SETUP_CACHE, key, ttlMs);
}

function writeUsd1RouteSetupCache(key, value) {
  writeTimedCache(USD1_ROUTE_SETUP_CACHE, key, value);
}

function addUsd1QuoteMetric(metrics, key, amount = 1) {
  if (metrics && Object.prototype.hasOwnProperty.call(metrics, key)) {
    metrics[key] += amount;
  }
}

function formatUsd1QuoteMetrics(metrics) {
  if (!metrics || (!metrics.quoteCalls && !metrics.routeSetupCacheHits && !metrics.routeSetupLocalHits)) {
    return null;
  }
  return {
    quoteCalls: metrics.quoteCalls,
    quoteTotalMs: metrics.quoteTotalMs,
    averageQuoteMs: metrics.quoteCalls ? Number((metrics.quoteTotalMs / metrics.quoteCalls).toFixed(1)) : 0,
    quoteCacheHits: metrics.quoteCacheHits,
    routeSetupLocalHits: metrics.routeSetupLocalHits,
    routeSetupCacheHits: metrics.routeSetupCacheHits,
    routeSetupCacheMisses: metrics.routeSetupCacheMisses,
    routeSetupFetchMs: metrics.routeSetupFetchMs,
    expansionQuoteCalls: metrics.expansionQuoteCalls,
    binarySearchQuoteCalls: metrics.binarySearchQuoteCalls,
    bufferQuoteCalls: metrics.bufferQuoteCalls,
    searchIterations: metrics.searchIterations,
  };
}

function normalizeBonkLaunchMode(mode) {
  return String(mode || "").trim().toLowerCase() === "bonkers" ? "bonkers" : "regular";
}

function resolveBonkPlatform(mode) {
  const launchMode = normalizeBonkLaunchMode(mode);
  return {
    launchMode,
    platformId: launchMode === "bonkers" ? BONKERS_PLATFORM : LETSBONK_PLATFORM,
  };
}

function trimTrailingZeroes(value) {
  return value.replace(/\.?0+$/, "");
}

function formatBn(value, decimals, precision = 6) {
  const negative = value.isNeg();
  const absolute = negative ? value.neg() : value;
  const divisor = new BN(10).pow(new BN(decimals));
  const whole = absolute.div(divisor).toString(10);
  let fraction = absolute.mod(divisor).toString(10).padStart(decimals, "0");
  fraction = fraction.slice(0, precision);
  const rendered = fraction ? `${whole}.${fraction}` : whole;
  const trimmed = trimTrailingZeroes(rendered);
  return negative && trimmed !== "0" ? `-${trimmed}` : trimmed;
}

function parseDecimalToBn(raw, decimals, label) {
  const value = String(raw || "").trim();
  if (!value) throw new Error(`${label} is required.`);
  if (!/^\d+(\.\d+)?$/.test(value)) {
    throw new Error(`Invalid ${label}: ${value}`);
  }
  const [wholePart, fractionPart = ""] = value.split(".");
  const paddedFraction = `${fractionPart}${"0".repeat(decimals)}`.slice(0, decimals);
  return new BN(wholePart, 10)
    .mul(new BN(10).pow(new BN(decimals)))
    .add(new BN(paddedFraction || "0", 10));
}

function estimateSupplyPercent(amount, supply) {
  if (supply.isZero()) return "0";
  const scaled = amount.mul(new BN(100_000_000)).div(supply);
  return trimTrailingZeroes(formatBn(scaled, 6, 4));
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

function txVersionFromFormat(format) {
  return String(format || "").trim().toLowerCase() === "legacy"
    ? TxVersion.LEGACY
    : TxVersion.V0;
}

function atomicUsd1TxVersion(request) {
  return resolveQuoteAssetConfig(request && request.quoteAsset).asset === "usd1"
    ? TxVersion.V0
    : txVersionFromFormat(request && request.txFormat);
}

function readTransactionBlockhash(transaction) {
  if (transaction instanceof VersionedTransaction) {
    return transaction.message.recentBlockhash;
  }
  return transaction.recentBlockhash || "";
}

function serializeTransaction(transaction) {
  if (transaction instanceof VersionedTransaction) {
    return Buffer.from(transaction.serialize()).toString("base64");
  }
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

function extractTransactions(result) {
  return Array.isArray(result && result.transactions)
    ? result.transactions
    : result && result.transaction
      ? [result.transaction]
      : [];
}

function normalizeTransactions(result, { labelPrefix, computeUnitLimit, computeUnitPriceMicroLamports, inlineTipLamports, inlineTipAccount, lastValidBlockHeight }) {
  const transactions = extractTransactions(result);
  return transactions.map((transaction, index) => {
    const label = transactions.length === 1 ? labelPrefix : `${labelPrefix}-${index + 1}`;
    return {
      label,
      format: transaction instanceof VersionedTransaction ? "v0" : "legacy",
      blockhash: readTransactionBlockhash(transaction),
      lastValidBlockHeight,
      serializedBase64: serializeTransaction(transaction),
      lookupTablesUsed: lookupTablesUsedOnTransaction(transaction),
      computeUnitLimit: computeUnitLimit || null,
      computeUnitPriceMicroLamports: computeUnitPriceMicroLamports || null,
      inlineTipLamports: inlineTipLamports || null,
      inlineTipAccount: inlineTipLamports && inlineTipAccount ? inlineTipAccount : null,
      serializedLength: Buffer.from(serializeTransaction(transaction), "base64").length,
    };
  });
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForWalletTokenAccountVisibility(raydium, owner, mint, ata, commitment) {
  if (!raydium || !raydium.account || typeof raydium.account.fetchWalletTokenAccounts !== "function") {
    return false;
  }
  for (let attempt = 0; attempt < 6; attempt += 1) {
    const refreshed = await raydium.account.fetchWalletTokenAccounts({ forceUpdate: true, commitment });
    const visible = (refreshed.tokenAccountRawInfos || []).some((entry) => (
      entry.pubkey.equals(ata) || entry.accountInfo.mint.equals(mint)
    ));
    if (visible) {
      return true;
    }
    if (attempt < 5) {
      await sleep(400 * (attempt + 1));
    }
  }
  return false;
}

async function ensureAssociatedTokenAccountExists(connection, owner, mint, request, raydium) {
  const commitment = request.commitment || "confirmed";
  const mintInfo = await connection.getAccountInfo(mint, commitment);
  if (!mintInfo) {
    throw new Error(`Token mint account not found: ${mint.toBase58()}`);
  }
  const tokenProgramId = mintInfo.owner;
  const ata = getAssociatedTokenAddressSync(mint, owner.publicKey, false, tokenProgramId);
  const existingAta = await connection.getAccountInfo(ata, commitment);
  if (existingAta) {
    const visible = await waitForWalletTokenAccountVisibility(
      raydium,
      owner.publicKey,
      mint,
      ata,
      commitment,
    );
    if (!visible) {
      throw new Error(`Associated token account exists on-chain but is not yet visible to Raydium: ${ata.toBase58()}`);
    }
    return ata;
  }
  const transaction = new Transaction();
  transaction.feePayer = owner.publicKey;
  if (request.txConfig && request.txConfig.computeUnitPriceMicroLamports) {
    transaction.add(
      ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: Number(request.txConfig.computeUnitPriceMicroLamports),
      }),
    );
  }
  if (request.txConfig && request.txConfig.computeUnitLimit) {
    transaction.add(
      ComputeBudgetProgram.setComputeUnitLimit({
        units: Number(request.txConfig.computeUnitLimit),
      }),
    );
  }
  transaction.add(
    createAssociatedTokenAccountIdempotentInstruction(
      owner.publicKey,
      ata,
      owner.publicKey,
      mint,
      tokenProgramId,
    ),
  );
  const tipInstruction = buildInlineTipInstruction(
    owner.publicKey,
    request.txConfig && request.txConfig.tipAccount,
    request.txConfig && request.txConfig.tipLamports,
  );
  if (tipInstruction) {
    transaction.add(tipInstruction);
  }
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash(commitment);
  transaction.recentBlockhash = blockhash;
  transaction.sign(owner);
  const signature = await connection.sendRawTransaction(transaction.serialize(), {
    preflightCommitment: commitment,
  });
  const confirmation = await connection.confirmTransaction(
    { signature, blockhash, lastValidBlockHeight },
    commitment,
  );
  if (confirmation && confirmation.value && confirmation.value.err) {
    throw new Error(`USD1 ATA creation failed: ${JSON.stringify(confirmation.value.err)}`);
  }
  const visible = await waitForWalletTokenAccountVisibility(
    raydium,
    owner.publicKey,
    mint,
    ata,
    commitment,
  );
  if (!visible) {
    throw new Error(`Created associated token account is not yet visible to Raydium: ${ata.toBase58()}`);
  }
  return ata;
}

async function ensureQuoteTokenAccountReady(connection, owner, request, raydium, mintOverride) {
  if (!allowAtaCreation(request)) {
    return null;
  }
  const quote = resolveQuoteAssetConfig(request && request.quoteAsset);
  if (quote.asset === "sol") {
    return null;
  }
  const mint = mintOverride || quote.mint;
  return ensureAssociatedTokenAccountExists(connection, owner, mint, request, raydium);
}

function allowAtaCreation(request) {
  return Boolean(request && request.allowAtaCreation);
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

function mergeLookupTableAccounts(...lists) {
  const merged = new Map();
  for (const list of lists) {
    for (const account of list || []) {
      merged.set(account.key.toBase58(), account);
    }
  }
  return Array.from(merged.values());
}

async function resolveLookupTableAccountsForAddresses(connection, addresses) {
  const resolved = await Promise.all(addresses.map(async (address) => {
    const key = new PublicKey(address);
    const response = await connection.getAddressLookupTable(key);
    if (!response || !response.value) {
      throw new Error(`Address lookup table not found: ${key.toBase58()}`);
    }
    return response.value;
  }));
  return resolved;
}

async function resolveBonkUsd1AtomicLookupTables(connection, request) {
  if (resolveQuoteAssetConfig(request && request.quoteAsset).asset !== "usd1") {
    return [];
  }
  return resolveLookupTableAccountsForAddresses(connection, [BONK_USD1_SUPER_LOOKUP_TABLE]);
}

function estimatedAccountKeyCount(ownerPubkey, instructions) {
  const keys = new Set([ownerPubkey.toBase58()]);
  for (const instruction of instructions || []) {
    if (instruction.programId) {
      keys.add(instruction.programId.toBase58());
    }
    for (const key of instruction.keys || []) {
      if (key && key.pubkey) {
        keys.add(key.pubkey.toBase58());
      }
    }
  }
  return keys.size;
}

function lookupTableAddressCount(lookupTables) {
  return (lookupTables || []).reduce((total, table) => (
    total + (((table && table.state && table.state.addresses) || []).length)
  ), 0);
}

function printAtomicUsd1Diagnostics(label, diagnostics) {
  console.error(`[bonk-usd1-atomic] ${label}: ${JSON.stringify(diagnostics)}`);
}

function versionedMessageSerializedLength(message) {
  try {
    return Buffer.from(message.serialize()).length;
  } catch (_error) {
    return null;
  }
}

function isComputeBudgetInstruction(instruction) {
  return instruction.programId && instruction.programId.equals(ComputeBudgetProgram.programId);
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

function buildAtomicEnvelopeInstructions(txConfig) {
  const instructions = [];
  if (txConfig && txConfig.computeUnitPriceMicroLamports) {
    instructions.push(
      ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: Number(txConfig.computeUnitPriceMicroLamports),
      }),
    );
  }
  if (txConfig && txConfig.computeUnitLimit) {
    instructions.push(
      ComputeBudgetProgram.setComputeUnitLimit({
        units: Number(txConfig.computeUnitLimit),
      }),
    );
  }
  return instructions;
}

function isAtomicMessageOverflowError(error) {
  const message = error && error.message ? error.message : String(error || "");
  return message.includes("encoding overruns Uint8Array")
    || message.includes("Transaction too large")
    || message.includes("encoding overruns");
}

async function ensureInlineTipOnTransaction(connection, owner, transaction, txConfig) {
  const tipInstruction = buildInlineTipInstruction(
    owner.publicKey,
    txConfig && txConfig.tipAccount,
    txConfig && txConfig.tipLamports,
  );
  if (!tipInstruction) {
    return transaction;
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
      return transaction;
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
    return transaction;
  }
  const rebuilt = new Transaction();
  rebuilt.feePayer = owner.publicKey;
  rebuilt.recentBlockhash = readTransactionBlockhash(transaction);
  instructions.forEach((instruction) => rebuilt.add(instruction));
  rebuilt.add(tipInstruction);
  rebuilt.sign(owner);
  return rebuilt;
}

async function ensureInlineTipOnSwapResult(connection, owner, result, txConfig) {
  const transactions = extractTransactions(result);
  if (!transactions.length || !txConfig || !txConfig.tipLamports || !txConfig.tipAccount) {
    return result;
  }
  const rebuiltTransactions = [];
  for (const transaction of transactions) {
    rebuiltTransactions.push(await ensureInlineTipOnTransaction(connection, owner, transaction, txConfig));
  }
  return rebuiltTransactions.length === 1
    ? { transaction: rebuiltTransactions[0] }
    : { transactions: rebuiltTransactions };
}

async function buildBonkUsd1LookupTableCandidates(connection, request, baseLookupTables) {
  const customLookupTables = await resolveBonkUsd1AtomicLookupTables(connection, request);
  const mergedLookupTables = mergeLookupTableAccounts(customLookupTables, baseLookupTables);
  const candidates = [];
  if (customLookupTables.length) {
    candidates.push({
      label: "custom-only",
      lookupTables: customLookupTables,
    });
  }
  candidates.push({
    label: customLookupTables.length ? "custom+merged" : "sdk-merged",
    lookupTables: mergedLookupTables,
  });
  return { customLookupTables, candidates };
}

function compileVersionedTransactionWithLookupTables(owner, recentBlockhash, instructions, lookupTables, extraSigners = []) {
  const message = new TransactionMessage({
    payerKey: owner.publicKey,
    recentBlockhash,
    instructions,
  }).compileToV0Message(lookupTables);
  const transaction = new VersionedTransaction(message);
  transaction.sign([owner, ...extraSigners]);
  return { message, transaction };
}

async function preferBonkUsd1LookupTableOnTransaction(connection, owner, request, transaction, extraSigners = []) {
  if (!(transaction instanceof VersionedTransaction)) {
    return transaction;
  }
  if (resolveQuoteAssetConfig(request && request.quoteAsset).asset !== "usd1") {
    return transaction;
  }
  const { instructions, addressLookupTableAccounts } = await decompileTransactionInstructions(connection, transaction);
  const { candidates } = await buildBonkUsd1LookupTableCandidates(connection, request, addressLookupTableAccounts);
  const originalLookupCount = lookupTablesUsedOnTransaction(transaction).length;
  let bestTransaction = transaction;
  let bestLookupCount = originalLookupCount;
  for (const candidate of candidates) {
    try {
      const rebuilt = compileVersionedTransactionWithLookupTables(
        owner,
        readTransactionBlockhash(transaction),
        instructions,
        candidate.lookupTables,
        extraSigners,
      ).transaction;
      const rebuiltLookupCount = lookupTablesUsedOnTransaction(rebuilt).length;
      if (
        rebuiltLookupCount < bestLookupCount
        || (
          rebuiltLookupCount === bestLookupCount
          && candidate.label === "custom-only"
          && rebuiltLookupCount <= 1
        )
      ) {
        bestTransaction = rebuilt;
        bestLookupCount = rebuiltLookupCount;
      }
      if (bestLookupCount <= 1) {
        return bestTransaction;
      }
    } catch (_error) {
      // Preserve the SDK-built transaction if the custom remap cannot compile cleanly.
    }
  }
  return bestTransaction;
}

async function combineAtomicUsd1ActionTransaction(connection, owner, request, swapTransaction, actionTransaction, extraSigners = []) {
  const [swapBundle, actionBundle] = await Promise.all([
    decompileTransactionInstructions(connection, swapTransaction),
    decompileTransactionInstructions(connection, actionTransaction),
  ]);
  const swapInstructions = swapBundle.instructions.filter((instruction) => (
    !isComputeBudgetInstruction(instruction)
    && !isInlineTipInstruction(
      instruction,
      owner.publicKey,
      request.txConfig && request.txConfig.tipAccount,
      request.txConfig && request.txConfig.tipLamports,
    )
  ));
  const actionInstructions = actionBundle.instructions.filter((instruction) => (
    !isComputeBudgetInstruction(instruction)
    && !isInlineTipInstruction(
      instruction,
      owner.publicKey,
      request.txConfig && request.txConfig.tipAccount,
      request.txConfig && request.txConfig.tipLamports,
    )
  ));
  const instructions = [
    ...buildAtomicEnvelopeInstructions(request.txConfig),
    ...swapInstructions,
    ...actionInstructions,
  ];
  const tipInstruction = buildInlineTipInstruction(
    owner.publicKey,
    request.txConfig && request.txConfig.tipAccount,
    request.txConfig && request.txConfig.tipLamports,
  );
  if (tipInstruction) {
    instructions.push(tipInstruction);
  }
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  const txVersion = atomicUsd1TxVersion(request);
  const baseLookupTables = mergeLookupTableAccounts(
    swapBundle.addressLookupTableAccounts,
    actionBundle.addressLookupTableAccounts,
  );
  const { customLookupTables, candidates } = await buildBonkUsd1LookupTableCandidates(
    connection,
    request,
    baseLookupTables,
  );
  const precompileDiagnostics = {
    txVersion: txVersion === TxVersion.LEGACY ? "legacy" : "v0",
    swapInstructionCount: swapInstructions.length,
    actionInstructionCount: actionInstructions.length,
    mergedInstructionCount: instructions.length,
    estimatedAccountKeyCount: estimatedAccountKeyCount(owner.publicKey, instructions),
    lookupTableCount: baseLookupTables.length,
    lookupTableAddressCount: lookupTableAddressCount(baseLookupTables),
    customLookupTableCount: customLookupTables.length,
    customLookupTables: customLookupTables.map((table) => table.key.toBase58()),
  };
  printAtomicUsd1Diagnostics("precompile", precompileDiagnostics);
  if (txVersion === TxVersion.LEGACY) {
    const transaction = new Transaction();
    transaction.feePayer = owner.publicKey;
    transaction.recentBlockhash = blockhash;
    instructions.forEach((instruction) => transaction.add(instruction));
    transaction.sign(owner, ...extraSigners);
    const serialized = transaction.serialize();
    printAtomicUsd1Diagnostics("compiled", {
      ...precompileDiagnostics,
      serializedBytes: serialized.length,
      serializedBase64Length: Buffer.from(serialized).toString("base64").length,
      packetLimit: PACKET_LIMIT,
    });
    if (serialized.length > PACKET_LIMIT) {
      throw new Error(
        `Atomic USD1 action exceeded packet limits after serialize: raw ${serialized.length} > ${PACKET_LIMIT} bytes`,
      );
    }
    return { transaction, lastValidBlockHeight };
  }
  let lastError = null;
  for (const candidate of candidates) {
    let message;
    try {
      message = new TransactionMessage({
        payerKey: owner.publicKey,
        recentBlockhash: blockhash,
        instructions,
      }).compileToV0Message(candidate.lookupTables);
    } catch (error) {
      lastError = error;
      printAtomicUsd1Diagnostics("overflow", {
        ...precompileDiagnostics,
        lookupStrategy: candidate.label,
        lookupTableCount: candidate.lookupTables.length,
        lookupTableAddressCount: lookupTableAddressCount(candidate.lookupTables),
        error: error && error.message ? error.message : String(error),
      });
      continue;
    }
    const compiledDiagnostics = {
      ...precompileDiagnostics,
      lookupStrategy: candidate.label,
      lookupTableCount: candidate.lookupTables.length,
      lookupTableAddressCount: lookupTableAddressCount(candidate.lookupTables),
      staticAccountKeyCount: message.staticAccountKeys.length,
      staticAccountKeys: message.staticAccountKeys.map((key) => key.toBase58()),
      mergedLookupTables: candidate.lookupTables.map((table) => table.key.toBase58()),
      lookupReferenceCount: (message.addressTableLookups || []).reduce(
        (total, lookup) => total + lookup.readonlyIndexes.length + lookup.writableIndexes.length,
        0,
      ),
      lookupTableLookups: (message.addressTableLookups || []).map((lookup) => ({
        table: lookup.accountKey.toBase58(),
        writableIndexes: lookup.writableIndexes.length,
        readonlyIndexes: lookup.readonlyIndexes.length,
      })),
      messageSerializedBytes: versionedMessageSerializedLength(message),
      packetLimit: PACKET_LIMIT,
    };
    let transaction;
    try {
      transaction = new VersionedTransaction(message);
      transaction.sign([owner, ...extraSigners]);
    } catch (error) {
      lastError = error;
      printAtomicUsd1Diagnostics("sign-overflow", {
        ...compiledDiagnostics,
        error: error && error.message ? error.message : String(error),
      });
      continue;
    }
    try {
      const serialized = transaction.serialize();
      compiledDiagnostics.serializedBytes = serialized.length;
      compiledDiagnostics.serializedBase64Length = Buffer.from(serialized).toString("base64").length;
      printAtomicUsd1Diagnostics("compiled", compiledDiagnostics);
      if (
        compiledDiagnostics.serializedBytes > PACKET_LIMIT
        || compiledDiagnostics.serializedBase64Length > BASE64_PACKET_LIMIT
      ) {
        lastError = new Error(
          `Atomic USD1 action exceeded packet limits after serialize: raw ${compiledDiagnostics.serializedBytes} > ${PACKET_LIMIT} bytes or base64 ${compiledDiagnostics.serializedBase64Length} > ${BASE64_PACKET_LIMIT}`,
        );
        printAtomicUsd1Diagnostics("packet-limit", {
          ...compiledDiagnostics,
          error: lastError.message,
        });
        continue;
      }
      return { transaction, lastValidBlockHeight };
    } catch (error) {
      lastError = error;
      printAtomicUsd1Diagnostics("serialize-overflow", {
        ...compiledDiagnostics,
        error: error && error.message ? error.message : String(error),
      });
    }
  }
  throw lastError || new Error("Atomic USD1 action assembly failed.");
}

async function loadLaunchDefaults(raydium, connection, ownerPubkey, mode = "regular", quoteAsset = "sol") {
  const launchMode = normalizeBonkLaunchMode(mode);
  const quote = resolveQuoteAssetConfig(quoteAsset);
  const cacheKey = `${launchMode}:${quote.asset}`;
  const ttlMs = bonkLaunchDefaultsCacheTtlMs();
  const cached = readTimedCache(BONK_LAUNCH_DEFAULTS_CACHE, cacheKey, ttlMs);
  const staticDefaults = cached || await (async () => {
    if (BONK_LAUNCH_DEFAULTS_IN_FLIGHT.has(cacheKey)) {
      return BONK_LAUNCH_DEFAULTS_IN_FLIGHT.get(cacheKey);
    }
    const loader = (async () => {
      const { platformId } = resolveBonkPlatform(launchMode);
      const configId = getPdaLaunchpadConfigId(LAUNCHPAD_PROGRAM, quote.mint, 0, 0).publicKey;
      const launchConfigsKey = "launch-configs";
      const launchConfigsCached = readTimedCache(
        RAYDIUM_LAUNCH_CONFIGS_CACHE,
        launchConfigsKey,
        ttlMs,
      );
      const launchConfigsPromise = launchConfigsCached
        ? Promise.resolve(launchConfigsCached)
        : (() => {
          if (RAYDIUM_LAUNCH_CONFIGS_IN_FLIGHT.has(launchConfigsKey)) {
            return RAYDIUM_LAUNCH_CONFIGS_IN_FLIGHT.get(launchConfigsKey);
          }
          const promise = raydium.api.fetchLaunchConfigs()
            .then((value) => {
              writeTimedCache(RAYDIUM_LAUNCH_CONFIGS_CACHE, launchConfigsKey, value);
              return value;
            })
            .finally(() => {
              RAYDIUM_LAUNCH_CONFIGS_IN_FLIGHT.delete(launchConfigsKey);
            });
          RAYDIUM_LAUNCH_CONFIGS_IN_FLIGHT.set(launchConfigsKey, promise);
          return promise;
        })();
      const [configAccount, platformAccount, launchConfigs] = await Promise.all([
        connection.getAccountInfo(configId),
        connection.getAccountInfo(platformId),
        launchConfigsPromise,
      ]);
      if (!configAccount) {
        throw new Error(`Launch config account not found: ${configId.toBase58()}`);
      }
      if (!platformAccount) {
        throw new Error(`Launch platform account not found: ${platformId.toBase58()}`);
      }
      const apiConfig = launchConfigs.find((entry) => entry.key.pubKey === configId.toBase58());
      if (!apiConfig) {
        throw new Error(`Raydium launch config defaults not found for ${configId.toBase58()}`);
      }
      const configInfo = LaunchpadConfig.decode(configAccount.data);
      const platformInfo = PlatformConfig.decode(platformAccount.data);
      const supply = new BN(apiConfig.defaultParams.supplyInit);
      const totalSellA = new BN(apiConfig.defaultParams.totalSellA);
      const totalFundRaisingB = new BN(apiConfig.defaultParams.totalFundRaisingB);
      const totalLockedAmount = new BN(0);
      const init = Curve.getCurve(configInfo.curveType).getInitParam({
        supply,
        totalFundRaising: totalFundRaisingB,
        totalSell: totalSellA,
        totalLockedAmount,
        migrateFee: configInfo.migrateFee,
      });
      const dummyMint = Keypair.generate().publicKey;
      const dummyPoolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, dummyMint, quote.mint).publicKey;
      const defaults = {
        mode: launchMode,
        configId,
        configInfo,
        platformInfo,
        platformId,
        supply,
        totalFundRaisingB,
        quoteAsset: quote.asset,
        quoteAssetLabel: quote.label,
        quoteMint: quote.mint,
        quoteDecimals: quote.decimals,
        poolInfoTemplate: {
          epoch: new BN(0),
          bump: 0,
          status: 0,
          mintDecimalsA: TOKEN_DECIMALS,
          mintDecimalsB: quote.decimals,
          supply,
          totalSellA,
          mintA: dummyMint,
          mintB: quote.mint,
          virtualA: init.a,
          virtualB: init.b,
          realA: new BN(0),
          realB: new BN(0),
          migrateFee: configInfo.migrateFee,
          migrateType: 1,
          protocolFee: new BN(0),
          platformFee: platformInfo.feeRate,
          platformId,
          configId,
          vaultA: getPdaLaunchpadVaultId(LAUNCHPAD_PROGRAM, dummyPoolId, dummyMint).publicKey,
          vaultB: getPdaLaunchpadVaultId(LAUNCHPAD_PROGRAM, dummyPoolId, quote.mint).publicKey,
          creator: PublicKey.default,
          totalFundRaisingB,
          vestingSchedule: {
            totalLockedAmount,
            cliffPeriod: new BN(0),
            unlockPeriod: new BN(0),
            startTime: new BN(0),
            totalAllocatedShare: new BN(0),
          },
          mintProgramFlag: 0,
          cpmmCreatorFeeOn: 0,
          platformVestingShare: platformInfo.platformVestingScale || new BN(0),
          configInfo,
          quoteAsset: quote.asset,
          quoteAssetLabel: quote.label,
          quoteMint: quote.mint,
          quoteDecimals: quote.decimals,
        },
      };
      writeTimedCache(BONK_LAUNCH_DEFAULTS_CACHE, cacheKey, defaults);
      return defaults;
    })().finally(() => {
      BONK_LAUNCH_DEFAULTS_IN_FLIGHT.delete(cacheKey);
    });
    BONK_LAUNCH_DEFAULTS_IN_FLIGHT.set(cacheKey, loader);
    return loader;
  })();
  const creator = ownerPubkey || PublicKey.default;
  return {
    ...staticDefaults,
    poolInfo: {
      ...staticDefaults.poolInfoTemplate,
      creator,
    },
  };
}

function buildPrelaunchPoolInfo(defaults, mint, creator) {
  const poolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, mint, defaults.quoteMint).publicKey;
  return {
    ...defaults.poolInfo,
    poolId,
    mintA: mint,
    vaultA: getPdaLaunchpadVaultId(LAUNCHPAD_PROGRAM, poolId, mint).publicKey,
    vaultB: getPdaLaunchpadVaultId(LAUNCHPAD_PROGRAM, poolId, defaults.quoteMint).publicKey,
    creator,
  };
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function loadLivePoolContext(raydium, connection, mint, quoteAsset) {
  const requestedQuote = resolveQuoteAssetConfig(quoteAsset);
  const candidateAssets = requestedQuote.asset === "usd1"
    ? [requestedQuote, resolveQuoteAssetConfig("sol")]
    : [requestedQuote, resolveQuoteAssetConfig("usd1")];
  const errors = [];
  for (const quote of candidateAssets) {
    const poolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, mint, quote.mint).publicKey;
    for (let attempt = 0; attempt < 6; attempt += 1) {
      try {
        const poolInfo = await raydium.launchpad.getRpcPoolInfo({ poolId });
        const configId = poolInfo.configId && poolInfo.configId.toBase58
          ? poolInfo.configId
          : new PublicKey(String(poolInfo.configId || ""));
        const platformId = poolInfo.platformId && poolInfo.platformId.toBase58
          ? poolInfo.platformId
          : new PublicKey(String(poolInfo.platformId || ""));
        const [configAccount, platformAccount] = await Promise.all([
          connection.getAccountInfo(configId),
          connection.getAccountInfo(platformId),
        ]);
        if (!configAccount) {
          throw new Error(`Launch config account not found: ${configId.toBase58()}`);
        }
        if (!platformAccount) {
          throw new Error(`Launch platform account not found: ${platformId.toBase58()}`);
        }
        return {
          poolId,
          poolInfo,
          configId,
          platformId,
          configInfo: LaunchpadConfig.decode(configAccount.data),
          platformInfo: PlatformConfig.decode(platformAccount.data),
          quoteAsset: quote.asset,
          quoteAssetLabel: quote.label,
          quoteMint: quote.mint,
          quoteDecimals: quote.decimals,
        };
      } catch (error) {
        errors.push(`${quote.asset}:${poolId.toBase58()}: ${error && error.message ? error.message : String(error)}`);
        if (attempt < 5) {
          await sleep(200);
        }
      }
    }
  }
  throw new Error(`Unable to resolve Bonk live pool context. Attempts: ${errors.join(" | ")}`);
}

async function loadPoolContextByPoolId(raydium, connection, poolIdInput, quoteAsset) {
  const poolId = new PublicKey(poolIdInput);
  const poolInfo = await raydium.launchpad.getRpcPoolInfo({ poolId });
  const configId = poolInfo.configId && poolInfo.configId.toBase58
    ? poolInfo.configId
    : new PublicKey(String(poolInfo.configId || ""));
  const platformId = poolInfo.platformId && poolInfo.platformId.toBase58
    ? poolInfo.platformId
    : new PublicKey(String(poolInfo.platformId || ""));
  const [configAccount, platformAccount] = await Promise.all([
    connection.getAccountInfo(configId),
    connection.getAccountInfo(platformId),
  ]);
  if (!configAccount) {
    throw new Error(`Launch config account not found: ${configId.toBase58()}`);
  }
  if (!platformAccount) {
    throw new Error(`Launch platform account not found: ${platformId.toBase58()}`);
  }
  const quote = resolveQuoteAssetConfig(quoteAsset);
  return {
    poolId,
    poolInfo,
    configId,
    platformId,
    configInfo: LaunchpadConfig.decode(configAccount.data),
    platformInfo: PlatformConfig.decode(platformAccount.data),
    quoteAsset: quote.asset,
    quoteAssetLabel: quote.label,
    quoteMint: quote.mint,
    quoteDecimals: quote.decimals,
  };
}

async function buildPrelaunchPoolContext(raydium, connection, mint, launchCreator, mode, quoteAsset) {
  const creator = new PublicKey(launchCreator);
  const defaults = await loadLaunchDefaults(raydium, connection, creator, mode, quoteAsset);
  const poolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, mint, defaults.quoteMint).publicKey;
  return {
    poolId,
    poolInfo: buildPrelaunchPoolInfo(defaults, mint, creator),
    configId: defaults.configId,
    platformId: defaults.platformId,
    configInfo: defaults.configInfo,
    platformInfo: defaults.platformInfo,
    quoteAsset: defaults.quoteAsset,
    quoteAssetLabel: defaults.quoteAssetLabel,
    quoteMint: defaults.quoteMint,
    quoteDecimals: defaults.quoteDecimals,
  };
}

function buildQuote(defaults, mode, amount) {
  const common = {
    poolInfo: defaults.poolInfo,
    protocolFeeRate: defaults.configInfo.tradeFeeRate,
    platformFeeRate: defaults.platformInfo.feeRate,
    curveType: defaults.configInfo.curveType,
    shareFeeRate: new BN(0),
    creatorFeeRate: defaults.platformInfo.creatorFeeRate,
    transferFeeConfigA: undefined,
    slot: 0,
  };
  if (mode === "tokens") {
    const tokenAmount = parseDecimalToBn(amount, TOKEN_DECIMALS, "buy amount");
    const quote = Curve.buyExactOut({
      ...common,
      amountA: tokenAmount,
    });
    return {
      mode,
      input: amount,
      estimatedTokens: formatBn(tokenAmount, TOKEN_DECIMALS, 6),
      estimatedSol: formatBn(quote.amountB, defaults.quoteDecimals, 6),
      estimatedQuoteAmount: formatBn(quote.amountB, defaults.quoteDecimals, 6),
      quoteAsset: defaults.quoteAsset,
      quoteAssetLabel: defaults.quoteAssetLabel,
      estimatedSupplyPercent: estimateSupplyPercent(tokenAmount, defaults.supply),
    };
  }
  const buyAmount = parseDecimalToBn(amount, defaults.quoteDecimals, `buy amount ${defaults.quoteAssetLabel}`);
  const quote = Curve.buyExactIn({
    ...common,
    amountB: buyAmount,
  });
  return {
    mode,
    input: amount,
    estimatedTokens: formatBn(quote.amountA.amount, TOKEN_DECIMALS, 6),
    estimatedSol: formatBn(buyAmount, defaults.quoteDecimals, 6),
    estimatedQuoteAmount: formatBn(buyAmount, defaults.quoteDecimals, 6),
    quoteAsset: defaults.quoteAsset,
    quoteAssetLabel: defaults.quoteAssetLabel,
    estimatedSupplyPercent: estimateSupplyPercent(quote.amountA.amount, defaults.supply),
  };
}

function buildCurveQuoteCommon(defaults) {
  return {
    poolInfo: defaults.poolInfo,
    protocolFeeRate: defaults.configInfo.tradeFeeRate,
    platformFeeRate: defaults.platformInfo.feeRate,
    curveType: defaults.configInfo.curveType,
    shareFeeRate: new BN(0),
    creatorFeeRate: defaults.platformInfo.creatorFeeRate,
    transferFeeConfigA: undefined,
    slot: 0,
  };
}

async function estimateDevBuyTokenAmount(raydium, connection, defaults, devBuy, slippageBps, requestContext = null) {
  if (!devBuy || !devBuy.mode || !devBuy.amount) {
    return null;
  }
  if (devBuy.mode === "tokens") {
    const requested = parseDecimalToBn(devBuy.amount, TOKEN_DECIMALS, "dev buy tokens");
    return buildMinAmountFromBps(requested, slippageBps);
  }
  if (defaults.quoteAsset === "usd1") {
    const solInput = parseDecimalToBn(devBuy.amount, 9, "dev buy SOL");
    const usd1RouteQuote = await quoteUsd1OutputFromSolInput(
      raydium,
      connection,
      solInput,
      slippageBps,
      requestContext,
    );
    const curveQuote = Curve.buyExactIn({
      ...buildCurveQuoteCommon(defaults),
      amountB: usd1RouteQuote.minOut,
    });
    return new BN(curveQuote.amountA.amount.toString());
  }
  const buyAmount = parseDecimalToBn(
    devBuy.amount,
    defaults.quoteDecimals,
    `dev buy ${defaults.quoteAssetLabel}`,
  );
  const curveQuote = Curve.buyExactIn({
    ...buildCurveQuoteCommon(defaults),
    amountB: buyAmount,
  });
  return new BN(curveQuote.amountA.amount.toString());
}

function buildUsd1RouteSetupCacheKey() {
  return `usd1-route-setup:${PINNED_USD1_ROUTE_POOL_ID}:${PREFERRED_USD1_ROUTE_CONFIG}`;
}

function toBasicPoolInfo(pool) {
  const version = pool.type === "Concentrated" ? 6 : pool.type === "Standard" ? 4 : 7;
  return {
    id: new PublicKey(pool.id),
    version,
    mintA: new PublicKey(pool.mintA.address || pool.mintA),
    mintB: new PublicKey(pool.mintB.address || pool.mintB),
  };
}

async function loadUsd1RouteSetup(raydium, connection, requestContext = null) {
  const context = ensureUsd1QuoteRequestContext(requestContext);
  const policy = getUsd1TopupPolicy();
  const cacheKey = buildUsd1RouteSetupCacheKey();
  if (context.localCache.has(cacheKey)) {
    addUsd1QuoteMetric(context.metrics, "routeSetupLocalHits");
    return context.localCache.get(cacheKey);
  }
  const cached = readUsd1RouteSetupCache(cacheKey, policy.routeSetupCacheTtlMs);
  if (cached) {
    addUsd1QuoteMetric(context.metrics, "routeSetupCacheHits");
    context.localCache.set(cacheKey, cached);
    return cached;
  }
  addUsd1QuoteMetric(context.metrics, "routeSetupCacheMisses");
  const startedAt = Date.now();
  const pool = await loadPinnedUsd1RoutePool(raydium);
  const inputMint = NATIVE_MINT;
  const outputMint = USD1_MINT;
  const basicPool = toBasicPoolInfo(pool);
  const routes = raydium.tradeV2.getAllRoute({
    inputMint,
    outputMint,
    clmmPools: basicPool.version === 6 ? [basicPool] : [],
    ammPools: basicPool.version === 4 ? [basicPool] : [],
    cpmmPools: basicPool.version === 7 ? [basicPool] : [],
  });
  const [routeData, swapPoolKeys, epochInfo] = await Promise.all([
    raydium.tradeV2.fetchSwapRoutesData({
      routes,
      inputMint,
      outputMint,
    }),
    raydium.api.fetchPoolKeysById({ idList: [pool.id] }),
    connection.getEpochInfo(),
  ]);
  if (!swapPoolKeys.length) {
    throw new Error(`Raydium pool keys not found for ${pool.id}.`);
  }
  const inputTokenInfo = routeData.mintInfos[inputMint.toBase58()];
  const outputTokenInfo = routeData.mintInfos[outputMint.toBase58()];
  const directPath = routes.directPath
    .map((entry) =>
      routeData.computeClmmPoolInfo[entry.id.toBase58()]
      || routeData.ammSimulateCache[entry.id.toBase58()]
      || routeData.computeCpmmData[entry.id.toBase58()])
    .filter(Boolean);
  const setup = {
    pool,
    inputMint,
    outputMint,
    routeData,
    swapPoolKeys,
    epochInfo,
    inputTokenInfo,
    outputTokenInfo,
    directPath,
    simulateCache: {
      ...routeData.ammSimulateCache,
      ...routeData.computeClmmPoolInfo,
      ...routeData.computeCpmmData,
    },
  };
  context.metrics.routeSetupFetchMs += Date.now() - startedAt;
  context.localCache.set(cacheKey, setup);
  writeUsd1RouteSetupCache(cacheKey, setup);
  return setup;
}

async function computeDirectRouteSwap(raydium, connection, pool, inputAmountBn, slippageBps, requestContext = null, phase = "general") {
  const context = ensureUsd1QuoteRequestContext(requestContext);
  const quoteCacheKey = `usd1-route-quote:${inputAmountBn.toString(10)}:${Number(slippageBps || 0)}`;
  if (context.localCache.has(quoteCacheKey)) {
    addUsd1QuoteMetric(context.metrics, "quoteCacheHits");
    return context.localCache.get(quoteCacheKey);
  }
  const routeSetup = await loadUsd1RouteSetup(raydium, connection, context);
  const inputTokenAmount = new TokenAmount(
    new Token({
      mint: routeSetup.inputMint,
      decimals: routeSetup.inputTokenInfo.decimals,
      symbol: routeSetup.inputTokenInfo.symbol,
      name: routeSetup.inputTokenInfo.name,
    }),
    inputAmountBn.toString(10),
    true,
  );
  addUsd1QuoteMetric(context.metrics, "quoteCalls");
  if (phase === "expansion") addUsd1QuoteMetric(context.metrics, "expansionQuoteCalls");
  if (phase === "binary") addUsd1QuoteMetric(context.metrics, "binarySearchQuoteCalls");
  if (phase === "buffer") addUsd1QuoteMetric(context.metrics, "bufferQuoteCalls");
  const startedAt = Date.now();
  const swapCandidates = raydium.tradeV2.getAllRouteComputeAmountOut({
    directPath: routeSetup.directPath,
    routePathDict: routeSetup.routeData.routePathDict,
    simulateCache: routeSetup.simulateCache,
    tickCache: routeSetup.routeData.computePoolTickData,
    mintInfos: routeSetup.routeData.mintInfos,
    inputTokenAmount,
    outputToken: routeSetup.outputTokenInfo,
    slippage: Number(slippageBps || 0) / 10_000,
    chainTime: Math.floor(Date.now() / 1000),
    epochInfo: routeSetup.epochInfo,
  });
  context.metrics.quoteTotalMs += Date.now() - startedAt;
  if (!swapCandidates.length) {
    throw new Error(`No Raydium route quote found for pool ${(pool && pool.id) || routeSetup.pool.id}.`);
  }
  const swapInfo = swapCandidates[0];
  const quote = {
    swapInfo,
    swapPoolKeys: routeSetup.swapPoolKeys,
    expectedOut: new BN(swapInfo.amountOut.amount.raw.toString()),
    minOut: new BN(swapInfo.minAmountOut.amount.raw.toString()),
    priceImpactPct: Number(swapInfo.priceImpact.toString()) * 100,
    pool: routeSetup.pool,
  };
  context.localCache.set(quoteCacheKey, quote);
  return quote;
}

function buildUsd1SearchGuessLamports(requiredQuoteAmount, referencePrice, maxInputLamports) {
  let guess = parseDecimalToBn(
    String(requiredQuoteAmount.toNumber() / 1_000_000 / referencePrice * 1.05 || 0.01),
    9,
    "top-up search guess",
  );
  if (guess.lte(new BN(0))) {
    guess = parseDecimalToBn("0.01", 9, "top-up search floor");
  }
  return guess.gt(maxInputLamports) ? maxInputLamports.clone() : guess;
}

function usd1SearchToleranceLamports(high, policy) {
  const minLamports = new BN(String(policy.searchMinLamports));
  const bpsLamports = high.mul(new BN(String(policy.searchToleranceBps))).div(new BN(10_000));
  return minLamports.gte(bpsLamports) ? minLamports : bpsLamports;
}

function addUsd1SearchBufferLamports(high, maxInputLamports, policy) {
  const minBufferLamports = new BN(String(policy.searchBufferMinLamports));
  const bpsBufferLamports = high.mul(new BN(String(policy.searchBufferBps))).div(new BN(10_000));
  const bufferLamports = minBufferLamports.gte(bpsBufferLamports) ? minBufferLamports : bpsBufferLamports;
  return minBn(high.add(bufferLamports), maxInputLamports);
}

async function quoteSolInputForUsd1Output(
  raydium,
  connection,
  requiredQuoteAmount,
  slippageBps,
  requestContext = null,
  maxInputLamportsOverride = null,
) {
  const context = ensureUsd1QuoteRequestContext(requestContext);
  const policy = getUsd1TopupPolicy();
  const routeSetup = await loadUsd1RouteSetup(raydium, connection, context);
  const referencePrice = Number(routeSetup.pool.price || 0);
  if (!Number.isFinite(referencePrice) || referencePrice <= 0) {
    throw new Error(`Pinned USD1 route pool has invalid price metadata: ${PINNED_USD1_ROUTE_POOL_ID}`);
  }
  const maxInputLamports = maxInputLamportsOverride || parseDecimalToBn("100000", 9, "maximum SOL quote search");
  let low = new BN(1);
  let high = buildUsd1SearchGuessLamports(requiredQuoteAmount, referencePrice, maxInputLamports);
  let quote = await computeDirectRouteSwap(
    raydium,
    connection,
    routeSetup.pool,
    high,
    slippageBps,
    context,
    "expansion",
  );
  while (quote.minOut.lt(requiredQuoteAmount) && high.lt(maxInputLamports)) {
    low = high.add(new BN(1));
    high = minBn(high.mul(new BN(2)), maxInputLamports);
    quote = await computeDirectRouteSwap(
      raydium,
      connection,
      routeSetup.pool,
      high,
      slippageBps,
      context,
      "expansion",
    );
    if (high.eq(maxInputLamports)) break;
  }
  if (quote.minOut.lt(requiredQuoteAmount)) {
    throw new Error(
      `Pinned USD1 route pool could not satisfy required USD1 output: ${PINNED_USD1_ROUTE_POOL_ID}. `
      + `requiredUsd1=${formatBn(requiredQuoteAmount, 6, 6)} `
      + `maxQuotedSol=${formatBn(maxInputLamports, 9, 6)} `
      + `quotedUsd1=${formatBn(quote.expectedOut, 6, 6)} `
      + `minUsd1=${formatBn(quote.minOut, 6, 6)} `
      + `priceImpactPct=${quote.priceImpactPct}`
    );
  }
  for (let index = 0; index < policy.maxSearchIterations && low.lt(high); index += 1) {
    addUsd1QuoteMetric(context.metrics, "searchIterations");
    if (high.sub(low).lte(usd1SearchToleranceLamports(high, policy))) {
      break;
    }
    const mid = low.add(high).div(new BN(2));
    const midQuote = await computeDirectRouteSwap(
      raydium,
      connection,
      routeSetup.pool,
      mid,
      slippageBps,
      context,
      "binary",
    );
    if (midQuote.minOut.gte(requiredQuoteAmount)) {
      high = mid;
      quote = midQuote;
    } else {
      low = mid.add(new BN(1));
    }
  }
  const bufferedInputLamports = addUsd1SearchBufferLamports(high, maxInputLamports, policy);
  if (bufferedInputLamports.gt(high)) {
    high = bufferedInputLamports;
    quote = await computeDirectRouteSwap(
      raydium,
      connection,
      routeSetup.pool,
      high,
      slippageBps,
      context,
      "buffer",
    );
  }
  return {
    inputLamports: high,
    expectedOut: new BN(quote.expectedOut.toString()),
    minOut: new BN(quote.minOut.toString()),
    swapInfo: quote.swapInfo,
    swapPoolKeys: quote.swapPoolKeys,
    priceImpactPct: quote.priceImpactPct,
    pool: routeSetup.pool,
  };
}

async function quoteUsd1OutputFromSolInput(raydium, connection, inputLamports, slippageBps, requestContext = null) {
  const routeSetup = await loadUsd1RouteSetup(raydium, connection, requestContext);
  const quote = await computeDirectRouteSwap(
    raydium,
    connection,
    routeSetup.pool,
    inputLamports,
    slippageBps,
    requestContext,
  );
  return {
    inputLamports,
    expectedOut: new BN(quote.expectedOut.toString()),
    minOut: new BN(quote.minOut.toString()),
  };
}

async function quoteLaunch(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const usd1QuoteContext = createUsd1QuoteRequestContext();
  const raydium = await Raydium.load({
    connection,
    owner: null,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const buyMode = String(request.mode || "").trim().toLowerCase();
  const defaults = await loadLaunchDefaults(
    raydium,
    connection,
    null,
    request.launchMode || "regular",
    request.quoteAsset,
  );
  if (defaults.quoteAsset === "usd1" && buyMode === "sol") {
    const solInput = parseDecimalToBn(request.amount, 9, "buy amount SOL");
    const usd1RouteQuote = await quoteUsd1OutputFromSolInput(
      raydium,
      connection,
      solInput,
      request.slippageBps,
      usd1QuoteContext,
    );
    const curveQuote = Curve.buyExactIn({
      poolInfo: defaults.poolInfo,
      protocolFeeRate: defaults.configInfo.tradeFeeRate,
      platformFeeRate: defaults.platformInfo.feeRate,
      curveType: defaults.configInfo.curveType,
      shareFeeRate: new BN(0),
      creatorFeeRate: defaults.platformInfo.creatorFeeRate,
      transferFeeConfigA: undefined,
      slot: 0,
      amountB: usd1RouteQuote.minOut,
    });
    return {
      mode: buyMode,
      input: request.amount,
      estimatedTokens: formatBn(curveQuote.amountA.amount, TOKEN_DECIMALS, 6),
      estimatedSol: formatBn(solInput, 9, 6),
      estimatedQuoteAmount: formatBn(solInput, 9, 6),
      quoteAsset: "sol",
      quoteAssetLabel: "SOL",
      estimatedSupplyPercent: estimateSupplyPercent(curveQuote.amountA.amount, defaults.supply),
    };
  }
  if (defaults.quoteAsset === "usd1" && buyMode === "tokens") {
    const tokenAmount = parseDecimalToBn(request.amount, TOKEN_DECIMALS, "buy amount");
    const curveQuote = Curve.buyExactOut({
      poolInfo: defaults.poolInfo,
      protocolFeeRate: defaults.configInfo.tradeFeeRate,
      platformFeeRate: defaults.platformInfo.feeRate,
      curveType: defaults.configInfo.curveType,
      shareFeeRate: new BN(0),
      creatorFeeRate: defaults.platformInfo.creatorFeeRate,
      transferFeeConfigA: undefined,
      slot: 0,
      amountA: tokenAmount,
    });
    const solQuote = await quoteSolInputForUsd1Output(
      raydium,
      connection,
      new BN(curveQuote.amountB.toString()),
      request.slippageBps,
      usd1QuoteContext,
    );
    return {
      mode: buyMode,
      input: request.amount,
      estimatedTokens: formatBn(tokenAmount, TOKEN_DECIMALS, 6),
      estimatedSol: formatBn(solQuote.inputLamports, 9, 6),
      estimatedQuoteAmount: formatBn(solQuote.inputLamports, 9, 6),
      quoteAsset: "sol",
      quoteAssetLabel: "SOL",
      estimatedSupplyPercent: estimateSupplyPercent(tokenAmount, defaults.supply),
      usd1QuoteMetrics: formatUsd1QuoteMetrics(usd1QuoteContext.metrics),
    };
  }
  return buildQuote(defaults, buyMode, request.amount);
}

function buildComputeBudgetConfig(input) {
  if (!input || !input.computeUnitLimit) return undefined;
  return {
    units: Number(input.computeUnitLimit),
    microLamports: Number(input.computeUnitPriceMicroLamports || 0),
  };
}

function buildTipConfig(input) {
  if (!input || !input.tipLamports || !input.tipAccount) return undefined;
  return {
    address: input.tipAccount,
    amount: new BN(String(input.tipLamports)),
  };
}

function minBn(left, right) {
  return left.lte(right) ? left : right;
}

function buildMinAmountFromBps(amount, slippageBps) {
  const safeBps = Math.max(0, Math.min(10_000, Number(slippageBps || 0)));
  return amount.mul(new BN(10_000 - safeBps)).div(new BN(10_000));
}

async function fetchWalletTokenBalance(connection, owner, mint) {
  const ata = getAssociatedTokenAddressSync(mint, owner, false, TOKEN_PROGRAM_ID);
  try {
    const balance = await connection.getTokenAccountBalance(ata, "processed");
    return new BN(balance.value.amount || "0");
  } catch (_error) {
    return new BN(0);
  }
}

async function loadPinnedUsd1RoutePool(raydium) {
  const pools = await raydium.api.fetchPoolById({ ids: PINNED_USD1_ROUTE_POOL_ID });
  const pool = (pools || []).find((entry) => entry && entry.id === PINNED_USD1_ROUTE_POOL_ID);
  if (!pool) {
    throw new Error(`Pinned USD1 route pool not found: ${PINNED_USD1_ROUTE_POOL_ID}`);
  }
  const mintA = pool.mintA && (pool.mintA.address || pool.mintA);
  const mintB = pool.mintB && (pool.mintB.address || pool.mintB);
  const isExpectedPair = [mintA, mintB].includes(NATIVE_MINT.toBase58())
    && [mintA, mintB].includes(USD1_MINT.toBase58());
  if (!isExpectedPair) {
    throw new Error(`Pinned USD1 route pool no longer matches SOL/USD1: ${PINNED_USD1_ROUTE_POOL_ID}`);
  }
  if (!pool.config || pool.config.id !== PREFERRED_USD1_ROUTE_CONFIG) {
    throw new Error(`Pinned USD1 route pool config changed: ${PINNED_USD1_ROUTE_POOL_ID}`);
  }
  return pool;
}

async function prepareUsd1Topup(raydium, connection, owner, request, requiredQuoteAmountRaw, requestContext = null) {
  if (resolveQuoteAssetConfig(request.quoteAsset).asset !== "usd1") {
    return null;
  }
  const context = ensureUsd1QuoteRequestContext(requestContext);
  const policy = getUsd1TopupPolicy();
  const requiredQuoteAmount = parseDecimalToBn(requiredQuoteAmountRaw, 6, "required USD1 amount");
  if (requiredQuoteAmount.lte(new BN(0))) {
    return null;
  }
  const currentUsd1Balance = await fetchWalletTokenBalance(connection, owner.publicKey, USD1_MINT);
  if (currentUsd1Balance.gte(requiredQuoteAmount)) {
    return {
      swapResult: null,
      requiredQuoteAmount: formatBn(requiredQuoteAmount, 6, 6),
      currentQuoteAmount: formatBn(currentUsd1Balance, 6, 6),
      shortfallQuoteAmount: "0",
    };
  }
  const shortfall = requiredQuoteAmount.sub(currentUsd1Balance);
  const balanceLamports = await connection.getBalance(owner.publicKey, "processed");
  const minRemainingLamports = parseDecimalToBn(String(policy.minRemainingSol), 9, "minimum remaining SOL");
  const maxSpendableLamports = new BN(String(balanceLamports)).sub(minRemainingLamports);
  if (maxSpendableLamports.lte(new BN(0))) {
    throw new Error(`Insufficient SOL headroom for USD1 top-up. Need at least ${policy.minRemainingSol} SOL reserved after swap.`);
  }
  const quote = await quoteSolInputForUsd1Output(
    raydium,
    connection,
    shortfall,
    request.slippageBps,
    context,
    maxSpendableLamports,
  );
  const swapResult = await raydium.tradeV2.swap({
    txVersion: txVersionFromFormat(request.txFormat),
    swapInfo: quote.swapInfo,
    swapPoolKeys: quote.swapPoolKeys,
    ownerInfo: {
      associatedOnly: false,
      checkCreateATAOwner: true,
    },
    routeProgram: RAYDIUM_ROUTE_PROGRAM,
    computeBudgetConfig: buildComputeBudgetConfig(request.txConfig),
    txTipConfig: buildTipConfig(request.txConfig),
    feePayer: owner.publicKey,
  });
  const normalizedSwapResult = await ensureInlineTipOnSwapResult(
    connection,
    owner,
    swapResult,
    request.txConfig,
  );
  return {
    swapResult: normalizedSwapResult,
    requiredQuoteAmount: formatBn(requiredQuoteAmount, 6, 6),
    currentQuoteAmount: formatBn(currentUsd1Balance, 6, 6),
    shortfallQuoteAmount: formatBn(shortfall, 6, 6),
      inputSol: formatBn(quote.inputLamports, 9, 6),
    expectedQuoteOut: formatBn(quote.expectedOut, 6, 6),
    minQuoteOut: formatBn(quote.minOut, 6, 6),
    priceImpactPct: String(quote.priceImpactPct),
      routePassedPolicy: Number(quote.pool.tvl || 0) >= policy.minPoolTvlUsd
      && quote.priceImpactPct <= policy.maxPriceImpactPct,
      routePoolId: quote.pool.id,
      routeConfigId: quote.pool.config && quote.pool.config.id ? quote.pool.config.id : "",
      routePoolType: quote.pool.type,
      routePoolTvlUsd: String(quote.pool.tvl || 0),
      usd1QuoteMetrics: formatUsd1QuoteMetrics(context.metrics),
  };
}

async function buildUsd1Topup(request) {
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const usd1QuoteContext = createUsd1QuoteRequestContext();
  const raydium = await Raydium.load({
    connection,
    owner,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  await ensureQuoteTokenAccountReady(connection, owner, request, raydium, USD1_MINT);
  const prepared = await prepareUsd1Topup(
    raydium,
    connection,
    owner,
    request,
    request.requiredQuoteAmount,
    usd1QuoteContext,
  );
  if (!prepared || !prepared.swapResult) {
    return {
      compiledTransaction: null,
      requiredQuoteAmount: prepared && prepared.requiredQuoteAmount ? prepared.requiredQuoteAmount : undefined,
      currentQuoteAmount: prepared && prepared.currentQuoteAmount ? prepared.currentQuoteAmount : undefined,
      shortfallQuoteAmount: prepared && prepared.shortfallQuoteAmount ? prepared.shortfallQuoteAmount : undefined,
      usd1QuoteMetrics: prepared && prepared.usd1QuoteMetrics ? prepared.usd1QuoteMetrics : undefined,
    };
  }
  const { lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  return {
    compiledTransaction: normalizeTransactions(prepared.swapResult, {
      labelPrefix: request.labelPrefix || "usd1-topup",
      computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
      computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
      inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
      lastValidBlockHeight,
    })[0],
    requiredQuoteAmount: prepared.requiredQuoteAmount,
    currentQuoteAmount: prepared.currentQuoteAmount,
    shortfallQuoteAmount: prepared.shortfallQuoteAmount,
    inputSol: prepared.inputSol,
    expectedQuoteOut: prepared.expectedQuoteOut,
    minQuoteOut: prepared.minQuoteOut,
    priceImpactPct: prepared.priceImpactPct,
    routePoolId: prepared.routePoolId,
    routeConfigId: prepared.routeConfigId,
    routePoolType: prepared.routePoolType,
    routePoolTvlUsd: prepared.routePoolTvlUsd,
    usd1QuoteMetrics: prepared.usd1QuoteMetrics,
  };
}

async function buildLaunch(request) {
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const usd1QuoteContext = createUsd1QuoteRequestContext();
  const raydium = await Raydium.load({
    connection,
    owner,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const defaults = await loadLaunchDefaults(
    raydium,
    connection,
    owner.publicKey,
    request.mode,
    request.quoteAsset,
  );
  await ensureQuoteTokenAccountReady(connection, owner, request, raydium, defaults.quoteMint);
  const mintKeypair = request.vanitySecret
    ? parseKeypair(request.vanitySecret)
    : Keypair.generate();
  const txVersion = atomicUsd1TxVersion(request);
  let buyAmount;
  let minMintAAmount;
  let createOnly = true;
  const predictedDevBuyTokenAmount = await estimateDevBuyTokenAmount(
    raydium,
    connection,
    defaults,
    request.devBuy,
    request.slippageBps,
    usd1QuoteContext,
  );
  if (request.devBuy && request.devBuy.mode && request.devBuy.amount) {
    createOnly = false;
    if (request.devBuy.mode === "tokens") {
      const quote = buildQuote(defaults, "tokens", request.devBuy.amount);
      const tokenAmount = parseDecimalToBn(request.devBuy.amount, TOKEN_DECIMALS, "dev buy tokens");
      buyAmount = parseDecimalToBn(
        quote.estimatedQuoteAmount || quote.estimatedSol,
        defaults.quoteDecimals,
        `dev buy ${defaults.quoteAssetLabel}`,
      );
      minMintAAmount = buildMinAmountFromBps(tokenAmount, request.slippageBps);
    } else if (defaults.quoteAsset === "usd1") {
      const solInput = parseDecimalToBn(request.devBuy.amount, 9, "dev buy SOL");
      const usd1RouteQuote = await quoteUsd1OutputFromSolInput(
        raydium,
        connection,
        solInput,
        request.slippageBps,
        usd1QuoteContext,
      );
      buyAmount = usd1RouteQuote.minOut;
    } else {
      buyAmount = parseDecimalToBn(
        request.devBuy.amount,
        defaults.quoteDecimals,
        `dev buy ${defaults.quoteAssetLabel}`,
      );
    }
  }
  const usd1Topup = !createOnly && defaults.quoteAsset === "usd1" && buyAmount
    ? await prepareUsd1Topup(
      raydium,
      connection,
      owner,
      {
        ...request,
        requiredQuoteAmount: formatBn(buyAmount, defaults.quoteDecimals, 6),
      },
      formatBn(buyAmount, defaults.quoteDecimals, 6),
      usd1QuoteContext,
    )
    : null;
  const buildResult = await raydium.launchpad.createLaunchpad({
    programId: LAUNCHPAD_PROGRAM,
    platformId: defaults.platformId,
    configId: defaults.configId,
    mintA: mintKeypair.publicKey,
    decimals: TOKEN_DECIMALS,
    name: request.token.name,
    symbol: request.token.symbol,
    uri: request.token.uri,
    migrateType: "cpmm",
    createOnly,
    buyAmount,
    minMintAAmount,
    slippage: new BN(String(request.slippageBps || 0)),
    txVersion,
    extraSigners: [mintKeypair],
    computeBudgetConfig: buildComputeBudgetConfig(request.txConfig),
    txTipConfig: buildTipConfig(request.txConfig),
    associatedOnly: false,
    checkCreateATAOwner: true,
  });
  const launchTransactions = extractTransactions(buildResult);
  const topupTransactions = usd1Topup && usd1Topup.swapResult
    ? extractTransactions(usd1Topup.swapResult)
    : [];
  const { lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  let compiledTransactions;
  let atomicCombined = false;
  let atomicFallbackReason = null;
  if (topupTransactions.length) {
    if (topupTransactions.length === 1 && launchTransactions.length === 1) {
      try {
        const combined = await combineAtomicUsd1ActionTransaction(
          connection,
          owner,
          request,
          topupTransactions[0],
          launchTransactions[0],
          [mintKeypair],
        );
        atomicCombined = true;
        compiledTransactions = normalizeTransactions({ transactions: [combined.transaction] }, {
          labelPrefix: "launch",
          computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
          computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
          inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
          inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
          lastValidBlockHeight: combined.lastValidBlockHeight,
        });
      } catch (error) {
        atomicFallbackReason = `Atomic USD1 launch fallback: ${error && error.message ? error.message : String(error)}`;
      }
    } else {
      atomicFallbackReason = "Atomic USD1 launch requires exactly one top-up transaction and one launch transaction.";
    }
  }
  if (!compiledTransactions) {
    const remappedLaunchTransactions = await Promise.all(launchTransactions.map((transaction, index) => (
      preferBonkUsd1LookupTableOnTransaction(
        connection,
        owner,
        request,
        transaction,
        index === 0 ? [mintKeypair] : [],
      )
    )));
    compiledTransactions = normalizeTransactions({ transactions: remappedLaunchTransactions }, {
      labelPrefix: "launch",
      computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
      computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
      inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
      lastValidBlockHeight,
    });
    if (topupTransactions.length) {
      const remappedTopupTransactions = await Promise.all(topupTransactions.map((transaction) => (
        preferBonkUsd1LookupTableOnTransaction(connection, owner, request, transaction)
      )));
      compiledTransactions.unshift(...normalizeTransactions({ transactions: remappedTopupTransactions }, {
        labelPrefix: request.labelPrefix || "launch-usd1-topup",
        computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
        computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
        inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
        inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
        lastValidBlockHeight,
      }));
      if (!atomicFallbackReason) {
        atomicFallbackReason = "USD1 launch path is using split top-up plus launch transactions.";
      }
    }
  }
  const usd1LaunchDetails = usd1Topup ? {
    compilePath: topupTransactions.length
      ? (atomicCombined ? "atomic-topup+launch" : "split-topup+launch")
      : "launch-only",
    requiredQuoteAmount: usd1Topup.requiredQuoteAmount,
    currentQuoteAmount: usd1Topup.currentQuoteAmount,
    shortfallQuoteAmount: usd1Topup.shortfallQuoteAmount,
    inputSol: usd1Topup.inputSol,
    expectedQuoteOut: usd1Topup.expectedQuoteOut,
    minQuoteOut: usd1Topup.minQuoteOut,
  } : null;
  return {
    mint: mintKeypair.publicKey.toBase58(),
    launchCreator: owner.publicKey.toBase58(),
    predictedDevBuyTokenAmountRaw: predictedDevBuyTokenAmount ? predictedDevBuyTokenAmount.toString(10) : null,
    compiledTransactions,
    atomicCombined,
    atomicFallbackReason,
    usd1LaunchDetails,
    usd1QuoteMetrics: formatUsd1QuoteMetrics(usd1QuoteContext.metrics),
  };
}

async function compileFollowBuy(request, labelPrefix, atomic = false) {
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const usd1QuoteContext = createUsd1QuoteRequestContext();
  const raydium = await Raydium.load({
    connection,
    owner,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const mint = new PublicKey(request.mint);
  const quote = resolveQuoteAssetConfig(request.quoteAsset);
  await ensureQuoteTokenAccountReady(connection, owner, request, raydium, quote.mint);
  const buyAmount = parseDecimalToBn(request.buyAmountSol, quote.decimals, `follow buy amount ${quote.label}`);
  const txVersion = atomic && quote.asset === "usd1"
    ? atomicUsd1TxVersion(request)
    : txVersionFromFormat(request.txFormat);
  const options = {
    programId: LAUNCHPAD_PROGRAM,
    mintA: mint,
    mintB: quote.mint,
    buyAmount,
    slippage: new BN(String(request.slippageBps || 0)),
    txVersion,
    computeBudgetConfig: buildComputeBudgetConfig(request.txConfig),
    txTipConfig: buildTipConfig(request.txConfig),
  };
  if (atomic) {
    const defaults = await loadLaunchDefaults(
      raydium,
      connection,
      request.launchCreator ? new PublicKey(request.launchCreator) : owner.publicKey,
      request.mode,
      request.quoteAsset,
    );
    const creator = request.launchCreator ? new PublicKey(request.launchCreator) : owner.publicKey;
    Object.assign(options, {
      poolInfo: buildPrelaunchPoolInfo(defaults, mint, creator),
      configInfo: defaults.configInfo,
      platformFeeRate: defaults.platformInfo.feeRate,
      mintAProgram: TOKEN_PROGRAM_ID,
      skipCheckMintA: true,
    });
  } else {
    const livePool = await loadLivePoolContext(raydium, connection, mint, request.quoteAsset);
    Object.assign(options, {
      poolInfo: livePool.poolInfo,
      configInfo: livePool.configInfo,
      platformFeeRate: livePool.platformInfo.feeRate,
      mintAProgram: TOKEN_PROGRAM_ID,
      skipCheckMintA: true,
    });
  }
  const buildResult = await raydium.launchpad.buyToken({
    ...options,
    associatedOnly: false,
    checkCreateATAOwner: true,
  });
  if (atomic && quote.asset === "usd1") {
    const usd1Topup = await prepareUsd1Topup(
      raydium,
      connection,
      owner,
      {
        ...request,
        requiredQuoteAmount: formatBn(buyAmount, quote.decimals, 6),
      },
      formatBn(buyAmount, quote.decimals, 6),
      usd1QuoteContext,
    );
    if (usd1Topup && usd1Topup.swapResult) {
      const topupTransactions = extractTransactions(usd1Topup.swapResult);
      const buyTransactions = extractTransactions(buildResult);
      if (topupTransactions.length !== 1 || buyTransactions.length !== 1) {
        throw new Error("Atomic USD1 follow buy requires exactly one top-up transaction and one buy transaction.");
      }
      const combined = await combineAtomicUsd1ActionTransaction(
        connection,
        owner,
        request,
        topupTransactions[0],
        buyTransactions[0],
      );
      return {
        compiledTransaction: normalizeTransactions({ transactions: [combined.transaction] }, {
          labelPrefix,
          computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
          computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
          inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
          inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
          lastValidBlockHeight: combined.lastValidBlockHeight,
        })[0],
        usd1QuoteMetrics: formatUsd1QuoteMetrics(usd1QuoteContext.metrics),
      };
    }
  }
  const { lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  return {
    compiledTransaction: normalizeTransactions(buildResult, {
      labelPrefix,
      computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
      computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
      inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
      lastValidBlockHeight,
    })[0],
    usd1QuoteMetrics: formatUsd1QuoteMetrics(usd1QuoteContext.metrics),
  };
}

async function compileFollowSell(request) {
  const owner = parseKeypair(request.ownerSecret);
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const raydium = await Raydium.load({
    connection,
    owner,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const mint = new PublicKey(request.mint);
  await ensureQuoteTokenAccountReady(connection, owner, request, raydium);
  let rawAmount;
  if (request.exactTokenAmountRaw) {
    rawAmount = new BN(String(request.exactTokenAmountRaw));
  } else {
    const tokenAccount = getAssociatedTokenAddressSync(mint, owner.publicKey, false, TOKEN_PROGRAM_ID);
    let balanceInfo;
    try {
      balanceInfo = await connection.getTokenAccountBalance(tokenAccount, request.commitment || "processed");
    } catch (_error) {
      return { compiledTransaction: null };
    }
    rawAmount = new BN(balanceInfo.value.amount || "0");
  }
  if (rawAmount.isZero()) {
    return { compiledTransaction: null };
  }
  const sellAmount = rawAmount.mul(new BN(Number(request.sellPercent || 0))).div(new BN(100));
  if (sellAmount.isZero()) {
    return { compiledTransaction: null };
  }
  const livePool = request.exactTokenAmountRaw && request.poolId && request.mode && request.launchCreator
    ? await buildPrelaunchPoolContext(
      raydium,
      connection,
      mint,
      request.launchCreator,
      request.mode,
      request.quoteAsset,
    )
    : request.poolId
      ? await loadPoolContextByPoolId(raydium, connection, request.poolId, request.quoteAsset)
      : await loadLivePoolContext(raydium, connection, mint, request.quoteAsset);
  const buildResult = await raydium.launchpad.sellToken({
    programId: LAUNCHPAD_PROGRAM,
    mintA: mint,
    mintB: livePool.quoteMint,
    sellAmount,
    poolInfo: livePool.poolInfo,
    configInfo: livePool.configInfo,
    platformFeeRate: livePool.platformInfo.feeRate,
    slippage: new BN(String(request.slippageBps || 0)),
    txVersion: txVersionFromFormat(request.txFormat),
    computeBudgetConfig: buildComputeBudgetConfig(request.txConfig),
    txTipConfig: buildTipConfig(request.txConfig),
    mintAProgram: TOKEN_PROGRAM_ID,
    skipCheckMintA: true,
    associatedOnly: false,
    checkCreateATAOwner: true,
  });
  const { lastValidBlockHeight } = await connection.getLatestBlockhash(request.commitment || "confirmed");
  return {
    compiledTransaction: normalizeTransactions(buildResult, {
      labelPrefix: "follow-sell",
      computeUnitLimit: request.txConfig && request.txConfig.computeUnitLimit,
      computeUnitPriceMicroLamports: request.txConfig && request.txConfig.computeUnitPriceMicroLamports,
      inlineTipLamports: request.txConfig && request.txConfig.tipLamports,
      inlineTipAccount: request.txConfig && request.txConfig.tipAccount,
      lastValidBlockHeight,
    })[0],
  };
}

async function deriveCanonicalPoolId(request) {
  const mint = new PublicKey(request.mint);
  const quote = resolveQuoteAssetConfig(request.quoteAsset);
  const poolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, mint, quote.mint).publicKey;
  return {
    poolId: poolId.toBase58(),
  };
}

async function predictDevBuyTokenAmount(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const raydium = await Raydium.load({
    connection,
    owner: null,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const defaults = await loadLaunchDefaults(
    raydium,
    connection,
    null,
    request.mode,
    request.quoteAsset,
  );
  const predicted = await estimateDevBuyTokenAmount(
    raydium,
    connection,
    defaults,
    request.devBuy,
    request.slippageBps,
  );
  return {
    predictedDevBuyTokenAmountRaw: predicted ? predicted.toString(10) : null,
  };
}

async function fetchMarketSnapshot(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "processed");
  const raydium = await Raydium.load({
    connection,
    owner: null,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const mint = new PublicKey(request.mint);
  const quote = resolveQuoteAssetConfig(request.quoteAsset);
  const poolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, mint, quote.mint).publicKey;
  const poolInfo = await raydium.launchpad.getRpcPoolInfo({ poolId });
  const supply = new BN(poolInfo.supply.toString());
  const virtualA = new BN(poolInfo.virtualA.toString());
  const virtualB = new BN(poolInfo.virtualB.toString());
  const realA = new BN(poolInfo.realA.toString());
  const realB = new BN(poolInfo.realB.toString());
  const totalSellA = new BN(poolInfo.totalSellA.toString());
  const marketCapLamports = virtualA.isZero() ? new BN(0) : supply.mul(virtualB).div(virtualA);
  return {
    mint: mint.toBase58(),
    quoteAsset: quote.asset,
    quoteAssetLabel: quote.label,
    creator: poolInfo.creator.toBase58 ? poolInfo.creator.toBase58() : String(poolInfo.creator),
    virtualTokenReserves: virtualA.toString(10),
    virtualSolReserves: virtualB.toString(10),
    realTokenReserves: totalSellA.sub(realA).toString(10),
    realSolReserves: realB.toString(10),
    tokenTotalSupply: supply.toString(10),
    complete: Number(poolInfo.status || 0) !== 0,
    marketCapLamports: marketCapLamports.toString(10),
    marketCapSol: formatBn(marketCapLamports, quote.decimals, 6),
  };
}

async function detectImportContext(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "processed");
  const raydium = await Raydium.load({
    connection,
    owner: null,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const mint = new PublicKey(request.mint);
  const candidates = [];
  for (const asset of ["sol", "usd1"]) {
    try {
      const quote = resolveQuoteAssetConfig(asset);
      const poolId = getPdaLaunchpadPoolId(LAUNCHPAD_PROGRAM, mint, quote.mint).publicKey;
      const poolInfo = await raydium.launchpad.getRpcPoolInfo({ poolId });
      const platformId = poolInfo.platformId && poolInfo.platformId.toBase58
        ? poolInfo.platformId.toBase58()
        : String(poolInfo.platformId || "");
      const configId = poolInfo.configId && poolInfo.configId.toBase58
        ? poolInfo.configId.toBase58()
        : String(poolInfo.configId || "");
      candidates.push({
        launchpad: "bonk",
        mode: platformId === BONKERS_PLATFORM.toBase58() ? "bonkers" : "regular",
        quoteAsset: quote.asset,
        creator: poolInfo.creator && poolInfo.creator.toBase58
          ? poolInfo.creator.toBase58()
          : String(poolInfo.creator || ""),
        platformId,
        configId,
        poolId: poolId.toBase58(),
        realQuoteReserves: poolInfo.realB ? poolInfo.realB.toString() : "0",
        complete: Number(poolInfo.status || 0) !== 0,
        detectionSource: "raydium-launchpad",
      });
    } catch (_error) {
      // Ignore missing pool shapes and keep probing the other quote asset.
    }
  }
  if (!candidates.length) {
    return null;
  }
  candidates.sort((left, right) => {
    const leftLiquidity = BigInt(left.realQuoteReserves || "0");
    const rightLiquidity = BigInt(right.realQuoteReserves || "0");
    if (leftLiquidity === rightLiquidity) {
      return left.quoteAsset === "sol" ? -1 : 1;
    }
    return rightLiquidity > leftLiquidity ? 1 : -1;
  });
  return candidates[0];
}

async function warmState(request) {
  const connection = new Connection(request.rpcUrl, request.commitment || "confirmed");
  const usd1QuoteContext = createUsd1QuoteRequestContext();
  const raydium = await Raydium.load({
    connection,
    owner: null,
    disableLoadToken: true,
    disableFeatureCheck: true,
  });
  const launchDefaultsPromise = Promise.all([
    loadLaunchDefaults(raydium, connection, null, "regular", "sol"),
    loadLaunchDefaults(raydium, connection, null, "regular", "usd1"),
    loadLaunchDefaults(raydium, connection, null, "bonkers", "sol"),
    loadLaunchDefaults(raydium, connection, null, "bonkers", "usd1"),
  ]);
  const routeSetupPromise = loadUsd1RouteSetup(raydium, connection, usd1QuoteContext);
  const [launchDefaults, routeSetup] = await Promise.all([launchDefaultsPromise, routeSetupPromise]);
  return {
    warmedLaunchDefaults: launchDefaults.map((entry) => ({
      mode: entry.mode,
      quoteAsset: entry.quoteAsset,
      platformId: entry.platformId.toBase58(),
      configId: entry.configId.toBase58(),
      quoteMint: entry.quoteMint.toBase58(),
    })),
    usd1RoutePoolId: routeSetup.pool.id,
    usd1RouteConfigId: routeSetup.pool.config && routeSetup.pool.config.id ? routeSetup.pool.config.id : "",
    usd1QuoteMetrics: formatUsd1QuoteMetrics(usd1QuoteContext.metrics),
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

async function main() {
  const request = await readRequest();
  let response;
  switch (request.action) {
    case "quote":
      response = await quoteLaunch(request);
      break;
    case "build-launch":
      response = await buildLaunch(request);
      break;
    case "compile-follow-buy":
      response = await compileFollowBuy(request, "follow-buy", false);
      break;
    case "compile-follow-buy-atomic":
      response = await compileFollowBuy(request, "follow-buy-atomic", true);
      break;
    case "compile-sol-to-usd1-topup":
      response = await buildUsd1Topup(request);
      break;
    case "compile-follow-sell":
      response = await compileFollowSell(request);
      break;
    case "predict-dev-buy-token-amount":
      response = await predictDevBuyTokenAmount(request);
      break;
    case "derive-pool-id":
      response = await deriveCanonicalPoolId(request);
      break;
    case "fetch-market-snapshot":
      response = await fetchMarketSnapshot(request);
      break;
    case "detect-import-context":
      response = await detectImportContext(request);
      break;
    case "warm-state":
      response = await warmState(request);
      break;
    default:
      throw new Error(`Unsupported bonk helper action: ${request.action || "(missing)"}`);
  }
  process.stdout.write(JSON.stringify(response));
}

main().catch((error) => {
  process.stderr.write(`${error && error.stack ? error.stack : String(error)}\n`);
  process.exit(1);
});
