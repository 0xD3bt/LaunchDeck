"use strict";

const fs = require("fs");
const path = require("path");

const PRODUCT_SLUG = "launchdeck";
const PROVIDERS = ["helius-sender", "hellomoon", "standard-rpc", "jito-bundle"];
const ENDPOINT_PROFILE_AGGREGATES = ["global", "us", "eu", "asia"];
const ENDPOINT_PROFILE_METROS = ["slc", "ewr", "lon", "fra", "ams", "sg", "tyo"];
const ENDPOINT_PROFILES = [...ENDPOINT_PROFILE_AGGREGATES, ...ENDPOINT_PROFILE_METROS];
const PROVIDER_ENDPOINT_PROFILES = {
  "helius-sender": ENDPOINT_PROFILES,
  hellomoon: ENDPOINT_PROFILES,
  "jito-bundle": ENDPOINT_PROFILES,
  "standard-rpc": [],
};
const LEGACY_PROVIDER_ALIASES = {
  auto: "helius-sender",
  helius: "helius-sender",
  jito: "jito-bundle",
  astralane: "standard-rpc",
  bloxroute: "standard-rpc",
  hellomoon: "hellomoon",
};
const VERIFIED_PROVIDERS = new Set(PROVIDERS);
const LAUNCHPADS = ["pump", "bonk", "bagsapp"];
const EXECUTION_POLICIES = ["fast", "safe"];
const AUTO_GAS_MODES = ["auto", "manual"];
const POST_LAUNCH_STRATEGIES = ["none", "dev-buy", "snipe-own-launch", "automatic-dev-sell"];
const MEV_MODES = ["off", "reduced", "secure"];
const PRESET_IDS = ["preset1", "preset2", "preset3"];
const DEFAULT_POLICY = "safe";
const DEFAULT_PROVIDER = "helius-sender";
const DEFAULT_ENDPOINT_PROFILE = "global";
const DEFAULT_DEV_BUY_AMOUNTS = ["0.5", "1", "2"];
const DEFAULT_CREATION_PRIORITY_FEE_SOL = "0.000001";
const DEFAULT_CREATION_TIP_SOL = "0.0002";
const DEFAULT_TRADE_PRIORITY_FEE_SOL = "0.000001";
const DEFAULT_TRADE_TIP_SOL = "0.0002";
const DEFAULT_TRADE_SLIPPAGE_PERCENT = "20";

function normalizeProvider(provider, fallback = DEFAULT_PROVIDER) {
  const normalized = String(provider || "").trim().toLowerCase();
  if (!normalized) return fallback;
  const migrated = LEGACY_PROVIDER_ALIASES[normalized] || normalized;
  return PROVIDERS.includes(migrated) ? migrated : fallback;
}

function providerSupportsEndpointProfiles(provider) {
  return Array.isArray(PROVIDER_ENDPOINT_PROFILES[provider]) && PROVIDER_ENDPOINT_PROFILES[provider].length > 0;
}

/**
 * Validates endpoint profile strings (aggregates, one metro, or comma-separated metros).
 * `ny` normalizes to `ewr`. `west` is rejected (falls back at call sites).
 */
function validateEndpointProfileLiteral(profile) {
  const raw = String(profile || "").trim().toLowerCase();
  if (!raw) return { ok: false };
  if (raw === "west") return { ok: false };
  if (raw.includes(",")) {
    const seen = new Set();
    const out = [];
    for (const part of raw.split(",")) {
      const p = part.trim().toLowerCase();
      if (!p) continue;
      if (ENDPOINT_PROFILE_AGGREGATES.includes(p)) return { ok: false };
      const metro = p === "ny" ? "ewr" : p;
      if (!ENDPOINT_PROFILE_METROS.includes(metro)) return { ok: false };
      if (!seen.has(metro)) {
        seen.add(metro);
        out.push(metro);
      }
    }
    return out.length ? { ok: true, value: out.join(",") } : { ok: false };
  }
  if (ENDPOINT_PROFILE_AGGREGATES.includes(raw)) return { ok: true, value: raw };
  const metro = raw === "ny" ? "ewr" : raw;
  if (ENDPOINT_PROFILE_METROS.includes(metro)) return { ok: true, value: metro };
  return { ok: false };
}

