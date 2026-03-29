const form = document.getElementById("launch-form");
const output = document.getElementById("output");
const statusNode = document.getElementById("status");
const metaNode = document.getElementById("meta");
const outputSection = document.getElementById("output-section");
const reportsTerminalSection = document.getElementById("reports-terminal-section");
const reportsTerminalList = document.getElementById("reports-terminal-list");
const reportsTerminalOutput = document.getElementById("reports-terminal-output");
const reportsTerminalMeta = document.getElementById("reports-terminal-meta");
const reportsTerminalResizeHandle = document.getElementById("reports-terminal-resize-handle");
const buttons = Array.from(document.querySelectorAll("[data-action]"));
const shellMain = document.querySelector(".shell");
const workspaceShell = document.querySelector(".workspace-shell");
const walletBox = document.querySelector(".wallet-box");
const walletSelect = document.getElementById("wallet-select");
const walletBalance = document.getElementById("wallet-balance");
const walletTriggerButton = document.getElementById("wallet-trigger-button");
const walletDropdown = document.getElementById("wallet-dropdown");
const walletDropdownList = document.getElementById("wallet-dropdown-list");
const walletRefreshButton = document.getElementById("wallet-refresh-button");
const walletSummarySol = document.getElementById("wallet-summary-sol");
const walletSummaryUsd = document.getElementById("wallet-summary-usd");
const topPresetChipBar = document.getElementById("top-preset-chip-bar");
const openVampButton = document.getElementById("open-vamp-button");
const themeToggleButton = document.getElementById("toggle-theme-button");
const feeSplitPill = document.getElementById("fee-split-pill");
const imageInput = document.getElementById("image-input");
const openImageLibraryButton = document.getElementById("open-image-library-button");
const imageLayoutToggle = document.getElementById("image-layout-toggle");
const tokenSurfaceSection = document.getElementById("token-surface-section");
const imagePreview = document.getElementById("image-preview");
const imageEmpty = document.getElementById("image-empty");
const imageStatus = document.getElementById("image-status");
const imagePath = document.getElementById("image-path");
const imageLibraryModal = document.getElementById("image-library-modal");
const imageLibraryClose = document.getElementById("image-library-close");
const imageLibrarySearchInput = document.getElementById("image-library-search-input");
const imageLibraryUploadButton = document.getElementById("image-library-upload-button");
const imageLibraryGrid = document.getElementById("image-library-grid");
const imageLibraryEmpty = document.getElementById("image-library-empty");
const imageCategoryChips = document.getElementById("image-category-chips");
const newImageCategoryButton = document.getElementById("new-image-category-button");
const imageItemMenu = document.getElementById("image-item-menu");
const imageMenuFavorite = document.getElementById("image-menu-favorite");
const imageMenuEdit = document.getElementById("image-menu-edit");
const imageMenuDelete = document.getElementById("image-menu-delete");
const imageDetailsModal = document.getElementById("image-details-modal");
const imageDetailsTitle = document.getElementById("image-details-title");
const imageDetailsClose = document.getElementById("image-details-close");
const imageDetailsCancel = document.getElementById("image-details-cancel");
const imageDetailsSave = document.getElementById("image-details-save");
const imageDetailsName = document.getElementById("image-details-name");
const imageDetailsTags = document.getElementById("image-details-tags");
const imageDetailsAddTag = document.getElementById("image-details-add-tag");
const imageDetailsTagList = document.getElementById("image-details-tag-list");
const imageDetailsError = document.getElementById("image-details-error");
const imageDetailsCategoryRow = document.getElementById("image-details-category-row");
const imageDetailsCategory = document.getElementById("image-details-category");
const imageDetailsNewCategory = document.getElementById("image-details-new-category");
const imageCategoryModal = document.getElementById("image-category-modal");
const imageCategoryClose = document.getElementById("image-category-close");
const imageCategoryCancel = document.getElementById("image-category-cancel");
const imageCategorySave = document.getElementById("image-category-save");
const imageCategoryName = document.getElementById("image-category-name");
const imageCategoryError = document.getElementById("image-category-error");
const metadataUri = document.getElementById("metadata-uri");
const nameInput = form.querySelector('[name="name"]');
const symbolInput = form.querySelector('[name="symbol"]');
const descriptionInput = form.querySelector('[name="description"]');
const websiteInput = form.querySelector('[name="website"]');
const twitterInput = form.querySelector('[name="twitter"]');
const telegramInput = form.querySelector('[name="telegram"]');
const descriptionDisclosure = document.getElementById("description-disclosure");
const descriptionToggle = document.getElementById("description-toggle");
const descriptionPanelBody = document.getElementById("description-panel-body");
const descriptionCharCount = document.getElementById("description-char-count");
const nameCharCount = document.getElementById("name-char-count");
const symbolCharCount = document.getElementById("symbol-char-count");
const tickerCapsToggle = document.getElementById("ticker-caps-toggle");
const devBuyModeInput = getNamedInput("devBuyMode");
const devBuyAmountInput = getNamedInput("devBuyAmount");
const postLaunchStrategyInput = getNamedInput("postLaunchStrategy");
const snipeBuyAmountInput = getNamedInput("snipeBuyAmountSol");
const sniperEnabledInput = getNamedInput("sniperEnabled");
const sniperConfigJsonInput = getNamedInput("sniperConfigJson");
const vanityPrivateKeyInput = getNamedInput("vanityPrivateKey");
const devBuyQuickButtons = document.getElementById("dev-buy-quick-buttons");
const changeDevBuyPresetsButton = document.getElementById("change-dev-buy-presets-button");
const cancelDevBuyPresetsButton = document.getElementById("cancel-dev-buy-presets-button");
const saveDevBuyPresetsButton = document.getElementById("save-dev-buy-presets-button");
const devBuySolInput = document.getElementById("dev-buy-sol-input");
const devBuyPercentInput = document.getElementById("dev-buy-percent-input");
const devBuyCustomDeployButton = document.getElementById("dev-buy-custom-deploy");
const quoteOutput = document.getElementById("quote-output");
const creationTipInput = document.getElementById("creation-tip-input");
const creationPriorityInput = document.getElementById("creation-priority-input");
const launchpadInputs = Array.from(document.querySelectorAll('input[name="launchpad"]'));
const providerSelect = document.getElementById("provider-select");
const buyProviderSelect = document.getElementById("buy-provider-select");
const sellProviderSelect = document.getElementById("sell-provider-select");
const settingsBackendRegionSummary = document.getElementById("settings-backend-region-summary");
const buyPriorityFeeInput = document.getElementById("buy-priority-fee-input");
const buyTipInput = document.getElementById("buy-tip-input");
const buySlippageInput = document.getElementById("buy-slippage-input");
const sellPriorityFeeInput = document.getElementById("sell-priority-fee-input");
const sellTipInput = document.getElementById("sell-tip-input");
const sellSlippageInput = document.getElementById("sell-slippage-input");
const settingsPresetChipBar = document.getElementById("settings-preset-chip-bar");
const presetEditToggle = document.getElementById("preset-edit-toggle");
const agentUnlockedAuthority = document.getElementById("agent-unlocked-authority");
const agentSplitList = document.getElementById("agent-split-list");
const agentSplitAdd = document.getElementById("agent-split-add");
const agentSplitReset = document.getElementById("agent-split-reset");
const agentSplitEven = document.getElementById("agent-split-even");
const agentSplitTotal = document.getElementById("agent-split-total");
const agentSplitBar = document.getElementById("agent-split-bar");
const agentSplitLegendList = document.getElementById("agent-split-legend-list");
const agentSplitModal = document.getElementById("agent-split-modal");
const agentSplitClose = document.getElementById("agent-split-close");
const agentSplitCancel = document.getElementById("agent-split-cancel");
const agentSplitSave = document.getElementById("agent-split-save");
const agentSplitModalError = document.getElementById("agent-split-modal-error");
const feeSplitEnabled = form.querySelector('[name="feeSplitEnabled"]');
const feeSplitList = document.getElementById("fee-split-list");
const feeSplitAdd = document.getElementById("fee-split-add");
const feeSplitReset = document.getElementById("fee-split-reset");
const feeSplitEven = document.getElementById("fee-split-even");
const feeSplitTotal = document.getElementById("fee-split-total");
const feeSplitBar = document.getElementById("fee-split-bar");
const feeSplitLegendList = document.getElementById("fee-split-legend-list");
const feeSplitModal = document.getElementById("fee-split-modal");
const feeSplitClose = document.getElementById("fee-split-close");
const feeSplitDisable = document.getElementById("fee-split-disable");
const feeSplitSave = document.getElementById("fee-split-save");
const deployModal = document.getElementById("deploy-modal");
const modalBody = document.getElementById("modal-body");
const modalClose = document.getElementById("modal-close");
const modalCancel = document.getElementById("modal-cancel");
const modalConfirm = document.getElementById("modal-confirm");
const testFillButton = document.getElementById("test-fill-button");
const openPopoutButton = document.getElementById("open-popout-button");
const toggleOutputButton = document.getElementById("toggle-output-button");
const toggleReportsButton = document.getElementById("toggle-reports-button");
const reportsRefreshButton = document.getElementById("reports-refresh-button");
const reportsSortButton = document.getElementById("reports-sort-button");
const openSettingsButton = document.getElementById("open-settings-button");
const saveSettingsButton = document.getElementById("save-settings-button");
const settingsModal = document.getElementById("settings-modal");
const settingsClose = document.getElementById("settings-close");
const settingsCancel = document.getElementById("settings-cancel");
const modeSniperButton = document.getElementById("mode-sniper-button");
const modeVanityButton = document.getElementById("mode-vanity-button");
const devAutoSellButton = document.getElementById("dev-auto-sell-button");
const devAutoSellPanel = document.getElementById("dev-auto-sell-panel");
const autoSellEnabledInput = document.getElementById("auto-sell-enabled-input");
const autoSellToggleState = document.getElementById("auto-sell-toggle-state");
const autoSellTriggerValue = document.getElementById("auto-sell-trigger-value");
const autoSellTriggerDescription = document.getElementById("auto-sell-trigger-description");
const autoSellDelaySlider = document.getElementById("auto-sell-delay-slider");
const autoSellDelayControl = document.getElementById("auto-sell-delay-control");
const autoSellPercentSlider = document.getElementById("auto-sell-percent-slider");
const autoSellDelayValue = document.getElementById("auto-sell-delay-value");
const autoSellBlockControl = document.getElementById("auto-sell-block-control");
const autoSellBlockValue = document.getElementById("auto-sell-block-value");
const autoSellPercentValue = document.getElementById("auto-sell-percent-value");
const autoSellSettings = document.getElementById("auto-sell-settings");
const autoSellTriggerModeButtons = Array.from(document.querySelectorAll("[data-auto-sell-trigger-mode]"));
const autoSellBlockOffsetButtons = Array.from(document.querySelectorAll("[data-auto-sell-block-offset]"));
const sniperModal = document.getElementById("sniper-modal");
const sniperClose = document.getElementById("sniper-close");
const sniperCancel = document.getElementById("sniper-cancel");
const sniperSave = document.getElementById("sniper-save");
const sniperRefreshButton = document.getElementById("sniper-refresh-button");
const sniperResetButton = document.getElementById("sniper-reset-button");
const sniperEnabledToggle = document.getElementById("sniper-enabled-toggle");
const sniperEnabledState = document.getElementById("sniper-enabled-state");
const sniperWalletsSection = document.getElementById("sniper-wallets-section");
const sniperWalletList = document.getElementById("sniper-wallet-list");
const sniperSelectionSummary = document.getElementById("sniper-selection-summary");
const sniperTotalSummary = document.getElementById("sniper-total-summary");
const sniperModalError = document.getElementById("sniper-modal-error");
const vanityModal = document.getElementById("vanity-modal");
const vanityClose = document.getElementById("vanity-close");
const vanitySave = document.getElementById("vanity-save");
const vanityClear = document.getElementById("vanity-clear");
const vanityPrivateKeyText = document.getElementById("vanity-private-key-input");
const vanityModalError = document.getElementById("vanity-modal-error");
const vampModal = document.getElementById("vamp-modal");
const vampClose = document.getElementById("vamp-close");
const vampCancel = document.getElementById("vamp-cancel");
const vampImport = document.getElementById("vamp-import");
const vampContractInput = document.getElementById("vamp-contract-input");
const vampStatus = document.getElementById("vamp-status");
const vampError = document.getElementById("vamp-error");
const OUTPUT_SECTION_VISIBILITY_KEY = "launchdeck.outputSectionVisible";
const REPORTS_TERMINAL_VISIBILITY_KEY = "launchdeck.reportsTerminalVisible";
const REPORTS_TERMINAL_SORT_KEY = "launchdeck.reportsTerminalSort";
const REPORTS_TERMINAL_LIST_WIDTH_KEY = "launchdeck.reportsTerminalListWidth";
const THEME_MODE_STORAGE_KEY = "launchdeck.themeMode";
const SELECTED_WALLET_STORAGE_KEY = "launchdeck.selectedWalletKey";
const SNIPER_DRAFT_STORAGE_KEY = "launchdeck.sniperDraft.v1";
const IMAGE_LAYOUT_COMPACT_STORAGE_KEY = "launchdeck.imageLayoutCompact";
const SELECTED_MODE_STORAGE_KEY = "launchdeck.selectedMode";
const FEE_SPLIT_DRAFT_STORAGE_KEY = "launchdeck.feeSplitDraft.v1";
const AGENT_SPLIT_DRAFT_STORAGE_KEY = "launchdeck.agentSplitDraft.v1";
const POPOUT_FORM_WIDTH = 532;
const POPOUT_REPORTS_WIDTH = 560;
const POPOUT_WORKSPACE_GAP = 12;
const pageSearchParams = new URLSearchParams(window.location.search);
const isPopoutMode = pageSearchParams.get("popout") === "1";
const popoutOutputParam = pageSearchParams.get("output");
const popoutReportsParam = pageSearchParams.get("reports");
let popoutAutosizeFrame = 0;
const RequestUtils = window.LaunchDeckRequestUtils || {};
const RenderUtils = window.LaunchDeckRenderUtils || {};
const DEFAULT_LAUNCHPAD_TOKEN_METADATA = Object.freeze({
  nameMaxLength: 32,
  symbolMaxLength: 10,
});

if (isPopoutMode) {
  document.body.classList.add("popout-mode");
  document.title = "LaunchDeck Popout";
  window.addEventListener("load", () => {
    schedulePopoutAutosize();
  });
  if (document.fonts && document.fonts.ready) {
    document.fonts.ready.then(() => {
      schedulePopoutAutosize();
    }).catch(() => {});
  }
}

setThemeMode(getStoredThemeMode(), { persist: false });
setOutputSectionVisible(
  isPopoutMode && popoutOutputParam != null
    ? popoutOutputParam === "1"
    : getStoredOutputSectionVisible(),
);
setImageLayoutCompact(getStoredImageLayoutCompact(), { persist: false });

let uploadedImage = null;
let latestWalletStatus = null;
let latestRuntimeStatus = null;
let latestLaunchpadRegistry = {};
let walletStatusRequestSerial = 0;
let appBootstrapState = {
  started: false,
  staticLoaded: false,
  configLoaded: false,
  walletsLoaded: false,
  runtimeLoaded: false,
};
let quoteTimer = null;
let defaultsApplied = false;
const requestStates = {
  bootstrap: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  walletStatus: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  runtimeStatus: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  reports: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  reportView: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  images: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  quote: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
};
const renderCache = {
  walletDropdown: "",
  sniperWalletList: "",
  reportsList: "",
  imageGrid: "",
  backendRegion: "",
};
let metadataUploadState = {
  debounceTimer: null,
  inFlightPromise: null,
  inFlightFingerprint: "",
  completedFingerprint: "",
  latestScheduledFingerprint: "",
  lastCanPreupload: false,
  staleWhileUploading: false,
  autoRetryFailures: 0,
  autoRetryDisabled: false,
};
let imageLibraryState = {
  images: [],
  categories: [],
  search: "",
  category: "all",
  activeImageId: "",
};
let activeImageMenuId = "";
let activeImageDetailsId = "";
let imageDetailsTagsState = [];
let isEditingNewImageUpload = false;
let imageCategoryModalContext = "library";
let tickerManuallyEdited = false;
let syncingTickerFromName = false;
let tickerClearedForManualEntry = false;
let syncingDevBuyInputs = false;
let devBuyPresetEditorOpen = false;
let lastDevBuyEditSource = "sol";
let syncingPresetInputs = false;
let lastTopPresetMarkup = "";
let lastSettingsPresetMarkup = "";
let lastQuickDevBuyMarkup = "";
let reportsTerminalState = {
  entries: [],
  activeId: "",
  activePayload: null,
  activeText: "",
  activeTab: "overview",
  sort: getStoredReportsTerminalSort(),
};
let reportsTerminalResizeState = null;
const REPORTS_TERMINAL_DEFAULT_LIST_WIDTH = 152;
const REPORTS_TERMINAL_MIN_LIST_WIDTH = 120;
const REPORTS_TERMINAL_MAX_LIST_WIDTH = 240;
const SPLIT_COLORS = ["#5b7cff", "#ff5d5d", "#14c38e", "#ffb020", "#7c5cff", "#00b8d9", "#ef5da8", "#8b5cf6"];
const DEFAULT_QUICK_DEV_BUY_AMOUNTS = ["0.5", "1", "2"];
const DEFAULT_PRESET_ID = "preset1";
const METADATA_PREUPLOAD_DEBOUNCE_MS = 500;
const MAX_FEE_SPLIT_RECIPIENTS = 10;
const SNIPER_EXECUTION_RESERVE_SOL = 0.005;
const SNIPER_BALANCE_PRESETS = [
  { label: "MAX", ratio: 1 },
  { label: "75%", ratio: 0.75 },
  { label: "50%", ratio: 0.5 },
  { label: "25%", ratio: 0.25 },
];
const PROVIDER_LABELS = {
  "helius-sender": "Helius Sender",
  "standard-rpc": "Standard RPC",
  "jito-bundle": "Jito Bundle",
};
const ROUTE_CAPABILITIES = {
  "helius-sender": {
    creation: { tip: true, priority: true, slippage: false },
    buy: { tip: true, priority: true, slippage: true },
    sell: { tip: true, priority: true, slippage: true },
  },
  "standard-rpc": {
    creation: { tip: false, priority: true, slippage: false },
    buy: { tip: false, priority: true, slippage: true },
    sell: { tip: false, priority: true, slippage: true },
  },
  "jito-bundle": {
    creation: { tip: true, priority: true, slippage: false },
    buy: { tip: true, priority: true, slippage: true },
    sell: { tip: true, priority: true, slippage: true },
  },
};
const TOTAL_SUPPLY_TOKENS = 1_000_000_000n;
const TOKEN_DECIMALS = 6;
const TEST_PRESET = {
  name: "test",
  symbol: "test",
  description: "test",
  website: "https://test.com/",
  twitter: "https://x.com/test",
  telegram: "https://t.me/test",
  devBuyMode: "sol",
  devBuyAmount: "0.001",
};

function setBusy(busy, label) {
  statusNode.textContent = label;
  buttons.forEach((button) => {
    button.disabled = busy;
  });
  if (openSettingsButton) openSettingsButton.disabled = busy;
  if (modeSniperButton) modeSniperButton.disabled = busy;
  if (modeVanityButton) modeVanityButton.disabled = busy;
  if (devAutoSellButton) devAutoSellButton.disabled = busy;
  if (saveSettingsButton) saveSettingsButton.disabled = busy;
  if (changeDevBuyPresetsButton) changeDevBuyPresetsButton.disabled = busy;
  if (saveDevBuyPresetsButton) saveDevBuyPresetsButton.disabled = busy;
  if (cancelDevBuyPresetsButton) cancelDevBuyPresetsButton.disabled = busy;
  if (devBuyCustomDeployButton) devBuyCustomDeployButton.disabled = busy;
}

function getNamedInput(name) {
  return document.querySelector(`[name="${name}"]`);
}

function getNamedValue(name) {
  const input = getNamedInput(name);
  return input ? input.value : "";
}

function setNamedValue(name, value) {
  const input = getNamedInput(name);
  if (input) input.value = value;
}

function isNamedChecked(name) {
  const input = getNamedInput(name);
  return Boolean(input && input.checked);
}

function formatSliderValue(value, suffix, digits = 0) {
  const numeric = Number(value || 0);
  if (!Number.isFinite(numeric)) return `0${suffix}`;
  return `${numeric.toFixed(digits)}${suffix}`;
}

function normalizeLaunchMode(value) {
  const mode = String(value || "").trim();
  if (["regular", "cashback", "agent-custom", "agent-unlocked", "agent-locked"].includes(mode)) {
    return mode;
  }
  return "regular";
}

function getStoredLaunchMode() {
  try {
    const stored = window.localStorage.getItem(SELECTED_MODE_STORAGE_KEY);
    return stored ? normalizeLaunchMode(stored) : "";
  } catch (_error) {
    return "";
  }
}

function setStoredLaunchMode(mode) {
  try {
    window.localStorage.setItem(SELECTED_MODE_STORAGE_KEY, normalizeLaunchMode(mode));
  } catch (_error) {
    // Ignore storage failures and keep mode controls functional.
  }
}

function serializeFeeSplitDraft() {
  return {
    enabled: Boolean(feeSplitEnabled && feeSplitEnabled.checked),
    rows: getFeeSplitRows().map((row) => ({
      type: row.dataset.type || "wallet",
      value: row.querySelector(".recipient-target")?.value?.trim() || "",
      sharePercent: row.querySelector(".recipient-share")?.value?.trim() || "",
      defaultReceiver: row.dataset.defaultReceiver === "true",
      targetLocked: row.dataset.targetLocked === "true",
    })),
  };
}

function normalizeFeeSplitDraft(value) {
  const rows = Array.isArray(value && value.rows)
    ? value.rows.map((entry) => ({
      type: entry && entry.type === "github" ? "github" : "wallet",
      value: String(entry && entry.value || "").trim(),
      sharePercent: normalizeDecimalInput(entry && entry.sharePercent || "", 2),
      defaultReceiver: Boolean(entry && entry.defaultReceiver),
      targetLocked: Boolean(entry && entry.targetLocked),
    }))
    : [];
  return {
    enabled: Boolean(value && value.enabled),
    rows,
  };
}

function getStoredFeeSplitDraft() {
  try {
    const raw = window.localStorage.getItem(FEE_SPLIT_DRAFT_STORAGE_KEY);
    if (!raw) return null;
    return normalizeFeeSplitDraft(JSON.parse(raw));
  } catch (_error) {
    return null;
  }
}

