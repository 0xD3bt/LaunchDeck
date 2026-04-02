#!/usr/bin/env node

require("dotenv").config();

const fs = require("fs");
const bs58 = require("bs58").default;
const {
  AddressLookupTableProgram,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  VersionedTransaction,
} = require("@solana/web3.js");

const DEFAULT_STATE_PATH = ".local/launchdeck/follow-daemon-state.json";
const DEFAULT_WALLET_ENV_KEY = "SOLANA_PRIVATE_KEY";
const EXTEND_CHUNK_SIZE = 20;
const BONK_USD1_SUPER_LOOKUP_TABLE = "GHVFasDr4sFtF2fMNBLnaRUKeSxX77DgK5SsThB3Ro7U";

function parseArgs(argv) {
  const options = {
    statePath: DEFAULT_STATE_PATH,
    walletEnvKey: DEFAULT_WALLET_ENV_KEY,
    dryRun: false,
    traceId: "",
    table: "",
    extraAddresses: [],
  };
  for (let index = 0; index < argv.length; index += 1) {
    const entry = argv[index];
    switch (entry) {
      case "--trace-id":
        options.traceId = argv[++index] || "";
        break;
      case "--state-path":
        options.statePath = argv[++index] || DEFAULT_STATE_PATH;
        break;
      case "--table":
        options.table = argv[++index] || "";
        break;
      case "--wallet-env":
        options.walletEnvKey = argv[++index] || DEFAULT_WALLET_ENV_KEY;
        break;
      case "--add":
        options.extraAddresses.push(...splitCsv(argv[++index] || ""));
        break;
      case "--dry-run":
        options.dryRun = true;
        break;
      default:
        throw new Error(`Unknown argument: ${entry}`);
    }
  }
  return options;
}

function splitCsv(value) {
  return String(value || "")
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}

function loadWallet(envKey) {
  const raw = process.env[envKey];
  if (!raw) {
    throw new Error(`Missing env wallet: ${envKey}`);
  }
  const encoded = raw.split(",")[0].trim();
  const secret = bs58.decode(encoded);
  return Keypair.fromSecretKey(secret);
}

function resolveRpcUrl() {
  return process.env.SOLANA_RPC_URL || process.env.RPC_URL || process.env.HELIUS_RPC_URL || "";
}

function resolveDestinationTable(options) {
  return options.table || BONK_USD1_SUPER_LOOKUP_TABLE;
}

function loadTraceTransactions(statePath, traceId) {
  const payload = JSON.parse(fs.readFileSync(statePath, "utf8"));
  const jobs = Array.isArray(payload.jobs) ? payload.jobs : [];
  const job = jobs.find((entry) => entry.traceId === traceId);
  if (!job) {
    throw new Error(`Trace id not found in state file: ${traceId}`);
  }
  const transactions = [];
  for (const action of job.actions || []) {
    for (const tx of action.preSignedTransactions || []) {
      if (!tx || !tx.serializedBase64) {
        continue;
      }
      transactions.push({
        actionId: action.actionId,
        transaction: VersionedTransaction.deserialize(Buffer.from(tx.serializedBase64, "base64")),
      });
    }
  }
  if (!transactions.length) {
    throw new Error(`Trace id ${traceId} has no pre-signed transactions in ${statePath}.`);
  }
  return transactions;
}

async function loadLookupTable(connection, address) {
  const response = await connection.getAddressLookupTable(new PublicKey(address));
  if (!response || !response.value) {
    throw new Error(`Address lookup table not found: ${address}`);
  }
  return response.value;
}

async function collectLookedUpAddresses(connection, transactions) {
  const addresses = new Map();
  for (const { actionId, transaction } of transactions) {
    for (const lookup of transaction.message.addressTableLookups || []) {
      const table = await loadLookupTable(connection, lookup.accountKey.toBase58());
      const indexes = [
        ...Array.from(lookup.writableIndexes || []),
        ...Array.from(lookup.readonlyIndexes || []),
      ];
      for (const index of indexes) {
        const key = table.state.addresses[index];
        if (!key) {
          throw new Error(
            `Lookup table ${table.key.toBase58()} missing address at index ${index} for action ${actionId}.`,
          );
        }
        addresses.set(key.toBase58(), {
          address: key.toBase58(),
          sourceTable: table.key.toBase58(),
          actionId,
        });
      }
    }
  }
  return Array.from(addresses.values());
}