function normalizeEndpointProfile(provider, profile, fallback = DEFAULT_ENDPOINT_PROFILE) {
  const normalizedProvider = normalizeProvider(provider);
  if (!providerSupportsEndpointProfiles(normalizedProvider)) return "";
  const checked = validateEndpointProfileLiteral(profile);
  if (checked.ok) return checked.value;
  return fallback;
}

function normalizePolicy(policy, fallback = DEFAULT_POLICY) {
  const normalized = String(policy || "").trim().toLowerCase();
  return EXECUTION_POLICIES.includes(normalized) ? normalized : fallback;
}

function normalizeDecimalString(value, fallback = "") {
  const normalized = String(value || "").trim();
  return normalized || fallback;
}

function coerceBoolean(value, fallback = false) {
  if (value === undefined || value === null || value === "") return fallback;
  if (typeof value === "boolean") return value;
  if (typeof value === "string") {
    const normalized = value.trim().toLowerCase();
    if (normalized === "true") return true;
    if (normalized === "false") return false;
  }
  if (typeof value === "number") return value !== 0;
  return Boolean(value);
}

function normalizeMevMode(value, fallback = "off") {
  if (typeof value === "boolean") return value ? "reduced" : "off";
  const normalized = String(value || "").trim().toLowerCase();
  if (MEV_MODES.includes(normalized)) return normalized;
  return MEV_MODES.includes(fallback) ? fallback : "off";
}

function defaultMevModeForProvider(provider) {
  return normalizeProvider(provider) === "hellomoon" ? "reduced" : "off";
}

function coerceNumber(value, fallback = 0) {
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
}

function createCreationSettings({
  provider = DEFAULT_PROVIDER,
  endpointProfile = DEFAULT_ENDPOINT_PROFILE,
  policy = DEFAULT_POLICY,
  tipSol = DEFAULT_CREATION_TIP_SOL,
  priorityFeeSol = DEFAULT_CREATION_PRIORITY_FEE_SOL,
  devBuySol = "",
  mevMode,
} = {}) {
  const normalizedProvider = normalizeProvider(provider);
  return {
    provider: normalizedProvider,
    endpointProfile: normalizeEndpointProfile(normalizedProvider, endpointProfile),
    policy: normalizePolicy(policy),
    tipSol: normalizeDecimalString(tipSol, DEFAULT_CREATION_TIP_SOL),
    priorityFeeSol: normalizeDecimalString(priorityFeeSol, DEFAULT_CREATION_PRIORITY_FEE_SOL),
    devBuySol: normalizeDecimalString(devBuySol),
    mevMode: normalizeMevMode(mevMode, defaultMevModeForProvider(normalizedProvider)),
  };
}

function createTradeSettings({
  provider = DEFAULT_PROVIDER,
  endpointProfile = DEFAULT_ENDPOINT_PROFILE,
  policy = DEFAULT_POLICY,
  priorityFeeSol = DEFAULT_TRADE_PRIORITY_FEE_SOL,
  tipSol = DEFAULT_TRADE_TIP_SOL,
  slippagePercent = DEFAULT_TRADE_SLIPPAGE_PERCENT,
  mevMode,
  mevProtect = false,
} = {}) {
  const normalizedProvider = normalizeProvider(provider);
  return {
    provider: normalizedProvider,
    endpointProfile: normalizeEndpointProfile(normalizedProvider, endpointProfile),
    policy: normalizePolicy(policy),
    priorityFeeSol: normalizeDecimalString(priorityFeeSol, DEFAULT_TRADE_PRIORITY_FEE_SOL),
    tipSol: normalizeDecimalString(tipSol, DEFAULT_TRADE_TIP_SOL),
    slippagePercent: normalizeDecimalString(slippagePercent, DEFAULT_TRADE_SLIPPAGE_PERCENT),
    mevMode: normalizeMevMode(
      mevMode,
      coerceBoolean(mevProtect, false) ? "reduced" : defaultMevModeForProvider(normalizedProvider)
    ),
  };
}