function setStoredFeeSplitDraft(value) {
  try {
    const normalized = normalizeFeeSplitDraft(value);
    if (!normalized.enabled && normalized.rows.length === 0) {
      window.localStorage.removeItem(FEE_SPLIT_DRAFT_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(FEE_SPLIT_DRAFT_STORAGE_KEY, JSON.stringify(normalized));
  } catch (_error) {
    // Ignore storage failures and keep fee split controls functional.
  }
}

function applyFeeSplitDraft(value, { persist = false } = {}) {
  const draft = normalizeFeeSplitDraft(value);
  if (feeSplitEnabled) feeSplitEnabled.checked = draft.enabled;
  if (feeSplitList) {
    feeSplitList.innerHTML = "";
    draft.rows.forEach((entry) => {
      feeSplitList.appendChild(createFeeSplitRow(entry));
    });
  }
  if (draft.enabled) ensureFeeSplitDefaultRow();
  syncFeeSplitTotals();
  if (persist) setStoredFeeSplitDraft(draft);
}

function serializeAgentSplitDraft() {
  return {
    rows: getAgentSplitRows().map((row) => ({
      locked: row.dataset.locked === "true",
      type: row.dataset.type || "wallet",
      value: row.querySelector(".recipient-target")?.value?.trim() || "",
      sharePercent: row.querySelector(".recipient-share")?.value?.trim() || "",
      defaultReceiver: row.dataset.defaultReceiver === "true",
      targetLocked: row.dataset.targetLocked === "true",
    })),
  };
}

function normalizeAgentSplitDraft(value) {
  const rows = Array.isArray(value && value.rows)
    ? value.rows.map((entry) => ({
      locked: Boolean(entry && entry.locked),
      type: entry && entry.type === "github" ? "github" : "wallet",
      value: String(entry && entry.value || "").trim(),
      sharePercent: normalizeDecimalInput(entry && entry.sharePercent || "", 2),
      defaultReceiver: Boolean(entry && entry.defaultReceiver),
      targetLocked: Boolean(entry && entry.targetLocked),
    }))
    : [];
  return { rows };
}

function getStoredAgentSplitDraft() {
  try {
    const raw = window.localStorage.getItem(AGENT_SPLIT_DRAFT_STORAGE_KEY);
    if (!raw) return null;
    return normalizeAgentSplitDraft(JSON.parse(raw));
  } catch (_error) {
    return null;
  }
}

function setStoredAgentSplitDraft(value) {
  try {
    const normalized = normalizeAgentSplitDraft(value);
    if (normalized.rows.length === 0) {
      window.localStorage.removeItem(AGENT_SPLIT_DRAFT_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(AGENT_SPLIT_DRAFT_STORAGE_KEY, JSON.stringify(normalized));
  } catch (_error) {
    // Ignore storage failures and keep agent split controls functional.
  }
}

function applyAgentSplitDraft(value, { persist = false } = {}) {
  const draft = normalizeAgentSplitDraft(value);
  if (agentSplitList) {
    agentSplitList.innerHTML = "";
    draft.rows.forEach((entry) => {
      agentSplitList.appendChild(createAgentSplitRow(entry));
    });
  }
  normalizeAgentSplitStructure();
  syncAgentSplitTotals();
  if (persist) setStoredAgentSplitDraft(draft);
}

function normalizeDecimalInput(value, maxDecimals = 6) {
  const raw = String(value || "").replace(/,/g, ".").trim();
  if (!raw) return "";
  const sanitized = raw.replace(/[^\d.]/g, "");
  const [whole = "", fractional = ""] = sanitized.split(".");
  const safeWhole = whole.replace(/^0+(?=\d)/, "") || (whole ? "0" : "");
  return fractional !== undefined && sanitized.includes(".")
    ? `${safeWhole || "0"}.${fractional.slice(0, maxDecimals)}`
    : safeWhole;
}

const reportsFeature = window.ReportsFeature.create({
  elements: {
    reportsTerminalSection,
    reportsTerminalList,
    reportsTerminalOutput,
    reportsTerminalMeta,
    reportsTerminalResizeHandle,
    openPopoutButton,
    toggleOutputButton,
    toggleReportsButton,
    reportsRefreshButton,
    reportsSortButton,
  },
  storage: {
    visibilityKey: REPORTS_TERMINAL_VISIBILITY_KEY,
    sortKey: REPORTS_TERMINAL_SORT_KEY,
    listWidthKey: REPORTS_TERMINAL_LIST_WIDTH_KEY,
  },
  requestStates,
  renderCache,
  state: reportsTerminalState,
  getResizeState: () => reportsTerminalResizeState,
  setResizeState: (value) => {
    reportsTerminalResizeState = value;
  },
  constants: {
    defaultListWidth: REPORTS_TERMINAL_DEFAULT_LIST_WIDTH,
    minListWidth: REPORTS_TERMINAL_MIN_LIST_WIDTH,
    maxListWidth: REPORTS_TERMINAL_MAX_LIST_WIDTH,
  },
  schedulePopoutAutosize,
  refreshOnVisible: () => refreshReportsTerminal(),
  renderOutput: () => renderReportsTerminalOutput(),
  renderList: () => renderReportsTerminalList(),
  loadEntry: (id) => loadReportsTerminalEntry(id),
  refreshReports: (options) => refreshReportsTerminal(options),
  normalizeTab: (tab) => normalizeReportsTerminalTab(tab),
  shortenAddress,
  openPopoutWindow,
});

reportsFeature.bindEvents();

function getStoredReportsTerminalListWidth() {
  return reportsFeature.getStoredListWidth();
}

function setReportsTerminalListWidth(width, options) {
  return reportsFeature.setListWidth(width, options);
}

function setReportsTerminalVisible(isVisible, options) {
  return reportsFeature.setVisible(isVisible, options);
}

function setReportsTerminalSort(sort, options) {
  return reportsFeature.setSort(sort, options);
}

setReportsTerminalSort(reportsTerminalState.sort, { persist: false });
setReportsTerminalVisible(
  isPopoutMode && popoutReportsParam != null
    ? popoutReportsParam === "1"
    : getStoredReportsTerminalVisible(),
  { persist: false },
);
setReportsTerminalListWidth(getStoredReportsTerminalListWidth(), { persist: false });

const imagesFeature = window.ImagesFeature.create({
  elements: {
    imageStatus,
    imagePath,
    imagePreview,
    imageEmpty,
    imageLibraryModal,
    imageLibrarySearchInput,
    imageLibraryUploadButton,
    imageLibraryGrid,
    imageLibraryEmpty,
    imageCategoryChips,
    newImageCategoryButton,
    imageItemMenu,
    imageMenuFavorite,
    imageMenuEdit,
    imageMenuDelete,
    imageDetailsModal,
    imageDetailsTitle,
    imageDetailsClose,
    imageDetailsCancel,
    imageDetailsSave,
    imageDetailsName,
    imageDetailsTags,
    imageDetailsAddTag,
    imageDetailsTagList,
    imageDetailsError,
    imageDetailsCategoryRow,
    imageDetailsCategory,
    imageDetailsNewCategory,
    imageCategoryModal,
    imageCategoryClose,
    imageCategoryCancel,
    imageCategorySave,
    imageCategoryName,
    imageCategoryError,
    imageLibraryClose,
    imageInput,
    openImageLibraryButton,
  },
  renderCache,
  requestStates,
  getImageLibraryState: () => imageLibraryState,
  getActiveImageMenuId: () => activeImageMenuId,
  setActiveImageMenuId: (value) => {
    activeImageMenuId = value;
  },
  getActiveImageDetailsId: () => activeImageDetailsId,
  setActiveImageDetailsId: (value) => {
    activeImageDetailsId = value;
  },
  getImageDetailsTagsState: () => imageDetailsTagsState,
  setImageDetailsTagsState: (value) => {
    imageDetailsTagsState = value;
  },
  getIsEditingNewImageUpload: () => isEditingNewImageUpload,
  setIsEditingNewImageUpload: (value) => {
    isEditingNewImageUpload = value;
  },
  getImageCategoryModalContext: () => imageCategoryModalContext,
  setImageCategoryModalContext: (value) => {
    imageCategoryModalContext = value;
  },
  getUploadedImage: () => uploadedImage,
  setUploadedImage: (value) => {
    uploadedImage = value;
  },
  clearMetadataUploadCache,
  setImagePreview,
  scheduleMetadataPreupload,
  escapeHTML,
  fetchJsonLatest: RequestUtils.fetchJsonLatest,
});

imagesFeature.bindEvents();

function hideImageItemMenu() {
  imagesFeature.hideItemMenu();
}

function setSelectedImage(image) {
  imagesFeature.setSelectedImage(image);
}

function renderImageCategoryChips() {
  imagesFeature.renderCategoryChips();
}

function renderImageLibraryGrid() {
  imagesFeature.renderLibraryGrid();
}

function fetchImageLibrary() {
  return imagesFeature.fetchLibrary();
}

function showImageLibraryModal() {
  imagesFeature.showLibraryModal();
}

function hideImageLibraryModal() {
  imagesFeature.hideLibraryModal();
}

function showImageDetailsModal(image, options = {}) {
  imagesFeature.showDetailsModal(image, options);
}

function hideImageDetailsModal() {
  imagesFeature.hideDetailsModal();
}

function showImageCategoryModal(context = "library") {
  imagesFeature.showCategoryModal(context);
}

function hideImageCategoryModal() {
  imagesFeature.hideCategoryModal();
}

const autoSellFeature = window.AutoSellFeature.create({
  elements: {
    devAutoSellButton,
    devAutoSellPanel,
    autoSellEnabledInput,
    autoSellToggleState,
    autoSellTriggerValue,
    autoSellTriggerDescription,
    autoSellDelaySlider,
    autoSellDelayControl,
    autoSellPercentSlider,
    autoSellDelayValue,
    autoSellBlockControl,
    autoSellBlockValue,
    autoSellPercentValue,
    autoSellSettings,
    autoSellTriggerModeButtons,
    autoSellBlockOffsetButtons,
  },
  getNamedValue,
  setNamedValue,
  isNamedChecked,
  formatSliderValue,
  syncSettingsCapabilities,
  syncActivePresetFromInputs,
  validateFieldByName,
  documentNode: document,
});

autoSellFeature.bindEvents();

function normalizeAutoSellTriggerMode(value) {
  return autoSellFeature.normalizeTriggerMode(value);
}

function getAutoSellTriggerMode() {
  return autoSellFeature.getTriggerMode();
}

function getAutoSellDelayMs() {
  return autoSellFeature.getDelayMs();
}

function getAutoSellBlockOffset() {
  return autoSellFeature.getBlockOffset();
}

function getAutoSellTriggerLabel(mode = getAutoSellTriggerMode()) {
  return autoSellFeature.getTriggerLabel(mode);
}

function getAutoSellTriggerDescription(mode = getAutoSellTriggerMode()) {
  return autoSellFeature.getTriggerDescription(mode);
}

function getAutoSellSummaryText(formValues = readForm()) {
  return autoSellFeature.getSummaryText(formValues);
}

function syncDevAutoSellUI() {
  autoSellFeature.syncUI();
}

function toggleDevAutoSellPanel(forceOpen) {
  autoSellFeature.togglePanel(forceOpen);
}

const sniperFeature = window.SniperFeature.create({
  storageKey: SNIPER_DRAFT_STORAGE_KEY,
  renderCache,
  balancePresets: SNIPER_BALANCE_PRESETS,
  executionReserveSol: SNIPER_EXECUTION_RESERVE_SOL,
  elements: {
    postLaunchStrategyInput,
    snipeBuyAmountInput,
    sniperEnabledInput,
    sniperConfigJsonInput,
    modeSniperButton,
    sniperModal,
    sniperClose,
    sniperCancel,
    sniperSave,
    sniperRefreshButton,
    sniperResetButton,
    sniperEnabledToggle,
    sniperEnabledState,
    sniperWalletsSection,
    sniperWalletList,
    sniperSelectionSummary,
    sniperTotalSummary,
    sniperModalError,
  },
  getLatestWalletStatus: () => latestWalletStatus,
  getAppBootstrapState: () => appBootstrapState,
  getSelectedWalletKey: () => selectedWalletKey(),
  getNamedValue,
  walletDisplayName,
  walletIndexFromEnvKey,
  shortenAddress,
  escapeHTML,
  normalizeDecimalInput,
  getRouteCapabilities,
  getBuyProvider,
  getSellProvider,
  refreshWalletStatus,
  metaNode,
});

sniperFeature.bindEvents();

function normalizeSniperDraftState(value) {
  return sniperFeature.normalizeDraftState(value);
}

function getStoredSniperDraft() {
  return sniperFeature.getStoredDraft();
}

function getSniperTriggerSummary(entry = {}) {
  return sniperFeature.getTriggerSummary(entry);
}

function setSniperModalError(message = "") {
  sniperFeature.setModalError(message);
}

function resetSniperState() {
  sniperFeature.resetState();
}

function applySniperStateToForm() {
  sniperFeature.applyStateToForm();
}

function renderSniperUI() {
  sniperFeature.renderUI();
}

function showSniperModal() {
  sniperFeature.showModal();
}

function hideSniperModal() {
  sniperFeature.hideModal();
}

function validateSniperState() {
  return sniperFeature.validateState();
}

function updateDescriptionDisclosure() {
  const currentLength = descriptionInput ? descriptionInput.value.length : 0;
  const maxLength = descriptionInput ? Number(descriptionInput.getAttribute("maxlength") || 500) : 500;
  const expanded = Boolean(descriptionPanelBody && !descriptionPanelBody.hidden);
  if (descriptionCharCount) descriptionCharCount.textContent = `${currentLength}/${maxLength}`;
  if (descriptionToggle) descriptionToggle.setAttribute("aria-expanded", expanded ? "true" : "false");
  if (descriptionDisclosure) {
    descriptionDisclosure.classList.toggle("is-open", expanded);
    descriptionDisclosure.classList.toggle("has-content", currentLength > 0);
  }
}

function toggleDescriptionDisclosure(forceOpen) {
  if (!descriptionPanelBody) return;
  const nextOpen = typeof forceOpen === "boolean" ? forceOpen : descriptionPanelBody.hidden;
  descriptionPanelBody.hidden = !nextOpen;
  updateDescriptionDisclosure();
}

function parseDecimalToBigInt(rawValue, decimals) {
  const raw = String(rawValue || "").trim();
  if (!raw) return 0n;
  if (!/^\d+(\.\d+)?$/.test(raw)) {
    throw new Error("Invalid decimal input.");
  }
  const [whole, fractional = ""] = raw.split(".");
  if (fractional.length > decimals) {
    throw new Error(`Too many decimal places (max ${decimals}).`);
  }
  const combined = `${whole}${fractional.padEnd(decimals, "0")}`.replace(/^0+(?=\d)/, "");
  return BigInt(combined || "0");
}

function formatBigIntDecimal(value, decimals, maxFractionDigits = decimals) {
  const negative = value < 0n;
  const absolute = negative ? -value : value;
  const base = 10n ** BigInt(decimals);
  const whole = absolute / base;
  const fraction = absolute % base;
  if (fraction === 0n) return `${negative ? "-" : ""}${whole.toString()}`;
  let fractionText = fraction.toString().padStart(decimals, "0").slice(0, maxFractionDigits);
  fractionText = fractionText.replace(/0+$/, "");
  return `${negative ? "-" : ""}${whole.toString()}${fractionText ? `.${fractionText}` : ""}`;
}

function getQuickDevBuyPresetAmounts(config = latestWalletStatus && latestWalletStatus.config) {
  const presetItems = config && config.presets && Array.isArray(config.presets.items)
    ? config.presets.items
    : [];
  return DEFAULT_QUICK_DEV_BUY_AMOUNTS.map((fallback, index) => {
    const preset = presetItems[index];
    const value = preset && preset.creationSettings && typeof preset.creationSettings.devBuySol === "string"
      ? preset.creationSettings.devBuySol.trim()
      : "";
    return value || fallback;
  });
}

function renderQuickDevBuyButtons(config = latestWalletStatus && latestWalletStatus.config) {
  if (!devBuyQuickButtons) return;
  const presetItems = getPresetItems(config);
  const markup = presetItems.map((preset, index) => {
    const amount = preset.creationSettings.devBuySol || DEFAULT_QUICK_DEV_BUY_AMOUNTS[index];
    if (devBuyPresetEditorOpen) {
      return `
        <label class="dev-buy-quick-button dev-buy-quick-button-editing" data-quick-buy-index="${index}">
          <span class="dev-buy-quick-content dev-buy-quick-content-editing">
            <input
              type="text"
              inputmode="decimal"
              class="dev-buy-quick-value dev-buy-quick-editor-input"
              data-dev-buy-preset-input="${index}"
              value="${escapeHTML(amount)}"
              placeholder="${escapeHTML(DEFAULT_QUICK_DEV_BUY_AMOUNTS[index])}"
            >
          </span>
        </label>
      `;
    }
    return `
      <button type="button" class="dev-buy-quick-button" data-quick-buy-index="${index}" data-quick-buy-preset-id="${escapeHTML(preset.id)}" data-quick-buy-amount="${escapeHTML(amount)}">
        <span class="dev-buy-quick-content">
          <img src="/solana-mark.png" alt="SOL" class="sol-logo inline-sol-logo quick-buy-sol-logo">
          <strong class="dev-buy-quick-value">${escapeHTML(amount)}</strong>
        </span>
      </button>
    `;
  }).join("");
  if (markup === lastQuickDevBuyMarkup) return;
  devBuyQuickButtons.innerHTML = markup;
  lastQuickDevBuyMarkup = markup;
}

function populateDevBuyPresetEditor(config = latestWalletStatus && latestWalletStatus.config) {
  const amounts = getQuickDevBuyPresetAmounts(config);
  getDevBuyPresetEditorInputs().forEach((input, index) => {
    if (input) input.value = amounts[index] || "";
  });
}

function getDevBuyPresetEditorInputs() {
  return devBuyQuickButtons
    ? Array.from(devBuyQuickButtons.querySelectorAll("[data-dev-buy-preset-input]"))
    : [];
}

function setDevBuyPresetEditorOpen(isOpen) {
  devBuyPresetEditorOpen = Boolean(isOpen);
  if (changeDevBuyPresetsButton) {
    changeDevBuyPresetsButton.hidden = devBuyPresetEditorOpen;
    changeDevBuyPresetsButton.setAttribute("aria-expanded", devBuyPresetEditorOpen ? "true" : "false");
  }
  if (cancelDevBuyPresetsButton) cancelDevBuyPresetsButton.hidden = !devBuyPresetEditorOpen;
  if (saveDevBuyPresetsButton) saveDevBuyPresetsButton.hidden = !devBuyPresetEditorOpen;
  renderQuickDevBuyButtons(getConfig());
}

function buildConfigWithUpdatedDevBuyPresets() {
  const config = cloneConfig(getConfig()) || createFallbackConfig();
  const presetItems = getPresetItems(config);
  const editorInputs = getDevBuyPresetEditorInputs();
  config.presets = config.presets || {};
  config.presets.items = presetItems.map((preset, index) => {
    const input = editorInputs[index];
    const nextValue = input ? String(input.value || "").trim() : "";
    return {
      ...preset,
      creationSettings: {
        ...(preset.creationSettings || {}),
        devBuySol: nextValue,
      },
    };
  });
  return config;
}

async function saveDevBuyPresetEditor() {
  const nextConfig = buildConfigWithUpdatedDevBuyPresets();
  if (saveDevBuyPresetsButton) saveDevBuyPresetsButton.disabled = true;
  if (cancelDevBuyPresetsButton) cancelDevBuyPresetsButton.disabled = true;
  if (changeDevBuyPresetsButton) changeDevBuyPresetsButton.disabled = true;
  try {
    const response = await fetch("/api/settings/save", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ config: nextConfig }),
    });
    const payload = await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to save quick deploy presets.");
    }
    setRegionRouting(payload.regionRouting || (latestWalletStatus && latestWalletStatus.regionRouting));
    setConfig(payload.config);
    if (latestWalletStatus) latestWalletStatus.config = payload.config;
    renderQuickDevBuyButtons(payload.config);
    populateDevBuyPresetEditor(payload.config);
    renderBackendRegionSummary(payload.regionRouting);
    setDevBuyPresetEditorOpen(false);
  } catch (error) {
    statusNode.textContent = "Error";
    output.textContent = error.message;
  } finally {
    if (saveDevBuyPresetsButton) saveDevBuyPresetsButton.disabled = false;
    if (cancelDevBuyPresetsButton) cancelDevBuyPresetsButton.disabled = false;
    if (changeDevBuyPresetsButton) changeDevBuyPresetsButton.disabled = false;
  }
}

function cloneConfig(value) {
  return value ? JSON.parse(JSON.stringify(value)) : null;
}

function createFallbackConfig() {
  return {
    defaults: {
      launchpad: "pump",
      mode: "regular",
      activePresetId: DEFAULT_PRESET_ID,
      presetEditing: false,
      misc: {
        trackSendBlockHeight: false,
      },
    },
    presets: {
      items: DEFAULT_QUICK_DEV_BUY_AMOUNTS.map((amount, index) => ({
        id: `preset${index + 1}`,
        label: `P${index + 1}`,
        creationSettings: {
          provider: "helius-sender",
          tipSol: "0.01",
          priorityFeeSol: "0.001",
          devBuySol: amount,
        },
        buySettings: {
          provider: "helius-sender",
          priorityFeeSol: "0.009",
          tipSol: "0.01",
          slippagePercent: "90",
          snipeBuyAmountSol: "",
        },
        sellSettings: {
          provider: "helius-sender",
          priorityFeeSol: "0.009",
          tipSol: "0.01",
          slippagePercent: "90",
        },
        automaticDevSell: {
          enabled: false,
          percent: 100,
          triggerMode: "confirmation",
          delayMs: 0,
          targetBlockOffset: 0,
        },
        postLaunchStrategy: "none",
      })),
    },
  };
}

function getConfig() {
  return latestWalletStatus && latestWalletStatus.config
    ? latestWalletStatus.config
    : createFallbackConfig();
}

function getPresetItems(config = getConfig()) {
  return config && config.presets && Array.isArray(config.presets.items)
    ? config.presets.items
    : createFallbackConfig().presets.items;
}

function getActivePresetId(config = getConfig()) {
  return config && config.defaults && config.defaults.activePresetId
    ? config.defaults.activePresetId
    : DEFAULT_PRESET_ID;
}

function getActivePreset(config = getConfig()) {
  const items = getPresetItems(config);
  return items.find((entry) => entry.id === getActivePresetId(config)) || items[0];
}

function getPresetDisplayLabel(preset, index = 0) {
  const rawLabel = String((preset && preset.label) || "").trim();
  const labelMatch = rawLabel.match(/^preset\s*([0-9]+)$/i);
  if (labelMatch) return `P${labelMatch[1]}`;
  const idMatch = String((preset && preset.id) || "").trim().match(/^preset([0-9]+)$/i);
  if (!rawLabel && idMatch) return `P${idMatch[1]}`;
  return rawLabel || `P${index + 1}`;
}

function isPresetEditing(config = getConfig()) {
  return Boolean(config && config.defaults && config.defaults.presetEditing);
}

function setConfig(nextConfig) {
  if (!latestWalletStatus) {
    latestWalletStatus = {
      connected: false,
      config: cloneConfig(nextConfig),
    };
  } else {
    latestWalletStatus = {
      ...latestWalletStatus,
      config: cloneConfig(nextConfig),
    };
  }
  renderPresetChips();
  renderQuickDevBuyButtons(nextConfig);
}

function setRegionRouting(nextRegionRouting) {
  if (!latestWalletStatus) {
    latestWalletStatus = {
      connected: false,
      config: cloneConfig(getConfig()),
      regionRouting: nextRegionRouting || null,
    };
    return;
  }
  latestWalletStatus = {
    ...latestWalletStatus,
    regionRouting: nextRegionRouting || latestWalletStatus.regionRouting || null,
  };
}

function formatBackendRegionValue(value, fallback = "global") {
  const normalized = String(value || "").trim();
  return normalized || fallback;
}

function renderBackendRegionSummary(regionRouting = latestWalletStatus && latestWalletStatus.regionRouting) {
  if (!settingsBackendRegionSummary) return;
  if (!regionRouting || typeof regionRouting !== "object") {
    if (RenderUtils.setCachedHTML) {
      RenderUtils.setCachedHTML(
        renderCache,
        "backendRegion",
        settingsBackendRegionSummary,
        '<div class="settings-section-copy">Loading backend routing defaults...</div>',
      );
    } else {
      settingsBackendRegionSummary.innerHTML = '<div class="settings-section-copy">Loading backend routing defaults...</div>';
    }
    return;
  }
  const shared = regionRouting && regionRouting.shared ? regionRouting.shared : {};
  const providers = regionRouting && regionRouting.providers ? regionRouting.providers : {};
  const sharedConfigured = formatBackendRegionValue(shared.configured, "None");
  const sharedEffective = formatBackendRegionValue(shared.effective);
  const providerRows = ["helius-sender", "jito-bundle"].map((provider) => {
    const entry = providers[provider] || {};
    const configured = formatBackendRegionValue(entry.configured, "None");
    const effective = formatBackendRegionValue(entry.effective);
    const overrideNote = entry.endpointOverrideActive
      ? '<div class="settings-region-note">Explicit endpoint override active. Region fanout is bypassed.</div>'
      : "";
    return `
      <div class="settings-region-card">
        <div class="settings-region-card-head">
          <strong>${escapeHTML(PROVIDER_LABELS[provider] || provider)}</strong>
          <span class="settings-region-effective">${escapeHTML(effective)}</span>
        </div>
        <div class="settings-region-meta">Override: ${escapeHTML(configured)}</div>
        ${overrideNote}
      </div>
    `;
  }).join("");
  const markup = `
    <div class="settings-region-card settings-region-card-shared">
      <div class="settings-region-card-head">
        <strong>Shared backend region</strong>
        <span class="settings-region-effective">${escapeHTML(sharedEffective)}</span>
      </div>
      <div class="settings-region-meta">Configured: ${escapeHTML(sharedConfigured)}</div>
    </div>
    ${providerRows}
    <div class="settings-sidebar-note">
      Region defaults are recommended because provider fanout usually reaches more nearby supported endpoints and lands faster and more reliably than pinning one endpoint. Change backend env values, then run <code>npm restart</code>.
    </div>
  `;
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "backendRegion", settingsBackendRegionSummary, markup);
  } else {
    settingsBackendRegionSummary.innerHTML = markup;
  }
}

function renderPresetChipMarkup(config = getConfig(), { topBar = false } = {}) {
  const activePresetId = getActivePresetId(config);
  return getPresetItems(config).map((preset, index) => `
    <button
      type="button"
      class="preset-chip${preset.id === activePresetId ? " active" : ""}${topBar ? " compact" : ""}"
      data-preset-id="${escapeHTML(preset.id)}"
    >
      ${escapeHTML(topBar ? getPresetDisplayLabel(preset, index) : `Preset ${index + 1}`)}
    </button>
  `).join("");
}