function chunk(array, size) {
  const chunks = [];
  for (let index = 0; index < array.length; index += size) {
    chunks.push(array.slice(index, index + size));
  }
  return chunks;
}

async function extendLookupTable(connection, wallet, tableAddress, addresses) {
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
  const instruction = AddressLookupTableProgram.extendLookupTable({
    lookupTable: new PublicKey(tableAddress),
    authority: wallet.publicKey,
    payer: wallet.publicKey,
    addresses: addresses.map((entry) => new PublicKey(entry)),
  });
  const transaction = new Transaction();
  transaction.feePayer = wallet.publicKey;
  transaction.recentBlockhash = blockhash;
  transaction.lastValidBlockHeight = lastValidBlockHeight;
  transaction.add(instruction);
  transaction.sign(wallet);
  const signature = await connection.sendRawTransaction(transaction.serialize(), {
    skipPreflight: false,
    maxRetries: 3,
    preflightCommitment: "confirmed",
  });
  await connection.confirmTransaction(
    { signature, blockhash, lastValidBlockHeight },
    "confirmed",
  );
  return signature;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!options.traceId && !options.extraAddresses.length) {
    throw new Error("Provide --trace-id and/or at least one --add address.");
  }
  const rpcUrl = resolveRpcUrl();
  if (!rpcUrl) {
    throw new Error("SOLANA_RPC_URL is not configured.");
  }

  const wallet = loadWallet(options.walletEnvKey);
  const connection = new Connection(rpcUrl, "confirmed");
  const tableAddress = resolveDestinationTable(options);
  const destinationTable = await loadLookupTable(connection, tableAddress);

  if (!destinationTable.state.authority || !destinationTable.state.authority.equals(wallet.publicKey)) {
    throw new Error(
      `Wallet ${wallet.publicKey.toBase58()} is not the authority for ${tableAddress}.`,
    );
  }

  const transactions = options.traceId
    ? loadTraceTransactions(options.statePath, options.traceId)
    : [];
  const collected = transactions.length
    ? await collectLookedUpAddresses(connection, transactions)
    : [];
  const requested = new Map();
  collected.forEach((entry) => requested.set(entry.address, entry));
  options.extraAddresses.forEach((entry) => {
    requested.set(entry, {
      address: entry,
      sourceTable: "manual",
      actionId: "manual",
    });
  });

  const existing = new Set(destinationTable.state.addresses.map((entry) => entry.toBase58()));
  const missing = Array.from(requested.values()).filter((entry) => !existing.has(entry.address));

  console.log(`wallet=${wallet.publicKey.toBase58()}`);
  console.log(`table=${tableAddress}`);
  console.log(`existingAddressCount=${destinationTable.state.addresses.length}`);
  console.log(`candidateAddressCount=${requested.size}`);
  console.log(`missingAddressCount=${missing.length}`);
  missing.forEach((entry) => {
    console.log(`missing ${entry.address} source=${entry.sourceTable} action=${entry.actionId}`);
  });

  if (!missing.length) {
    return;
  }
  if (destinationTable.state.addresses.length + missing.length > 256) {
    throw new Error(
      `Extending ${tableAddress} with ${missing.length} addresses would exceed the 256-address ALT limit.`,
    );
  }
  if (options.dryRun) {
    return;
  }

  const chunks = chunk(
    missing.map((entry) => entry.address),
    EXTEND_CHUNK_SIZE,
  );
  for (const [index, group] of chunks.entries()) {
    const signature = await extendLookupTable(connection, wallet, tableAddress, group);
    console.log(`extended chunk=${index + 1}/${chunks.length} count=${group.length} signature=${signature}`);
  }

  const refreshed = await loadLookupTable(connection, tableAddress);
  console.log(`finalAddressCount=${refreshed.state.addresses.length}`);
}

main().catch((error) => {
  console.error(error && error.message ? error.message : String(error));
  process.exit(1);
});