function createDefaultPreset(id, label, devBuySol = "", overrides = {}) {
  return {
    id,
    label,
    creationSettings: createCreationSettings({ devBuySol, ...(overrides.creationSettings || {}) }),
    buySettings: {
      ...createTradeSettings(overrides.buySettings || {}),
      snipeBuyAmountSol: normalizeDecimalString(overrides.buySettings && overrides.buySettings.snipeBuyAmountSol),
    },
    sellSettings: createTradeSettings(overrides.sellSettings || {}),
    postLaunchStrategy: String(overrides.postLaunchStrategy || "none").trim() || "none",
  };
}

function createDefaultPersistentConfig() {
  return {
    defaults: {
      launchpad: "pump",
      mode: "regular",
      activePresetId: "preset1",
      presetEditing: false,
      misc: {
        trackSendBlockHeight: false,
      },
      automaticDevSell: {
        enabled: false,
        percent: 0,
        delaySeconds: 0,
      },
    },
    presets: {
      items: PRESET_IDS.map((id, index) => createDefaultPreset(id, `P${index + 1}`, DEFAULT_DEV_BUY_AMOUNTS[index])),
    },
  };
}

function ensureDir(dirPath) {
  fs.mkdirSync(dirPath, { recursive: true });
}

function getLocalDataDir(baseDir) {
  return path.join(baseDir, ".local", PRODUCT_SLUG);
}

function getPersistentConfigPath(baseDir) {
  return path.join(getLocalDataDir(baseDir), "app-config.json");
}

function deepClone(value) {
  return JSON.parse(JSON.stringify(value));
}

function mergeObjects(target, source) {
  if (!source || typeof source !== "object" || Array.isArray(source)) {
    return source === undefined ? target : source;
  }
  const output = { ...(target || {}) };
  for (const [key, value] of Object.entries(source)) {
    if (Array.isArray(value)) {
      output[key] = value.map((item) => deepClone(item));
    } else if (value && typeof value === "object") {
      output[key] = mergeObjects(output[key], value);
    } else if (value !== undefined) {
      output[key] = value;
    }
  }
  return output;
}

function firstNonEmpty(...values) {
  for (const value of values) {
    const normalized = String(value || "").trim();
    if (normalized) return normalized;
  }
  return "";
}