function renderPresetChips() {
  const config = getConfig();
  const topMarkup = renderPresetChipMarkup(config, { topBar: true });
  const settingsMarkup = renderPresetChipMarkup(config, { topBar: false });
  if (topPresetChipBar && topMarkup !== lastTopPresetMarkup) {
    topPresetChipBar.innerHTML = topMarkup;
    lastTopPresetMarkup = topMarkup;
  }
  if (settingsPresetChipBar && settingsMarkup !== lastSettingsPresetMarkup) {
    settingsPresetChipBar.innerHTML = settingsMarkup;
    lastSettingsPresetMarkup = settingsMarkup;
  }
  if (presetEditToggle) {
    const editing = isPresetEditing(config);
    presetEditToggle.classList.toggle("active", editing);
    presetEditToggle.setAttribute("aria-pressed", editing ? "true" : "false");
    presetEditToggle.innerHTML = editing ? "Lock" : "&#9998;";
    presetEditToggle.title = editing ? "Lock active preset" : "Unlock active preset for editing";
  }
}

function setDevBuyHiddenState(mode, amount) {
  if (devBuyModeInput) devBuyModeInput.value = mode || "sol";
  if (devBuyAmountInput) devBuyAmountInput.value = amount || "";
}

function clearDevBuyState() {
  setDevBuyHiddenState("sol", "");
  syncingDevBuyInputs = true;
  if (devBuySolInput) devBuySolInput.value = "";
  if (devBuyPercentInput) devBuyPercentInput.value = "";
  syncingDevBuyInputs = false;
  quoteOutput.textContent = "No dev buy selected.";
  syncActivePresetDevBuy("");
}

function syncActivePresetDevBuy(amount) {
  if (!isPresetEditing(getConfig())) return;
  const config = cloneConfig(getConfig());
  const activePreset = getActivePreset(config);
  if (!activePreset) return;
  activePreset.creationSettings = {
    ...activePreset.creationSettings,
    devBuySol: String(amount || "").trim(),
  };
  setConfig(config);
}

function percentToTokenAmount(percentValue) {
  const percentRaw = parseDecimalToBigInt(percentValue, 4);
  const rawTokens = (TOTAL_SUPPLY_TOKENS * (10n ** BigInt(TOKEN_DECIMALS)) * percentRaw) / 1_000_000n;
  return formatBigIntDecimal(rawTokens, TOKEN_DECIMALS, TOKEN_DECIMALS);
}

async function updateDevBuyFromSolInput(value) {
  const amount = normalizeDecimalInput(value, 9);
  syncingDevBuyInputs = true;
  if (devBuySolInput) devBuySolInput.value = amount;
  syncingDevBuyInputs = false;
  lastDevBuyEditSource = "sol";
  if (!amount) {
    clearDevBuyState();
    return;
  }
  setDevBuyHiddenState("sol", amount);
  syncActivePresetDevBuy(amount);
  await updateQuote();
}

async function updateDevBuyFromPercentInput(value) {
  const percent = normalizeDecimalInput(value, 4);
  syncingDevBuyInputs = true;
  if (devBuyPercentInput) devBuyPercentInput.value = percent;
  syncingDevBuyInputs = false;
  lastDevBuyEditSource = "percent";
  if (!percent) {
    clearDevBuyState();
    return;
  }
  try {
    const tokenAmount = percentToTokenAmount(percent);
    if (!tokenAmount || Number(percent) <= 0) {
      clearDevBuyState();
      return;
    }
    setDevBuyHiddenState("tokens", tokenAmount);
    await updateQuote();
  } catch (_error) {
    setDevBuyHiddenState("tokens", "");
    quoteOutput.textContent = "Enter a valid percentage.";
  }
}

async function triggerDeployWithDevBuy(mode, amount, source = "sol") {
  setDevBuyHiddenState(mode, amount);
  lastDevBuyEditSource = source;
  if (source === "sol") {
    syncingDevBuyInputs = true;
    if (devBuySolInput) devBuySolInput.value = amount;
    syncingDevBuyInputs = false;
    await updateQuote();
  } else {
    await updateQuote();
  }
  const errors = validateForm();
  if (showValidationErrors(errors)) return;
  clearValidationErrors();
  await run("deploy");
}

function isTickerCapsEnabled() {
  return Boolean(tickerCapsToggle && tickerCapsToggle.classList.contains("active"));
}

function getLaunchpadTokenMetadata(launchpad = getLaunchpad()) {
  const entry = latestLaunchpadRegistry && latestLaunchpadRegistry[launchpad];
  const metadata = entry && entry.tokenMetadata ? entry.tokenMetadata : {};
  const nameMaxLength = Number(metadata.nameMaxLength || DEFAULT_LAUNCHPAD_TOKEN_METADATA.nameMaxLength);
  const symbolMaxLength = Number(metadata.symbolMaxLength || DEFAULT_LAUNCHPAD_TOKEN_METADATA.symbolMaxLength);
  return {
    nameMaxLength: Number.isFinite(nameMaxLength) && nameMaxLength > 0
      ? nameMaxLength
      : DEFAULT_LAUNCHPAD_TOKEN_METADATA.nameMaxLength,
    symbolMaxLength: Number.isFinite(symbolMaxLength) && symbolMaxLength > 0
      ? symbolMaxLength
      : DEFAULT_LAUNCHPAD_TOKEN_METADATA.symbolMaxLength,
  };
}

function formatTickerValue(value) {
  const { symbolMaxLength } = getLaunchpadTokenMetadata();
  const normalized = String(value || "").replace(/\s+/g, " ").trimStart();
  const clipped = normalized.slice(0, symbolMaxLength);
  return isTickerCapsEnabled() ? clipped.toUpperCase() : clipped;
}

function getAutoTickerValue() {
  return formatTickerValue(nameInput ? nameInput.value : "");
}

function updateTokenFieldCounts() {
  const { nameMaxLength, symbolMaxLength } = getLaunchpadTokenMetadata();
  if (nameInput) {
    nameInput.maxLength = nameMaxLength;
    if (nameInput.value.length > nameMaxLength) {
      nameInput.value = nameInput.value.slice(0, nameMaxLength);
    }
  }
  if (symbolInput) {
    symbolInput.maxLength = symbolMaxLength;
    const formatted = formatTickerValue(symbolInput.value);
    if (symbolInput.value !== formatted) {
      syncingTickerFromName = true;
      symbolInput.value = formatted;
      syncingTickerFromName = false;
    }
  }
  if (nameCharCount && nameInput) {
    nameCharCount.textContent = `${nameInput.value.length}/${nameMaxLength}`;
  }
  if (symbolCharCount && symbolInput) {
    symbolCharCount.textContent = `${symbolInput.value.length}/${symbolMaxLength}`;
  }
}

function applyLaunchpadTokenMetadata() {
  updateTokenFieldCounts();
  if (!tickerManuallyEdited) {
    syncTickerFromName();
    return;
  }
  applyTickerCapsMode();
}

function syncTickerFromName() {
  if (!nameInput || !symbolInput || tickerManuallyEdited) {
    updateTokenFieldCounts();
    return;
  }
  syncingTickerFromName = true;
  symbolInput.value = getAutoTickerValue();
  syncingTickerFromName = false;
  tickerClearedForManualEntry = false;
  updateTokenFieldCounts();
}

function applyTickerCapsMode() {
  if (!symbolInput) return;
  syncingTickerFromName = true;
  symbolInput.value = formatTickerValue(symbolInput.value);
  syncingTickerFromName = false;
  updateTokenFieldCounts();
}

function getMode() {
  const checked = form.querySelector('input[name="mode"]:checked');
  return checked ? checked.value : "regular";
}

function setMode(mode) {
  const normalized = String(mode || "").trim();
  const target = normalized || "regular";
  const next = form.querySelector(`input[name="mode"][value="${CSS.escape(target)}"]`)
    || form.querySelector('input[name="mode"][value="regular"]');
  if (!next) return;
  next.checked = true;
  updateModeVisibility();
}

function getDevBuyMode() {
  const explicit = getNamedValue("devBuyMode");
  return explicit || "sol";
}

function getLaunchpad() {
  const checked = document.querySelector('input[name="launchpad"]:checked');
  return checked ? checked.value : "pump";
}

function getProvider() {
  return providerSelect ? providerSelect.value || "helius-sender" : "helius-sender";
}

function getBuyProvider() {
  return buyProviderSelect ? buyProviderSelect.value || "helius-sender" : "helius-sender";
}

function getSellProvider() {
  return sellProviderSelect ? sellProviderSelect.value || "helius-sender" : "helius-sender";
}

function getRouteCapabilities(route, rowType) {
  const normalizedRoute = String(route || "helius-sender").trim().toLowerCase();
  return ROUTE_CAPABILITIES[normalizedRoute] && ROUTE_CAPABILITIES[normalizedRoute][rowType]
    ? ROUTE_CAPABILITIES[normalizedRoute][rowType]
    : ROUTE_CAPABILITIES["helius-sender"][rowType];
}

function setFieldEnabled(input, enabled) {
  if (!input) return;
  input.disabled = !enabled;
  const label = input.closest("label");
  if (label) label.classList.toggle("is-disabled", !enabled);
}

function setFieldVisibility(input, visible) {
  if (!input) return;
  const label = input.closest("label");
  if (label) label.hidden = !visible;
}

function syncSettingsCapabilities() {
  const editing = isPresetEditing(getConfig());
  const creationCapabilities = getRouteCapabilities(getProvider(), "creation");
  const buyCapabilities = getRouteCapabilities(getBuyProvider(), "buy");
  const sellCapabilities = getRouteCapabilities(getSellProvider(), "sell");

  if (providerSelect) providerSelect.disabled = !editing;
  if (buyProviderSelect) buyProviderSelect.disabled = !editing;
  if (sellProviderSelect) sellProviderSelect.disabled = !editing;
  setFieldVisibility(creationTipInput, creationCapabilities.tip);
  setFieldVisibility(creationPriorityInput, creationCapabilities.priority);
  setFieldVisibility(buyPriorityFeeInput, buyCapabilities.priority);
  setFieldVisibility(buyTipInput, buyCapabilities.tip);
  setFieldVisibility(buySlippageInput, buyCapabilities.slippage);
  setFieldVisibility(sellPriorityFeeInput, sellCapabilities.priority);
  setFieldVisibility(sellTipInput, sellCapabilities.tip);
  setFieldVisibility(sellSlippageInput, sellCapabilities.slippage);
  setFieldEnabled(creationTipInput, editing && creationCapabilities.tip);
  setFieldEnabled(creationPriorityInput, editing && creationCapabilities.priority);
  setFieldEnabled(buyPriorityFeeInput, editing && buyCapabilities.priority);
  setFieldEnabled(buyTipInput, editing && buyCapabilities.tip);
  setFieldEnabled(buySlippageInput, editing && buyCapabilities.slippage);
  setFieldEnabled(sellPriorityFeeInput, editing && sellCapabilities.priority);
  setFieldEnabled(sellTipInput, editing && sellCapabilities.tip);
  setFieldEnabled(sellSlippageInput, editing && sellCapabilities.slippage);
}

function applyPresetToSettingsInputs(preset, options = {}) {
  if (!preset) return;
  const { syncToMainForm = true } = options;
  syncingPresetInputs = true;
  if (providerSelect) providerSelect.value = preset.creationSettings.provider || "helius-sender";
  if (creationTipInput) creationTipInput.value = preset.creationSettings.tipSol || "";
  if (creationPriorityInput) creationPriorityInput.value = preset.creationSettings.priorityFeeSol || "";
  if (buyProviderSelect) buyProviderSelect.value = preset.buySettings.provider || "helius-sender";
  if (buyPriorityFeeInput) buyPriorityFeeInput.value = preset.buySettings.priorityFeeSol || "";
  if (buyTipInput) buyTipInput.value = preset.buySettings.tipSol || "";
  if (buySlippageInput) buySlippageInput.value = preset.buySettings.slippagePercent || "";
  if (sellProviderSelect) sellProviderSelect.value = preset.sellSettings.provider || "helius-sender";
  if (sellPriorityFeeInput) sellPriorityFeeInput.value = preset.sellSettings.priorityFeeSol || "";
  if (sellTipInput) sellTipInput.value = preset.sellSettings.tipSol || "";
  if (sellSlippageInput) sellSlippageInput.value = preset.sellSettings.slippagePercent || "";
  syncingPresetInputs = false;

  if (syncToMainForm) {
    clearDevBuyState();
  }

  syncDevAutoSellUI();
  syncSettingsCapabilities();
  renderPresetChips();
  renderQuickDevBuyButtons(getConfig());
}

function syncActivePresetFromInputs() {
  if (syncingPresetInputs) return;
  const config = cloneConfig(getConfig());
  const activePreset = getActivePreset(config);
  if (!activePreset) return;
  activePreset.creationSettings = {
    ...activePreset.creationSettings,
    provider: getProvider(),
    tipSol: creationTipInput ? creationTipInput.value.trim() : "",
    priorityFeeSol: creationPriorityInput ? creationPriorityInput.value.trim() : "",
    devBuySol: activePreset.creationSettings && activePreset.creationSettings.devBuySol
      ? activePreset.creationSettings.devBuySol.trim()
      : "",
  };
  activePreset.buySettings = {
    ...activePreset.buySettings,
    provider: getBuyProvider(),
    priorityFeeSol: buyPriorityFeeInput ? buyPriorityFeeInput.value.trim() : "",
    tipSol: buyTipInput ? buyTipInput.value.trim() : "",
    slippagePercent: buySlippageInput ? buySlippageInput.value.trim() : "",
  };
  activePreset.sellSettings = {
    ...activePreset.sellSettings,
    provider: getSellProvider(),
    priorityFeeSol: sellPriorityFeeInput ? sellPriorityFeeInput.value.trim() : "",
    tipSol: sellTipInput ? sellTipInput.value.trim() : "",
    slippagePercent: sellSlippageInput ? sellSlippageInput.value.trim() : "",
  };
  setConfig(config);
  syncSettingsCapabilities();
}

function setActivePreset(presetId, options = {}) {
  const config = cloneConfig(getConfig());
  const exists = getPresetItems(config).some((entry) => entry.id === presetId);
  config.defaults = {
    ...(config.defaults || {}),
    activePresetId: exists ? presetId : DEFAULT_PRESET_ID,
  };
  setConfig(config);
  applyPresetToSettingsInputs(getActivePreset(config), options);
}

function setPresetEditing(editing) {
  const config = cloneConfig(getConfig());
  config.defaults = {
    ...(config.defaults || {}),
    presetEditing: Boolean(editing),
  };
  setConfig(config);
  const inputs = [
    providerSelect,
    creationTipInput,
    creationPriorityInput,
    buyProviderSelect,
    buyPriorityFeeInput,
    buyTipInput,
    buySlippageInput,
    sellProviderSelect,
    sellPriorityFeeInput,
    sellTipInput,
    sellSlippageInput,
  ];
  inputs.forEach((input) => {
    if (!input) return;
    input.disabled = !editing;
  });
  syncSettingsCapabilities();
}


function renderVanityButtonState() {
  if (!modeVanityButton) return;
  modeVanityButton.classList.toggle("active", Boolean(getNamedValue("vanityPrivateKey").trim()));
}


function showVanityModal() {
  if (vanityPrivateKeyText) vanityPrivateKeyText.value = getNamedValue("vanityPrivateKey");
  if (vanityModalError) vanityModalError.textContent = "";
  if (vanityModal) vanityModal.hidden = false;
}

function hideVanityModal() {
  if (vanityModal) vanityModal.hidden = true;
}

function showVampModal() {
  if (vampError) vampError.textContent = "";
  if (vampStatus) {
    vampStatus.hidden = true;
    vampStatus.textContent = "";
  }
  if (vampContractInput) vampContractInput.value = "";
  if (vampModal) vampModal.hidden = false;
  if (vampContractInput) queueMicrotask(() => vampContractInput.focus());
}

function hideVampModal() {
  if (vampModal) vampModal.hidden = true;
}

function setVampStatus(message = "") {
  if (!vampStatus) return;
  vampStatus.hidden = !message;
  vampStatus.textContent = message;
}

async function importVampToken() {
  const contractAddress = vampContractInput ? vampContractInput.value.trim() : "";
  if (!contractAddress) {
    if (vampError) vampError.textContent = "Contract address is required.";
    return;
  }
  if (vampError) vampError.textContent = "";
  setVampStatus("Importing token metadata...");
  if (vampImport) vampImport.disabled = true;
  if (vampCancel) vampCancel.disabled = true;
  if (vampClose) vampClose.disabled = true;
  try {
    const response = await fetch("/api/vamp", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ contractAddress }),
    });
    const payload = await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to import token metadata.");
    }

    if (nameInput) nameInput.value = payload.token && payload.token.name ? payload.token.name : "";
    if (symbolInput) {
      syncingTickerFromName = true;
      symbolInput.value = formatTickerValue(payload.token && payload.token.symbol ? payload.token.symbol : "");
      syncingTickerFromName = false;
      tickerManuallyEdited = Boolean(String(payload.token && payload.token.symbol ? payload.token.symbol : "").trim());
      tickerClearedForManualEntry = false;
    }
    if (descriptionInput) {
      descriptionInput.value = payload.token && payload.token.description ? payload.token.description : "";
      toggleDescriptionDisclosure(Boolean(descriptionInput.value.trim()));
      updateDescriptionDisclosure();
    }
    if (websiteInput) websiteInput.value = payload.token && payload.token.website ? payload.token.website : "";
    if (twitterInput) twitterInput.value = payload.token && payload.token.twitter ? payload.token.twitter : "";
    if (telegramInput) telegramInput.value = payload.token && payload.token.telegram ? payload.token.telegram : "";
    setMode(payload.token && payload.token.mode ? payload.token.mode : "regular");
    clearMetadataUploadCache({ clearInput: true });
    updateTokenFieldCounts();

    if (payload.image) {
      imageLibraryState.activeImageId = payload.image.id || "";
      setSelectedImage(payload.image);
      try {
        await fetchImageLibrary();
      } catch (_error) {
        // Keep the imported image selected even if the library refresh fails.
      }
    }

    imageStatus.textContent = payload.image
      ? "Token image imported to library."
      : (payload.warning || "");
    imagePath.textContent = "";
    hideVampModal();
  } catch (error) {
    if (vampError) vampError.textContent = error.message;
    setVampStatus("");
  } finally {
    if (vampImport) vampImport.disabled = false;
    if (vampCancel) vampCancel.disabled = false;
    if (vampClose) vampClose.disabled = false;
  }
}

function applyVanityValue(rawValue) {
  const nextValue = String(rawValue || "").trim();
  if (vanityPrivateKeyInput) vanityPrivateKeyInput.value = nextValue;
  renderVanityButtonState();
}

function hydrateModeActionState() {
  const storedDraft = getStoredSniperDraft();
  const enabled = getNamedValue("sniperEnabled") === "true";
  let wallets = {};
  if (storedDraft) {
    sniperFeature.setState(storedDraft);
    applySniperStateToForm();
    renderSniperUI();
    renderVanityButtonState();
    return;
  }
  try {
    const parsed = JSON.parse(getNamedValue("sniperConfigJson") || "[]");
    if (Array.isArray(parsed)) {
      wallets = parsed.reduce((accumulator, entry) => {
        if (!entry || !entry.envKey) return accumulator;
        accumulator[entry.envKey] = {
          selected: true,
          amountSol: entry.amountSol || "",
          triggerMode: entry.targetBlockOffset != null
            ? "block-offset"
            : (entry.submitWithLaunch
              ? "same-time"
              : "on-submit"),
          submitDelayMs: entry.submitDelayMs || 0,
          targetBlockOffset: entry.targetBlockOffset,
          retryOnce: Boolean(entry.retryOnce),
        };
        return accumulator;
      }, {});
    }
  } catch (_error) {
    wallets = {};
  }
  sniperFeature.setState({ enabled, wallets });
  applySniperStateToForm();
  renderSniperUI();
  renderVanityButtonState();
}

function setImagePreview(previewUrl) {
  if (!previewUrl) {
    imagePreview.removeAttribute("src");
    imagePreview.hidden = true;
    imageEmpty.hidden = false;
    return;
  }
  imagePreview.src = previewUrl;
  imagePreview.hidden = false;
  imageEmpty.hidden = true;
}

function selectedWalletKey() {
  return walletSelect.value || "";
}

function hasBootstrapConfig() {
  return Boolean(appBootstrapState.staticLoaded && appBootstrapState.configLoaded && latestWalletStatus && latestWalletStatus.config);
}

function ensureInteractiveBootstrapReady(message = "App settings are still loading from the backend.") {
  if (hasBootstrapConfig()) return true;
  statusNode.textContent = "Loading";
  metaNode.textContent = message;
  return false;
}

function markBootstrapState(nextState = {}) {
  appBootstrapState = {
    ...appBootstrapState,
    ...nextState,
  };
}

function setSettingsLoadingState(isLoading) {
  if (!settingsModal) return;
  settingsModal.classList.toggle("settings-loading", Boolean(isLoading));
  const controls = settingsModal.querySelectorAll("input, select, button");
  controls.forEach((control) => {
    if (control === settingsClose || control === settingsCancel) return;
    control.disabled = Boolean(isLoading);
  });
}

function selectedWalletRecord() {
  const wallets = latestWalletStatus && Array.isArray(latestWalletStatus.wallets) ? latestWalletStatus.wallets : [];
  return wallets.find((wallet) => wallet.envKey === selectedWalletKey()) || null;
}

function getStoredSelectedWalletKey() {
  try {
    return window.localStorage.getItem(SELECTED_WALLET_STORAGE_KEY) || "";
  } catch (_error) {
    return "";
  }
}

function setStoredSelectedWalletKey(walletKey) {
  try {
    const normalized = String(walletKey || "").trim();
    if (!normalized) {
      window.localStorage.removeItem(SELECTED_WALLET_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(SELECTED_WALLET_STORAGE_KEY, normalized);
  } catch (_error) {
    // Ignore storage access failures and keep the UI functional.
  }
}

function shortAddress(value) {
  if (!value) return "-";
  if (value.length <= 10) return value;
  return `${value.slice(0, 4)}...${value.slice(-4)}`;
}

function walletIndexFromEnvKey(envKey) {
  if (!envKey) return "?";
  const suffix = envKey.replace("SOLANA_PRIVATE_KEY", "");
  return suffix ? suffix : "1";
}

function walletLabel(wallet, balanceSol) {
  if (!wallet) return "No wallet";
  const displayName = walletDisplayName(wallet);
  if (!wallet.publicKey) return `${displayName}: invalid`;
  const bal = balanceSol != null ? ` | ${Number(balanceSol).toFixed(4)} SOL` : "";
  return `${displayName} - ${wallet.publicKey}${bal}`;
}

function walletDisplayName(wallet) {
  if (!wallet) return "No wallet";
  if (wallet.customName && String(wallet.customName).trim()) {
    return String(wallet.customName).trim();
  }
  const index = walletIndexFromEnvKey(wallet.envKey);
  return `#${index}`;
}

function walletBalanceSol(wallet) {
  if (!wallet || wallet.balanceSol == null || Number.isNaN(Number(wallet.balanceSol))) return null;
  return Number(wallet.balanceSol);
}

function formatWalletSol(value) {
  if (value == null || Number.isNaN(Number(value))) return "--";
  return Number(value).toFixed(2);
}

function formatWalletUsd(value) {
  if (value == null || Number.isNaN(Number(value))) return "--";
  return Number(value).toFixed(2);
}

function walletUsdValue(wallet) {
  if (!wallet || wallet.usd1Balance == null || Number.isNaN(Number(wallet.usd1Balance))) return null;
  return Number(wallet.usd1Balance);
}

function renderWalletSummary() {
  if (!walletSummarySol || !walletSummaryUsd) return;
  const selectedWallet = selectedWalletRecord();
  walletSummarySol.textContent = formatWalletSol(walletBalanceSol(selectedWallet));
  walletSummaryUsd.textContent = formatWalletUsd(walletUsdValue(selectedWallet));
}

function renderWalletDropdownList(wallets = [], selectedKey = "") {
  if (!walletDropdownList) return;
  if (!wallets.length) {
    const emptyMarkup = `<div class="wallet-empty-state">${appBootstrapState.walletsLoaded ? "No wallets found" : "Loading wallets..."}</div>`;
    if (RenderUtils.setCachedHTML) {
      RenderUtils.setCachedHTML(renderCache, "walletDropdown", walletDropdownList, emptyMarkup);
    } else {
      walletDropdownList.innerHTML = emptyMarkup;
    }
    return;
  }
  const markup = wallets.map((wallet) => {
    const solValue = walletBalanceSol(wallet);
    const usdValue = walletUsdValue(wallet);
    return `
      <button
        type="button"
        class="wallet-option-button${wallet.envKey === selectedKey ? " is-selected" : ""}"
        data-wallet-key="${escapeHTML(wallet.envKey || "")}"
      >
        <span class="wallet-option-main">
          <span class="wallet-option-name">${escapeHTML(walletDisplayName(wallet))}</span>
          <span class="wallet-option-meta">${escapeHTML(shortenAddress(wallet.publicKey || "Unavailable"))}</span>
        </span>
        <span class="wallet-option-balances">
          <span class="wallet-option-sol">${escapeHTML(formatWalletSol(solValue))} SOL</span>
          <span class="wallet-option-usd">${escapeHTML(formatWalletUsd(usdValue))} USD1</span>
        </span>
      </button>
    `;
  }).join("");
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "walletDropdown", walletDropdownList, markup);
  } else {
    walletDropdownList.innerHTML = markup;
  }
}

function setWalletDropdownOpen(isOpen) {
  if (walletDropdown) walletDropdown.hidden = !isOpen;
  if (walletTriggerButton) walletTriggerButton.setAttribute("aria-expanded", String(isOpen));
}

function toggleWalletDropdown() {
  setWalletDropdownOpen(!walletDropdown || walletDropdown.hidden);
}

function connectedWalletText() {
  return latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "Connected wallet";
}

function shortenAddress(addr, chars = 6) {
  if (!addr || addr.length <= chars * 2 + 3) return addr;
  return addr.slice(0, chars) + "..." + addr.slice(-chars);
}

function connectedWalletShort() {
  return latestWalletStatus && latestWalletStatus.wallet
    ? shortenAddress(latestWalletStatus.wallet)
    : "Connected wallet";
}

function getFeeSplitRows() {
  return Array.from(feeSplitList.querySelectorAll(".fee-split-row"));
}

function createFeeSplitRow(entry = {}) {
  const row = document.createElement("div");
  row.className = "fee-split-row";
  row.dataset.type = entry.type || "wallet";
  if (entry.defaultReceiver) row.dataset.defaultReceiver = "true";
  row.innerHTML = `
    <div class="fee-split-row-top">
      <div class="recipient-type-tabs">
        <button type="button" class="recipient-type-tab" data-type="wallet">Wallet</button>
        <button type="button" class="recipient-type-tab" data-type="github">GitHub</button>
      </div>
      <button type="button" class="recipient-remove" aria-label="Remove recipient">×</button>
    </div>
    <div class="fee-split-row-main">
      <div class="recipient-target-wrap">
        <input class="recipient-target" type="text" placeholder="Wallet address">
        <button type="button" class="recipient-lock-toggle">Set</button>
      </div>
      <div class="recipient-share-box">
        <input class="recipient-share" type="number" min="0" max="100" step="0.01" placeholder="0">
        <span>%</span>
      </div>
    </div>
    <input class="recipient-slider" type="range" min="0" max="100" step="0.01" value="0">
  `;

  row.querySelector(".recipient-target").value = entry.value || "";
  row.querySelector(".recipient-share").value = entry.sharePercent || "";
  row.querySelector(".recipient-slider").value = entry.sharePercent || "0";
  updateFeeSplitRowType(row, row.dataset.type);
  setRecipientTargetLocked(row, Boolean(entry.targetLocked));
  return row;
}

function updateFeeSplitRowType(row, type) {
  row.dataset.type = type;
  row.querySelectorAll(".recipient-type-tab").forEach((button) => {
    button.classList.toggle("active", button.dataset.type === type);
  });
  const target = row.querySelector(".recipient-target");
  target.placeholder = type === "github" ? "GitHub username" : "Wallet address";
}

function setRecipientTargetLocked(row, locked) {
  if (!row || row.dataset.locked === "true") return;
  const target = row.querySelector(".recipient-target");
  const toggle = row.querySelector(".recipient-lock-toggle");
  if (!target || !toggle) return;

  if (locked) {
    if (!target.value.trim()) {
      target.focus();
      return;
    }
    row.dataset.targetLocked = "true";
    target.readOnly = true;
    target.title = target.value.trim();
    toggle.textContent = "Set";
    toggle.classList.add("is-set");
  } else {
    delete row.dataset.targetLocked;
    target.readOnly = false;
    target.title = "";
    toggle.textContent = "Set";
    toggle.classList.remove("is-set");
  }

  row.querySelectorAll(".recipient-type-tab").forEach((button) => {
    button.disabled = locked;
  });
}

function ensureFeeSplitDefaultRow() {
  if (!feeSplitList) return;
  const deployerAddress = latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "";
  let defaultRow = getFeeSplitRows().find((row) => row.dataset.defaultReceiver === "true");
  if (!defaultRow) {
    feeSplitList.appendChild(createFeeSplitRow({
      type: "wallet",
      value: deployerAddress,
      sharePercent: "100",
      defaultReceiver: true,
      targetLocked: true,
    }));
    defaultRow = getFeeSplitRows().find((row) => row.dataset.defaultReceiver === "true");
  }
  if (!defaultRow) return;
  const target = defaultRow.querySelector(".recipient-target");
  const share = defaultRow.querySelector(".recipient-share");
  const slider = defaultRow.querySelector(".recipient-slider");
  if (target && deployerAddress && (!target.value.trim() || target.value !== deployerAddress)) {
    target.value = deployerAddress;
  }
  if (share && !share.value.trim()) share.value = "100";
  if (slider && !Number(slider.value || 0)) slider.value = share && share.value.trim() ? share.value.trim() : "100";
  setRecipientTargetLocked(defaultRow, true);
}

function syncFeeSplitTotals() {
  const rows = getFeeSplitRows();
  const total = rows.reduce((sum, row) => {
    const value = Number(row.querySelector(".recipient-share").value || 0);
    return sum + (Number.isFinite(value) ? value : 0);
  }, 0);
  feeSplitTotal.textContent = `${total.toFixed(2).replace(/\.00$/, "")}%`;
  feeSplitTotal.classList.toggle("invalid", Math.abs(total - 100) > 0.001 && total !== 0);
  feeSplitReset.disabled = rows.length === 0;
  feeSplitEven.disabled = rows.length === 0;
  if (feeSplitAdd) feeSplitAdd.disabled = rows.length >= MAX_FEE_SPLIT_RECIPIENTS;

  if (rows.length === 0 || total === 0) {
    feeSplitBar.style.background = "#1e2630";
    feeSplitLegendList.innerHTML = "";
    return;
  }

  let running = 0;
  const gradientStops = [];
  const legendItems = [];
  rows.forEach((row, index) => {
    const share = Number(row.querySelector(".recipient-share").value || 0);
    const color = SPLIT_COLORS[index % SPLIT_COLORS.length];
    const targetValue = row.querySelector(".recipient-target").value.trim();
    const label = targetValue
      ? row.dataset.type === "github"
        ? `@${targetValue.replace(/^@+/, "")}`
        : targetValue
      : row.dataset.type === "github"
        ? "@github"
        : "wallet";
    if (share > 0) {
      const start = running;
      running += share;
      gradientStops.push(`${color} ${start}%`, `${color} ${running}%`);
      legendItems.push(
        `<span class="legend-chip"><span class="legend-dot" style="background:${color}"></span>${label} ${share.toFixed(2).replace(/\.00$/, "")}%</span>`
      );
    }
  });

  if (running < 100) {
    gradientStops.push(`#1e2630 ${running}%`, "#1e2630 100%");
  }

  feeSplitBar.style.background = gradientStops.length
    ? `linear-gradient(90deg, ${gradientStops.join(", ")})`
    : "#1e2630";
  feeSplitLegendList.innerHTML = legendItems.join("");
}

function updateFeeSplitVisibility() {
  const mode = getMode();
  const active = mode === "agent-custom" || (mode === "regular" && feeSplitEnabled.checked);
  feeSplitPill.classList.toggle("active", active);
  feeSplitPill.disabled = mode !== "regular" && mode !== "agent-custom";
  if (mode === "regular" && active) ensureFeeSplitDefaultRow();
  if (mode !== "regular" && feeSplitModal) feeSplitModal.hidden = true;
  syncFeeSplitTotals();
  syncSettingsCapabilities();
}

function showFeeSplitModal() {
  const mode = getMode();
  if (mode === "regular") {
    feeSplitEnabled.checked = true;
    updateFeeSplitVisibility();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    if (feeSplitModal) feeSplitModal.hidden = false;
    return;
  }
  if (mode === "agent-custom") {
    showAgentSplitModal();
  }
}

function hideFeeSplitModal() {
  if (feeSplitModal) feeSplitModal.hidden = true;
}

function createAgentSplitRow(entry = {}) {
  const isAgent = entry.locked === true;
  const row = document.createElement("div");
  row.className = "fee-split-row";
  row.dataset.type = isAgent ? "agent" : (entry.type || "wallet");
  if (isAgent) row.dataset.locked = "true";
  if (entry.defaultReceiver) row.dataset.defaultReceiver = "true";

  if (isAgent) {
    row.innerHTML = `
      <div class="fee-split-row-top">
        <div class="recipient-type-tabs">
          <span class="recipient-type-tab active locked-tab">Agent Buyback</span>
        </div>
      </div>
      <div class="fee-split-row-main">
        <input class="recipient-target" type="text" value="Agent fee split receiver (derived)" disabled>
        <div class="recipient-share-box">
          <input class="recipient-share" type="number" min="0" max="100" step="0.01" placeholder="0">
          <span>%</span>
        </div>
      </div>
      <input class="recipient-slider" type="range" min="0" max="100" step="0.01" value="0">
    `;
  } else {
    row.innerHTML = `
      <div class="fee-split-row-top">
        <div class="recipient-type-tabs">
          <button type="button" class="recipient-type-tab" data-type="wallet">Wallet</button>
          <button type="button" class="recipient-type-tab" data-type="github">GitHub</button>
        </div>
        <button type="button" class="recipient-remove" aria-label="Remove recipient">×</button>
      </div>
      <div class="fee-split-row-main">
        <div class="recipient-target-wrap">
          <input class="recipient-target" type="text" placeholder="Wallet address">
          <button type="button" class="recipient-lock-toggle">Set</button>
        </div>
        <div class="recipient-share-box">
          <input class="recipient-share" type="number" min="0" max="100" step="0.01" placeholder="0">
          <span>%</span>
        </div>
      </div>
      <input class="recipient-slider" type="range" min="0" max="100" step="0.01" value="0">
    `;
    row.querySelector(".recipient-target").value = entry.value || "";
    updateFeeSplitRowType(row, row.dataset.type);
    setRecipientTargetLocked(row, Boolean(entry.targetLocked));
  }
  row.querySelector(".recipient-share").value = entry.sharePercent || "";
  row.querySelector(".recipient-slider").value = entry.sharePercent || "0";
  return row;
}

function getAgentSplitRows() {
  return Array.from(agentSplitList.querySelectorAll(".fee-split-row"));
}

function syncAgentSplitTotals() {
  const rows = getAgentSplitRows();
  const total = rows.reduce((sum, row) => {
    const value = Number(row.querySelector(".recipient-share").value || 0);
    return sum + (Number.isFinite(value) ? value : 0);
  }, 0);
  agentSplitTotal.textContent = `${total.toFixed(2).replace(/\.00$/, "")}%`;
  agentSplitTotal.classList.toggle("invalid", Math.abs(total - 100) > 0.001 && total !== 0);
  agentSplitReset.disabled = rows.length === 0;
  agentSplitEven.disabled = rows.length === 0;
  if (agentSplitAdd) agentSplitAdd.disabled = rows.length >= MAX_FEE_SPLIT_RECIPIENTS;

  if (rows.length === 0 || total === 0) {
    agentSplitBar.style.background = "#1e2630";
    agentSplitLegendList.innerHTML = "";
    return;
  }

  let running = 0;
  const gradientStops = [];
  const legendItems = [];
  rows.forEach((row, index) => {
    const share = Number(row.querySelector(".recipient-share").value || 0);
    const color = SPLIT_COLORS[index % SPLIT_COLORS.length];
    const targetValue = row.querySelector(".recipient-target").value.trim();
    const label = row.dataset.locked
      ? "Agent Buyback"
      : targetValue
        ? row.dataset.type === "github"
          ? `@${targetValue.replace(/^@+/, "")}`
          : shortenAddress(targetValue, 4) || "wallet"
        : "wallet";
    if (share > 0) {
      const start = running;
      running += share;
      gradientStops.push(`${color} ${start}%`, `${color} ${running}%`);
      legendItems.push(
        `<span class="legend-chip"><span class="legend-dot" style="background:${color}"></span>${label} ${share.toFixed(2).replace(/\.00$/, "")}%</span>`
      );
    }
  });

  if (running < 100) {
    gradientStops.push(`#1e2630 ${running}%`, "#1e2630 100%");
  }

  agentSplitBar.style.background = gradientStops.length
    ? `linear-gradient(90deg, ${gradientStops.join(", ")})`
    : "#1e2630";
  agentSplitLegendList.innerHTML = legendItems.join("");
}

function initAgentSplitIfEmpty() {
  if (agentSplitList.children.length === 0) {
    if (!seedAgentSplitFromFeeSplit()) {
      resetAgentSplitToDefault();
    }
  }
}

function showAgentSplitModal() {
  if (getMode() !== "agent-custom") return;
  initAgentSplitIfEmpty();
  syncAgentSplitTotals();
  if (agentSplitModalError) agentSplitModalError.textContent = "";
  if (agentSplitModal) agentSplitModal.hidden = false;
}

function hideAgentSplitModal() {
  if (agentSplitModal) agentSplitModal.hidden = true;
}

function setAgentSplitModalError(message = "") {
  if (agentSplitModalError) agentSplitModalError.textContent = message;
}

function seedAgentSplitFromFeeSplit() {
  if (!agentSplitList) return false;
  const regularRows = getFeeSplitRows();
  if (!regularRows.length) return false;
  const defaultReceiverRow = regularRows.find((row) => row.dataset.defaultReceiver === "true");
  if (!defaultReceiverRow) return false;

  const agentSharePercent = defaultReceiverRow.querySelector(".recipient-share").value.trim() || "0";
  const carriedRows = regularRows
    .filter((row) => row !== defaultReceiverRow)
    .map((row) => ({
      type: row.dataset.type || "wallet",
      value: row.querySelector(".recipient-target").value.trim(),
      sharePercent: row.querySelector(".recipient-share").value.trim(),
      targetLocked: row.dataset.targetLocked === "true",
    }))
    .filter((entry) => entry.value || entry.sharePercent);

  agentSplitList.innerHTML = "";
  agentSplitList.appendChild(createAgentSplitRow({ locked: true, sharePercent: agentSharePercent }));
  carriedRows.forEach((entry) => {
    agentSplitList.appendChild(createAgentSplitRow(entry));
  });
  syncAgentSplitTotals();
  setAgentSplitModalError("");
  return true;
}

function resetAgentSplitToDefault() {
  if (!agentSplitList) return;
  agentSplitList.innerHTML = "";
  agentSplitList.appendChild(createAgentSplitRow({ locked: true, sharePercent: "50" }));
  agentSplitList.appendChild(
    createAgentSplitRow({
      type: "wallet",
      value: latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "",
      sharePercent: "50",
      defaultReceiver: true,
      targetLocked: true,
    })
  );
  syncAgentSplitTotals();
  setAgentSplitModalError("");
}

function attemptCloseAgentSplitModal() {
  const errors = validateAgentSplit();
  if (errors.length > 0) {
    setAgentSplitModalError(errors[0]);
    return false;
  }
  setStoredAgentSplitDraft(serializeAgentSplitDraft());
  setAgentSplitModalError("");
  hideAgentSplitModal();
  return true;
}

function normalizeAgentSplitStructure({ afterAdd = false } = {}) {
  const rows = getAgentSplitRows();
  const agentRow = rows.find((row) => row.dataset.locked === "true");
  const otherRows = rows.filter((row) => row.dataset.locked !== "true");
  if (!agentRow) return;

  const agentShareInput = agentRow.querySelector(".recipient-share");
  const agentSliderInput = agentRow.querySelector(".recipient-slider");

  if (otherRows.length === 0) {
    agentShareInput.value = "100";
    agentSliderInput.value = "100";
    return;
  }

  if (afterAdd && otherRows.length === 1) {
    const currentAgentShare = Number(agentShareInput.value || 0);
    const currentOtherShare = Number(otherRows[0].querySelector(".recipient-share").value || 0);
    if (Math.abs(currentAgentShare - 100) < 0.001 && Math.abs(currentOtherShare) < 0.001) {
      agentShareInput.value = "50";
      agentSliderInput.value = "50";
      otherRows[0].querySelector(".recipient-share").value = "50";
      otherRows[0].querySelector(".recipient-slider").value = "50";
    }
  }
}

function collectAgentSplitRecipients() {
  return getAgentSplitRows().map((row) => {
    if (row.dataset.locked) {
      const sharePercent = row.querySelector(".recipient-share").value.trim();
      const numericShare = Number(sharePercent);
      return {
        type: "agent",
        shareBps: Number.isFinite(numericShare) ? Math.round(numericShare * 100) : NaN,
      };
    }
    const type = row.dataset.type || "wallet";
    const value = row.querySelector(".recipient-target").value.trim();
    const sharePercent = row.querySelector(".recipient-share").value.trim();
    if (!value && !sharePercent) return null;
    const numericShare = Number(sharePercent);
    return {
      type,
      address: type === "wallet" ? value : "",
      githubUsername: type === "github" ? value.replace(/^@+/, "") : "",
      shareBps: Number.isFinite(numericShare) ? Math.round(numericShare * 100) : NaN,
    };
  }).filter(Boolean);
}

function updateLockedModeFields() {
  const full = connectedWalletText();
  const short = connectedWalletShort();
  if (agentUnlockedAuthority) {
    agentUnlockedAuthority.value = short;
    agentUnlockedAuthority.title = full;
  }

  const defaultReceiverRow = getAgentSplitRows().find((row) => row.dataset.defaultReceiver === "true");
  if (defaultReceiverRow) {
    const target = defaultReceiverRow.querySelector(".recipient-target");
    if (target && !target.value.trim() && latestWalletStatus && latestWalletStatus.wallet) {
      target.value = latestWalletStatus.wallet;
      setRecipientTargetLocked(defaultReceiverRow, true);
      syncAgentSplitTotals();
    }
  }

  ensureFeeSplitDefaultRow();
}

function updateModeVisibility() {
  const mode = getMode();
  document.querySelectorAll("[data-mode-panel]").forEach((node) => {
    node.hidden = node.getAttribute("data-mode-panel") !== mode;
  });
  updateFeeSplitVisibility();
  updateLockedModeFields();
  if (mode === "agent-custom") initAgentSplitIfEmpty();
  if (mode !== "agent-custom") hideAgentSplitModal();
  updateJitoVisibility();
}

function usesBundledJito() {
  return getProvider() === "jito-bundle" && Number(creationTipInput ? creationTipInput.value || 0 : 0) > 0;
}

function updateJitoVisibility() {
  syncSettingsCapabilities();
}

function applyProviderAvailability(providers = {}) {
  [providerSelect, buyProviderSelect, sellProviderSelect].forEach((select) => {
    if (!select) return;
    Array.from(select.options).forEach((option) => {
      const entry = providers[option.value];
      option.disabled = Boolean(entry && !entry.available);
      option.textContent = PROVIDER_LABELS[option.value] || option.textContent.replace(/ \(unverified\)$/, "");
      if (entry && entry.reason) {
        option.title = entry.reason;
      }
    });
    if (select.selectedOptions[0] && select.selectedOptions[0].disabled) {
      const fallback = Array.from(select.options).find((option) => !option.disabled);
      if (fallback) select.value = fallback.value;
    }
  });
  syncSettingsCapabilities();
}

function applyLaunchpadAvailability(launchpads = {}) {
  if (!launchpadInputs.length) return;
  latestLaunchpadRegistry = launchpads && typeof launchpads === "object" ? launchpads : {};
  launchpadInputs.forEach((input) => {
    const label = input.closest(".launchpad-option");
    const titleNode = label ? label.querySelector(".launchpad-title") : null;
    const entry = launchpads[input.value];
    const unavailable = Boolean(entry && !entry.available);
    input.disabled = unavailable;
    if (label) {
      label.classList.toggle("is-disabled", unavailable);
      if (entry && entry.reason) {
        label.title = entry.reason;
      } else {
        label.removeAttribute("title");
      }
    }
    if (titleNode) {
      const baseLabel = input.value === "bagsapp"
        ? "Bagsapp"
        : input.value.charAt(0).toUpperCase() + input.value.slice(1);
      titleNode.textContent = baseLabel;
    }
  });

  const checked = document.querySelector('input[name="launchpad"]:checked');
  if (!checked || checked.disabled) {
    const fallback = launchpadInputs.find((input) => !input.disabled);
    if (fallback) fallback.checked = true;
  }

  applyLaunchpadTokenMetadata();
}

function applyPersistentDefaults(config) {
  if (!config || defaultsApplied) return;
  const defaults = config.defaults || {};
  const storedSniperDraft = getStoredSniperDraft();
  const storedMode = getStoredLaunchMode();
  const storedFeeSplitDraft = getStoredFeeSplitDraft();
  const storedAgentSplitDraft = getStoredAgentSplitDraft();
  if (defaults.launchpad) {
    const launchpadInput = document.querySelector(`input[name="launchpad"][value="${defaults.launchpad}"]`);
    if (launchpadInput) launchpadInput.checked = true;
  }
  setConfig(config);
  applyPresetToSettingsInputs(getActivePreset(config));
  if (defaults.automaticDevSell) {
    if (autoSellEnabledInput) autoSellEnabledInput.checked = Boolean(defaults.automaticDevSell.enabled);
    setNamedValue(
      "automaticDevSellPercent",
      String(defaults.automaticDevSell.enabled
        ? Math.max(1, Number(defaults.automaticDevSell.percent || 100))
        : Number(defaults.automaticDevSell.percent || 100)),
    );
    setNamedValue(
      "automaticDevSellTriggerMode",
      normalizeAutoSellTriggerMode(
        defaults.automaticDevSell.triggerMode
          || (Number(defaults.automaticDevSell.delaySeconds || 0) > 0 ? "submit-delay" : "confirmation"),
      ),
    );
    setNamedValue(
      "automaticDevSellDelayMs",
      String(defaults.automaticDevSell.delayMs != null
        ? defaults.automaticDevSell.delayMs
        : Number(defaults.automaticDevSell.delaySeconds || 0) * 1000),
    );
    setNamedValue("automaticDevSellBlockOffset", String(defaults.automaticDevSell.targetBlockOffset || 0));
  }
  applyFeeSplitDraft(
    storedFeeSplitDraft || (defaults.misc && defaults.misc.feeSplitDraft) || null,
    { persist: false },
  );
  applyAgentSplitDraft(
    storedAgentSplitDraft || (defaults.misc && defaults.misc.agentSplitDraft) || null,
    { persist: false },
  );
  setMode(storedMode || defaults.mode || "regular");
  syncDevAutoSellUI();
  setPresetEditing(Boolean(defaults.presetEditing));
  if (!storedSniperDraft && defaults.misc && defaults.misc.sniperDraft) {
    sniperFeature.setState(normalizeSniperDraftState(defaults.misc.sniperDraft));
    applySniperStateToForm();
  }
  renderQuickDevBuyButtons(config);
  populateDevBuyPresetEditor(config);
  defaultsApplied = true;
}

function collectFeeSplitRecipients() {
  return Array.from(feeSplitList.querySelectorAll(".fee-split-row"))
    .map((row) => {
      const type = row.dataset.type || "wallet";
      const value = row.querySelector(".recipient-target").value.trim();
      const sharePercent = row.querySelector(".recipient-share").value.trim();
      if (!value && !sharePercent) return null;
      const numericShare = Number(sharePercent);
      return {
        type,
        address: type === "wallet" ? value : "",
        githubUsername: type === "github" ? value.replace(/^@+/, "") : "",
        shareBps: Number.isFinite(numericShare) ? Math.round(numericShare * 100) : NaN,
      };
    })
    .filter(Boolean);
}