function normalizePresetShape(preset, fallbackPreset, index) {
  const creationProvider = normalizeProvider(
    preset && preset.creationSettings && preset.creationSettings.provider,
    fallbackPreset.creationSettings.provider
  );
  const buyProvider = normalizeProvider(
    preset && preset.buySettings && preset.buySettings.provider,
    fallbackPreset.buySettings.provider
  );
  const sellProvider = normalizeProvider(
    preset && preset.sellSettings && preset.sellSettings.provider,
    fallbackPreset.sellSettings.provider
  );
  return {
    ...fallbackPreset,
    ...(preset || {}),
    id: String((preset && preset.id) || fallbackPreset.id || `preset${index + 1}`).trim() || `preset${index + 1}`,
    label: String((preset && preset.label) || fallbackPreset.label || `P${index + 1}`).trim() || `P${index + 1}`,
    creationSettings: {
      ...fallbackPreset.creationSettings,
      ...(preset && preset.creationSettings ? preset.creationSettings : {}),
      provider: creationProvider,
      endpointProfile: normalizeEndpointProfile(
        creationProvider,
        preset && preset.creationSettings && preset.creationSettings.endpointProfile,
        fallbackPreset.creationSettings.endpointProfile
      ),
      policy: normalizePolicy(preset && preset.creationSettings && preset.creationSettings.policy, fallbackPreset.creationSettings.policy),
      tipSol: normalizeDecimalString(preset && preset.creationSettings && preset.creationSettings.tipSol, fallbackPreset.creationSettings.tipSol),
      priorityFeeSol: normalizeDecimalString(preset && preset.creationSettings && preset.creationSettings.priorityFeeSol, fallbackPreset.creationSettings.priorityFeeSol),
      devBuySol: normalizeDecimalString(preset && preset.creationSettings && preset.creationSettings.devBuySol, fallbackPreset.creationSettings.devBuySol),
      mevMode: normalizeMevMode(
        preset && preset.creationSettings && preset.creationSettings.mevMode,
        defaultMevModeForProvider(creationProvider)
      ),
    },
    buySettings: {
      ...fallbackPreset.buySettings,
      ...(preset && preset.buySettings ? preset.buySettings : {}),
      provider: buyProvider,
      endpointProfile: normalizeEndpointProfile(
        buyProvider,
        preset && preset.buySettings && preset.buySettings.endpointProfile,
        fallbackPreset.buySettings.endpointProfile
      ),
      policy: normalizePolicy(preset && preset.buySettings && preset.buySettings.policy, fallbackPreset.buySettings.policy),
      priorityFeeSol: normalizeDecimalString(preset && preset.buySettings && preset.buySettings.priorityFeeSol, fallbackPreset.buySettings.priorityFeeSol),
      tipSol: normalizeDecimalString(preset && preset.buySettings && preset.buySettings.tipSol, fallbackPreset.buySettings.tipSol),
      slippagePercent: normalizeDecimalString(preset && preset.buySettings && preset.buySettings.slippagePercent, fallbackPreset.buySettings.slippagePercent),
      mevMode: normalizeMevMode(
        preset && preset.buySettings && (preset.buySettings.mevMode ?? preset.buySettings.mevProtect),
        defaultMevModeForProvider(buyProvider)
      ),
      snipeBuyAmountSol: normalizeDecimalString(preset && preset.buySettings && preset.buySettings.snipeBuyAmountSol, fallbackPreset.buySettings.snipeBuyAmountSol),
    },
    sellSettings: {
      ...fallbackPreset.sellSettings,
      ...(preset && preset.sellSettings ? preset.sellSettings : {}),
      provider: sellProvider,
      endpointProfile: normalizeEndpointProfile(
        sellProvider,
        preset && preset.sellSettings && preset.sellSettings.endpointProfile,
        fallbackPreset.sellSettings.endpointProfile
      ),
      policy: normalizePolicy(preset && preset.sellSettings && preset.sellSettings.policy, fallbackPreset.sellSettings.policy),
      priorityFeeSol: normalizeDecimalString(preset && preset.sellSettings && preset.sellSettings.priorityFeeSol, fallbackPreset.sellSettings.priorityFeeSol),
      tipSol: normalizeDecimalString(preset && preset.sellSettings && preset.sellSettings.tipSol, fallbackPreset.sellSettings.tipSol),
      slippagePercent: normalizeDecimalString(preset && preset.sellSettings && preset.sellSettings.slippagePercent, fallbackPreset.sellSettings.slippagePercent),
      mevMode: normalizeMevMode(
        preset && preset.sellSettings && (preset.sellSettings.mevMode ?? preset.sellSettings.mevProtect),
        defaultMevModeForProvider(sellProvider)
      ),
    },
    postLaunchStrategy: String((preset && preset.postLaunchStrategy) || fallbackPreset.postLaunchStrategy || "none").trim() || "none",
  };
}