function readForm() {
  const data = new FormData(form);
  const values = Object.fromEntries(data.entries());
  const mode = values.mode || "regular";
  const creationCapabilities = getRouteCapabilities(getProvider(), "creation");
  const buyCapabilities = getRouteCapabilities(getBuyProvider(), "buy");
  const sellCapabilities = getRouteCapabilities(getSellProvider(), "sell");
  const devBuyAmount = String(values.devBuyAmount || "").trim();
  const agentSplitRecipients = mode === "agent-custom" ? collectAgentSplitRecipients() : [];
  const agentBuyback = agentSplitRecipients.find((entry) => entry.type === "agent");
  let sniperWallets = [];
  try {
    const parsed = JSON.parse(getNamedValue("sniperConfigJson") || "[]");
    if (Array.isArray(parsed)) {
      sniperWallets = parsed.filter((entry) => entry && entry.envKey);
    }
  } catch (_error) {
    sniperWallets = [];
  }

  return {
    selectedWalletKey: selectedWalletKey(),
    launchpad: getLaunchpad(),
    provider: getProvider(),
    buyProvider: getBuyProvider(),
    sellProvider: getSellProvider(),
    activePresetId: getActivePresetId(),
    mode,
    name: values.name || "",
    symbol: values.symbol || "",
    description: values.description || "",
    website: values.website || "",
    twitter: values.twitter || "",
    telegram: values.telegram || "",
    mayhemMode: data.get("mayhemMode") === "on",
    agentAuthority: values.agentAuthority || "",
    buybackPercent:
      mode === "agent-custom"
        ? agentBuyback ? String(agentBuyback.shareBps / 100) : ""
        : values.agentUnlockedBuybackPercent || "",
    agentSplitRecipients,
    devBuyMode: devBuyAmount ? getDevBuyMode() : "",
    devBuyAmount,
    autoGas: true,
    buyAutoGas: true,
    priorityFeeSol: creationCapabilities.priority ? (getNamedValue("creationPriorityFeeSol") || "") : "",
    creationTipSol: creationCapabilities.tip ? (getNamedValue("creationTipSol") || "") : "",
    maxPriorityFeeSol: creationCapabilities.priority ? (getNamedValue("creationPriorityFeeSol") || "") : "",
    maxTipSol: creationCapabilities.tip ? (getNamedValue("creationTipSol") || "") : "",
    buyPriorityFeeSol: buyCapabilities.priority ? (getNamedValue("buyPriorityFeeSol") || "") : "",
    buyTipSol: buyCapabilities.tip ? (getNamedValue("buyTipSol") || "") : "",
    buySlippagePercent: getNamedValue("buySlippagePercent") || "",
    buyMaxPriorityFeeSol: buyCapabilities.priority ? (getNamedValue("buyPriorityFeeSol") || "") : "",
    buyMaxTipSol: buyCapabilities.tip ? (getNamedValue("buyTipSol") || "") : "",
    sellPriorityFeeSol: sellCapabilities.priority ? (getNamedValue("sellPriorityFeeSol") || "") : "",
    sellTipSol: sellCapabilities.tip ? (getNamedValue("sellTipSol") || "") : "",
    sellSlippagePercent: getNamedValue("sellSlippagePercent") || "",
    enableJito: getProvider() === "jito-bundle" || Number(getNamedValue("creationTipSol") || 0) > 0,
    jitoTipSol: creationCapabilities.tip ? (getNamedValue("creationTipSol") || "") : "",
    skipPreflight: getNamedValue("skipPreflight") === "true",
    trackSendBlockHeight: true,
    feeSplitEnabled: mode === "regular" && feeSplitEnabled.checked,
    feeSplitRecipients: mode === "regular" && feeSplitEnabled.checked ? collectFeeSplitRecipients() : [],
    postLaunchStrategy: getNamedValue("postLaunchStrategy") || "none",
    snipeBuyAmountSol: getNamedValue("snipeBuyAmountSol") || "",
    sniperEnabled: getNamedValue("sniperEnabled") === "true",
    sniperWallets,
    sniperConfigJson: getNamedValue("sniperConfigJson") || "[]",
    automaticDevSellEnabled: isNamedChecked("automaticDevSellEnabled"),
    automaticDevSellPercent: getNamedValue("automaticDevSellPercent") || "0",
    automaticDevSellTriggerMode: getAutoSellTriggerMode(),
    automaticDevSellDelayMs: String(getAutoSellDelayMs()),
    automaticDevSellBlockOffset: String(getAutoSellBlockOffset()),
    vanityPrivateKey: getNamedValue("vanityPrivateKey") || "",
    imageFileName: uploadedImage ? uploadedImage.fileName : "",
    metadataUri: metadataUri.value || "",
  };
}

function metadataFingerprintFromForm(formValues = readForm()) {
  return JSON.stringify({
    imageId: uploadedImage ? (uploadedImage.id || uploadedImage.fileName || "") : "",
    imageFileName: formValues.imageFileName || "",
    name: String(formValues.name || "").trim(),
    symbol: String(formValues.symbol || "").trim(),
    description: String(formValues.description || "").trim(),
    website: String(formValues.website || "").trim(),
    twitter: String(formValues.twitter || "").trim(),
    telegram: String(formValues.telegram || "").trim(),
  });
}

function canPreuploadMetadata(formValues = readForm()) {
  return Boolean(
    formValues.imageFileName
    && String(formValues.name || "").trim()
    && String(formValues.symbol || "").trim()
  );
}

function hasFreshPreuploadedMetadata(formValues = readForm()) {
  if (!canPreuploadMetadata(formValues) || !metadataUri.value) return false;
  return metadataUploadState.completedFingerprint === metadataFingerprintFromForm(formValues);
}

function clearMetadataUploadDebounce() {
  if (!metadataUploadState.debounceTimer) return;
  clearTimeout(metadataUploadState.debounceTimer);
  metadataUploadState.debounceTimer = null;
}

function clearMetadataUploadCache({ clearInput = false } = {}) {
  clearMetadataUploadDebounce();
  metadataUploadState.completedFingerprint = "";
  metadataUploadState.latestScheduledFingerprint = "";
  metadataUploadState.lastCanPreupload = false;
  metadataUploadState.autoRetryFailures = 0;
  metadataUploadState.autoRetryDisabled = false;
  if (clearInput && metadataUri) {
    metadataUri.value = "";
  }
}

function markMetadataUploadDirty() {
  const formValues = readForm();
  if (hasFreshPreuploadedMetadata(formValues)) return;
  metadataUploadState.completedFingerprint = "";
  metadataUploadState.autoRetryFailures = 0;
  metadataUploadState.autoRetryDisabled = false;
  if (metadataUri) {
    metadataUri.value = "";
  }
}

function currentMetadataRetryDelayMs() {
  return metadataUploadState.autoRetryFailures >= 2
    ? METADATA_PREUPLOAD_DEBOUNCE_MS * 2
    : METADATA_PREUPLOAD_DEBOUNCE_MS;
}

async function uploadMetadataForCurrentForm(source = "background") {
  const formValues = readForm();
  if (!canPreuploadMetadata(formValues)) {
    if (source === "send") {
      throw new Error("Select an image and fill in both name and ticker before deploying.");
    }
    return "";
  }
  const fingerprint = metadataFingerprintFromForm(formValues);
  if (hasFreshPreuploadedMetadata(formValues)) {
    return metadataUri.value;
  }
  if (metadataUploadState.inFlightPromise) {
    if (metadataUploadState.inFlightFingerprint === fingerprint) {
      return metadataUploadState.inFlightPromise;
    }
    metadataUploadState.staleWhileUploading = true;
    metadataUploadState.latestScheduledFingerprint = fingerprint;
    if (source !== "send") {
      await metadataUploadState.inFlightPromise.catch(() => "");
      if (hasFreshPreuploadedMetadata(readForm())) {
        return metadataUri.value;
      }
    }
  }

  metadataUploadState.inFlightFingerprint = fingerprint;
  metadataUploadState.latestScheduledFingerprint = fingerprint;
  imageStatus.textContent = source === "send" ? "Preparing metadata..." : "Uploading metadata...";
  imagePath.textContent = "";

  const request = fetch("/api/metadata/upload", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      form: formValues,
    }),
  })
    .then(async (response) => {
      const payload = await response.json();
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Metadata upload failed.");
      }
      const liveForm = readForm();
      const liveFingerprint = canPreuploadMetadata(liveForm) ? metadataFingerprintFromForm(liveForm) : "";
      if (liveFingerprint === fingerprint) {
        metadataUri.value = payload.metadataUri || "";
        metadataUploadState.completedFingerprint = fingerprint;
        metadataUploadState.autoRetryFailures = 0;
        metadataUploadState.autoRetryDisabled = false;
        imageStatus.textContent = "Metadata ready.";
      } else {
        metadataUploadState.staleWhileUploading = true;
      }
      return payload.metadataUri || "";
    })
    .catch((error) => {
      if (source === "background") {
        metadataUploadState.autoRetryFailures += 1;
        metadataUploadState.autoRetryDisabled = metadataUploadState.autoRetryFailures >= 4;
        if (metadataUploadState.autoRetryDisabled) {
          imageStatus.textContent = `${error.message} Auto retry paused until deploy.`;
        } else {
          const nextDelayMs = currentMetadataRetryDelayMs();
          imageStatus.textContent = `${error.message} Retrying in ${nextDelayMs}ms.`;
        }
      }
      throw error;
    })
    .finally(() => {
      if (metadataUploadState.inFlightPromise === request) {
        metadataUploadState.inFlightPromise = null;
        metadataUploadState.inFlightFingerprint = "";
      }
      if (
        metadataUploadState.staleWhileUploading
        && metadataUploadState.latestScheduledFingerprint
        && metadataUploadState.latestScheduledFingerprint !== metadataUploadState.completedFingerprint
      ) {
        metadataUploadState.staleWhileUploading = false;
        scheduleMetadataPreupload({ immediate: true });
      } else {
        metadataUploadState.staleWhileUploading = false;
      }
    });

  metadataUploadState.inFlightPromise = request;
  return request;
}

function scheduleMetadataPreupload({ immediate = false } = {}) {
  clearMetadataUploadDebounce();
  if (!uploadedImage) return;
  const formValues = readForm();
  if (!canPreuploadMetadata(formValues)) {
    metadataUploadState.lastCanPreupload = false;
    markMetadataUploadDirty();
    imageStatus.textContent = "Waiting for name and ticker to pre-upload metadata.";
    imagePath.textContent = "";
    return;
  }
  const becameReady = !metadataUploadState.lastCanPreupload;
  metadataUploadState.lastCanPreupload = true;
  const fingerprint = metadataFingerprintFromForm(formValues);
  metadataUploadState.latestScheduledFingerprint = fingerprint;
  if (hasFreshPreuploadedMetadata(formValues)) return;
  if (metadataUploadState.inFlightPromise && metadataUploadState.inFlightFingerprint === fingerprint) {
    return;
  }
  if (metadataUploadState.autoRetryDisabled) {
    imageStatus.textContent = "Metadata auto retry paused until deploy.";
    imagePath.textContent = "";
    return;
  }
  const delayMs = immediate || becameReady ? 0 : currentMetadataRetryDelayMs();
  metadataUploadState.debounceTimer = setTimeout(() => {
    metadataUploadState.debounceTimer = null;
    uploadMetadataForCurrentForm("background").catch(() => {});
  }, delayMs);
}

async function ensureMetadataReadyForAction(action) {
  const formValues = readForm();
  if (!formValues.imageFileName) return;
  if (hasFreshPreuploadedMetadata(formValues)) return;
  if (canPreuploadMetadata(formValues)) {
    await uploadMetadataForCurrentForm(action === "send" ? "send" : "action");
    return;
  }
  if (metadataUploadState.inFlightPromise) {
    await metadataUploadState.inFlightPromise.catch(() => "");
    if (hasFreshPreuploadedMetadata(readForm())) {
      return;
    }
  }
  throw new Error(
    action === "send"
      ? "Select an image and fill in both name and ticker before deploying."
      : `Select an image and fill in both name and ticker before ${action}.`,
  );
}

function renderWalletOptions(wallets, selectedKey, balanceSol) {
  walletSelect.innerHTML = "";
  if (!wallets || wallets.length === 0) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = "No wallets found";
    walletSelect.appendChild(option);
    walletSelect.disabled = true;
    renderWalletDropdownList([], "");
    renderWalletSummary();
    return;
  }

  walletSelect.disabled = false;
  wallets.forEach((wallet) => {
    const option = document.createElement("option");
    option.value = wallet.envKey;
    const bal = wallet.envKey === selectedKey ? balanceSol : null;
    option.textContent = walletLabel(wallet, bal);
    if (wallet.envKey === selectedKey) {
      option.selected = true;
    }
    walletSelect.appendChild(option);
  });
  renderWalletDropdownList(wallets, selectedKey);
  renderWalletSummary();
}

function applySelectedWalletLocally(nextKey) {
  if (!latestWalletStatus || !Array.isArray(latestWalletStatus.wallets)) return;
  const selectedWallet = latestWalletStatus.wallets.find((wallet) => wallet.envKey === nextKey) || null;
  latestWalletStatus = {
    ...latestWalletStatus,
    selectedWalletKey: nextKey,
    connected: Boolean(selectedWallet && selectedWallet.publicKey),
    wallet: selectedWallet && selectedWallet.publicKey ? selectedWallet.publicKey : null,
    balanceLamports: selectedWallet && selectedWallet.balanceLamports != null ? selectedWallet.balanceLamports : null,
    balanceSol: selectedWallet && selectedWallet.balanceSol != null ? selectedWallet.balanceSol : null,
    usd1Balance: selectedWallet && selectedWallet.usd1Balance != null ? selectedWallet.usd1Balance : null,
  };
  if (walletSelect) walletSelect.value = nextKey;
  renderWalletOptions(latestWalletStatus.wallets, nextKey, latestWalletStatus.balanceSol);
  renderSniperUI();
  if (!selectedWallet || !selectedWallet.publicKey) {
    if (walletBalance) walletBalance.textContent = "-";
    metaNode.textContent = selectedWallet && selectedWallet.error ? selectedWallet.error : "Wallet unavailable";
    updateLockedModeFields();
    return;
  }
  if (walletBalance) {
    walletBalance.textContent = latestWalletStatus.balanceSol == null
      ? "--"
      : `${Number(latestWalletStatus.balanceSol).toFixed(4)} SOL`;
  }
  metaNode.textContent = `Using ${walletLabel(selectedWallet)}`;
  updateLockedModeFields();
}

async function refreshWalletStatus(preserveSelection = true) {
  try {
    const wallet = preserveSelection ? selectedWalletKey() : "";
    const url = wallet ? `/api/wallet-status?wallet=${encodeURIComponent(wallet)}` : "/api/wallet-status";
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("wallet-status", url, {}, requestStates.walletStatus)
      : null;
    if (result && result.aborted) return;
    const response = result ? result.response : await fetch(url);
    const payload = result ? result.payload : await response.json();
    if (result && !result.isLatest) return;
    walletStatusRequestSerial += 1;
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to load wallet status.");
    }
    applyWalletStatusPayload(payload);
  } catch (error) {
    if (walletBalance && !latestWalletStatus) walletBalance.textContent = "-";
    metaNode.textContent = error.message;
  }
}

function applyBootstrapFastPayload(payload) {
  latestWalletStatus = {
    ...(latestWalletStatus || {}),
    selectedWalletKey: payload.selectedWalletKey || "",
    wallets: Array.isArray(payload.wallets) ? payload.wallets : [],
    wallet: payload.wallet || null,
    connected: Boolean(payload.connected),
    balanceLamports: payload.balanceLamports == null ? null : payload.balanceLamports,
    balanceSol: payload.balanceSol == null ? null : payload.balanceSol,
    usd1Balance: payload.usd1Balance == null ? null : payload.usd1Balance,
    config: payload.config,
    regionRouting: payload.regionRouting || null,
    providers: payload.providers || {},
    launchpads: payload.launchpads || {},
  };
  renderWalletOptions(latestWalletStatus.wallets || [], latestWalletStatus.selectedWalletKey || "", latestWalletStatus.balanceSol);
  applyPersistentDefaults(payload.config);
  applyProviderAvailability(payload.providers || {});
  applyLaunchpadAvailability(payload.launchpads || {});
  renderQuickDevBuyButtons(payload.config);
  populateDevBuyPresetEditor(payload.config);
  updateQuote().catch(() => {});
  renderSniperUI();
  renderBackendRegionSummary(payload.regionRouting);
  markBootstrapState({
    staticLoaded: true,
    configLoaded: Boolean(payload.config),
  });
  setSettingsLoadingState(!hasBootstrapConfig());
  schedulePopoutAutosize();
}

function applyRuntimeStatusPayload(payload) {
  latestRuntimeStatus = payload;
  markBootstrapState({ runtimeLoaded: true });
}

function applyWalletStatusPayload(payload) {
  latestWalletStatus = {
    ...(latestWalletStatus || {}),
    ...payload,
    config: payload.config || (latestWalletStatus && latestWalletStatus.config) || null,
    regionRouting: payload.regionRouting || (latestWalletStatus && latestWalletStatus.regionRouting) || null,
    providers: payload.providers || (latestWalletStatus && latestWalletStatus.providers) || {},
    launchpads: payload.launchpads || (latestWalletStatus && latestWalletStatus.launchpads) || {},
  };
  const wallets = latestWalletStatus.wallets || [];
  const selectedWalletKeyValue = latestWalletStatus.selectedWalletKey || "";
  renderWalletOptions(wallets, selectedWalletKeyValue, latestWalletStatus.balanceSol);
  renderSniperUI();
  markBootstrapState({ walletsLoaded: true });
  if (!latestWalletStatus.connected) {
    if (walletBalance) walletBalance.textContent = "-";
    metaNode.textContent = "No wallet configured. Add SOLANA_PRIVATE_KEY to .env.";
    updateLockedModeFields();
    schedulePopoutAutosize();
    return;
  }

  if (walletBalance) {
    walletBalance.textContent = latestWalletStatus.balanceSol == null
      ? "--"
      : `${Number(latestWalletStatus.balanceSol).toFixed(4)} SOL`;
  }
  const selectedWallet = wallets.find((walletEntry) => walletEntry.envKey === selectedWalletKeyValue);
  metaNode.textContent = selectedWallet ? `Using ${walletLabel(selectedWallet)}` : "Wallet ready";
  updateLockedModeFields();
  schedulePopoutAutosize();
}

async function bootstrapApp() {
  markBootstrapState({ started: true });
  setSettingsLoadingState(true);
  const storedWalletKey = getStoredSelectedWalletKey();
  const bootstrapUrl = storedWalletKey
    ? `/api/bootstrap-fast?wallet=${encodeURIComponent(storedWalletKey)}`
    : "/api/bootstrap-fast";
  const result = RequestUtils.fetchJsonLatest
    ? await RequestUtils.fetchJsonLatest("bootstrap-fast", bootstrapUrl, {}, requestStates.bootstrap)
    : null;
  if (result && result.aborted) return;
  const response = result ? result.response : await fetch(bootstrapUrl);
  const payload = result ? result.payload : await response.json();
  if (result && !result.isLatest) return;
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to load app bootstrap.");
  }
  applyBootstrapFastPayload(payload);
  refreshWalletStatus(true).catch(() => {});
  refreshRuntimeStatus().catch(() => {});
  fetch("/api/lookup-tables/warm", { method: "POST" }).catch(() => {});
  fetch("/api/pump-global/warm", { method: "POST" }).catch(() => {});
}

async function refreshRuntimeStatus() {
  try {
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("runtime-status", "/api/runtime-status", {}, requestStates.runtimeStatus)
      : null;
    if (result && result.aborted) return;
    const response = result ? result.response : await fetch("/api/runtime-status");
    const payload = result ? result.payload : await response.json();
    if (result && !result.isLatest) return;
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to load runtime status.");
    }
    applyRuntimeStatusPayload(payload);
  } catch (_error) {
    // Keep runtime hydration best-effort so boot remains responsive.
  }
}

async function updateQuote() {
  const buyAmount = getNamedValue("devBuyAmount").trim();
  if (!buyAmount) {
    quoteOutput.textContent = "No dev buy selected.";
    if (!syncingDevBuyInputs) {
      syncingDevBuyInputs = true;
      if (devBuyPercentInput) devBuyPercentInput.value = "";
      if (lastDevBuyEditSource !== "percent" && devBuySolInput) devBuySolInput.value = "";
      syncingDevBuyInputs = false;
    }
    return;
  }

  try {
    const mode = getDevBuyMode();
    const url = `/api/quote?mode=${encodeURIComponent(mode)}&amount=${encodeURIComponent(buyAmount)}`;
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("quote", url, {}, requestStates.quote)
      : null;
    if (result && result.aborted) return;
    const response = result ? result.response : await fetch(url);
    const payload = result ? result.payload : await response.json();
    if (result && !result.isLatest) return;
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Quote failed.");
    }
    if (!payload.quote) {
      quoteOutput.textContent = "Enter a valid dev buy amount.";
      return;
    }
    syncingDevBuyInputs = true;
    if (mode === "sol") {
      if (devBuyPercentInput) devBuyPercentInput.value = payload.quote.estimatedSupplyPercent;
    } else {
      if (devBuySolInput) devBuySolInput.value = payload.quote.estimatedSol;
      if (devBuyPercentInput) devBuyPercentInput.value = payload.quote.estimatedSupplyPercent;
    }
    syncingDevBuyInputs = false;
    quoteOutput.textContent =
      mode === "sol"
        ? `Estimated tokens out: ${payload.quote.estimatedTokens} (${payload.quote.estimatedSupplyPercent}% supply)`
        : `Estimated SOL required: ${payload.quote.estimatedSol} for ${payload.quote.estimatedSupplyPercent}% supply`;
  } catch (error) {
    quoteOutput.textContent = error.message;
  }
}

function queueQuoteUpdate() {
  if (RequestUtils.scheduleDebounced) {
    RequestUtils.scheduleDebounced(requestStates.quote, 250, () => {
      updateQuote().catch((error) => {
        quoteOutput.textContent = error.message;
      });
    });
    return;
  }
  clearTimeout(quoteTimer);
  quoteTimer = setTimeout(updateQuote, 250);
}

async function uploadSelectedImage(file) {
  const formData = new FormData();
  formData.append("file", file, file.name);
  const response = await fetch("/api/upload-image", {
    method: "POST",
    body: formData,
  });

  const payload = await response.json();
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Image upload failed.");
  }

  imageStatus.textContent = "Image uploaded to library.";
  imagePath.textContent = "";
  imageLibraryState.activeImageId = payload.id || "";
  try {
    await fetchImageLibrary();
  } catch (error) {
    imageStatus.textContent = error.message;
  }
  showImageDetailsModal(payload, { isNewUpload: true });
}

async function applyTestPreset() {
  form.querySelector('[name="name"]').value = TEST_PRESET.name;
  tickerManuallyEdited = false;
  syncTickerFromName();
  form.querySelector('[name="description"]').value = TEST_PRESET.description;
  form.querySelector('[name="website"]').value = TEST_PRESET.website;
  form.querySelector('[name="twitter"]').value = TEST_PRESET.twitter;
  form.querySelector('[name="telegram"]').value = TEST_PRESET.telegram;
  setDevBuyHiddenState(TEST_PRESET.devBuyMode, TEST_PRESET.devBuyAmount);
  syncingDevBuyInputs = true;
  if (devBuySolInput) devBuySolInput.value = TEST_PRESET.devBuyMode === "sol" ? TEST_PRESET.devBuyAmount : "";
  if (devBuyPercentInput) devBuyPercentInput.value = "";
  syncingDevBuyInputs = false;

  clearValidationErrors();
  Object.keys(fieldValidators).forEach((name) => setFieldError(name, ""));
  updateTokenFieldCounts();
  updateJitoVisibility();
  queueQuoteUpdate();

  if (!uploadedImage) {
    try {
      imageStatus.textContent = "Uploading image...";
      imagePath.textContent = "";
      clearMetadataUploadCache({ clearInput: true });
      const response = await fetch("/solana-mark.png");
      if (!response.ok) {
        throw new Error("Failed to load test image.");
      }
      const blob = await response.blob();
      const file = new File([blob], "solana-mark.png", { type: blob.type || "image/png" });
      try {
        const dataTransfer = new DataTransfer();
        dataTransfer.items.add(file);
        imageInput.files = dataTransfer.files;
      } catch (_error) {
        // Some browsers restrict programmatic file input assignment.
      }
      setImagePreview(URL.createObjectURL(file));
      await uploadSelectedImage(file);
    } catch (error) {
      imageStatus.textContent = error.message;
      imagePath.textContent = "";
    }
  }
}

const fieldValidators = {
  website(v) {
    if (!v) return "";
    if (!/^https?:\/\/.+/i.test(v)) return "Must start with https://";
    return "";
  },
  twitter(v) {
    if (!v) return "";
    if (/^https?:\/\/(x\.com|twitter\.com)\/.+/i.test(v)) return "";
    if (/^@?[\w]{1,15}$/.test(v)) return "";
    return "Enter a valid URL (https://x.com/...) or @username";
  },
  telegram(v) {
    if (!v) return "";
    if (/^https?:\/\/t\.me\/.+/i.test(v)) return "";
    return "Enter a valid Telegram link (https://t.me/username)";
  },
  devBuyAmount(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n <= 0) return "Must be a positive number";
    return "";
  },
  creationTipSol(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  creationPriorityFeeSol(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  buyPriorityFeeSol(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  buyTipSol(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  buySlippagePercent(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  sellPriorityFeeSol(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  sellTipSol(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  sellSlippagePercent(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  automaticDevSellPercent(v) {
    if (!isNamedChecked("automaticDevSellEnabled")) return "";
    const n = Number(v);
    if (isNaN(n) || n <= 0 || n > 100) return "Must be between 1 and 100";
    return "";
  },
  automaticDevSellDelayMs(v) {
    if (!isNamedChecked("automaticDevSellEnabled") || getAutoSellTriggerMode() !== "submit-delay") return "";
    const n = Number(v);
    if (isNaN(n) || n < 0 || n > 1500) return "Must be between 0 and 1500";
    return "";
  },
  automaticDevSellBlockOffset(v) {
    if (!isNamedChecked("automaticDevSellEnabled") || getAutoSellTriggerMode() !== "block-offset") return "";
    const n = Number(v);
    if (isNaN(n) || n < 0 || n > 5) return "Must be between 0 and 5";
    return "";
  },
  agentUnlockedBuybackPercent(v) {
    if (getMode() !== "agent-unlocked") return "";
    if (!v) return "Buyback % is required";
    const n = Number(v);
    if (isNaN(n) || n < 0 || n > 100) return "Must be between 0 and 100";
    return "";
  },
};

function setFieldError(name, msg) {
  const errorEl = document.querySelector(`.field-error[data-error-for="${name}"]`);
  const input = getNamedInput(name);
  if (errorEl) errorEl.textContent = msg || "";
  if (input) input.classList.toggle("input-error", !!msg);
}

function validateFieldByName(name) {
  const input = getNamedInput(name);
  if (!input || !fieldValidators[name]) return "";
  const msg = fieldValidators[name](input.value.trim());
  setFieldError(name, msg);
  return msg;
}

function validateAllInlineFields() {
  const errors = [];
  for (const name of Object.keys(fieldValidators)) {
    const msg = validateFieldByName(name);
    if (msg) errors.push(msg);
  }
  return errors;
}

function validateAgentSplit() {
  const errors = [];
  const recipients = collectAgentSplitRecipients();
  if (getMode() !== "agent-custom") return errors;

  if (recipients.length === 0) {
    errors.push("Agent fee split is required.");
    return errors;
  }
  if (recipients.length > MAX_FEE_SPLIT_RECIPIENTS) {
    errors.push(`Agent custom fee split supports at most ${MAX_FEE_SPLIT_RECIPIENTS} recipients.`);
  }

  const total = recipients.reduce((sum, entry) => sum + (Number(entry.shareBps) || 0), 0);
  const agentRows = recipients.filter((entry) => entry.type === "agent");
  if (agentRows.length !== 1) {
    errors.push("Agent custom mode requires exactly one agent buyback row.");
  }
  if (total !== 10_000) {
    errors.push("Agent custom fee split must total 100%.");
  }

  recipients.forEach((entry, index) => {
    if (!Number.isFinite(entry.shareBps) || entry.shareBps < 0) {
      errors.push(`Agent split recipient ${index + 1} has an invalid %.`);
      return;
    }
    if (entry.type === "agent") return;
    if (entry.type === "wallet" && !entry.address) {
      errors.push(`Agent split recipient ${index + 1} is missing a wallet address.`);
    }
    if (entry.type === "github" && !entry.githubUsername) {
      errors.push(`Agent split recipient ${index + 1} is missing a GitHub username.`);
    }
  });

  return errors;
}

function validateFeeSplit() {
  const errors = [];
  if (getMode() !== "regular" || !feeSplitEnabled.checked) return errors;
  const recipients = collectFeeSplitRecipients();
  if (recipients.length > MAX_FEE_SPLIT_RECIPIENTS) {
    errors.push(`Fee split supports at most ${MAX_FEE_SPLIT_RECIPIENTS} recipients.`);
  }
  return errors;
}

function validateForm() {
  const errors = [];
  const f = readForm();
  if (!f.name.trim()) errors.push("Token name is required.");
  if (!f.symbol.trim()) errors.push("Ticker is required.");
  if (!uploadedImage) errors.push("Token image is required.");
  if (!latestWalletStatus || !latestWalletStatus.connected) errors.push("No wallet connected.");
  if (f.automaticDevSellEnabled && !f.devBuyAmount) errors.push("Dev auto-sell requires a dev buy amount.");
  validateSniperState().forEach((msg) => errors.push(msg));
  const inlineErrors = validateAllInlineFields();
  inlineErrors.forEach((msg) => errors.push(msg));
  validateFeeSplit().forEach((msg) => errors.push(msg));
  validateAgentSplit().forEach((msg) => errors.push(msg));
  return errors;
}

function showValidationErrors(errors) {
  let container = document.getElementById("validation-errors");
  if (errors.length === 0) {
    if (container) container.remove();
    return false;
  }
  if (!container) {
    container = document.createElement("div");
    container.id = "validation-errors";
    container.className = "validation-errors";
    const outputAnchor = outputSection || document.querySelector(".output");
    outputAnchor.parentNode.insertBefore(container, outputAnchor);
  }
  container.innerHTML = errors.map((e) => `<span>${escapeHTML(e)}</span>`).join("");
  container.scrollIntoView({ behavior: "smooth", block: "nearest" });
  return true;
}

function clearValidationErrors() {
  const container = document.getElementById("validation-errors");
  if (container) container.remove();
}

function buildDeployPreviewHTML() {
  const f = readForm();
  const walletAddr = latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "Unknown";
  const bal = latestWalletStatus && latestWalletStatus.balanceSol ? `${Number(latestWalletStatus.balanceSol).toFixed(4)} SOL` : "-";

  const imgSrc = imagePreview.src && !imagePreview.hidden ? imagePreview.src : "";
  const tokenImgHTML = imgSrc
    ? `<img class="modal-token-img" src="${imgSrc}" alt="">`
    : `<div class="modal-token-img-empty">No img</div>`;

  const devBuyText = f.devBuyAmount
    ? `${f.devBuyAmount} ${f.devBuyMode === "tokens" ? "tokens" : "SOL"}`
    : "None";

  const quoteText = quoteOutput.textContent || "";

  const modeLabels = {
    regular: "Regular",
    cashback: "Cashback",
    "agent-custom": "Agent Custom",
    "agent-unlocked": "Agent Unlocked",
    "agent-locked": "Agent Locked",
  };

  let feesText = "Default (deployer)";
  if (f.mode === "cashback") feesText = "Cashback to traders";
  else if (f.mode === "agent-locked") feesText = "Agent escrow (locked)";
  else if (f.mode === "agent-custom") {
    const parts = (f.agentSplitRecipients || []).map((entry) => {
      const share = (Number(entry.shareBps || 0) / 100).toFixed(2).replace(/\.00$/, "");
      if (entry.type === "agent") return `Agent ${share}%`;
      if (entry.type === "github") return `@${entry.githubUsername} ${share}%`;
      return `${shortenAddress(entry.address || "wallet", 4)} ${share}%`;
    });
    feesText = parts.length > 0 ? parts.join(" | ") : "Agent split";
  } else if (f.mode === "agent-unlocked") {
    feesText = "Untouched on launch; configure once later";
  } else if (f.feeSplitEnabled) {
    feesText = `Fee split (${f.feeSplitRecipients.length} recipients)`;
  }

  let txSettingsText = usesBundledJito()
    ? "Priority: custom"
    : `Priority: ${f.priorityFeeSol || "off"}`;
  if (f.jitoTipSol) txSettingsText += ` | Tip: ${f.jitoTipSol || "default"} SOL`;
  if (f.skipPreflight) txSettingsText += " | Preflight off";

  const buybackText = f.mode === "agent-locked" ? "100%"
    : (f.mode === "agent-custom" || f.mode === "agent-unlocked") ? `${f.buybackPercent || "50"}%`
    : "-";
  const sniperText = f.sniperEnabled
    ? (f.sniperWallets.length
      ? f.sniperWallets.map((entry) => `#${walletIndexFromEnvKey(entry.envKey)} ${entry.amountSol} SOL @ ${entry.targetBlockOffset != null ? `b${entry.targetBlockOffset}` : (entry.submitWithLaunch ? "same-time" : getSniperTriggerSummary(entry).toLowerCase())}`).join(" | ")
      : "Enabled")
    : "Off";
  const vanityText = f.vanityPrivateKey ? "Custom vanity key attached" : "Off";

  const rows = [
    { label: "Wallet", value: walletAddr, cls: "" },
    { label: "Balance", value: bal, cls: "green" },
    { label: "Preset", value: f.activePresetId || DEFAULT_PRESET_ID, cls: "secondary" },
    { label: "Platform", value: f.launchpad || "pump", cls: "" },
    {
      label: "Creation",
      value: `${PROVIDER_LABELS[f.provider || "helius-sender"] || (f.provider || "helius-sender")}`,
      cls: "",
    },
    {
      label: "Buy Route",
      value: `${PROVIDER_LABELS[f.buyProvider || "helius-sender"] || (f.buyProvider || "helius-sender")} | slip ${f.buySlippagePercent || "90"}%`,
      cls: "secondary",
    },
    {
      label: "Sell Route",
      value: `${PROVIDER_LABELS[f.sellProvider || "helius-sender"] || (f.sellProvider || "helius-sender")} | slip ${f.sellSlippagePercent || "90"}%`,
      cls: "secondary",
    },
    { label: "Mode", value: `${modeLabels[f.mode] || f.mode}${f.mayhemMode ? " + Mayhem" : ""}`, cls: "" },
    ...(f.mode.startsWith("agent") ? [{ label: "Buyback", value: buybackText, cls: "" }] : []),
    { label: "Fees", value: feesText, cls: "secondary" },
    { label: "Dev Buy", value: devBuyText, cls: "" },
    ...(f.automaticDevSellEnabled ? [{
      label: "Dev Auto Sell",
      value: getAutoSellSummaryText(f),
      cls: "secondary",
    }] : []),
    { label: "Sniper", value: sniperText, cls: "secondary" },
    { label: "Vanity", value: vanityText, cls: "secondary" },
    ...(quoteText && f.devBuyAmount ? [{ label: "Estimate", value: quoteText, cls: "secondary" }] : []),
    { label: "Tx Settings", value: txSettingsText, cls: "secondary" },
  ];

  return `
    <div class="modal-token-header">
      ${tokenImgHTML}
      <div class="modal-token-info">
        <div class="modal-token-name">${escapeHTML(f.name || "Unnamed")}</div>
        <div class="modal-token-ticker">$${escapeHTML(f.symbol || "???")}</div>
      </div>
    </div>
    ${rows.map((r) => `
      <div class="modal-preview-row">
        <div class="modal-preview-label">${r.label}</div>
        <div class="modal-preview-value ${r.cls}">${escapeHTML(r.value)}</div>
      </div>
    `).join("")}
    <div class="modal-warning">This will broadcast a real transaction on Solana mainnet.</div>
  `;
}

function escapeHTML(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

function showDeployModal() {
  modalBody.innerHTML = buildDeployPreviewHTML();
  deployModal.hidden = false;
}

function hideDeployModal() {
  deployModal.hidden = true;
}

async function run(action) {
  if (!ensureInteractiveBootstrapReady()) return;
  const actualAction = action === "deploy" ? "send" : action;
  const label = action === "deploy" ? "Deploying..." : action === "simulate" ? "Simulating..." : "Building...";
  const clientActionStartedAt = performance.now();
  setBusy(true, label);
  output.textContent = "Working...";

  try {
    await new Promise((resolve) => requestAnimationFrame(() => resolve()));
    await ensureMetadataReadyForAction(actualAction);
    const formPayload = readForm();
    const clientPreRequestMs = Math.max(0, Math.round(performance.now() - clientActionStartedAt));
    const response = await fetch("/api/run", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        action: actualAction,
        form: formPayload,
        clientPreRequestMs,
      }),
    });
    const payload = await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Request failed.");
    }

    statusNode.textContent = action === "deploy" ? "Deployed" : action === "simulate" ? "Simulated" : "Built";
    const wallet = latestWalletStatus && latestWalletStatus.selectedWalletKey
      ? `Using #${walletIndexFromEnvKey(latestWalletStatus.selectedWalletKey)}`
      : "Wallet ready";
    const followMeta = payload.report && payload.report.followDaemon && payload.report.followDaemon.enabled
      ? ` | Follow: ${payload.report.followDaemon.job && payload.report.followDaemon.job.state ? payload.report.followDaemon.job.state : "armed"}`
      : "";
    metaNode.textContent = `${wallet} | ${payload.report.launchpad || "pump"} | ${payload.report.execution.resolvedProvider || payload.report.execution.provider || "helius-sender"} | Mint: ${shortAddress(payload.report.mint)}${followMeta}`;
    metadataUri.value = payload.metadataUri || "";
    if (payload.metadataUri) {
      metadataUploadState.completedFingerprint = metadataFingerprintFromForm(readForm());
    }
    output.textContent = payload.text;
    setBusy(false, statusNode.textContent || "Idle");
    if (payload.sendLogPath) {
      refreshReportsTerminal({
        preserveSelection: false,
        preferId: extractReportIdFromPath(payload.sendLogPath),
      }).catch((error) => {
        if (reportsTerminalOutput && reportsTerminalSection && !reportsTerminalSection.hidden) {
          reportsTerminalState.activePayload = null;
          reportsTerminalState.activeText = error.message || "Failed to refresh reports.";
          renderReportsTerminalOutput();
        }
      });
    }
    refreshWalletStatus(true).catch(() => {});
  } catch (error) {
    statusNode.textContent = "Error";
    output.textContent = error.message;
  } finally {
    if (buttons.some((button) => button.disabled)) {
      setBusy(false, statusNode.textContent || "Idle");
    }
  }
}

function buildSavedConfigFromForm() {
  const current = cloneConfig(getConfig());
  const base = current || createFallbackConfig();
  const f = readForm();

  base.defaults = {
    ...(base.defaults || {}),
    launchpad: f.launchpad || "pump",
    mode: f.mode || "regular",
    activePresetId: f.activePresetId || DEFAULT_PRESET_ID,
    presetEditing: isPresetEditing(base),
    misc: {
      ...(base.defaults && base.defaults.misc ? base.defaults.misc : {}),
      sniperDraft: normalizeSniperDraftState(sniperFeature.getState()),
      feeSplitDraft: normalizeFeeSplitDraft(serializeFeeSplitDraft()),
      agentSplitDraft: normalizeAgentSplitDraft(serializeAgentSplitDraft()),
    },
    automaticDevSell: {
      enabled: Boolean(f.automaticDevSellEnabled),
      percent: Number(f.automaticDevSellPercent || 100),
      triggerMode: normalizeAutoSellTriggerMode(f.automaticDevSellTriggerMode),
      delayMs: Number(f.automaticDevSellDelayMs || 0),
      targetBlockOffset: Number(f.automaticDevSellBlockOffset || 0),
    },
  };

  base.presets = base.presets || {};
  base.presets.items = getPresetItems(base).map((preset) => preset.id === f.activePresetId
    ? {
        ...preset,
        creationSettings: {
          ...preset.creationSettings,
          provider: f.provider || "helius-sender",
          tipSol: f.creationTipSol || "",
          priorityFeeSol: f.priorityFeeSol || "",
          devBuySol: (preset.creationSettings && preset.creationSettings.devBuySol) || "",
        },
        buySettings: {
          ...preset.buySettings,
          provider: f.buyProvider || "helius-sender",
          priorityFeeSol: f.buyPriorityFeeSol || "",
          tipSol: f.buyTipSol || "",
          slippagePercent: f.buySlippagePercent || "",
        },
        sellSettings: {
          ...preset.sellSettings,
          provider: f.sellProvider || "helius-sender",
          priorityFeeSol: f.sellPriorityFeeSol || "",
          tipSol: f.sellTipSol || "",
          slippagePercent: f.sellSlippagePercent || "",
        },
      }
    : preset);

  return base;
}

async function saveSettings() {
  if (!hasBootstrapConfig()) {
    statusNode.textContent = "Loading";
    metaNode.textContent = "Settings are still loading from the backend.";
    return;
  }
  setBusy(true, "Saving defaults...");
  try {
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("settings-save", "/api/settings", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          config: buildSavedConfigFromForm(),
        }),
      })
      : null;
    const response = result ? result.response : await fetch("/api/settings", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        config: buildSavedConfigFromForm(),
      }),
    });
    const payload = result ? result.payload : await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to save settings.");
    }
    statusNode.textContent = "Defaults saved";
    setRegionRouting(payload.regionRouting || (latestWalletStatus && latestWalletStatus.regionRouting));
    setConfig(payload.config);
    metaNode.textContent = "Launch defaults and selected presets saved.";
    renderQuickDevBuyButtons(payload.config);
    populateDevBuyPresetEditor(payload.config);
    renderBackendRegionSummary(payload.regionRouting);
    hideSettingsModal();
  } catch (error) {
    statusNode.textContent = "Error";
    output.textContent = error.message;
  } finally {
    setBusy(false, statusNode.textContent || "Ready");
  }
}

function showSettingsModal() {
  if (!hasBootstrapConfig()) {
    setSettingsLoadingState(true);
    renderBackendRegionSummary(null);
    if (settingsModal) settingsModal.hidden = false;
    return;
  }
  setSettingsLoadingState(false);
  renderPresetChips();
  applyPresetToSettingsInputs(getActivePreset(getConfig()), { syncToMainForm: false });
  setPresetEditing(isPresetEditing(getConfig()));
  renderBackendRegionSummary();
  if (settingsModal) settingsModal.hidden = false;
}

function hideSettingsModal() {
  if (settingsModal) settingsModal.hidden = true;
}

function getStoredOutputSectionVisible() {
  try {
    const stored = window.localStorage.getItem(OUTPUT_SECTION_VISIBILITY_KEY);
    if (stored === "true") return true;
    if (stored === "false") return false;
  } catch (_error) {
    // Ignore storage access failures and fall back to default visible state.
  }
  return true;
}

function setOutputSectionVisible(isVisible) {
  document.documentElement.classList.toggle("output-hidden", !isVisible);
  document.body.classList.toggle("output-hidden", !isVisible);
  if (outputSection) outputSection.hidden = !isVisible;
  if (toggleOutputButton) {
    toggleOutputButton.classList.toggle("active", isVisible);
    toggleOutputButton.setAttribute("aria-pressed", String(isVisible));
  }
  try {
    window.localStorage.setItem(OUTPUT_SECTION_VISIBILITY_KEY, String(isVisible));
  } catch (_error) {
    // Ignore storage access failures and keep the UI functional.
  }
  schedulePopoutAutosize();
}

function getStoredReportsTerminalVisible() {
  try {
    const stored = window.localStorage.getItem(REPORTS_TERMINAL_VISIBILITY_KEY);
    if (stored === "true") return true;
    if (stored === "false") return false;
  } catch (_error) {
    // Ignore storage access failures and fall back to hidden state.
  }
  return false;
}

function getStoredImageLayoutCompact() {
  try {
    return window.localStorage.getItem(IMAGE_LAYOUT_COMPACT_STORAGE_KEY) === "true";
  } catch (_error) {
    return false;
  }
}

function setImageLayoutCompact(isCompact, { persist = true } = {}) {
  const compact = Boolean(isCompact);
  if (tokenSurfaceSection) {
    tokenSurfaceSection.classList.toggle("is-image-compact", compact);
  }
  if (imageLayoutToggle) {
    imageLayoutToggle.setAttribute("aria-pressed", String(compact));
    const label = compact ? "Expand image field" : "Compact image field";
    imageLayoutToggle.setAttribute("aria-label", label);
    imageLayoutToggle.setAttribute("title", label);
  }
  if (persist) {
    try {
      window.localStorage.setItem(IMAGE_LAYOUT_COMPACT_STORAGE_KEY, String(compact));
    } catch (_error) {
      // Ignore storage failures and keep the UI functional.
    }
  }
  schedulePopoutAutosize();
}

function getStoredReportsTerminalSort() {
  try {
    return window.localStorage.getItem(REPORTS_TERMINAL_SORT_KEY) === "oldest" ? "oldest" : "newest";
  } catch (_error) {
    return "newest";
  }
}

function clampReportsTerminalListWidth(width) {
  const numeric = Number(width);
  if (!Number.isFinite(numeric)) return REPORTS_TERMINAL_DEFAULT_LIST_WIDTH;
  return Math.min(REPORTS_TERMINAL_MAX_LIST_WIDTH, Math.max(REPORTS_TERMINAL_MIN_LIST_WIDTH, Math.round(numeric)));
}

function getCurrentReportsTerminalListWidth() {
  if (!reportsTerminalSection) return REPORTS_TERMINAL_DEFAULT_LIST_WIDTH;
  const inlineWidth = reportsTerminalSection.style.getPropertyValue("--reports-terminal-list-width");
  if (inlineWidth) {
    const parsedInlineWidth = Number.parseInt(inlineWidth, 10);
    if (Number.isFinite(parsedInlineWidth)) return clampReportsTerminalListWidth(parsedInlineWidth);
  }
  return REPORTS_TERMINAL_DEFAULT_LIST_WIDTH;
}

function describeReportEntry(entry) {
  const parts = [];
  if (entry.displayTime) parts.push(entry.displayTime);
  if (entry.provider) parts.push(entry.provider);
  if (entry.signatureCount) parts.push(`${entry.signatureCount} sig${entry.signatureCount === 1 ? "" : "s"}`);
  if (entry.followEnabled) {
    const followBits = [];
    if (entry.followState) followBits.push(`follow ${entry.followState}`);
    if (entry.followActionCount) followBits.push(`${entry.followConfirmedCount || 0}/${entry.followActionCount} done`);
    if (entry.followProblemCount) followBits.push(`${entry.followProblemCount} issue${entry.followProblemCount === 1 ? "" : "s"}`);
    if (followBits.length) parts.push(followBits.join(" | "));
  }
  return parts.join(" | ");
}

function normalizeReportsTerminalTab(tab) {
  const normalized = String(tab || "").trim().toLowerCase();
  return ["overview", "actions", "benchmarks", "raw"].includes(normalized) ? normalized : "overview";
}

function currentReportsTerminalEntry() {
  return reportsTerminalState.entries.find((entry) => entry.id === reportsTerminalState.activeId) || null;
}

function currentReportsTerminalPayload() {
  return reportsTerminalState.activePayload && typeof reportsTerminalState.activePayload === "object"
    ? reportsTerminalState.activePayload
    : null;
}

function currentReportsTerminalReport() {
  const payload = currentReportsTerminalPayload();
  return payload && payload.report && typeof payload.report === "object" ? payload.report : null;
}

function currentReportsTerminalFollowJob() {
  const report = currentReportsTerminalReport();
  return report && report.followDaemon && report.followDaemon.job && typeof report.followDaemon.job === "object"
    ? report.followDaemon.job
    : null;
}

function currentReportsTerminalFollowActions() {
  const job = currentReportsTerminalFollowJob();
  return job && Array.isArray(job.actions) ? job.actions : [];
}

function currentReportsTerminalBenchmark() {
  const report = currentReportsTerminalReport();
  return report && report.benchmark && typeof report.benchmark === "object" ? report.benchmark : null;
}

function currentReportsTerminalExecution() {
  const report = currentReportsTerminalReport();
  return report && report.execution && typeof report.execution === "object" ? report.execution : null;
}

function currentReportsTerminalTimingProfiles() {
  const report = currentReportsTerminalReport();
  return report
    && report.followDaemon
    && Array.isArray(report.followDaemon.timingProfiles)
    ? report.followDaemon.timingProfiles
    : [];
}

function formatReportMetric(value, suffix = "", fallback = "--", digits = 0) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return `${numeric.toFixed(digits)}${suffix}`;
}

function formatReportTimestamp(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return "--";
  try {
    return new Date(numeric).toLocaleTimeString();
  } catch (_error) {
    return String(value);
  }
}

function formatReportLatencyDelta(startMs, endMs) {
  const start = Number(startMs);
  const end = Number(endMs);
  if (!Number.isFinite(start) || !Number.isFinite(end) || end < start) return "--";
  return `${Math.round(end - start)}ms`;
}