function migrateLegacyConfig(parsed) {
  const base = createDefaultPersistentConfig();
  const defaults = parsed && parsed.defaults ? parsed.defaults : {};
  const legacyLaunchDefaults = defaults.launchExecution || {};
  const legacyBuyDefaults = defaults.buyExecution || {};
  const legacyAutoSell = defaults.automaticDevSell || {};
  const legacyLaunchPresets = parsed && parsed.presets && Array.isArray(parsed.presets.launch) ? parsed.presets.launch : [];
  const legacySniperPresets = parsed && parsed.presets && Array.isArray(parsed.presets.sniper) ? parsed.presets.sniper : [];

  const items = PRESET_IDS.map((id, index) => {
    const fallbackPreset = base.presets.items[index];
    const launchPreset = legacyLaunchPresets[index] || {};
    const sniperPreset = legacySniperPresets[index] || {};
    const launchExecution = launchPreset.execution || legacyLaunchDefaults;
    const buyExecution = sniperPreset.execution || legacyBuyDefaults;
    const creationPriorityFeeSol = firstNonEmpty(launchExecution.priorityFeeSol, legacyLaunchDefaults.priorityFeeSol);
    const buyPriorityFeeSol = firstNonEmpty(
      buyExecution.priorityFeeSol,
      buyExecution.maxPriorityFeeSol,
      legacyBuyDefaults.priorityFeeSol,
      legacyBuyDefaults.maxPriorityFeeSol,
      fallbackPreset.buySettings.priorityFeeSol
    );
    const buyTipSol = firstNonEmpty(
      buyExecution.tipSol,
      buyExecution.maxTipSol,
      legacyBuyDefaults.tipSol,
      legacyBuyDefaults.maxTipSol,
      fallbackPreset.buySettings.tipSol
    );
    const sellPriorityFeeSol = buyPriorityFeeSol || fallbackPreset.sellSettings.priorityFeeSol;
    const sellTipSol = buyTipSol || fallbackPreset.sellSettings.tipSol;

    return normalizePresetShape({
      id: launchPreset.id || sniperPreset.id || id,
      label: launchPreset.label || sniperPreset.label || `P${index + 1}`,
      creationSettings: {
        provider: normalizeProvider(launchExecution.provider, fallbackPreset.creationSettings.provider),
        endpointProfile: normalizeEndpointProfile(
          launchExecution.provider,
          launchExecution.endpointProfile,
          fallbackPreset.creationSettings.endpointProfile
        ),
        policy: normalizePolicy(launchExecution.policy, fallbackPreset.creationSettings.policy),
        tipSol: firstNonEmpty(launchExecution.tipSol, launchExecution.maxTipSol, legacyLaunchDefaults.tipSol, legacyLaunchDefaults.maxTipSol, fallbackPreset.creationSettings.tipSol),
        priorityFeeSol: creationPriorityFeeSol,
        devBuySol: normalizeDecimalString(launchPreset.buyAmountSol, fallbackPreset.creationSettings.devBuySol),
      },
      buySettings: {
        provider: normalizeProvider(buyExecution.provider, fallbackPreset.buySettings.provider),
        endpointProfile: normalizeEndpointProfile(
          buyExecution.provider,
          buyExecution.endpointProfile,
          fallbackPreset.buySettings.endpointProfile
        ),
        policy: normalizePolicy(buyExecution.policy, fallbackPreset.buySettings.policy),
        priorityFeeSol: buyPriorityFeeSol,
        tipSol: buyTipSol,
        slippagePercent: fallbackPreset.buySettings.slippagePercent,
        snipeBuyAmountSol: normalizeDecimalString(sniperPreset.buyAmountSol, fallbackPreset.buySettings.snipeBuyAmountSol),
      },
      sellSettings: {
        provider: normalizeProvider(buyExecution.provider, fallbackPreset.sellSettings.provider),
        endpointProfile: normalizeEndpointProfile(
          buyExecution.provider,
          buyExecution.endpointProfile,
          fallbackPreset.sellSettings.endpointProfile
        ),
        policy: normalizePolicy(buyExecution.policy, fallbackPreset.sellSettings.policy),
        priorityFeeSol: sellPriorityFeeSol,
        tipSol: sellTipSol,
        slippagePercent: fallbackPreset.sellSettings.slippagePercent,
      },
      postLaunchStrategy: String(defaults.postLaunchStrategy || "none").trim() || "none",
    }, fallbackPreset, index);
  });

  return {
    defaults: {
      ...base.defaults,
      launchpad: String(defaults.launchpad || base.defaults.launchpad).trim() || base.defaults.launchpad,
      mode: String(defaults.mode || base.defaults.mode).trim() || base.defaults.mode,
      activePresetId: PRESET_IDS.includes(String(defaults.activePresetId || "")) ? String(defaults.activePresetId) : "preset1",
      presetEditing: coerceBoolean(defaults.presetEditing, false),
      automaticDevSell: {
        enabled: coerceBoolean(legacyAutoSell.enabled, false),
        percent: coerceNumber(legacyAutoSell.percent, 0),
        delaySeconds: coerceNumber(legacyAutoSell.delaySeconds, 0),
      },
    },
    presets: {
      items,
    },
  };
}

function normalizePersistentConfig(parsed) {
  const base = createDefaultPersistentConfig();
  const hasNewPresetShape = Boolean(parsed && parsed.presets && Array.isArray(parsed.presets.items));
  if (!hasNewPresetShape) {
    return migrateLegacyConfig(parsed || {});
  }

  const merged = mergeObjects(base, parsed);
  const items = PRESET_IDS.map((id, index) => {
    const fallbackPreset = base.presets.items[index];
    const existing = Array.isArray(merged.presets.items)
      ? merged.presets.items.find((entry) => String(entry && entry.id) === id) || merged.presets.items[index]
      : null;
    return normalizePresetShape(existing, fallbackPreset, index);
  });

  return {
    defaults: {
      ...base.defaults,
      ...(merged.defaults || {}),
      launchpad: String((merged.defaults && merged.defaults.launchpad) || base.defaults.launchpad).trim() || base.defaults.launchpad,
      mode: String((merged.defaults && merged.defaults.mode) || base.defaults.mode).trim() || base.defaults.mode,
      activePresetId: PRESET_IDS.includes(String(merged.defaults && merged.defaults.activePresetId))
        ? String(merged.defaults.activePresetId)
        : "preset1",
      presetEditing: coerceBoolean(merged.defaults && merged.defaults.presetEditing, false),
      misc: {
        trackSendBlockHeight: coerceBoolean(
          merged.defaults && merged.defaults.misc && merged.defaults.misc.trackSendBlockHeight,
          base.defaults.misc.trackSendBlockHeight
        ),
      },
      automaticDevSell: {
        enabled: coerceBoolean(merged.defaults && merged.defaults.automaticDevSell && merged.defaults.automaticDevSell.enabled, base.defaults.automaticDevSell.enabled),
        percent: coerceNumber(merged.defaults && merged.defaults.automaticDevSell && merged.defaults.automaticDevSell.percent, base.defaults.automaticDevSell.percent),
        delaySeconds: coerceNumber(merged.defaults && merged.defaults.automaticDevSell && merged.defaults.automaticDevSell.delaySeconds, base.defaults.automaticDevSell.delaySeconds),
      },
    },
    presets: {
      items,
    },
  };
}

function readPersistentConfig(baseDir) {
  const filePath = getPersistentConfigPath(baseDir);
  if (!fs.existsSync(filePath)) {
    return createDefaultPersistentConfig();
  }

  const raw = fs.readFileSync(filePath, "utf8").trim();
  if (!raw) {
    return createDefaultPersistentConfig();
  }

  const parsed = JSON.parse(raw);
  return normalizePersistentConfig(parsed);
}

function writePersistentConfig(baseDir, nextConfig) {
  const filePath = getPersistentConfigPath(baseDir);
  ensureDir(path.dirname(filePath));
  fs.writeFileSync(filePath, JSON.stringify(nextConfig, null, 2), "utf8");
  return filePath;
}

function resolveProviderSupport(provider) {
  return {
    verified: VERIFIED_PROVIDERS.has(provider),
    supportsSingle: true,
    supportsBundle: provider === "jito-bundle" || provider === "hellomoon",
    supportsSequential: provider !== "jito-bundle",
    supportsEndpointProfiles: providerSupportsEndpointProfiles(provider),
    endpointProfiles: PROVIDER_ENDPOINT_PROFILES[provider] || [],
    supportState: VERIFIED_PROVIDERS.has(provider) ? "verified" : "unverified",
  };
}

module.exports = {
  AUTO_GAS_MODES,
  EXECUTION_POLICIES,
  ENDPOINT_PROFILES,
  LAUNCHPADS,
  MEV_MODES,
  POST_LAUNCH_STRATEGIES,
  PRODUCT_SLUG,
  PROVIDERS,
  createCreationSettings,
  createTradeSettings,
  createDefaultPreset,
  createDefaultPersistentConfig,
  getLocalDataDir,
  getPersistentConfigPath,
  normalizePersistentConfig,
  readPersistentConfig,
  normalizeEndpointProfile,
  normalizeMevMode,
  providerSupportsEndpointProfiles,
  resolveProviderSupport,
  writePersistentConfig,
};