function reportStateClass(state) {
  const normalized = String(state || "").trim().toLowerCase();
  if (["confirmed", "completed", "success", "healthy"].includes(normalized)) return "is-good";
  if (["running", "eligible", "armed", "queued", "sent"].includes(normalized)) return "is-warn";
  if (["failed", "cancelled", "expired", "completed-with-failures"].includes(normalized)) return "is-bad";
  return "";
}

function inferReportErrorCategory(message) {
  const normalized = String(message || "").toLowerCase();
  if (!normalized) return "";
  if (normalized.includes("os error 1224") || normalized.includes("user-mapped section")) return "local write race";
  if (normalized.includes("insufficient funds")) return "insufficient funds";
  if (normalized.includes("was not found") || normalized.includes("account")) return "account visibility";
  if (normalized.includes("custom") || normalized.includes("instructionerror")) return "on-chain failure";
  if (normalized.includes("timeout") || normalized.includes("too many requests") || normalized.includes("unavailable")) return "transport/rpc";
  return "action failure";
}

function describeFollowActionTrigger(action) {
  if (!action || typeof action !== "object") return "Immediate";
  if (action.requireConfirmation) return "After confirmation";
  if (action.targetBlockOffset != null) return `Block ${action.targetBlockOffset}`;
  if (Number(action.submitDelayMs || 0) > 0) return `Submit + ${action.submitDelayMs}ms`;
  if (Number(action.delayMs || 0) > 0) return `Delay ${action.delayMs}ms`;
  return "Immediate";
}

function describeFollowActionSize(action) {
  if (!action || typeof action !== "object") return "--";
  if (action.buyAmountSol) return `${action.buyAmountSol} SOL`;
  if (action.sellPercent != null) return `${action.sellPercent}%`;
  return "--";
}

function describeFollowActionWallet(action) {
  if (!action || !action.walletEnvKey) return "--";
  return `Wallet #${walletIndexFromEnvKey(action.walletEnvKey)}`;
}

function shortenReportEndpoint(endpoint) {
  const raw = String(endpoint || "").trim();
  if (!raw) return "--";
  try {
    const url = new URL(raw);
    const host = url.hostname.toLowerCase();
    if (host.includes("helius-rpc.com")) {
      if (host.startsWith("mainnet.")) return "Helius WS";
      if (host.includes("-sender.")) {
        const region = host.split("-sender.")[0];
        return `${region.toUpperCase()} sender`;
      }
      return "Helius";
    }
    if (host.includes("jito")) return host.replace(/^https?:\/\//, "");
    const label = host.replace(/^www\./, "");
    return label.length > 24 ? shortenAddress(label, 10) : label;
  } catch (_error) {
    return raw.length > 24 ? shortenAddress(raw, 10) : raw;
  }
}

function renderCopyableHash(value, label = "Copy hash") {
  const raw = String(value || "").trim();
  if (!raw) return "--";
  return `
    <button
      type="button"
      class="reports-copy-hash"
      data-copy-value="${escapeHTML(raw)}"
      title="${escapeHTML(label)}"
    >
      ${escapeHTML(shortenAddress(raw, 8))}
    </button>
  `;
}

function buildFollowActionMetricItems(action, followJob) {
  const isBuy = String(action && action.kind || "").toLowerCase() === "sniper-buy";
  const metrics = [
    { label: "Wallet", value: describeFollowActionWallet(action) },
    { label: "Trigger", value: describeFollowActionTrigger(action) },
    { label: "Size", value: describeFollowActionSize(action) },
    { label: "Start Block", value: action && action.sendObservedBlockHeight != null ? String(action.sendObservedBlockHeight) : isBuy && followJob && followJob.sendObservedBlockHeight != null ? `launch ${followJob.sendObservedBlockHeight}` : "--" },
    { label: "Confirm Block", value: action && action.confirmedObservedBlockHeight != null ? String(action.confirmedObservedBlockHeight) : "--" },
    { label: "Blocks", value: action && action.blocksToConfirm != null ? String(action.blocksToConfirm) : "--" },
    { label: "Endpoint", value: shortenReportEndpoint(action && action.endpoint) },
    { label: "Attempts", value: String(action && action.attemptCount != null ? action.attemptCount : 0) },
  ];
  if (!isBuy) {
    metrics.push(
      { label: "Scheduled", value: formatReportTimestamp(action && action.scheduledForMs) },
      { label: "Started", value: formatReportTimestamp(action && action.submitStartedAtMs) },
      { label: "Submitted", value: formatReportTimestamp(action && action.submittedAtMs) },
      { label: "Confirmed", value: formatReportTimestamp(action && action.confirmedAtMs) },
      { label: "Submit->Confirm", value: formatReportLatencyDelta(action && action.submittedAtMs, action && action.confirmedAtMs) },
    );
  } else {
    metrics.push(
      { label: "Launch->Submit", value: followJob ? formatReportLatencyDelta(followJob.submitAtMs, action && action.submittedAtMs) : "--" },
      { label: "Submit Time", value: formatReportTimestamp(action && action.submittedAtMs) },
    );
  }
  return metrics;
}

function renderReportMetricGrid(items = []) {
  const visible = items.filter((item) => item && item.value != null && item.value !== "");
  if (!visible.length) {
    return '<div class="reports-terminal-empty">No metrics available for this section.</div>';
  }
  return `
    <div class="reports-metric-grid">
      ${visible.map((item) => `
        <div class="reports-metric-card">
          <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
          <strong class="reports-metric-value">${escapeHTML(String(item.value))}</strong>
        </div>
      `).join("")}
    </div>
  `;
}

function buildReportsOverviewMarkup() {
  const entry = currentReportsTerminalEntry();
  const payload = currentReportsTerminalPayload();
  const report = currentReportsTerminalReport();
  const execution = currentReportsTerminalExecution() || {};
  const benchmark = currentReportsTerminalBenchmark() || {};
  const timings = benchmark.timings || execution.timings || {};
  const health = report && report.followDaemon && report.followDaemon.health ? report.followDaemon.health : null;
  const job = currentReportsTerminalFollowJob();
  const actions = currentReportsTerminalFollowActions();
  const problemCount = actions.filter((action) => ["failed", "cancelled", "expired"].includes(String(action.state || "").toLowerCase())).length;
  const runningCount = actions.filter((action) => ["running", "eligible", "armed", "queued", "sent"].includes(String(action.state || "").toLowerCase())).length;
  const overviewCards = [
    { label: "Action", value: entry && entry.action ? entry.action : payload && payload.action ? payload.action : "--" },
    { label: "Mint", value: entry && entry.mint ? shortenAddress(entry.mint, 6) : report && report.mint ? shortenAddress(report.mint, 6) : "--" },
    { label: "Provider", value: execution.resolvedProvider || execution.provider || "--" },
    { label: "Transport", value: execution.transportType || (entry && entry.transportType) || "--" },
    { label: "Signatures", value: entry ? String(entry.signatureCount || 0) : String(Array.isArray(payload && payload.signatures) ? payload.signatures.length : 0) },
    { label: "Follow", value: job ? (job.state || "armed") : "Off" },
    { label: "Selected Wallet", value: job && job.selectedWalletKey ? `Wallet #${walletIndexFromEnvKey(job.selectedWalletKey)}` : "--" },
    { label: "Follow Actions", value: actions.length ? `${actions.length} total` : "0" },
    { label: "Problems", value: String(problemCount) },
    { label: "Running", value: String(runningCount) },
    { label: "Submit", value: formatReportMetric(timings.sendSubmitMs, "ms") },
    { label: "Confirm", value: formatReportMetric(timings.sendConfirmMs, "ms") },
  ];
  const watcherCards = health
    ? [
      { label: "Slot Watcher", value: health.slotWatcher || "--" },
      { label: "Signature Watcher", value: health.signatureWatcher || "--" },
      { label: "Market Watcher", value: health.marketWatcher || "--" },
      { label: "Queue Depth", value: String(health.queueDepth != null ? health.queueDepth : "--") },
      { label: "Compile Slots", value: String(health.availableCompileSlots != null ? health.availableCompileSlots : "--") },
      { label: "Send Slots", value: String(health.availableSendSlots != null ? health.availableSendSlots : "--") },
    ]
    : [];
  return `
    <div class="reports-panel-stack">
      <section class="reports-panel-section">
        <div class="reports-panel-title">Overview</div>
        ${renderReportMetricGrid(overviewCards)}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Primary Benchmarks</div>
        ${renderReportMetricGrid([
          { label: "Total", value: formatReportMetric(timings.totalElapsedMs, "ms") },
          { label: "Backend", value: formatReportMetric(timings.backendTotalElapsedMs, "ms") },
          { label: "Compile", value: formatReportMetric(timings.compileTransactionsMs, "ms") },
          { label: "Serialize", value: formatReportMetric(timings.compileTxSerializeMs, "ms") },
          { label: "Send", value: formatReportMetric(timings.sendMs, "ms") },
          { label: "Persist", value: formatReportMetric(timings.persistReportMs, "ms") },
        ])}
      </section>
      ${watcherCards.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Follow Health</div>
          ${renderReportMetricGrid(watcherCards)}
          ${health && health.lastError ? `<div class="reports-callout is-bad">${escapeHTML(String(health.lastError))}</div>` : ""}
        </section>
      ` : ""}
    </div>
  `;
}

function buildReportsActionsMarkup() {
  const execution = currentReportsTerminalExecution() || {};
  const followJob = currentReportsTerminalFollowJob();
  const actions = currentReportsTerminalFollowActions();
  const launchSends = Array.isArray(execution.sent) ? execution.sent : [];
  if (!launchSends.length && !actions.length) {
    return '<div class="reports-terminal-empty">No action data available in this report.</div>';
  }
  return `
    <div class="reports-panel-stack">
      ${launchSends.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Launch Send</div>
          <div class="reports-action-list">
            ${launchSends.map((sent) => `
              <article class="reports-action-card">
                <div class="reports-action-head">
                  <div>
                    <strong>${escapeHTML(sent.label || "launch")}</strong>
                    <div class="reports-action-subtitle">${escapeHTML(execution.resolvedProvider || execution.provider || execution.transportType || "--")}</div>
                  </div>
                  <span class="reports-state-badge ${reportStateClass(sent.confirmationStatus)}">${escapeHTML(sent.confirmationStatus || "sent")}</span>
                </div>
                ${renderReportMetricGrid([
                  { label: "Endpoint", value: shortenReportEndpoint(sent.endpoint) },
                  { label: "Send Block", value: sent.sendObservedBlockHeight != null ? String(sent.sendObservedBlockHeight) : "--" },
                  { label: "Confirm Block", value: sent.confirmedObservedBlockHeight != null ? String(sent.confirmedObservedBlockHeight) : "--" },
                  { label: "Blocks", value: sent.confirmedObservedBlockHeight != null && sent.sendObservedBlockHeight != null ? String(sent.confirmedObservedBlockHeight - sent.sendObservedBlockHeight) : "--" },
                  { label: "Format", value: sent.format || "--" },
                ])}
                ${sent.signature ? `<div class="reports-action-links">${renderCopyableHash(sent.signature, "Copy signature")} ${sent.explorerUrl ? `<a class="reports-inline-link" href="${escapeHTML(sent.explorerUrl)}" target="_blank" rel="noreferrer">Open explorer</a>` : ""}</div>` : ""}
              </article>
            `).join("")}
          </div>
        </section>
      ` : ""}
      ${actions.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Follow Actions</div>
          <div class="reports-action-list">
            ${actions.map((action) => {
              const errorCategory = inferReportErrorCategory(action.lastError);
              return `
                <article class="reports-action-card">
                  <div class="reports-action-head">
                    <div>
                      <strong>${escapeHTML(action.kind || action.actionId || "action")}</strong>
                      <div class="reports-action-subtitle">${escapeHTML(`${describeFollowActionWallet(action)} | ${describeFollowActionTrigger(action)} | ${describeFollowActionSize(action)}`)}</div>
                    </div>
                    <span class="reports-state-badge ${reportStateClass(action.state)}">${escapeHTML(action.state || "--")}</span>
                  </div>
                  ${renderReportMetricGrid(buildFollowActionMetricItems(action, followJob))}
                  ${(action.signature || action.explorerUrl) ? `<div class="reports-action-links">${action.signature ? renderCopyableHash(action.signature, "Copy signature") : ""} ${action.explorerUrl ? `<a class="reports-inline-link" href="${escapeHTML(action.explorerUrl)}" target="_blank" rel="noreferrer">Open explorer</a>` : ""}</div>` : ""}
                  ${action.lastError ? `<div class="reports-callout is-bad"><strong>${escapeHTML(errorCategory || "error")}</strong> | ${escapeHTML(String(action.lastError))}</div>` : ""}
                </article>
              `;
            }).join("")}
          </div>
        </section>
      ` : ""}
    </div>
  `;
}

function buildReportsBenchmarksMarkup() {
  const benchmark = currentReportsTerminalBenchmark() || {};
  const execution = currentReportsTerminalExecution() || {};
  const timings = benchmark.timings || execution.timings || {};
  const sent = Array.isArray(benchmark.sent) && benchmark.sent.length ? benchmark.sent : (Array.isArray(execution.sent) ? execution.sent : []);
  const timingProfiles = currentReportsTerminalTimingProfiles();
  return `
    <div class="reports-panel-stack">
      <section class="reports-panel-section">
        <div class="reports-panel-title">Timing Breakdown</div>
        ${renderReportMetricGrid([
          { label: "Total", value: formatReportMetric(timings.totalElapsedMs, "ms") },
          { label: "Backend", value: formatReportMetric(timings.backendTotalElapsedMs, "ms") },
          { label: "Pre-request", value: formatReportMetric(timings.clientPreRequestMs, "ms") },
          { label: "Form", value: formatReportMetric(timings.formToRawConfigMs, "ms") },
          { label: "Normalize", value: formatReportMetric(timings.normalizeConfigMs, "ms") },
          { label: "Wallet Load", value: formatReportMetric(timings.walletLoadMs, "ms") },
          { label: "Compile", value: formatReportMetric(timings.compileTransactionsMs, "ms") },
          { label: "ALT Load", value: formatReportMetric(timings.compileAltLoadMs, "ms") },
          { label: "Blockhash", value: formatReportMetric(timings.compileBlockhashFetchMs, "ms") },
          { label: "Serialize", value: formatReportMetric(timings.compileTxSerializeMs, "ms") },
          { label: "Send", value: formatReportMetric(timings.sendMs, "ms") },
          { label: "Submit", value: formatReportMetric(timings.sendSubmitMs, "ms") },
          { label: "Confirm", value: formatReportMetric(timings.sendConfirmMs, "ms") },
          { label: "Persist", value: formatReportMetric(timings.persistReportMs, "ms") },
        ])}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Chain Benchmark</div>
        ${sent.length ? `
          <div class="reports-action-list">
            ${sent.map((item) => `
              <article class="reports-action-card">
                <div class="reports-action-head">
                  <div>
                    <strong>${escapeHTML(item.label || "tx")}</strong>
                    <div class="reports-action-subtitle">${escapeHTML(item.signature ? shortenAddress(item.signature, 8) : "--")}</div>
                  </div>
                  <span class="reports-state-badge ${reportStateClass(item.confirmationStatus)}">${escapeHTML(item.confirmationStatus || "--")}</span>
                </div>
                ${renderReportMetricGrid([
                  { label: "Send Block", value: item.sendBlockHeight != null ? String(item.sendBlockHeight) : item.sendObservedBlockHeight != null ? String(item.sendObservedBlockHeight) : "--" },
                  { label: "Confirm Block", value: item.confirmedBlockHeight != null ? String(item.confirmedBlockHeight) : item.confirmedObservedBlockHeight != null ? String(item.confirmedObservedBlockHeight) : "--" },
                  { label: "Blocks To Confirm", value: item.blocksToConfirm != null ? String(item.blocksToConfirm) : "--" },
                  { label: "Confirmed Slot", value: item.confirmedSlot != null ? String(item.confirmedSlot) : "--" },
                ])}
              </article>
            `).join("")}
          </div>
        ` : '<div class="reports-terminal-empty">No chain benchmark entries recorded.</div>'}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Timing Profiles</div>
        ${timingProfiles.length ? `
          <div class="reports-action-list">
            ${timingProfiles.map((profile) => {
              const rec = profile.recommendation || {};
              return `
                <article class="reports-action-card">
                  <div class="reports-action-head">
                    <div>
                      <strong>${escapeHTML(profile.actionType || "unknown")}</strong>
                      <div class="reports-action-subtitle">${escapeHTML(`${profile.provider || "--"} | confidence ${rec.confidence || "low"}`)}</div>
                    </div>
                    <span class="reports-state-badge ${Number(rec.successRate || 0) >= 0.75 ? "is-good" : Number(rec.successRate || 0) >= 0.4 ? "is-warn" : "is-bad"}">${escapeHTML(formatReportMetric(Number(rec.successRate || 0) * 100, "%", "--", 0))}</span>
                  </div>
                  ${renderReportMetricGrid([
                    { label: "Samples", value: String(profile.sampleCount != null ? profile.sampleCount : rec.sampleCount || 0) },
                    { label: "Success", value: formatReportMetric(Number(rec.successRate || 0) * 100, "%", "--", 0) },
                    { label: "Quality", value: formatReportMetric(rec.weightedQualityScore, "", "--", 1) },
                    { label: "P50 Submit", value: formatReportMetric(profile.p50SubmitMs, "ms") },
                    { label: "P75 Submit", value: formatReportMetric(profile.p75SubmitMs, "ms") },
                    { label: "P90 Submit", value: formatReportMetric(profile.p90SubmitMs, "ms") },
                    { label: "Suggest Delay", value: formatReportMetric(rec.suggestedSubmitDelayMs, "ms") },
                    { label: "Suggest Jitter", value: formatReportMetric(rec.suggestedJitterMs, "ms") },
                  ])}
                </article>
              `;
            }).join("")}
          </div>
        ` : '<div class="reports-terminal-empty">No timing profiles recorded yet.</div>'}
      </section>
    </div>
  `;
}

function buildReportsRawMarkup() {
  return `<pre class="console reports-console">${escapeHTML(reportsTerminalState.activeText || "Report is empty.")}</pre>`;
}

function buildReportsTerminalOutputMarkup() {
  const payload = currentReportsTerminalPayload();
  const tab = normalizeReportsTerminalTab(reportsTerminalState.activeTab);
  const tabs = [
    { id: "overview", label: "Overview" },
    { id: "actions", label: "Actions" },
    { id: "benchmarks", label: "Benchmarks" },
    { id: "raw", label: "Raw" },
  ];
  const fallbackMessage = reportsTerminalState.activeText || "Structured report data is unavailable for this entry.";
  const content = !payload && tab !== "raw"
    ? `<div class="reports-callout">${escapeHTML(fallbackMessage)}</div>`
    : tab === "actions"
      ? buildReportsActionsMarkup()
      : tab === "benchmarks"
        ? buildReportsBenchmarksMarkup()
        : tab === "raw"
          ? buildReportsRawMarkup()
          : buildReportsOverviewMarkup();
  return `
    <div class="reports-terminal-tabs">
      ${tabs.map((item) => `
        <button
          type="button"
          class="reports-terminal-tab${tab === item.id ? " active" : ""}"
          data-report-tab="${item.id}"
        >
          ${escapeHTML(item.label)}
        </button>
      `).join("")}
    </div>
    <div class="reports-terminal-content">${content}</div>
  `;
}

function renderReportsTerminalOutput() {
  if (!reportsTerminalOutput) return;
  const markup = buildReportsTerminalOutputMarkup();
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "reportsOutput", reportsTerminalOutput, markup);
  } else {
    reportsTerminalOutput.innerHTML = markup;
  }
}

function renderReportsTerminalList() {
  if (!reportsTerminalList) return;
  if (!reportsTerminalState.entries.length) {
    const emptyMarkup = '<div class="reports-terminal-empty">No persisted reports found yet.</div>';
    if (RenderUtils.setCachedHTML) {
      RenderUtils.setCachedHTML(renderCache, "reportsList", reportsTerminalList, emptyMarkup);
    } else {
      reportsTerminalList.innerHTML = emptyMarkup;
    }
    return;
  }
  const markup = reportsTerminalState.entries.map((entry) => `
    <button
      type="button"
      class="reports-terminal-item${entry.id === reportsTerminalState.activeId ? " active" : ""}"
      data-report-id="${escapeHTML(entry.id)}"
    >
      <span class="reports-terminal-item-title">${escapeHTML(String(entry.action || "unknown"))}</span>
      <span class="reports-terminal-item-meta">${escapeHTML(String(entry.mint || entry.fileName || "Unknown mint"))}</span>
      <span class="reports-terminal-item-meta">${escapeHTML(describeReportEntry(entry) || "No metadata")}</span>
    </button>
  `).join("");
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "reportsList", reportsTerminalList, markup);
  } else {
    reportsTerminalList.innerHTML = markup;
  }
}

async function loadReportsTerminalEntry(id) {
  if (!id || !reportsTerminalOutput) return;
  reportsTerminalState.activePayload = null;
  reportsTerminalState.activeText = "Loading report...";
  renderReportsTerminalOutput();
  const url = `/api/reports/view?id=${encodeURIComponent(id)}`;
  const result = RequestUtils.fetchJsonLatest
    ? await RequestUtils.fetchJsonLatest("report-view", url, {}, requestStates.reportView)
    : null;
  if (result && result.aborted) return;
  const response = result ? result.response : await fetch(url);
  const payload = result ? result.payload : await response.json();
  if (result && !result.isLatest) return;
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to load report.");
  }
  reportsTerminalState.activeId = payload.entry && payload.entry.id ? payload.entry.id : id;
  reportsTerminalState.activePayload = payload.payload && typeof payload.payload === "object" ? payload.payload : null;
  reportsTerminalState.activeText = payload.text || "Report is empty.";
  if (reportsTerminalMeta) {
    reportsTerminalMeta.textContent = payload.entry
      ? [
          payload.entry.displayTime || "Unknown time",
          payload.entry.action || "unknown",
          payload.entry.provider || "unknown route",
          payload.entry.followEnabled && payload.entry.followState ? `follow ${payload.entry.followState}` : "",
        ].filter(Boolean).join(" | ")
      : "Report loaded.";
  }
  renderReportsTerminalOutput();
  renderReportsTerminalList();
}

async function refreshReportsTerminal({ preserveSelection = true, preferId = "" } = {}) {
  if (!reportsTerminalList || !reportsTerminalOutput) return;
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "reportsList", reportsTerminalList, '<div class="reports-terminal-empty">Loading reports...</div>');
  } else {
    reportsTerminalList.innerHTML = '<div class="reports-terminal-empty">Loading reports...</div>';
  }
  const url = `/api/reports?sort=${encodeURIComponent(reportsTerminalState.sort)}`;
  const result = RequestUtils.fetchJsonLatest
    ? await RequestUtils.fetchJsonLatest("reports", url, {}, requestStates.reports)
    : null;
  if (result && result.aborted) return;
  const response = result ? result.response : await fetch(url);
  const payload = result ? result.payload : await response.json();
  if (result && !result.isLatest) return;
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to list reports.");
  }
  reportsTerminalState.entries = Array.isArray(payload.reports) ? payload.reports : [];
  setReportsTerminalSort(payload.sort || reportsTerminalState.sort, { persist: false });
  const availableIds = new Set(reportsTerminalState.entries.map((entry) => entry.id));
  const nextId = preferId && availableIds.has(preferId)
    ? preferId
    : preserveSelection && reportsTerminalState.activeId && availableIds.has(reportsTerminalState.activeId)
      ? reportsTerminalState.activeId
      : reportsTerminalState.entries[0] && reportsTerminalState.entries[0].id
        ? reportsTerminalState.entries[0].id
        : "";
  reportsTerminalState.activeId = nextId;
  renderReportsTerminalList();
  if (!nextId) {
    reportsTerminalState.activePayload = null;
    reportsTerminalState.activeText = "Run Build, Simulate, or Deploy to create persisted reports.";
    if (reportsTerminalMeta) reportsTerminalMeta.textContent = "No persisted reports available yet.";
    renderReportsTerminalOutput();
    return;
  }
  await loadReportsTerminalEntry(nextId);
}

function extractReportIdFromPath(filePath) {
  const normalized = String(filePath || "").trim();
  if (!normalized) return "";
  const parts = normalized.split(/[\\/]+/);
  return parts[parts.length - 1] || "";
}

function getStoredThemeMode() {
  try {
    const stored = window.localStorage.getItem(THEME_MODE_STORAGE_KEY);
    return stored === "light" ? "light" : "dark";
  } catch (_error) {
    return "dark";
  }
}

function setThemeMode(mode, { persist = true } = {}) {
  const normalized = mode === "light" ? "light" : "dark";
  document.documentElement.classList.toggle("theme-light", normalized === "light");
  document.body.classList.toggle("theme-light", normalized === "light");
  if (themeToggleButton) {
    themeToggleButton.textContent = normalized === "light" ? "☾" : "☀";
    themeToggleButton.classList.toggle("active", normalized === "light");
    themeToggleButton.setAttribute("aria-pressed", normalized === "light" ? "true" : "false");
    themeToggleButton.setAttribute("title", normalized === "light" ? "Switch to dark mode" : "Switch to white mode");
    themeToggleButton.setAttribute("aria-label", normalized === "light" ? "Switch to dark mode" : "Switch to white mode");
  }
  if (!persist) return;
  try {
    window.localStorage.setItem(THEME_MODE_STORAGE_KEY, normalized);
  } catch (_error) {
    // Ignore storage failures and keep theme switching functional.
  }
}

function completeInitialBoot() {
  if (window.__launchdeckBootFallback) {
    window.clearTimeout(window.__launchdeckBootFallback);
    window.__launchdeckBootFallback = null;
  }
  if (isPopoutMode) {
    resizePopoutToVisibleLayout();
  }
  requestAnimationFrame(() => {
    document.documentElement.classList.remove("boot-pending");
    schedulePopoutAutosize();
    if (isPopoutMode) {
      window.setTimeout(() => {
        schedulePopoutAutosize();
      }, 120);
    }
  });
}

function isOutputSectionCurrentlyVisible() {
  return Boolean(outputSection && !outputSection.hidden);
}

function isReportsTerminalCurrentlyVisible() {
  return Boolean(reportsTerminalSection && !reportsTerminalSection.hidden);
}

function measureVisibleWorkspaceContent() {
  if (!workspaceShell) {
    return { width: 0, height: 0 };
  }
  const visibleChildren = Array.from(workspaceShell.children).filter((node) => !node.hidden);
  if (!visibleChildren.length) {
    return { width: 0, height: 0 };
  }
  const workspaceStyles = window.getComputedStyle(workspaceShell);
  const gap = Number.parseFloat(workspaceStyles.columnGap || workspaceStyles.gap || "0") || 0;
  const width = visibleChildren.reduce((sum, node, index) => {
    const rect = node.getBoundingClientRect();
    return sum + Math.ceil(rect.width) + (index > 0 ? gap : 0);
  }, 0);
  const workspaceRect = workspaceShell.getBoundingClientRect();
  const height = Math.ceil(
    Math.max(
      workspaceRect ? workspaceRect.height : 0,
      ...visibleChildren.map((node) => {
        const rect = node.getBoundingClientRect();
        return rect.height;
      }),
    ),
  );
  return { width, height };
}

function getPreferredPopoutContentWidth() {
  const measuredWidth = measureVisibleWorkspaceContent().width;
  let preferredWidth = 0;
  if (form && !form.hidden) {
    preferredWidth += POPOUT_FORM_WIDTH;
  }
  if (reportsTerminalSection && !reportsTerminalSection.hidden) {
    preferredWidth += preferredWidth > 0 ? POPOUT_WORKSPACE_GAP : 0;
    preferredWidth += POPOUT_REPORTS_WIDTH;
  }
  return Math.max(measuredWidth, preferredWidth);
}

function getPreferredPopoutContentHeight() {
  return measureVisibleWorkspaceContent().height;
}

function resizePopoutToVisibleLayout() {
  if (!isPopoutMode) return;
  const contentWidth = getPreferredPopoutContentWidth();
  const contentHeight = getPreferredPopoutContentHeight();
  if (!contentWidth || !contentHeight) return;
  const chromeWidth = Math.max(0, window.outerWidth - window.innerWidth);
  const chromeHeight = Math.max(0, window.outerHeight - window.innerHeight);
  const maxOuterWidth = Math.max(420, window.screen.availWidth - 24);
  const maxOuterHeight = Math.max(560, window.screen.availHeight - 24);
  const targetWidth = Math.min(Math.max(420, contentWidth + chromeWidth + 4), maxOuterWidth);
  const targetHeight = Math.min(Math.max(560, contentHeight + chromeHeight + 4), maxOuterHeight);
  if (Math.abs(window.outerWidth - targetWidth) < 4 && Math.abs(window.outerHeight - targetHeight) < 4) {
    return;
  }
  try {
    window.resizeTo(targetWidth, targetHeight);
  } catch (_error) {
    // Ignore resize failures on browsers that restrict popup resizing.
  }
}

function schedulePopoutAutosize() {
  if (!isPopoutMode) return;
  if (popoutAutosizeFrame) {
    window.cancelAnimationFrame(popoutAutosizeFrame);
  }
  popoutAutosizeFrame = window.requestAnimationFrame(() => {
    popoutAutosizeFrame = 0;
    window.requestAnimationFrame(() => {
      resizePopoutToVisibleLayout();
    });
  });
}

function openPopoutWindow() {
  const popoutUrl = new URL(window.location.href);
  popoutUrl.searchParams.set("popout", "1");
  const outputVisible = isOutputSectionCurrentlyVisible();
  const reportsVisible = isReportsTerminalCurrentlyVisible();
  popoutUrl.searchParams.set("output", outputVisible ? "1" : "0");
  popoutUrl.searchParams.set("reports", reportsVisible ? "1" : "0");
  const contentWidth = getPreferredPopoutContentWidth();
  const contentHeight = getPreferredPopoutContentHeight();
  const chromeWidth = Math.max(0, window.outerWidth - window.innerWidth);
  const chromeHeight = Math.max(0, window.outerHeight - window.innerHeight);
  const width = Math.min(
    Math.max(420, (contentWidth || 720) + chromeWidth + 4),
    Math.max(420, window.screen.availWidth - 24),
  );
  const height = Math.min(
    Math.max(560, (contentHeight || 760) + chromeHeight + 4),
    Math.max(560, window.screen.availHeight - 24),
  );
  window.open(
    popoutUrl.toString(),
    "launchdeck-popout",
    `popup=yes,width=${width},height=${height},menubar=no,toolbar=no,location=no,status=no,resizable=yes,scrollbars=yes`,
  );
}

form.querySelectorAll('input[name="mode"]').forEach((node) => {
  node.addEventListener("change", () => {
    setStoredLaunchMode(getMode());
    updateModeVisibility();
  });
});
launchpadInputs.forEach((input) => {
  input.addEventListener("change", () => {
    if (!input.checked) return;
    applyLaunchpadTokenMetadata();
  });
});
if (nameInput) {
  nameInput.addEventListener("input", () => {
    syncTickerFromName();
    markMetadataUploadDirty();
    scheduleMetadataPreupload({ immediate: true });
  });
}
if (descriptionToggle) {
  descriptionToggle.addEventListener("click", () => {
    toggleDescriptionDisclosure();
  });
}
if (descriptionInput) {
  descriptionInput.addEventListener("input", () => {
    updateDescriptionDisclosure();
    markMetadataUploadDirty();
    scheduleMetadataPreupload({ immediate: true });
  });
}
if (symbolInput) {
  symbolInput.addEventListener("focus", () => {
    const autoTickerValue = getAutoTickerValue();
    if (!tickerManuallyEdited && autoTickerValue && symbolInput.value === autoTickerValue) {
      syncingTickerFromName = true;
      symbolInput.value = "";
      syncingTickerFromName = false;
      tickerManuallyEdited = true;
      tickerClearedForManualEntry = true;
      updateTokenFieldCounts();
    }
  });
  symbolInput.addEventListener("input", () => {
    if (syncingTickerFromName) return;
    const formatted = formatTickerValue(symbolInput.value);
    syncingTickerFromName = true;
    if (symbolInput.value !== formatted) {
      symbolInput.value = formatted;
    }
    syncingTickerFromName = false;
    tickerManuallyEdited = true;
    tickerClearedForManualEntry = symbolInput.value.trim().length === 0;
    updateTokenFieldCounts();
    markMetadataUploadDirty();
    scheduleMetadataPreupload({ immediate: true });
  });
  symbolInput.addEventListener("blur", () => {
    if (!symbolInput.value.trim()) {
      tickerManuallyEdited = false;
      tickerClearedForManualEntry = false;
      syncTickerFromName();
      return;
    }
    tickerClearedForManualEntry = false;
  });
}
[
  websiteInput,
  twitterInput,
  telegramInput,
].filter(Boolean).forEach((input) => {
  input.addEventListener("input", () => {
    markMetadataUploadDirty();
    scheduleMetadataPreupload({ immediate: true });
  });
});
if (tickerCapsToggle) {
  tickerCapsToggle.addEventListener("click", () => {
    tickerCapsToggle.classList.toggle("active");
    applyTickerCapsMode();
    if (!tickerManuallyEdited) {
      syncTickerFromName();
    }
  });
}
if (changeDevBuyPresetsButton) {
  changeDevBuyPresetsButton.addEventListener("click", () => {
    setDevBuyPresetEditorOpen(true);
    populateDevBuyPresetEditor(getConfig());
  });
}
if (cancelDevBuyPresetsButton) {
  cancelDevBuyPresetsButton.addEventListener("click", () => {
    setDevBuyPresetEditorOpen(false);
  });
}
if (saveDevBuyPresetsButton) {
  saveDevBuyPresetsButton.addEventListener("click", async () => {
    await saveDevBuyPresetEditor();
  });
}
if (devBuySolInput) {
  devBuySolInput.addEventListener("input", async () => {
    if (syncingDevBuyInputs) return;
    await updateDevBuyFromSolInput(devBuySolInput.value);
  });
}
if (devBuyPercentInput) {
  devBuyPercentInput.addEventListener("input", async () => {
    if (syncingDevBuyInputs) return;
    await updateDevBuyFromPercentInput(devBuyPercentInput.value);
  });
}
if (providerSelect) providerSelect.addEventListener("change", () => {
  syncActivePresetFromInputs();
  updateJitoVisibility();
});
if (buyProviderSelect) buyProviderSelect.addEventListener("change", syncActivePresetFromInputs);
if (sellProviderSelect) sellProviderSelect.addEventListener("change", syncActivePresetFromInputs);
feeSplitPill.addEventListener("click", () => {
  const mode = getMode();
  if (mode !== "regular" && mode !== "agent-custom") return;
  if (!feeSplitEnabled.checked) {
    feeSplitEnabled.checked = true;
    showFeeSplitModal();
    return;
  }
  showFeeSplitModal();
});
if (walletTriggerButton) {
  walletTriggerButton.addEventListener("click", () => {
    toggleWalletDropdown();
  });
}
if (walletRefreshButton) {
  walletRefreshButton.addEventListener("click", async () => {
    await refreshWalletStatus(true);
  });
}
if (walletDropdownList) {
  walletDropdownList.addEventListener("click", (event) => {
    const button = event.target.closest(".wallet-option-button");
    if (!button || !walletSelect) return;
    const nextKey = String(button.dataset.walletKey || "").trim();
    if (!nextKey) return;
    walletSelect.value = nextKey;
    setStoredSelectedWalletKey(nextKey);
    applySelectedWalletLocally(nextKey);
    setWalletDropdownOpen(false);
    refreshWalletStatus(true);
  });
}
walletSelect.addEventListener("change", () => {
  const nextKey = selectedWalletKey();
  setStoredSelectedWalletKey(nextKey);
  applySelectedWalletLocally(nextKey);
  refreshWalletStatus(true);
});
document.addEventListener("click", (event) => {
  if (!walletDropdown || walletDropdown.hidden) return;
  const target = event.target;
  if (walletBox && walletBox.contains(target)) return;
  setWalletDropdownOpen(false);
});

feeSplitAdd.addEventListener("click", () => {
  if (getFeeSplitRows().length >= MAX_FEE_SPLIT_RECIPIENTS) return;
  feeSplitList.appendChild(createFeeSplitRow({ type: "wallet", sharePercent: "" }));
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
});

feeSplitReset.addEventListener("click", () => {
  getFeeSplitRows().forEach((row) => {
    row.querySelector(".recipient-share").value = "0";
    row.querySelector(".recipient-slider").value = "0";
  });
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
});

feeSplitEven.addEventListener("click", () => {
  const rows = getFeeSplitRows();
  const targetRows = rows;
  if (targetRows.length === 0) return;
  rows.forEach((row) => {
    row.querySelector(".recipient-share").value = "0";
    row.querySelector(".recipient-slider").value = "0";
  });
  const evenShare = Number((100 / targetRows.length).toFixed(2));
  let assigned = 0;
  targetRows.forEach((row, index) => {
    const share = index === targetRows.length - 1 ? Number((100 - assigned).toFixed(2)) : evenShare;
    assigned += share;
    row.querySelector(".recipient-share").value = String(share);
    row.querySelector(".recipient-slider").value = String(share);
  });
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
});

feeSplitList.addEventListener("click", (event) => {
  const lockToggle = event.target.closest(".recipient-lock-toggle");
  if (lockToggle) {
    const row = lockToggle.closest(".fee-split-row");
    setRecipientTargetLocked(row, row.dataset.targetLocked !== "true");
    syncFeeSplitTotals();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    return;
  }
  const tab = event.target.closest(".recipient-type-tab");
  if (tab) {
    updateFeeSplitRowType(tab.closest(".fee-split-row"), tab.dataset.type);
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    return;
  }
  const removeButton = event.target.closest(".recipient-remove");
  if (removeButton) {
    removeButton.closest(".fee-split-row").remove();
    syncFeeSplitTotals();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
  }
});

feeSplitList.addEventListener("input", (event) => {
  const row = event.target.closest(".fee-split-row");
  if (!row) return;
  if (event.target.classList.contains("recipient-slider")) {
    row.querySelector(".recipient-share").value = event.target.value;
  }
  if (event.target.classList.contains("recipient-share")) {
    row.querySelector(".recipient-slider").value = event.target.value || "0";
  }
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
});

agentSplitAdd.addEventListener("click", () => {
  if (getAgentSplitRows().length >= MAX_FEE_SPLIT_RECIPIENTS) {
    setAgentSplitModalError(`Agent custom fee split supports at most ${MAX_FEE_SPLIT_RECIPIENTS} recipients.`);
    return;
  }
  agentSplitList.appendChild(createAgentSplitRow({ type: "wallet", sharePercent: "" }));
  normalizeAgentSplitStructure({ afterAdd: true });
  syncAgentSplitTotals();
  setStoredAgentSplitDraft(serializeAgentSplitDraft());
  setAgentSplitModalError("");
});

agentSplitReset.addEventListener("click", () => {
  getAgentSplitRows().forEach((row) => {
    row.querySelector(".recipient-share").value = "0";
    row.querySelector(".recipient-slider").value = "0";
  });
  syncAgentSplitTotals();
  setStoredAgentSplitDraft(serializeAgentSplitDraft());
  setAgentSplitModalError("");
});

agentSplitEven.addEventListener("click", () => {
  const rows = getAgentSplitRows();
  const targetRows = rows;
  if (targetRows.length === 0) return;
  rows.forEach((row) => {
    row.querySelector(".recipient-share").value = "0";
    row.querySelector(".recipient-slider").value = "0";
  });
  const evenShare = Number((100 / targetRows.length).toFixed(2));
  let assigned = 0;
  targetRows.forEach((row, index) => {
    const share = index === targetRows.length - 1 ? Number((100 - assigned).toFixed(2)) : evenShare;
    assigned += share;
    row.querySelector(".recipient-share").value = String(share);
    row.querySelector(".recipient-slider").value = String(share);
  });
  syncAgentSplitTotals();
  setStoredAgentSplitDraft(serializeAgentSplitDraft());
  setAgentSplitModalError("");
});

agentSplitList.addEventListener("click", (event) => {
  const lockToggle = event.target.closest(".recipient-lock-toggle");
  if (lockToggle) {
    const row = lockToggle.closest(".fee-split-row");
    setRecipientTargetLocked(row, row.dataset.targetLocked !== "true");
    syncAgentSplitTotals();
    setStoredAgentSplitDraft(serializeAgentSplitDraft());
    setAgentSplitModalError("");
    return;
  }
  const tab = event.target.closest(".recipient-type-tab");
  if (tab && tab.dataset.type) {
    updateFeeSplitRowType(tab.closest(".fee-split-row"), tab.dataset.type);
    syncAgentSplitTotals();
    setStoredAgentSplitDraft(serializeAgentSplitDraft());
    setAgentSplitModalError("");
    return;
  }
  const removeButton = event.target.closest(".recipient-remove");
  if (removeButton) {
    removeButton.closest(".fee-split-row").remove();
    normalizeAgentSplitStructure();
    syncAgentSplitTotals();
    setStoredAgentSplitDraft(serializeAgentSplitDraft());
    setAgentSplitModalError("");
  }
});

agentSplitList.addEventListener("input", (event) => {
  const row = event.target.closest(".fee-split-row");
  if (!row) return;
  if (event.target.classList.contains("recipient-slider")) {
    row.querySelector(".recipient-share").value = event.target.value;
  }
  if (event.target.classList.contains("recipient-share")) {
    row.querySelector(".recipient-slider").value = event.target.value || "0";
  }
  if (event.target.classList.contains("recipient-target") && row.dataset.defaultReceiver === "true") {
    delete row.dataset.defaultReceiver;
  }
  syncAgentSplitTotals();
  setStoredAgentSplitDraft(serializeAgentSplitDraft());
  setAgentSplitModalError("");
});

Object.keys(fieldValidators).forEach((name) => {
  const input = getNamedInput(name);
  if (!input) return;
  input.addEventListener("blur", () => validateFieldByName(name));
  input.addEventListener("input", () => {
    if (input.classList.contains("input-error")) validateFieldByName(name);
  });
});

if (imageLayoutToggle) {
  imageLayoutToggle.addEventListener("click", () => {
    setImageLayoutCompact(!(tokenSurfaceSection && tokenSurfaceSection.classList.contains("is-image-compact")));
  });
}
imageInput.addEventListener("change", async () => {
  const [file] = imageInput.files || [];
  if (!file) return;
  imageStatus.textContent = "Uploading image to library...";

  try {
    await uploadSelectedImage(file);
  } catch (error) {
    imageStatus.textContent = error.message;
  } finally {
    imageInput.value = "";
  }
});

testFillButton.addEventListener("click", async () => {
  await applyTestPreset();
});
if (openVampButton) {
  openVampButton.addEventListener("click", showVampModal);
}
if (themeToggleButton) {
  themeToggleButton.addEventListener("click", () => {
    const nextMode = document.documentElement.classList.contains("theme-light") ? "dark" : "light";
    setThemeMode(nextMode);
  });
}
if (openSettingsButton) {
  openSettingsButton.addEventListener("click", showSettingsModal);
}
saveSettingsButton.addEventListener("click", async () => {
  await saveSettings();
});

buttons.forEach((button) => {
  button.addEventListener("click", () => {
    const action = button.dataset.action;
    const errors = validateForm();
    if (showValidationErrors(errors)) return;
    clearValidationErrors();
    if (action === "deploy") {
      showDeployModal();
    } else {
      run(action);
    }
  });
});

modalClose.addEventListener("click", hideDeployModal);
modalCancel.addEventListener("click", hideDeployModal);
modalConfirm.addEventListener("click", () => {
  hideDeployModal();
  run("deploy");
});
if (settingsClose) settingsClose.addEventListener("click", hideSettingsModal);
if (settingsCancel) settingsCancel.addEventListener("click", hideSettingsModal);
if (settingsModal) {
  settingsModal.addEventListener("click", (event) => {
    if (event.target === settingsModal) hideSettingsModal();
  });
}
if (topPresetChipBar) {
  topPresetChipBar.addEventListener("click", (event) => {
    const chip = event.target.closest("[data-preset-id]");
    if (!chip) return;
    setActivePreset(chip.getAttribute("data-preset-id") || DEFAULT_PRESET_ID);
  });
}
if (settingsPresetChipBar) {
  settingsPresetChipBar.addEventListener("click", (event) => {
    const chip = event.target.closest("[data-preset-id]");
    if (!chip) return;
    setActivePreset(chip.getAttribute("data-preset-id") || DEFAULT_PRESET_ID);
  });
}
if (presetEditToggle) {
  presetEditToggle.addEventListener("click", () => {
    setPresetEditing(!isPresetEditing(getConfig()));
  });
}
[
  creationTipInput,
  creationPriorityInput,
  buyPriorityFeeInput,
  buyTipInput,
  buySlippageInput,
  sellPriorityFeeInput,
  sellTipInput,
  sellSlippageInput,
].forEach((input) => {
  if (!input) return;
  const eventName = input.tagName === "SELECT" || input.type === "checkbox" ? "change" : "input";
  input.addEventListener(eventName, () => {
    syncActivePresetFromInputs();
    if (input.name) validateFieldByName(input.name);
    if (sniperModal && !sniperModal.hidden) {
      renderSniperUI();
    }
  });
});
if (devBuyQuickButtons) {
  devBuyQuickButtons.addEventListener("click", async (event) => {
    if (devBuyPresetEditorOpen) return;
    const button = event.target.closest("[data-quick-buy-amount]");
    if (!button) return;
    const presetId = button.getAttribute("data-quick-buy-preset-id") || DEFAULT_PRESET_ID;
    const amount = button.getAttribute("data-quick-buy-amount") || "";
    if (!amount) return;
    setActivePreset(presetId);
    await triggerDeployWithDevBuy("sol", amount, "sol");
  });
}
if (devBuyCustomDeployButton) {
  devBuyCustomDeployButton.addEventListener("click", async () => {
    const mode = getDevBuyMode();
    const amount = getNamedValue("devBuyAmount").trim();
    if (!amount) {
      clearDevBuyState();
      const errors = validateForm();
      if (showValidationErrors(errors)) return;
      clearValidationErrors();
      showDeployModal();
      return;
    }
    await triggerDeployWithDevBuy(mode, amount, lastDevBuyEditSource);
  });
}
if (modeVanityButton) {
  modeVanityButton.addEventListener("click", () => {
    showVanityModal();
  });
}
if (feeSplitClose) feeSplitClose.addEventListener("click", hideFeeSplitModal);
if (feeSplitSave) feeSplitSave.addEventListener("click", () => {
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
  hideFeeSplitModal();
});
if (feeSplitDisable) {
  feeSplitDisable.addEventListener("click", () => {
    feeSplitEnabled.checked = false;
    updateFeeSplitVisibility();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    hideFeeSplitModal();
  });
}
if (feeSplitModal) {
  feeSplitModal.addEventListener("click", (event) => {
    if (event.target === feeSplitModal) hideFeeSplitModal();
  });
}
if (agentSplitClose) agentSplitClose.addEventListener("click", attemptCloseAgentSplitModal);
if (agentSplitCancel) {
  agentSplitCancel.addEventListener("click", () => {
    resetAgentSplitToDefault();
    hideAgentSplitModal();
  });
}
if (agentSplitSave) agentSplitSave.addEventListener("click", attemptCloseAgentSplitModal);
if (agentSplitModal) {
  agentSplitModal.addEventListener("click", (event) => {
    if (event.target === agentSplitModal) attemptCloseAgentSplitModal();
  });
}
if (vanitySave) {
  vanitySave.addEventListener("click", () => {
    const nextValue = vanityPrivateKeyText ? vanityPrivateKeyText.value.trim() : "";
    if (vanityModalError) vanityModalError.textContent = "";
    applyVanityValue(nextValue);
    hideVanityModal();
  });
}
if (vanityClear) {
  vanityClear.addEventListener("click", () => {
    if (vanityPrivateKeyText) vanityPrivateKeyText.value = "";
    if (vanityModalError) vanityModalError.textContent = "";
    applyVanityValue("");
    hideVanityModal();
  });
}
if (vanityClose) vanityClose.addEventListener("click", hideVanityModal);
if (vanityModal) {
  vanityModal.addEventListener("click", (event) => {
    if (event.target === vanityModal) hideVanityModal();
  });
}
if (vampImport) {
  vampImport.addEventListener("click", async () => {
    await importVampToken();
  });
}
if (vampClose) vampClose.addEventListener("click", hideVampModal);
if (vampCancel) vampCancel.addEventListener("click", hideVampModal);
if (vampContractInput) {
  vampContractInput.addEventListener("keydown", async (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    await importVampToken();
  });
}
if (vampModal) {
  vampModal.addEventListener("click", (event) => {
    if (event.target === vampModal) hideVampModal();
  });
}
deployModal.addEventListener("click", (event) => {
  if (event.target === deployModal) hideDeployModal();
});

updateModeVisibility();
updateJitoVisibility();
syncDevAutoSellUI();
hydrateModeActionState();
updateTokenFieldCounts();
updateDescriptionDisclosure();
setSettingsLoadingState(true);
renderBackendRegionSummary(null);
renderSniperUI();
renderReportsTerminalOutput();
completeInitialBoot();
Promise.resolve(bootstrapApp()).catch((error) => {
  if (walletBalance) walletBalance.textContent = "-";
  metaNode.textContent = error.message;
});
