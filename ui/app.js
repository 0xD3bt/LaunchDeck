const bootOverlay = document.getElementById("boot-overlay");
const form = document.getElementById("launch-form");
const launchSurfaceCard = form ? form.querySelector(".launch-surface") : null;
const output = document.getElementById("output");
const statusNode = document.getElementById("status");
const metaNode = document.getElementById("meta");
const outputSection = document.getElementById("output-section");
const reportsTerminalSection = document.getElementById("reports-terminal-section");
const reportsTerminalList = document.getElementById("reports-terminal-list");
const reportsTerminalOutput = document.getElementById("reports-terminal-output");
const reportsTerminalMeta = document.getElementById("reports-terminal-meta");
const reportsTerminalResizeHandle = document.getElementById("reports-terminal-resize-handle");
const benchmarksPopoutModal = document.getElementById("benchmarks-popout-modal");
const benchmarksPopoutTitle = document.getElementById("benchmarks-popout-title");
const benchmarksPopoutClose = document.getElementById("benchmarks-popout-close");
const benchmarksPopoutBody = document.getElementById("benchmarks-popout-body");
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
const themeToggleSunIcon = themeToggleButton ? themeToggleButton.querySelector(".theme-icon-sun") : null;
const themeToggleMoonIcon = themeToggleButton ? themeToggleButton.querySelector(".theme-icon-moon") : null;
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
const bonkQuoteAssetInput = getNamedInput("quoteAsset");
const bonkQuoteAssetToggle = document.getElementById("bonk-quote-asset-toggle");
const bonkQuoteAssetToggleSolIcon = document.getElementById("bonk-quote-asset-toggle-sol-icon");
const bonkQuoteAssetToggleUsd1Icon = document.getElementById("bonk-quote-asset-toggle-usd1-icon");
const bagsIdentityButton = document.getElementById("bags-identity-button");
const bagsIdentityButtonLabel = document.getElementById("bags-identity-button-label");
const bagsIdentityModal = document.getElementById("bags-identity-modal");
const bagsIdentityClose = document.getElementById("bags-identity-close");
const bagsIdentityCancel = document.getElementById("bags-identity-cancel");
const bagsIdentityVerifyButton = document.getElementById("bags-identity-verify");
const bagsIdentityInitButton = document.getElementById("bags-identity-init-button");
const bagsIdentityCurrent = document.getElementById("bags-identity-current");
const bagsApiKeyInput = document.getElementById("bags-api-key-input");
const bagsApiKeySave = document.getElementById("bags-api-key-save");
const bagsAgentUsernameInput = document.getElementById("bags-agent-username-input");
const bagsVerificationContent = document.getElementById("bags-verification-content");
const bagsPostIdInput = document.getElementById("bags-post-id-input");
const bagsIdentityError = document.getElementById("bags-identity-error");
const devBuyQuotePrefixIcon = document.getElementById("dev-buy-quote-prefix-icon");
const devBuyQuotePrefixText = document.getElementById("dev-buy-quote-prefix-text");
const creationTipInput = document.getElementById("creation-tip-input");
const creationPriorityInput = document.getElementById("creation-priority-input");
const creationMevModeSelect = document.getElementById("creation-mev-mode-select");
const creationAutoFeeInput = document.getElementById("creation-auto-fee-input");
const creationAutoFeeButton = document.getElementById("creation-auto-fee-button");
const creationMaxFeeInput = document.getElementById("creation-max-fee-input");
const launchpadInputs = Array.from(document.querySelectorAll('input[name="launchpad"]'));
const providerSelect = document.getElementById("provider-select");
const buyProviderSelect = document.getElementById("buy-provider-select");
const sellProviderSelect = document.getElementById("sell-provider-select");
const settingsBackendRegionSummary = document.getElementById("settings-backend-region-summary");
const platformRuntimeIndicators = document.getElementById("platform-runtime-indicators");
const buyPriorityFeeInput = document.getElementById("buy-priority-fee-input");
const buyTipInput = document.getElementById("buy-tip-input");
const buySlippageInput = document.getElementById("buy-slippage-input");
const buyMevModeSelect = document.getElementById("buy-mev-mode-select");
const buyAutoFeeInput = document.getElementById("buy-auto-fee-input");
const buyAutoFeeButton = document.getElementById("buy-auto-fee-button");
const buyMaxFeeInput = document.getElementById("buy-max-fee-input");
const buyHelloMoonMevWarning = document.getElementById("buy-hellomoon-mev-warning");
const buyStandardRpcWarning = document.getElementById("buy-standard-rpc-warning");
const sellPriorityFeeInput = document.getElementById("sell-priority-fee-input");
const sellTipInput = document.getElementById("sell-tip-input");
const sellSlippageInput = document.getElementById("sell-slippage-input");
const sellMevModeSelect = document.getElementById("sell-mev-mode-select");
const sellAutoFeeInput = document.getElementById("sell-auto-fee-input");
const sellAutoFeeButton = document.getElementById("sell-auto-fee-button");
const sellMaxFeeInput = document.getElementById("sell-max-fee-input");
const sellHelloMoonMevWarning = document.getElementById("sell-hellomoon-mev-warning");
const sellStandardRpcWarning = document.getElementById("sell-standard-rpc-warning");
const settingsPresetChipBar = document.getElementById("settings-preset-chip-bar");
const presetEditToggle = document.getElementById("preset-edit-toggle");
const agentUnlockedAuthority = document.getElementById("agent-unlocked-authority");
const agentSplitList = document.getElementById("agent-split-list");
const agentSplitAdd = document.getElementById("agent-split-add");
const agentSplitReset = document.getElementById("agent-split-reset");
const agentSplitEven = document.getElementById("agent-split-even");
const agentSplitClearAll = document.getElementById("agent-split-clear-all");
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
const feeSplitClearAll = document.getElementById("fee-split-clear-all");
const feeSplitTotal = document.getElementById("fee-split-total");
const feeSplitBar = document.getElementById("fee-split-bar");
const feeSplitLegendList = document.getElementById("fee-split-legend-list");
const feeSplitModal = document.getElementById("fee-split-modal");
const feeSplitClose = document.getElementById("fee-split-close");
const feeSplitDisable = document.getElementById("fee-split-disable");
const feeSplitSave = document.getElementById("fee-split-save");
const feeSplitModalError = document.getElementById("fee-split-modal-error");
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
const reportsTransactionsButton = document.getElementById("reports-transactions-button");
const reportsLaunchesButton = document.getElementById("reports-launches-button");
const reportsActiveJobsButton = document.getElementById("reports-active-jobs-button");
const reportsActiveLogsButton = document.getElementById("reports-active-logs-button");
const openSettingsButton = document.getElementById("open-settings-button");
const saveSettingsButton = document.getElementById("save-settings-button");
const settingsModal = document.getElementById("settings-modal");
const settingsClose = document.getElementById("settings-close");
const settingsCancel = document.getElementById("settings-cancel");
const modeSniperButton = document.getElementById("mode-sniper-button");
const modeVanityButton = document.getElementById("mode-vanity-button");
let vanityDerivedAddressPill = document.getElementById("mode-vanity-address");
let vanityDerivedPublicKey = "";
const devAutoSellButton = document.getElementById("dev-auto-sell-button");
const devAutoSellPanel = document.getElementById("dev-auto-sell-panel");
const autoSellEnabledInput = document.getElementById("auto-sell-enabled-input");
const autoSellToggleState = document.getElementById("auto-sell-toggle-state");
const autoSellTriggerFamilyValue = document.getElementById("auto-sell-trigger-family-value");
const autoSellTriggerValue = document.getElementById("auto-sell-trigger-value");
const autoSellTimeSettings = document.getElementById("auto-sell-time-settings");
const autoSellTriggerFamilyButtons = Array.from(document.querySelectorAll("[data-auto-sell-trigger-family]"));
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
const autoSellMarketCapEnabledInput = document.getElementById("auto-sell-market-cap-enabled-input");
const autoSellMarketCapSettings = document.getElementById("auto-sell-market-cap-settings");
const autoSellMarketCapThresholdInput = document.getElementById("auto-sell-market-cap-threshold-input");
const autoSellMarketCapThresholdValue = document.getElementById("auto-sell-market-cap-threshold-value");
const autoSellMarketCapTimeoutInput = document.getElementById("auto-sell-market-cap-timeout-input");
const autoSellMarketCapTimeoutActionInput = document.getElementById("auto-sell-market-cap-timeout-action-input");
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
let vampAutoImportTimer = null;
let vampInFlightAddress = "";
const OUTPUT_SECTION_VISIBILITY_KEY = "launchdeck.outputSectionVisible";
const REPORTS_TERMINAL_VISIBILITY_KEY = "launchdeck.reportsTerminalVisible";
const REPORTS_TERMINAL_LIST_WIDTH_KEY = "launchdeck.reportsTerminalListWidth";
const REPORTS_TERMINAL_VIEW_KEY = "launchdeck.reportsTerminalView";
const REPORTS_ACTIVE_LOGS_VIEW_KEY = "launchdeck.reportsActiveLogsView";
const THEME_MODE_STORAGE_KEY = "launchdeck.themeMode";
const SELECTED_WALLET_STORAGE_KEY = "launchdeck.selectedWalletKey";
const SELECTED_LAUNCHPAD_STORAGE_KEY = "launchdeck.selectedLaunchpad";
const SNIPER_DRAFT_STORAGE_KEY = "launchdeck.sniperDraft.v1";
const IMAGE_LAYOUT_COMPACT_STORAGE_KEY = "launchdeck.imageLayoutCompact";
const SELECTED_MODE_STORAGE_KEY = "launchdeck.selectedMode";
const SELECTED_BONK_QUOTE_ASSET_STORAGE_KEY = "launchdeck.bonkQuoteAsset";
const FEE_SPLIT_DRAFT_STORAGE_KEY = "launchdeck.feeSplitDraft.v1";
const AGENT_SPLIT_DRAFT_STORAGE_KEY = "launchdeck.agentSplitDraft.v1";
const AUTO_SELL_DRAFT_STORAGE_KEY = "launchdeck.autoSellDraft.v1";
let settingsModalInitialConfig = null;
const bagsIdentityModeInput = getNamedInput("bagsIdentityMode");
const bagsAgentUsernameHiddenInput = getNamedInput("bagsAgentUsername");
const bagsAuthTokenInput = getNamedInput("bagsAuthToken");
const bagsIdentityVerifiedWalletInput = getNamedInput("bagsIdentityVerifiedWallet");

const POPOUT_FORM_WIDTH = 532;
const POPOUT_REPORTS_WIDTH = 560;
const POPOUT_WORKSPACE_GAP = 12;
const POPOUT_WINDOW_NAME = "launchdeck-popout";
const pageSearchParams = new URLSearchParams(window.location.search);
const hasLegacyPopoutQuery = pageSearchParams.get("popout") === "1";
const isPopoutMode = window.name === POPOUT_WINDOW_NAME || hasLegacyPopoutQuery;
let popoutAutosizeFrame = 0;
let liveSyncTimer = 0;
let liveSyncReady = false;
let isApplyingLiveSync = false;
const LIVE_SYNC_CHANNEL_NAME = "launchdeck-live-sync.v1";
const LIVE_SYNC_STORAGE_KEY = "launchdeck.liveSyncEvent.v1";
const LIVE_SYNC_SESSION_STORAGE_KEY = "launchdeck.liveSyncSnapshot.v1";
const EARLY_BOOT_STORAGE_KEY = "launchdeck.earlyBootSnapshot.v1";
const EARLY_BOOT_SESSION_STORAGE_KEY = "launchdeck.earlyBootSnapshot.session.v1";
const LIVE_SYNC_MAX_AGE_MS = 5 * 60 * 1000;
const LIVE_SYNC_SOURCE_ID = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
const liveSyncChannel = typeof BroadcastChannel === "function"
  ? new BroadcastChannel(LIVE_SYNC_CHANNEL_NAME)
  : null;
const RequestUtils = window.LaunchDeckRequestUtils || {};
const RenderUtils = window.LaunchDeckRenderUtils || {};
const DEFAULT_LAUNCHPAD_TOKEN_METADATA = Object.freeze({
  nameMaxLength: 32,
  symbolMaxLength: 10,
});
const STANDARD_RPC_SLIPPAGE_DEFAULT = "20";

function readEarlyLiveSyncSnapshot() {
  const isFreshPayload = (payload) => {
    if (!payload || typeof payload !== "object") return false;
    const timestampMs = Number(payload.timestampMs);
    return Number.isFinite(timestampMs) && (Date.now() - timestampMs) <= LIVE_SYNC_MAX_AGE_MS;
  };
  try {
    if (window.opener && window.opener !== window) {
      const openerPayload = window.opener.__launchdeckLiveSyncSnapshot;
      if (isFreshPayload(openerPayload)) return openerPayload;
    }
  } catch (_error) {
    // Ignore opener access failures and continue with storage fallbacks.
  }
  try {
    const sessionRaw = window.sessionStorage.getItem(EARLY_BOOT_SESSION_STORAGE_KEY);
    if (sessionRaw) {
      const sessionPayload = JSON.parse(sessionRaw);
      if (isFreshPayload(sessionPayload)) return sessionPayload;
    }
  } catch (_error) {
    // Ignore session storage failures and continue with other fallbacks.
  }
  try {
    const localRaw = window.localStorage.getItem(EARLY_BOOT_STORAGE_KEY);
    if (localRaw) {
      const localPayload = JSON.parse(localRaw);
      if (isFreshPayload(localPayload)) return localPayload;
    }
  } catch (_error) {
    // Ignore localStorage failures and continue with live sync fallback.
  }
  try {
    const sessionRaw = window.sessionStorage.getItem(LIVE_SYNC_SESSION_STORAGE_KEY);
    if (sessionRaw) {
      const sessionPayload = JSON.parse(sessionRaw);
      if (isFreshPayload(sessionPayload)) return sessionPayload;
    }
  } catch (_error) {
    // Ignore session storage failures and continue with localStorage fallback.
  }
  try {
    const localRaw = window.localStorage.getItem(LIVE_SYNC_STORAGE_KEY);
    if (localRaw) {
      const localPayload = JSON.parse(localRaw);
      if (isFreshPayload(localPayload)) return localPayload;
    }
  } catch (_error) {
    // Ignore localStorage failures and keep boot functional.
  }
  return null;
}

const earlyLiveSyncSnapshot = readEarlyLiveSyncSnapshot();
if (earlyLiveSyncSnapshot) {
  window.__launchdeckEarlyLiveSyncSnapshot = earlyLiveSyncSnapshot;
}

if (isPopoutMode) {
  try {
    if (window.name !== POPOUT_WINDOW_NAME) window.name = POPOUT_WINDOW_NAME;
  } catch (_error) {
    // Ignore window.name failures and continue with static popout behavior.
  }
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
if (pageSearchParams.has("popout") || pageSearchParams.has("output") || pageSearchParams.has("reports")) {
  const cleanUrl = new URL(window.location.href);
  cleanUrl.searchParams.delete("popout");
  cleanUrl.searchParams.delete("output");
  cleanUrl.searchParams.delete("reports");
  try {
    window.history.replaceState(null, "", `${cleanUrl.pathname}${cleanUrl.search}${cleanUrl.hash}`);
  } catch (_error) {
    // Ignore history replacement failures and keep boot functional.
  }
}

setThemeMode(getStoredThemeMode(), { persist: false });
setOutputSectionVisible(
  getStoredOutputSectionVisible(),
);
setImageLayoutCompact(getStoredImageLayoutCompact(), { persist: false });

if (!isPopoutMode) {
  if (output) output.textContent = "";
  if (metaNode) metaNode.textContent = "";
  setStatusLabel("");
}

let uploadedImage = null;
let latestWalletStatus = null;
let latestRuntimeStatus = null;
let latestLaunchpadRegistry = {};
let bagsIdentityState = {
  mode: "wallet-only",
  configuredApiKey: false,
  verified: false,
  agentUsername: "",
  authToken: "",
  verifiedWallet: "",
  publicIdentifier: "",
  secret: "",
  verificationPostContent: "",
  error: "",
};
let importedCreatorFeeState = {
  mode: "",
  address: "",
  githubUsername: "",
  githubUserId: "",
};
let feeSplitModalSnapshot = null;
let feeSplitClearAllRestoreSnapshot = null;
let agentSplitClearAllRestoreSnapshot = null;
let walletStatusRequestSerial = 0;
let appBootstrapState = {
  started: false,
  staticLoaded: false,
  configLoaded: false,
  walletsLoaded: false,
  runtimeLoaded: false,
};
let startupWarmState = {
  started: false,
  ready: false,
  promise: null,
  enabled: true,
  backendLoaded: false,
  backendPayload: null,
  backendError: "",
};
const STARTUP_WARM_REQUEST_TIMEOUT_MS = 4000;
const STARTUP_WARM_WAIT_TIMEOUT_MS = 1500;
const STARTUP_WARM_CACHE_STORAGE_KEY = "launchdeck.startupWarmCache.v1";
const WALLET_STATUS_LAST_REFRESH_STORAGE_KEY = "launchdeck.walletStatusLastRefreshAtMs";
let walletStatusRefreshIntervalMs = 30000;
const RUNTIME_STATUS_REFRESH_INTERVAL_MS = 15000;
const STARTUP_WARM_CACHE_MAX_AGE_MS = RUNTIME_STATUS_REFRESH_INTERVAL_MS;
const WARM_ACTIVITY_DEBOUNCE_MS = 1000;
let quoteTimer = null;
let defaultsApplied = false;
const requestStates = {
  bootstrap: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  walletStatus: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  runtimeStatus: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  followJobs: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
  logs: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
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
  lastAlertedWarning: "",
};
let runtimeStatusRefreshTimer = null;
let walletStatusRefreshTimer = null;
let warmActivityState = {
  debounceTimer: null,
  inFlightPromise: null,
  lastSentAtMs: 0,
  /** When true, run another flush after the current POST finishes (latest DOM/config). */
  pendingFlush: false,
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
const DEV_BUY_QUOTE_CACHE_TTL_MS = 30_000;
const DEV_BUY_QUOTE_DEBOUNCE_MS = 100;
const devBuyQuoteCache = new Map();
const devBuyQuoteWarmInFlight = new Set();
let syncingPresetInputs = false;
let lastTopPresetMarkup = "";
let lastSettingsPresetMarkup = "";
let lastQuickDevBuyMarkup = "";
let reportsTerminalState = {
  allEntries: [],
  entries: [],
  launches: [],
  activeLogs: {
    live: [],
    errors: [],
    error: "",
    updatedAtMs: 0,
  },
  launchBundles: {},
  launchMetadataByUri: {},
  activeId: "",
  activePayload: null,
  activeBenchmarkReportId: "",
  activeBenchmarkSnapshot: null,
  activeText: "",
  activeTab: "overview",
  view: getStoredReportsTerminalView(),
  activeLogsView: getStoredActiveLogsView(),
  sort: "newest",
};
let reportsTerminalLoadSerial = 0;
let reportsTerminalResizeState = null;
let followJobsState = {
  configured: false,
  reachable: false,
  jobs: [],
  health: null,
  error: "",
  loaded: false,
  refreshTimer: null,
};
let outputFollowRefreshState = {
  serial: 0,
  timer: null,
  reportId: "",
  startedAtMs: 0,
};
const REPORTS_TERMINAL_DEFAULT_LIST_WIDTH = 152;
const REPORTS_TERMINAL_MIN_LIST_WIDTH = 120;
const REPORTS_TERMINAL_MAX_LIST_WIDTH = 240;
const REPORTS_TERMINAL_ITEM_LIMIT = 25;
const OUTPUT_FOLLOW_REFRESH_INTERVAL_MS = 1500;
const OUTPUT_FOLLOW_REFRESH_TIMEOUT_MS = 90000;
const FOLLOW_JOBS_REFRESH_INTERVAL_MS = 5000;
const FOLLOW_JOBS_OFFLINE_RETRY_MS = 15000;
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
  hellomoon: "Hello Moon QUIC",
  "standard-rpc": "Standard RPC",
  "jito-bundle": "Jito Bundle",
};
const ROUTE_CAPABILITIES = {
  "helius-sender": {
    creation: { tip: true, priority: true, slippage: false },
    buy: { tip: true, priority: true, slippage: true },
    sell: { tip: true, priority: true, slippage: true },
  },
  hellomoon: {
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
const PROVIDER_FEE_REQUIREMENTS = {
  "helius-sender": { minTipSol: 0.0002, priorityRequired: true },
  hellomoon: { minTipSol: 0.001, priorityRequired: true },
  "jito-bundle": { minTipSol: 0.0002, priorityRequired: true },
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

function setStatusLabel(label = "") {
  const normalized = String(label || "").trim();
  const hidden = !normalized || /^(idle|ready)$/i.test(normalized);
  if (statusNode) {
    statusNode.textContent = hidden ? "" : normalized;
    statusNode.hidden = hidden;
  }
  if (imageStatus) {
    imageStatus.textContent = hidden ? "" : normalized;
  }
}

function currentStatusLabel() {
  if (imageStatus && imageStatus.textContent.trim()) return imageStatus.textContent.trim();
  if (statusNode && statusNode.textContent.trim()) return statusNode.textContent.trim();
  return "";
}

function setBusy(busy, label) {
  setStatusLabel(label);
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

function setNamedChecked(name, checked) {
  const input = getNamedInput(name);
  if (input) input.checked = Boolean(checked);
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
  if ([
    "regular",
    "bonkers",
    "cashback",
    "agent-custom",
    "agent-unlocked",
    "agent-locked",
    "bags-2-2",
    "bags-025-1",
    "bags-1-025",
  ].includes(mode)) {
    return mode;
  }
  return "regular";
}

function defaultLaunchModeForLaunchpad(launchpad) {
  const normalizedLaunchpad = normalizeLaunchpad(launchpad);
  if (normalizedLaunchpad === "bagsapp") return "bags-2-2";
  return "regular";
}

function normalizeLaunchModeForLaunchpad(mode, launchpad = getLaunchpad()) {
  const normalizedMode = normalizeLaunchMode(mode);
  const allowedModes = getLaunchpadUiCapabilities(normalizeLaunchpad(launchpad)).allowedModes || ["regular"];
  return allowedModes.includes(normalizedMode)
    ? normalizedMode
    : (allowedModes[0] || defaultLaunchModeForLaunchpad(launchpad));
}

function normalizeLaunchpad(value) {
  const launchpad = String(value || "").trim().toLowerCase();
  if (["pump", "bonk", "bagsapp"].includes(launchpad)) {
    return launchpad;
  }
  return "pump";
}

function normalizeStoredBonkQuoteAsset(value) {
  return normalizeQuoteAsset(value);
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

function getStoredLaunchpad() {
  try {
    const stored = window.localStorage.getItem(SELECTED_LAUNCHPAD_STORAGE_KEY);
    return stored ? normalizeLaunchpad(stored) : "";
  } catch (_error) {
    return "";
  }
}

function setStoredLaunchpad(launchpad) {
  try {
    window.localStorage.setItem(SELECTED_LAUNCHPAD_STORAGE_KEY, normalizeLaunchpad(launchpad));
  } catch (_error) {
    // Ignore storage failures and keep launchpad controls functional.
  }
}

function getStoredBonkQuoteAsset() {
  try {
    const stored = window.localStorage.getItem(SELECTED_BONK_QUOTE_ASSET_STORAGE_KEY);
    return stored ? normalizeStoredBonkQuoteAsset(stored) : "";
  } catch (_error) {
    return "";
  }
}

function setStoredBonkQuoteAsset(asset) {
  try {
    window.localStorage.setItem(
      SELECTED_BONK_QUOTE_ASSET_STORAGE_KEY,
      normalizeStoredBonkQuoteAsset(asset),
    );
  } catch (_error) {
    // Ignore storage failures and keep quote asset controls functional.
  }
}

function serializeFeeSplitDraft() {
  return {
    enabled: Boolean(feeSplitEnabled && feeSplitEnabled.checked),
    suppressDefaultRow: feeSplitList ? feeSplitList.dataset.suppressDefaultRow === "true" : false,
    rows: getFeeSplitRows().map((row) => ({
      type: row.dataset.type || "wallet",
      value: row.querySelector(".recipient-target")?.value?.trim() || "",
      githubUserId: row.dataset.githubUserId || "",
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
      githubUserId: String(entry && entry.githubUserId || "").trim(),
      sharePercent: normalizeDecimalInput(entry && entry.sharePercent || "", 2),
      defaultReceiver: Boolean(entry && entry.defaultReceiver),
      targetLocked: Boolean(entry && entry.targetLocked),
    }))
    : [];
  return {
    enabled: Boolean(value && value.enabled),
    suppressDefaultRow: Boolean(value && value.suppressDefaultRow),
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
    if (draft.suppressDefaultRow) {
      feeSplitList.dataset.suppressDefaultRow = "true";
    } else {
      delete feeSplitList.dataset.suppressDefaultRow;
    }
    feeSplitList.innerHTML = "";
    draft.rows.forEach((entry) => {
      feeSplitList.appendChild(createFeeSplitRow(entry));
    });
  }
  if (draft.enabled) ensureFeeSplitDefaultRow();
  syncFeeSplitTotals();
  if (persist) setStoredFeeSplitDraft(draft);
}

function feeSplitClearAllDraft() {
  const deployerAddress = latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "";
  return normalizeFeeSplitDraft({
    enabled: true,
    suppressDefaultRow: false,
    rows: [{
      type: "wallet",
      value: deployerAddress,
      githubUserId: "",
      sharePercent: "100",
      defaultReceiver: true,
      targetLocked: true,
    }],
  });
}

function updateFeeSplitClearAllButton() {
  if (!feeSplitClearAll) return;
  feeSplitClearAll.textContent = feeSplitClearAllRestoreSnapshot ? "Restore All" : "Clear All";
}

function clearFeeSplitRestoreState() {
  feeSplitClearAllRestoreSnapshot = null;
  updateFeeSplitClearAllButton();
}

function agentSplitClearAllDraft() {
  return normalizeAgentSplitDraft({
    rows: [{
      locked: true,
      type: "wallet",
      value: "",
      sharePercent: "100",
      defaultReceiver: false,
      targetLocked: true,
    }],
  });
}

function updateAgentSplitClearAllButton() {
  if (!agentSplitClearAll) return;
  agentSplitClearAll.textContent = agentSplitClearAllRestoreSnapshot ? "Restore All" : "Clear All";
}

function clearAgentSplitRestoreState() {
  agentSplitClearAllRestoreSnapshot = null;
  updateAgentSplitClearAllButton();
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

function buildAgentSplitDraftFromFeeSplitDraft(value) {
  const draft = normalizeFeeSplitDraft(value);
  if (!draft.enabled && draft.rows.length === 0) {
    return normalizeAgentSplitDraft({ rows: [] });
  }
  const defaultReceiverRow = draft.rows.find((row) => row.defaultReceiver);
  const carriedRows = draft.rows
    .filter((row) => !row.defaultReceiver)
    .map((row) => ({
      locked: false,
      type: row.type === "github" ? "github" : "wallet",
      value: row.value,
      sharePercent: row.sharePercent,
      defaultReceiver: false,
      targetLocked: Boolean(row.targetLocked),
    }));
  if (!defaultReceiverRow && carriedRows.length === 0) {
    return normalizeAgentSplitDraft({ rows: [] });
  }
  return normalizeAgentSplitDraft({
    rows: [
      {
        locked: true,
        type: "wallet",
        value: "",
        sharePercent: defaultReceiverRow ? defaultReceiverRow.sharePercent : (carriedRows.length > 0 ? "0" : ""),
        defaultReceiver: false,
        targetLocked: true,
      },
      ...carriedRows,
    ],
  });
}

function buildFeeSplitDraftFromAgentSplitDraft(value) {
  const draft = normalizeAgentSplitDraft(value);
  if (draft.rows.length === 0) {
    return normalizeFeeSplitDraft({ enabled: false, rows: [] });
  }
  const agentRow = draft.rows.find((row) => row.locked || row.type === "agent");
  const deployerAddress = latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "";
  const carriedRows = draft.rows
    .filter((row) => !row.locked && row.type !== "agent")
    .map((row) => ({
      type: row.type === "github" ? "github" : "wallet",
      value: row.value,
      githubUserId: "",
      sharePercent: row.sharePercent,
      defaultReceiver: false,
      targetLocked: Boolean(row.targetLocked),
    }));
  if ((!agentRow || !agentRow.sharePercent) && carriedRows.length === 0) {
    return normalizeFeeSplitDraft({ enabled: false, rows: [] });
  }
  return normalizeFeeSplitDraft({
    enabled: carriedRows.length > 0 || Boolean(agentRow && agentRow.sharePercent),
    suppressDefaultRow: false,
    rows: [
      {
        type: "wallet",
        value: deployerAddress,
        githubUserId: "",
        sharePercent: agentRow ? agentRow.sharePercent : "",
        defaultReceiver: true,
        targetLocked: true,
      },
      ...carriedRows,
    ],
  });
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

function normalizeAutoSellDraft(value) {
  if (!value || typeof value !== "object") return null;
  const triggerFamily = normalizeAutoSellTriggerFamily(
    value.triggerFamily || ((Boolean(value.marketCapEnabled) || String(value.marketCapThreshold || "").trim()) ? "market-cap" : "time")
  );
  const legacyTimeoutMinutesRaw = String(value.marketCapScanTimeoutMinutes || "").trim();
  const timeoutSeconds = value.marketCapScanTimeoutSeconds != null && value.marketCapScanTimeoutSeconds !== ""
    ? Math.max(1, Math.min(86400, Math.round(Number(value.marketCapScanTimeoutSeconds || 30) || 30)))
    : (legacyTimeoutMinutesRaw
      ? Math.max(1, Math.min(86400, Math.round((Number(legacyTimeoutMinutesRaw || 15) || 15) * 60)))
      : 30);
  return {
    enabled: Boolean(value.enabled),
    percent: Math.max(1, Math.min(100, Number(value.percent || 100) || 100)),
    triggerFamily,
    triggerMode: normalizeAutoSellTriggerMode(value.triggerMode),
    delayMs: Math.max(0, Number(value.delayMs || 0) || 0),
    blockOffset: Math.max(0, Math.min(23, Math.round(Number(value.blockOffset || 0) || 0))),
    marketCapEnabled: triggerFamily === "market-cap" || Boolean(value.marketCapEnabled),
    marketCapThreshold: String(value.marketCapThreshold || "").trim(),
    marketCapScanTimeoutSeconds: timeoutSeconds,
    marketCapTimeoutAction: String(value.marketCapTimeoutAction || "").trim().toLowerCase() === "sell" ? "sell" : "stop",
  };
}

function getStoredAutoSellDraft() {
  try {
    const raw = window.localStorage.getItem(AUTO_SELL_DRAFT_STORAGE_KEY);
    if (!raw) return null;
    return normalizeAutoSellDraft(JSON.parse(raw));
  } catch (_error) {
    return null;
  }
}

function setStoredAutoSellDraft(value) {
  try {
    const normalized = normalizeAutoSellDraft(value);
    if (!normalized) {
      window.localStorage.removeItem(AUTO_SELL_DRAFT_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(AUTO_SELL_DRAFT_STORAGE_KEY, JSON.stringify(normalized));
  } catch (_error) {
    // Ignore localStorage write failures.
  }
}

function applyAutoSellDraft(value, { persist = false } = {}) {
  const draft = normalizeAutoSellDraft(value);
  if (!draft) return;
  if (autoSellEnabledInput) autoSellEnabledInput.checked = Boolean(draft.enabled);
  setNamedValue("automaticDevSellTriggerFamily", draft.triggerFamily);
  setNamedValue("automaticDevSellPercent", String(draft.percent));
  setNamedValue("automaticDevSellTriggerMode", draft.triggerMode);
  setNamedValue("automaticDevSellDelayMs", String(draft.delayMs));
  setNamedValue("automaticDevSellBlockOffset", String(draft.blockOffset));
  setNamedChecked("automaticDevSellMarketCapEnabled", draft.triggerFamily === "market-cap");
  setNamedValue("automaticDevSellMarketCapThreshold", draft.marketCapThreshold);
  setNamedValue("automaticDevSellMarketCapScanTimeoutSeconds", String(draft.marketCapScanTimeoutSeconds));
  setNamedValue("automaticDevSellMarketCapTimeoutAction", draft.marketCapTimeoutAction);
  syncDevAutoSellUI();
  if (persist) setStoredAutoSellDraft(draft);
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

function syncAgentSplitDraftFromFeeSplitDraft(value) {
  const draft = buildAgentSplitDraftFromFeeSplitDraft(value);
  applyAgentSplitDraft(draft, { persist: false });
  setStoredAgentSplitDraft(draft);
  return draft;
}

function syncFeeSplitDraftFromAgentSplitDraft(value) {
  const draft = buildFeeSplitDraftFromAgentSplitDraft(value);
  applyFeeSplitDraft(draft, { persist: false });
  setStoredFeeSplitDraft(draft);
  return draft;
}

function formatShareBpsAsPercent(shareBps) {
  const numeric = Number(shareBps);
  if (!Number.isFinite(numeric) || numeric <= 0) return "";
  const raw = (numeric / 100).toFixed(2);
  return raw.replace(/\.00$/, "").replace(/(\.\d*[1-9])0$/, "$1");
}

function parseGithubRecipientTarget(value) {
  const normalized = String(value || "").trim().replace(/^@+/, "");
  if (!normalized) {
    return { githubUsername: "", githubUserId: "" };
  }
  if (/^\d+$/.test(normalized)) {
    return { githubUsername: "", githubUserId: normalized };
  }
  return { githubUsername: normalized, githubUserId: "" };
}

function looksLikeSolanaAddress(value) {
  const normalized = String(value || "").trim();
  if (!normalized || normalized.startsWith("@")) return false;
  return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(normalized);
}

function normalizeImportedCreatorFeeState(value) {
  const mode = String(value && value.mode || "").trim().toLowerCase();
  return {
    mode,
    address: String(value && value.address || "").trim(),
    githubUsername: String(value && value.githubUsername || "").trim().replace(/^@+/, ""),
    githubUserId: String(value && value.githubUserId || "").trim(),
  };
}

function setImportedCreatorFeeState(value) {
  importedCreatorFeeState = normalizeImportedCreatorFeeState(value);
}

function buildFeeSplitDraftFromRecipients(recipients, { enabled = false } = {}) {
  return {
    enabled,
    suppressDefaultRow: Array.isArray(recipients) && recipients.length > 0,
    rows: Array.isArray(recipients)
      ? recipients.map((entry) => ({
        type: entry && entry.type === "github" ? "github" : "wallet",
        value: entry && entry.type === "github"
          ? String(entry.githubUsername || entry.githubUserId || "").trim().replace(/^@+/, "")
          : String(entry && entry.address || "").trim(),
        githubUserId: String(entry && entry.githubUserId || "").trim(),
        sharePercent: formatShareBpsAsPercent(entry && entry.shareBps),
        defaultReceiver: false,
        targetLocked: false,
      })).filter((entry) => entry.value || entry.sharePercent)
      : [],
  };
}

function buildAgentSplitDraftFromRecipients(recipients) {
  return {
    rows: Array.isArray(recipients)
      ? recipients.map((entry) => {
        const type = String(entry && entry.type || "wallet").trim().toLowerCase();
        if (type === "agent") {
          return {
            locked: true,
            type: "wallet",
            value: "",
            sharePercent: formatShareBpsAsPercent(entry && entry.shareBps),
            defaultReceiver: false,
            targetLocked: true,
          };
        }
        return {
          locked: false,
          type: type === "github" ? "github" : "wallet",
          value: type === "github"
            ? String(entry && entry.githubUsername || "").trim().replace(/^@+/, "")
            : String(entry && entry.address || "").trim(),
          sharePercent: formatShareBpsAsPercent(entry && entry.shareBps),
          defaultReceiver: false,
          targetLocked: false,
        };
      }).filter((entry) => entry.value || entry.sharePercent || entry.locked)
      : [],
  };
}

function applyImportedRouteState({
  launchpad = getLaunchpad(),
  feeSharingRecipients = null,
  agentFeeRecipients = null,
  creatorFee = null,
} = {}) {
  const nextLaunchpad = normalizeLaunchpad(launchpad);
  const feeRecipients = Array.isArray(feeSharingRecipients) ? feeSharingRecipients : [];
  const agentRecipients = Array.isArray(agentFeeRecipients) ? agentFeeRecipients : [];
  applyFeeSplitDraft(
    buildFeeSplitDraftFromRecipients(feeRecipients, {
      enabled: nextLaunchpad === "bagsapp" || feeRecipients.length > 0,
    }),
    { persist: false },
  );
  applyAgentSplitDraft(buildAgentSplitDraftFromRecipients(agentRecipients), { persist: false });
  setImportedCreatorFeeState(creatorFee || null);
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
    reportsTransactionsButton,
    reportsLaunchesButton,
    reportsActiveJobsButton,
    reportsActiveLogsButton,
  },
  storage: {
    visibilityKey: REPORTS_TERMINAL_VISIBILITY_KEY,
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
  refreshOnVisible: () => refreshReportsTerminal({ showLoading: false }),
  renderOutput: () => renderReportsTerminalOutput(),
  renderList: () => renderReportsTerminalList(),
    loadEntry: (id) => loadReportsTerminalEntry(id),
  refreshReports: (options) => refreshReportsTerminal(options),
  getView: () => reportsTerminalState.view,
  setView: (value) => {
    setReportsTerminalView(value);
  },
  reuseEntry: (id) => reuseFromHistory(id),
  relaunchEntry: (id) => relaunchFromHistory(id),
  normalizeTab: (tab) => normalizeReportsTerminalTab(tab),
  shortenAddress,
  openPopoutWindow,
});

reportsFeature.bindEvents();

if (reportsTerminalOutput) {
  reportsTerminalOutput.addEventListener("click", async (event) => {
    const reportTabButton = event.target.closest("[data-report-tab]");
    if (reportTabButton) {
      const nextTab = normalizeReportsTerminalTab(reportTabButton.getAttribute("data-report-tab"));
      if (nextTab !== reportsTerminalState.activeTab) {
        reportsTerminalState.activeTab = nextTab;
        renderReportsTerminalOutput();
        scheduleLiveSyncBroadcast();
      }
      return;
    }
    const activeLogsViewButton = event.target.closest("[data-active-logs-view]");
    if (activeLogsViewButton) {
      const nextView = normalizeActiveLogsView(activeLogsViewButton.getAttribute("data-active-logs-view"));
      if (nextView !== reportsTerminalState.activeLogsView) {
        setReportsActiveLogsView(nextView);
        renderReportsTerminalOutput();
        if (normalizeReportsTerminalView(reportsTerminalState.view) === "active-logs") {
          refreshActiveLogs({ showLoading: false }).catch(() => {});
        }
      }
      return;
    }
    const benchmarkPopoutButton = event.target.closest("[data-benchmark-popout]");
    if (benchmarkPopoutButton) {
      if (benchmarkPopoutButton.disabled) return;
      showBenchmarksPopoutModal();
      return;
    }
    const cancelAllButton = event.target.closest("[data-follow-cancel-all]");
    if (cancelAllButton) {
      if (cancelAllButton.disabled) return;
      if (!window.confirm("Cancel all active follow launches?")) return;
      try {
        await cancelAllFollowJobs();
        await refreshReportsTerminal().catch(() => {});
      } catch (error) {
        metaNode.textContent = error && error.message ? error.message : "Failed to cancel active follow launches.";
      }
      return;
    }
    const cancelButton = event.target.closest("[data-follow-cancel-trace-id]");
    if (!cancelButton) return;
    const traceId = cancelButton.getAttribute("data-follow-cancel-trace-id") || "";
    if (!traceId) return;
    if (!window.confirm("Cancel this follow launch?")) return;
    try {
      await cancelFollowJob(traceId, { note: "Cancelled from History launch card" });
      await refreshReportsTerminal().catch(() => {});
    } catch (error) {
      metaNode.textContent = error && error.message ? error.message : "Failed to cancel follow launch.";
    }
  });
}

function getStoredReportsTerminalListWidth() {
  return reportsFeature.getStoredListWidth();
}

function setReportsTerminalListWidth(width, options) {
  const result = reportsFeature.setListWidth(width, options);
  scheduleLiveSyncBroadcast();
  return result;
}

function setReportsTerminalVisible(isVisible, options) {
  const result = reportsFeature.setVisible(isVisible, options);
  syncReportsTerminalLayoutHeight();
  scheduleLiveSyncBroadcast({ immediate: true });
  return result;
}

function setReportsTerminalView(view, { persist = true } = {}) {
  reportsTerminalState.view = normalizeReportsTerminalView(view);
  if (persist) setStoredReportsTerminalView(reportsTerminalState.view);
  syncReportsTerminalChrome();
}

function setReportsActiveLogsView(view, { persist = true } = {}) {
  reportsTerminalState.activeLogsView = normalizeActiveLogsView(view);
  if (persist) setStoredActiveLogsView(reportsTerminalState.activeLogsView);
}
setReportsTerminalVisible(
  getStoredReportsTerminalVisible(),
  { persist: false },
);
setReportsTerminalListWidth(getStoredReportsTerminalListWidth(), { persist: false });
syncReportsTerminalChrome();
window.addEventListener("resize", () => {
  syncReportsTerminalLayoutHeight();
});

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

function hasAttachedImage() {
  return Boolean(
    uploadedImage
    || (metadataUri && metadataUri.value)
    || (imagePreview && !imagePreview.hidden && imagePreview.src),
  );
}

async function ensureTestImageSelected() {
  let availableImages = Array.isArray(imageLibraryState.images) ? [...imageLibraryState.images] : [];
  if (!availableImages.length) {
    try {
      const response = await fetch("/api/images");
      const payload = await response.json();
      if (response.ok && payload.ok) {
        imageLibraryState.images = Array.isArray(payload.images) ? payload.images : [];
        imageLibraryState.categories = Array.isArray(payload.categories) ? payload.categories : [];
        availableImages = imageLibraryState.images;
      }
    } catch (_error) {
      // Fall through when the library fetch fails.
    }
  }

  const preferred =
    availableImages.find((entry) => entry && entry.fileName === "solana-mark.png")
    || availableImages.find((entry) => entry && entry.previewUrl === "/solana-mark.png")
    || availableImages[0];

  if (!preferred) return false;
  imageLibraryState.activeImageId = preferred.id || "";
  setSelectedImage(preferred);
  return true;
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
    autoSellTriggerFamilyValue,
    autoSellTriggerValue,
    autoSellTimeSettings,
    autoSellTriggerFamilyButtons,
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
    autoSellMarketCapEnabledInput,
    autoSellMarketCapSettings,
    autoSellMarketCapThresholdInput,
    autoSellMarketCapThresholdValue,
    autoSellMarketCapTimeoutInput,
    autoSellMarketCapTimeoutActionInput,
  },
  getNamedValue,
  setNamedValue,
  isNamedChecked,
  formatSliderValue,
  syncSettingsCapabilities,
  syncActivePresetFromInputs,
  validateFieldByName,
  setNamedChecked,
  documentNode: document,
  persistDraft: () => setStoredAutoSellDraft({
    enabled: isNamedChecked("automaticDevSellEnabled"),
    percent: getNamedValue("automaticDevSellPercent") || "100",
    triggerFamily: getNamedValue("automaticDevSellTriggerFamily") || "time",
    triggerMode: getNamedValue("automaticDevSellTriggerMode") || "block-offset",
    delayMs: getNamedValue("automaticDevSellDelayMs") || "0",
    blockOffset: getNamedValue("automaticDevSellBlockOffset") || "0",
    marketCapEnabled: (getNamedValue("automaticDevSellTriggerFamily") || "time") === "market-cap",
    marketCapThreshold: getNamedValue("automaticDevSellMarketCapThreshold") || "",
    marketCapScanTimeoutSeconds: getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds")
      || getNamedValue("automaticDevSellMarketCapScanTimeoutMinutes")
      || "30",
    marketCapTimeoutAction: getNamedValue("automaticDevSellMarketCapTimeoutAction") || "stop",
  }),
});

autoSellFeature.bindEvents();

function normalizeAutoSellTriggerMode(value) {
  return autoSellFeature.normalizeTriggerMode(value);
}

function normalizeAutoSellTriggerFamily(value) {
  return autoSellFeature.normalizeTriggerFamily(value);
}

function getAutoSellTriggerFamily() {
  return autoSellFeature.getTriggerFamily();
}

function getAutoSellTriggerMode() {
  return autoSellFeature.getTriggerMode();
}

function parseAutoSellMarketCapThreshold(value) {
  return autoSellFeature.parseMarketCapThreshold(value);
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

function hydrateDevAutoSellState() {
  const storedDraft = getStoredAutoSellDraft();
  if (storedDraft) {
    applyAutoSellDraft(storedDraft, { persist: false });
    return;
  }
  syncDevAutoSellUI();
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
    setStatusLabel("Error");
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
          mevMode: "off",
          autoFee: false,
          maxFeeSol: "",
          devBuySol: amount,
        },
        buySettings: {
          provider: "helius-sender",
          priorityFeeSol: "0.009",
          tipSol: "0.01",
          slippagePercent: "90",
          mevMode: "off",
          autoFee: false,
          maxFeeSol: "",
          snipeBuyAmountSol: "",
        },
        sellSettings: {
          provider: "helius-sender",
          priorityFeeSol: "0.009",
          tipSol: "0.01",
          slippagePercent: "90",
          mevMode: "off",
          autoFee: false,
          maxFeeSol: "",
        },
        automaticDevSell: {
          enabled: false,
          percent: 100,
          triggerFamily: "time",
          triggerMode: "block-offset",
          delayMs: 0,
          targetBlockOffset: 0,
          marketCapEnabled: false,
          marketCapThreshold: "",
          marketCapScanTimeoutSeconds: 30,
          marketCapTimeoutAction: "stop",
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

function isTrackSendBlockHeightEnabled(config = getConfig()) {
  return Boolean(
    config
    && config.defaults
    && config.defaults.misc
    && config.defaults.misc.trackSendBlockHeight,
  );
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
  scheduleLiveSyncBroadcast({ immediate: true });
}

function normalizeAutoFeeCapValue(value) {
  const trimmed = String(value || "").trim();
  if (!trimmed) return "";
  const numeric = Number(trimmed);
  if (!Number.isFinite(numeric)) return trimmed;
  return numeric <= 0 ? "" : trimmed;
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

function formatWarmProviders(values = []) {
  const normalized = Array.isArray(values)
    ? values.map((value) => String(value || "").trim()).filter(Boolean)
    : [];
  return normalized.length ? normalized.join(" | ") : "--";
}

function formatWarmTimestamp(value) {
  const timestampMs = Number(value || 0);
  if (!timestampMs) return "--";
  const date = new Date(timestampMs);
  if (Number.isNaN(date.getTime())) return "--";
  return date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function normalizeWatcherHealth(value) {
  return String(value || "").trim().toLowerCase();
}

function truncateStatusText(value, maxLength = 140) {
  const text = String(value || "").trim();
  if (!text) return "";
  return text.length > maxLength ? `${text.slice(0, maxLength - 3)}...` : text;
}

function providerFeeRequirements(provider) {
  return PROVIDER_FEE_REQUIREMENTS[String(provider || "").trim().toLowerCase()] || null;
}

function providerMinimumTipSol(provider) {
  const requirements = providerFeeRequirements(provider);
  return requirements ? Number(requirements.minTipSol || 0) : 0;
}

function providerRequiresPriorityFee(provider) {
  const requirements = providerFeeRequirements(provider);
  return Boolean(requirements && requirements.priorityRequired);
}

function providerRequirementLabel(provider) {
  const normalized = String(provider || "").trim().toLowerCase();
  return PROVIDER_LABELS[normalized] || normalized || "selected provider";
}

function validateNonNegativeSolField(value) {
  if (!value) return "";
  const n = Number(value);
  if (isNaN(n) || n < 0) return "Must be a valid number";
  return "";
}

function validateRequiredPriorityFeeField(value, provider) {
  const label = providerRequirementLabel(provider);
  if (!value) return `Priority fee is required for ${label}.`;
  const n = Number(value);
  if (isNaN(n) || n <= 0) return `Priority fee must be greater than 0 for ${label}.`;
  return "";
}

function validateRequiredTipField(value, provider) {
  const label = providerRequirementLabel(provider);
  const minimumTipSol = providerMinimumTipSol(provider);
  if (!value) return `Tip is required for ${label}.`;
  const n = Number(value);
  if (isNaN(n) || n < 0) return "Must be a valid number";
  if (minimumTipSol > 0 && n < minimumTipSol) {
    return `Tip must be at least ${minimumTipSol.toFixed(4)} SOL for ${label}.`;
  }
  return "";
}

function validateOptionalAutoFeeCapField(value, provider) {
  if (!value) return "";
  const n = Number(value);
  if (isNaN(n) || n <= 0) return "Must be greater than 0";
  const minimumTipSol = providerMinimumTipSol(provider);
  if (minimumTipSol > 0 && n < minimumTipSol) {
    return `Max auto fee must be at least ${minimumTipSol.toFixed(4)} SOL for ${providerRequirementLabel(provider)}.`;
  }
  return "";
}

function normalizeWarmTargets(values) {
  return Array.isArray(values)
    ? values.filter((value) => value && typeof value === "object")
    : [];
}

function tonePriority(tone) {
  switch (String(tone || "").trim()) {
    case "red":
      return 5;
    case "yellow":
      return 4;
    case "blue":
      return 3;
    case "green":
      return 2;
    case "gray":
    default:
      return 1;
  }
}

function strongestIndicatorTone(tones = []) {
  return tones
    .map((tone) => String(tone || "").trim())
    .filter(Boolean)
    .sort((left, right) => tonePriority(right) - tonePriority(left))[0] || "gray";
}

function formatWarmTargetName(target) {
  const label = String(target && target.label || target && target.provider || "Target").trim() || "Target";
  const rawTarget = String(target && target.target || "").trim();
  if (!rawTarget) return label;
  const normalizedTarget = /^https?:\/\//i.test(rawTarget) || /^wss?:\/\//i.test(rawTarget)
    ? shortenReportEndpoint(rawTarget)
    : rawTarget;
  return `${label} (${normalizedTarget})`;
}

function summarizeWarmFailures(targets, limit = 2) {
  const items = targets.slice(0, limit).map((target) => {
    const name = formatWarmTargetName(target);
    const error = truncateStatusText(target && target.lastError || "", 100);
    return error ? `${name}: ${error}` : name;
  });
  if (targets.length > limit) {
    items.push(`+${targets.length - limit} more`);
  }
  return items.join(" | ");
}

function summarizeWarmRateLimits(targets, limit = 2) {
  const items = targets.slice(0, limit).map((target) => {
    const name = formatWarmTargetName(target);
    const message = truncateStatusText(target && target.lastRateLimitMessage || "", 100);
    return message ? `${name}: ${message}` : `${name}: reachable but rate-limited`;
  });
  if (targets.length > limit) {
    items.push(`+${targets.length - limit} more`);
  }
  return items.join(" | ");
}

function latestWarmTargetSuccess(targets) {
  const values = targets
    .map((target) => Number(target && target.lastSuccessAtMs || 0))
    .filter((value) => Number.isFinite(value) && value > 0);
  return values.length ? Math.max(...values) : 0;
}

function uniqueWarmTargetProviders(targets) {
  return Array.from(new Set(
    targets
      .map((target) => String(target && target.provider || "").trim())
      .filter(Boolean),
  ));
}

function summarizeStartupWarmFailures(targets, limit = 2) {
  const items = targets.slice(0, limit).map((target) => {
    const label = String(target && target.label || "Target").trim() || "Target";
    const error = truncateStatusText(target && target.error || "", 100);
    return error ? `${label}: ${error}` : label;
  });
  if (targets.length > limit) {
    items.push(`+${targets.length - limit} more`);
  }
  return items.join(" | ");
}

function buildWarmTooltipRowsFromTargets(targets, nowMs) {
  return normalizeWarmTargets(targets).map((target) => {
    const label = formatWarmTargetName(target);
    if (String(target && target.lastError || "").trim()) {
      return {
        tone: "red",
        label,
        detail: truncateStatusText(target.lastError || "", 120) || "Failed",
      };
    }
    if (isWarmTelemetryTargetRateLimited(target)) {
      return {
        tone: "yellow",
        label,
        detail: truncateStatusText(target.lastRateLimitMessage || "", 120) || "Reachable but rate-limited",
      };
    }
    if (isWarmTelemetryTargetHealthy(target, nowMs)) {
      return {
        tone: "green",
        label,
        detail: `Healthy${target && target.lastSuccessAtMs ? ` • last success ${formatWarmTimestamp(target.lastSuccessAtMs)}` : ""}`,
      };
    }
    if (target && target.active) {
      return {
        tone: "yellow",
        label,
        detail: `Waiting for a fresh success probe (>${Math.round(WARM_TELEMETRY_FRESH_MS / 1000)}s)`,
      };
    }
    return {
      tone: "gray",
      label,
      detail: "Inactive",
    };
  });
}

function buildStartupWarmTooltipRows(targets) {
  return Array.isArray(targets)
    ? targets.map((target) => ({
        tone: target && target.ok ? "green" : String(target && target.error || "").trim() ? "red" : "yellow",
        label: String(target && target.label || "Target").trim() || "Target",
        detail: target && target.ok
          ? "Healthy"
          : truncateStatusText(target && target.error || "", 120) || "Waiting",
      }))
    : [];
}

function renderRuntimeIndicatorTooltipSections(sections = []) {
  return sections
    .filter((section) => section && Array.isArray(section.rows) && section.rows.length > 0)
    .map((section) => {
      const rowsMarkup = section.rows.map((row) => {
        const tone = ["green", "yellow", "blue", "red", "gray"].includes(row && row.tone) ? row.tone : "gray";
        const label = String(row && row.label || "Item").trim() || "Item";
        const detail = String(row && row.detail || "").trim();
        return `
          <div class="runtime-indicator-tooltip-row">
            <span class="runtime-indicator-tooltip-dot is-${tone}" aria-hidden="true"></span>
            <div class="runtime-indicator-tooltip-copy">
              <div class="runtime-indicator-tooltip-row-label">${escapeHTML(label)}</div>
              ${detail ? `<div class="runtime-indicator-tooltip-row-detail">${escapeHTML(detail)}</div>` : ""}
            </div>
          </div>
        `;
      }).join("");
      return `
        <div class="runtime-indicator-tooltip-section">
          <div class="runtime-indicator-tooltip-head">${escapeHTML(String(section.title || "").trim() || "Status")}</div>
          ${rowsMarkup}
        </div>
      `;
    }).join("");
}

function describeWarmReason(reason) {
  switch (String(reason || "").trim()) {
    case "disabled-by-env":
      return "disabled by settings";
    case "suspended-idle":
      return "paused automatically because the app is idle";
    case "idle-awaiting-browser-activity":
      return "waiting for activity before warming";
    case "active-in-flight-request":
      return "active because requests are in flight";
    case "active-operator-activity":
      return "active because the app is in use";
    default:
      return String(reason || "").trim() || "waiting";
  }
}

function isWarmAutoPausedReason(reason) {
  const normalized = String(reason || "").trim();
  return normalized === "suspended-idle" || normalized === "idle-awaiting-browser-activity";
}

function walletRefreshPausedByIdleSuspend() {
  const warm = latestRuntimeStatus && latestRuntimeStatus.warm && typeof latestRuntimeStatus.warm === "object"
    ? latestRuntimeStatus.warm
    : null;
  if (!warm || warm.idleSuspendEnabled === false) return false;
  if (warm.active === true) return false;
  return Boolean(warm.suspended) || isWarmAutoPausedReason(warm.reason);
}

function clearWalletStatusRefreshTimer() {
  if (!walletStatusRefreshTimer) return;
  window.clearTimeout(walletStatusRefreshTimer);
  walletStatusRefreshTimer = null;
}

function syncWalletStatusRefreshLoop({ immediateResume = false } = {}) {
  if (walletRefreshPausedByIdleSuspend()) {
    clearWalletStatusRefreshTimer();
    return;
  }
  if (walletStatusRefreshTimer) return;
  if (immediateResume) {
    refreshWalletStatus(true, true).catch(() => {});
    return;
  }
  scheduleWalletStatusRefresh();
}

/** Fresh success window (~3× default 10s keep-warm interval, capped). */
const WARM_TELEMETRY_FRESH_MS = 180000;

function warmTargetFreshSuccess(target, refNowMs) {
  const t = Number(target && target.lastSuccessAtMs || 0);
  return Number.isFinite(t) && t > 0 && (refNowMs - t) <= WARM_TELEMETRY_FRESH_MS;
}

function isWarmTelemetryTargetRateLimited(target) {
  return String(target && target.status || "").trim() === "rate-limited";
}

function isWarmTelemetryTargetHealthy(target, refNowMs) {
  if (isWarmTelemetryTargetRateLimited(target)) {
    return false;
  }
  if (String(target && target.lastError || "").trim()) {
    return false;
  }
  return warmTargetFreshSuccess(target, refNowMs);
}

function startupWarmSnapshot() {
  if (!startupWarmState.enabled) {
    return {
      disabled: true,
      stateTargets: [],
      endpointTargets: [],
      watchTargets: [],
      stateFailures: [],
      endpointFailures: [],
      watchFailures: [],
      error: "",
    };
  }
  const payload = startupWarmState.backendPayload && typeof startupWarmState.backendPayload === "object"
    ? startupWarmState.backendPayload
    : null;
  const summary = payload && payload.startupWarm && typeof payload.startupWarm === "object"
    ? payload.startupWarm
    : null;
  const stateTargets = summary && summary.stateTargets && typeof summary.stateTargets === "object"
    ? Array.from({ length: Number(summary.stateTargets.total || 0) }, (_, index) => ({
        label: `Startup state target ${index + 1}`,
        ok: index < Number(summary.stateTargets.healthy || 0),
        error: "",
      }))
    : payload ? [
        { label: "Lookup tables", ok: Boolean(payload.lookupTables && payload.lookupTables.ok), error: payload.lookupTables && payload.lookupTables.error ? String(payload.lookupTables.error) : "" },
        { label: "Pump global", ok: Boolean(payload.pumpGlobal && payload.pumpGlobal.ok), error: payload.pumpGlobal && payload.pumpGlobal.error ? String(payload.pumpGlobal.error) : "" },
        { label: "Bonk state", ok: Boolean(payload.bonkState && payload.bonkState.ok), error: payload.bonkState && payload.bonkState.error ? String(payload.bonkState.error) : "" },
        { label: "Fee market", ok: Boolean(payload.feeMarket && payload.feeMarket.ok), error: payload.feeMarket && payload.feeMarket.error ? String(payload.feeMarket.error) : "" },
      ].filter((target) => target.ok || target.error) : [];
  const endpointTargets = summary && summary.endpointTargets && typeof summary.endpointTargets === "object"
    ? Array.from({ length: Number(summary.endpointTargets.total || 0) }, (_, index) => ({
        label: String(summary.endpointTargets.label || "Endpoint warm").trim() || "Endpoint warm",
        ok: index < Number(summary.endpointTargets.healthy || 0),
        error: "",
      }))
    : payload && Array.isArray(payload.heliusSender)
      ? payload.heliusSender.map((entry) => ({
          label: String(entry && entry.endpoint || "Helius Sender").trim() || "Helius Sender",
          ok: Boolean(entry && entry.ok),
          error: entry && entry.error ? String(entry.error) : "",
        }))
      : [];
  const watchTargets = summary && summary.watchTargets && typeof summary.watchTargets === "object"
    ? Array.from({ length: Number(summary.watchTargets.total || 0) }, (_, index) => ({
        label: String(summary.watchTargets.label || "Watcher WS warm").trim() || "Watcher WS warm",
        ok: index < Number(summary.watchTargets.healthy || 0),
        error: "",
      }))
    : [];
  const stateFailures = summary && Array.isArray(summary.stateFailures)
    ? summary.stateFailures.map((entry) => ({
        label: String(entry && entry.label || "Startup state target").trim() || "Startup state target",
        error: entry && entry.error ? String(entry.error) : "",
      }))
    : [];
  const endpointFailures = summary && Array.isArray(summary.endpointFailures)
    ? summary.endpointFailures.map((entry) => ({
        label: String(entry && entry.label || summary && summary.endpointTargets && summary.endpointTargets.label || "Endpoint warm").trim() || "Endpoint warm",
        error: entry && entry.error ? String(entry.error) : "",
      }))
    : [];
  const watchFailures = summary && Array.isArray(summary.watchFailures)
    ? summary.watchFailures.map((entry) => ({
        label: String(entry && entry.label || summary && summary.watchTargets && summary.watchTargets.label || "Watcher WS warm").trim() || "Watcher WS warm",
        error: entry && entry.error ? String(entry.error) : "",
      }))
    : [];
  return {
    disabled: false,
    stateTargets,
    endpointTargets,
    watchTargets,
    stateFailures,
    endpointFailures,
    watchFailures,
    error: String(startupWarmState.backendError || "").trim(),
  };
}

function currentWatchPathSnapshot() {
  const runtimeFollow = runtimeFollowDaemonStatus();
  const health = followStatusSnapshot().health || null;
  const watcherHealthValues = [
    health && health.signatureWatcher,
    health && health.slotWatcher,
    health && health.marketWatcher,
  ].map(normalizeWatcherHealth).filter(Boolean);
  const watcherMode = [
    health && health.signatureWatcherMode,
    health && health.slotWatcherMode,
    health && health.marketWatcherMode,
  ].map((value) => String(value || "").trim()).find(Boolean) || "";
  const endpoint = health && health.watchEndpoint ? String(health.watchEndpoint).trim() : "";
  if (!runtimeFollow || runtimeFollow.configured === false) {
    return {
      tone: "gray",
      title: "Watch Path: disabled.",
    };
  }
  if (!runtimeFollow.reachable) {
    const reason = String(runtimeFollow.error || followJobsState.error || "Follow daemon is unreachable.").trim();
    return {
      tone: "red",
      title: `Watch Path: offline. ${reason}`,
    };
  }
  const healthyWatchers = watcherHealthValues.filter((value) => value === "healthy").length;
  const failedWatchers = watcherHealthValues.filter((value) => value === "failed").length;
  const degradedWatchers = watcherHealthValues.filter((value) => value === "degraded").length;
  if (!endpoint) {
    return {
      tone: failedWatchers > 0 || degradedWatchers > 0 ? "yellow" : "green",
      title: failedWatchers > 0 || degradedWatchers > 0
        ? "Watch Path: partial. Follow daemon is reachable, but watcher health is mixed and no active endpoint is reported yet."
        : "Watch Path: healthy. Ready, but no active watcher is running right now.",
    };
  }
  if (failedWatchers > 0 || degradedWatchers > 0) {
    const modeLabel = watcherMode || "websocket";
    return {
      tone: healthyWatchers > 0 ? "yellow" : "red",
      title: healthyWatchers > 0
        ? `Watch Path: partial. ${modeLabel} via ${shortenReportEndpoint(endpoint)}.`
        : `Watch Path: failing. ${modeLabel} via ${shortenReportEndpoint(endpoint)}.`,
    };
  }
  if (watcherMode === "helius-transaction-subscribe") {
    return {
      tone: "green",
      title: `Watch Path: healthy. Helius transactionSubscribe via ${shortenReportEndpoint(endpoint)}.`,
    };
  }
  return {
    tone: "green",
    title: `Watch Path: healthy. Standard websocket via ${shortenReportEndpoint(endpoint)}.`,
  };
}

function buildPlatformRuntimeIndicatorState() {
  const warm = latestRuntimeStatus && latestRuntimeStatus.warm && typeof latestRuntimeStatus.warm === "object"
    ? latestRuntimeStatus.warm
    : null;
  const startupWarm = startupWarmSnapshot();
  const useStartupWarmFallback = !appBootstrapState.runtimeLoaded && startupWarmState.backendLoaded;
  const nowMs = Date.now();
  const stateTargets = normalizeWarmTargets(warm && warm.stateTargets);
  const endpointTargets = normalizeWarmTargets(warm && warm.endpointTargets);
  const watchTargets = normalizeWarmTargets(warm && warm.watchTargets);
  const activeStateTargets = stateTargets.filter((target) => Boolean(target && target.active));
  const activeEndpointTargets = endpointTargets.filter((target) => Boolean(target && target.active));
  const activeWatchTargets = watchTargets.filter((target) => Boolean(target && target.active));
  const healthyStateTargets = stateTargets.filter((target) => isWarmTelemetryTargetHealthy(target, nowMs));
  const healthyEndpointTargets = endpointTargets.filter((target) => isWarmTelemetryTargetHealthy(target, nowMs));
  const healthyWatchTargets = watchTargets.filter((target) => isWarmTelemetryTargetHealthy(target, nowMs));
  const rateLimitedActiveStateTargets = activeStateTargets.filter((target) => isWarmTelemetryTargetRateLimited(target));
  const rateLimitedActiveEndpointTargets = activeEndpointTargets.filter((target) => isWarmTelemetryTargetRateLimited(target));
  const rateLimitedActiveWatchTargets = activeWatchTargets.filter((target) => isWarmTelemetryTargetRateLimited(target));
  const staleActiveStateTargets = activeStateTargets.filter((target) => !isWarmTelemetryTargetHealthy(target, nowMs) && !isWarmTelemetryTargetRateLimited(target) && !String(target && target.lastError || "").trim());
  const staleActiveEndpointTargets = activeEndpointTargets.filter((target) => !isWarmTelemetryTargetHealthy(target, nowMs) && !isWarmTelemetryTargetRateLimited(target) && !String(target && target.lastError || "").trim());
  const staleActiveWatchTargets = activeWatchTargets.filter((target) => !isWarmTelemetryTargetHealthy(target, nowMs) && !isWarmTelemetryTargetRateLimited(target) && !String(target && target.lastError || "").trim());
  const failingActiveStateTargets = activeStateTargets.filter((target) => String(target && target.lastError || "").trim());
  const failingActiveEndpointTargets = activeEndpointTargets.filter((target) => String(target && target.lastError || "").trim());
  const failingActiveWatchTargets = activeWatchTargets.filter((target) => String(target && target.lastError || "").trim());
  const failingStateTargets = stateTargets.filter((target) => String(target && target.lastError || "").trim());
  const failingEndpointTargets = endpointTargets.filter((target) => String(target && target.lastError || "").trim());
  const failingWatchTargets = watchTargets.filter((target) => String(target && target.lastError || "").trim());
  const endpointTargetProviders = uniqueWarmTargetProviders(activeEndpointTargets.length ? activeEndpointTargets : endpointTargets);
  const senderConnectionWarm = (endpointTargetProviders.length === 1 && endpointTargetProviders[0] === "helius-sender")
    || (!endpointTargetProviders.length && startupWarm.endpointTargets.length > 0);
  const endpointWarmLabel = senderConnectionWarm ? "Sender connection warm" : "Endpoint prewarm";
  const warmProviders = warm && Array.isArray(warm.selectedProviders)
    ? warm.selectedProviders.map((value) => String(value || "").trim()).filter(Boolean)
    : [];
  const lastWarmSuccess = warm ? formatWarmTimestamp(warm.lastWarmSuccessAtMs) : "--";
  const warmReason = warm && warm.reason ? String(warm.reason).trim() : "";
  const warmError = warm && warm.lastError ? String(warm.lastError).trim() : "";
  const warmDisabledByUser = warm && warm.continuousEnabled === false && warm.startupEnabled === false;
  const startupWarmInProgress = startupWarmState.enabled && startupWarmState.started && !startupWarmState.ready;
  const startupStateFailures = startupWarm.stateFailures && startupWarm.stateFailures.length
    ? startupWarm.stateFailures
    : startupWarm.stateTargets.filter((target) => !target.ok && target.error);
  const startupEndpointFailures = startupWarm.endpointFailures && startupWarm.endpointFailures.length
    ? startupWarm.endpointFailures
    : startupWarm.endpointTargets.filter((target) => !target.ok && target.error);
  const startupWatchFailures = startupWarm.watchFailures && startupWarm.watchFailures.length
    ? startupWarm.watchFailures
    : startupWarm.watchTargets.filter((target) => !target.ok && target.error);

  let stateWarm = {
    tone: startupWarmInProgress ? "yellow" : "gray",
    title: startupWarmInProgress ? "State Warm: starting." : "State Warm: disabled.",
  };
  if (useStartupWarmFallback) {
    if (startupWarm.disabled) {
      stateWarm = {
        tone: "gray",
        title: "State Warm: disabled by user.",
      };
    } else if (startupStateFailures.length > 0) {
      stateWarm = {
        tone: startupStateFailures.length < startupWarm.stateTargets.length ? "yellow" : "red",
        title: startupStateFailures.length < startupWarm.stateTargets.length
          ? `State Warm: partial. ${startupStateFailures.length}/${startupWarm.stateTargets.length} startup target${startupWarm.stateTargets.length === 1 ? "" : "s"} failed. ${summarizeStartupWarmFailures(startupStateFailures)}`
          : `State Warm: failing. ${startupStateFailures.length}/${startupWarm.stateTargets.length} startup target${startupWarm.stateTargets.length === 1 ? "" : "s"} failed. ${summarizeStartupWarmFailures(startupStateFailures)}`,
      };
    } else if (startupWarm.stateTargets.length > 0) {
      stateWarm = {
        tone: "green",
        title: `State Warm: healthy. ${startupWarm.stateTargets.length} startup target${startupWarm.stateTargets.length === 1 ? "" : "s"} succeeded.`,
      };
    } else if (startupWarm.error) {
      stateWarm = {
        tone: "red",
        title: `State Warm: failing. ${startupWarm.error}`,
      };
    }
  } else if (warm) {
    if (warmDisabledByUser) {
      stateWarm = {
        tone: "gray",
        title: "State Warm: disabled by user.",
      };
    } else if (failingActiveStateTargets.length > 0) {
      stateWarm = {
        tone: failingActiveStateTargets.length < activeStateTargets.length ? "yellow" : "red",
        title: failingActiveStateTargets.length < activeStateTargets.length
          ? `State Warm: partial. ${failingActiveStateTargets.length}/${activeStateTargets.length} active target${activeStateTargets.length === 1 ? "" : "s"} failed. ${summarizeWarmFailures(failingActiveStateTargets)}`
          : `State Warm: failing. ${failingActiveStateTargets.length}/${activeStateTargets.length} active target${activeStateTargets.length === 1 ? "" : "s"} failed. ${summarizeWarmFailures(failingActiveStateTargets)}`,
      };
    } else if (rateLimitedActiveStateTargets.length > 0) {
      stateWarm = {
        tone: "yellow",
        title: rateLimitedActiveStateTargets.length < activeStateTargets.length
          ? `State Warm: degraded. ${rateLimitedActiveStateTargets.length}/${activeStateTargets.length} active target${activeStateTargets.length === 1 ? "" : "s"} reachable but rate-limited. ${summarizeWarmRateLimits(rateLimitedActiveStateTargets)}`
          : `State Warm: rate-limited. ${rateLimitedActiveStateTargets.length}/${activeStateTargets.length} active target${activeStateTargets.length === 1 ? "" : "s"} reachable but rate-limited. ${summarizeWarmRateLimits(rateLimitedActiveStateTargets)}`,
      };
    } else if (staleActiveStateTargets.length > 0) {
      stateWarm = {
        tone: "yellow",
        title: `State Warm: waiting. ${staleActiveStateTargets.length} active target${staleActiveStateTargets.length === 1 ? "" : "s"} without a fresh success probe (>${Math.round(WARM_TELEMETRY_FRESH_MS / 1000)}s).`,
      };
    } else if (activeStateTargets.length > 0) {
      const latestSuccess = formatWarmTimestamp(latestWarmTargetSuccess(activeStateTargets));
      stateWarm = {
        tone: "green",
        title: `State Warm: healthy. ${activeStateTargets.length} active target${activeStateTargets.length === 1 ? "" : "s"}. Last success ${latestSuccess}.`,
      };
    } else if (warm.active && healthyStateTargets.length > 0 && failingStateTargets.length === 0) {
      const latestSuccess = formatWarmTimestamp(latestWarmTargetSuccess(healthyStateTargets));
      stateWarm = {
        tone: "green",
        title: `State Warm: healthy. Warm state is enabled and ready. ${healthyStateTargets.length} target${healthyStateTargets.length === 1 ? "" : "s"} succeeded. Last success ${latestSuccess}.`,
      };
    } else if (warm.active && healthyStateTargets.length > 0 && failingStateTargets.length > 0) {
      stateWarm = {
        tone: "yellow",
        title: `State Warm: partial. Warm state is enabled, but some targets failed. ${summarizeWarmFailures(failingStateTargets)}`,
      };
    } else if (warm.active && warmError) {
      stateWarm = {
        tone: "red",
        title: `State Warm: failing. ${warmError}`,
      };
    } else {
      const warmReasonText = describeWarmReason(warmReason);
      stateWarm = {
        tone: isWarmAutoPausedReason(warmReason) ? "blue" : "yellow",
        title: stateTargets.length > 0
          ? `State Warm: auto-paused. ${warmReasonText}. Last success ${lastWarmSuccess}.`
          : startupWarmInProgress
            ? "State Warm: starting."
            : isWarmAutoPausedReason(warmReason)
              ? `State Warm: auto-paused. ${warmReasonText}.`
              : `State Warm: waiting. ${warmReasonText}.`,
      };
    }
  }

  let endpointPrewarm = {
    tone: startupWarmInProgress ? "yellow" : "gray",
    title: startupWarmInProgress ? `${endpointWarmLabel}: starting.` : `${endpointWarmLabel}: disabled.`,
  };
  if (useStartupWarmFallback) {
    if (startupWarm.disabled) {
      endpointPrewarm = {
        tone: "gray",
        title: `${endpointWarmLabel}: disabled by user.`,
      };
    } else if (startupEndpointFailures.length > 0) {
      endpointPrewarm = {
        tone: startupEndpointFailures.length < startupWarm.endpointTargets.length ? "yellow" : "red",
        title: startupEndpointFailures.length < startupWarm.endpointTargets.length
          ? `${endpointWarmLabel}: partial. ${startupEndpointFailures.length}/${startupWarm.endpointTargets.length} startup target${startupWarm.endpointTargets.length === 1 ? "" : "s"} failed. ${summarizeStartupWarmFailures(startupEndpointFailures)}`
          : `${endpointWarmLabel}: failing. ${startupEndpointFailures.length}/${startupWarm.endpointTargets.length} startup target${startupWarm.endpointTargets.length === 1 ? "" : "s"} failed. ${summarizeStartupWarmFailures(startupEndpointFailures)}`,
      };
    } else if (startupWarm.endpointTargets.length > 0) {
      endpointPrewarm = {
        tone: "green",
        title: `${endpointWarmLabel}: healthy. ${startupWarm.endpointTargets.length} startup target${startupWarm.endpointTargets.length === 1 ? "" : "s"} succeeded.`,
      };
    } else if (startupWarm.error) {
      endpointPrewarm = {
        tone: "red",
        title: `${endpointWarmLabel}: failing. ${startupWarm.error}`,
      };
    }
  } else if (warm) {
    if (warmDisabledByUser || (!warmProviders.length && !activeEndpointTargets.length && !endpointTargets.length)) {
      endpointPrewarm = {
        tone: "gray",
        title: `${endpointWarmLabel}: disabled by user.`,
      };
    } else if (failingActiveEndpointTargets.length > 0) {
      endpointPrewarm = {
        tone: failingActiveEndpointTargets.length < activeEndpointTargets.length ? "yellow" : "red",
        title: failingActiveEndpointTargets.length < activeEndpointTargets.length
          ? `${endpointWarmLabel}: partial. ${failingActiveEndpointTargets.length}/${activeEndpointTargets.length} active target${activeEndpointTargets.length === 1 ? "" : "s"} failed. ${summarizeWarmFailures(failingActiveEndpointTargets)}`
          : `${endpointWarmLabel}: failing. ${failingActiveEndpointTargets.length}/${activeEndpointTargets.length} active target${activeEndpointTargets.length === 1 ? "" : "s"} failed. ${summarizeWarmFailures(failingActiveEndpointTargets)}`,
      };
    } else if (rateLimitedActiveEndpointTargets.length > 0) {
      endpointPrewarm = {
        tone: "yellow",
        title: rateLimitedActiveEndpointTargets.length < activeEndpointTargets.length
          ? `${endpointWarmLabel}: degraded. ${rateLimitedActiveEndpointTargets.length}/${activeEndpointTargets.length} active target${activeEndpointTargets.length === 1 ? "" : "s"} reachable but rate-limited. ${summarizeWarmRateLimits(rateLimitedActiveEndpointTargets)}`
          : `${endpointWarmLabel}: rate-limited. ${rateLimitedActiveEndpointTargets.length}/${activeEndpointTargets.length} active target${activeEndpointTargets.length === 1 ? "" : "s"} reachable but rate-limited. ${summarizeWarmRateLimits(rateLimitedActiveEndpointTargets)}`,
      };
    } else if (staleActiveEndpointTargets.length > 0) {
      endpointPrewarm = {
        tone: "yellow",
        title: `${endpointWarmLabel}: waiting. ${staleActiveEndpointTargets.length} active target${staleActiveEndpointTargets.length === 1 ? "" : "s"} without a fresh success probe (>${Math.round(WARM_TELEMETRY_FRESH_MS / 1000)}s).`,
      };
    } else if (activeEndpointTargets.length > 0) {
      const latestSuccess = formatWarmTimestamp(latestWarmTargetSuccess(activeEndpointTargets));
      endpointPrewarm = {
        tone: "green",
        title: `${endpointWarmLabel}: healthy. ${activeEndpointTargets.length} active target${activeEndpointTargets.length === 1 ? "" : "s"} across ${formatWarmProviders(warmProviders)}. Last success ${latestSuccess}.`,
      };
    } else if (warm.active && healthyEndpointTargets.length > 0 && failingEndpointTargets.length === 0) {
      const latestSuccess = formatWarmTimestamp(latestWarmTargetSuccess(healthyEndpointTargets));
      endpointPrewarm = {
        tone: "green",
        title: `${endpointWarmLabel}: healthy. Warm routing is enabled and ready across ${formatWarmProviders(warmProviders)}. ${healthyEndpointTargets.length} target${healthyEndpointTargets.length === 1 ? "" : "s"} succeeded. Last success ${latestSuccess}.`,
      };
    } else if (warm.active && healthyEndpointTargets.length > 0 && failingEndpointTargets.length > 0) {
      endpointPrewarm = {
        tone: "yellow",
        title: `${endpointWarmLabel}: partial. Warm routing is enabled, but some targets failed. ${summarizeWarmFailures(failingEndpointTargets)}`,
      };
    } else if (warmProviders.length > 0) {
      const warmReasonText = describeWarmReason(warmReason);
      endpointPrewarm = {
        tone: isWarmAutoPausedReason(warmReason) ? "blue" : "yellow",
        title: endpointTargets.length > 0
          ? `${endpointWarmLabel}: auto-paused. ${warmReasonText}. Selected providers: ${formatWarmProviders(warmProviders)}.`
          : startupWarmInProgress
            ? `${endpointWarmLabel}: starting. Selected providers: ${formatWarmProviders(warmProviders)}.`
            : isWarmAutoPausedReason(warmReason)
              ? `${endpointWarmLabel}: auto-paused. ${warmReasonText}. Selected providers: ${formatWarmProviders(warmProviders)}.`
              : `${endpointWarmLabel}: waiting for telemetry. ${warmReasonText}. Selected providers: ${formatWarmProviders(warmProviders)}.`,
      };
    } else {
      endpointPrewarm = {
        tone: "gray",
        title: `${endpointWarmLabel}: disabled by user.`,
      };
    }
  }


  let watchPrewarm = {
    tone: startupWarmInProgress ? "yellow" : "gray",
    title: startupWarmInProgress ? "Watcher WS warm: starting." : "Watcher WS warm: disabled.",
  };
  if (useStartupWarmFallback) {
    if (startupWarm.disabled) {
      watchPrewarm = {
        tone: "gray",
        title: "Watcher WS warm: disabled by user.",
      };
    } else if (startupWatchFailures.length > 0) {
      watchPrewarm = {
        tone: startupWatchFailures.length < startupWarm.watchTargets.length ? "yellow" : "red",
        title: startupWatchFailures.length < startupWarm.watchTargets.length
          ? `Watcher WS warm: partial. ${startupWatchFailures.length}/${startupWarm.watchTargets.length} startup target${startupWarm.watchTargets.length === 1 ? "" : "s"} failed. ${summarizeStartupWarmFailures(startupWatchFailures)}`
          : `Watcher WS warm: failing. ${startupWatchFailures.length}/${startupWarm.watchTargets.length} startup target${startupWarm.watchTargets.length === 1 ? "" : "s"} failed. ${summarizeStartupWarmFailures(startupWatchFailures)}`,
      };
    } else if (startupWarm.watchTargets.length > 0) {
      watchPrewarm = {
        tone: "green",
        title: `Watcher WS warm: healthy. ${startupWarm.watchTargets.length} startup target${startupWarm.watchTargets.length === 1 ? "" : "s"} succeeded.`,
      };
    } else if (startupWarm.error) {
      watchPrewarm = {
        tone: "red",
        title: `Watcher WS warm: failing. ${startupWarm.error}`,
      };
    }
  } else if (warm) {
    if (warmDisabledByUser || (!activeWatchTargets.length && !watchTargets.length)) {
      watchPrewarm = {
        tone: "gray",
        title: "Watcher WS warm: disabled by user.",
      };
    } else if (failingActiveWatchTargets.length > 0) {
      watchPrewarm = {
        tone: failingActiveWatchTargets.length < activeWatchTargets.length ? "yellow" : "red",
        title: failingActiveWatchTargets.length < activeWatchTargets.length
          ? `Watcher WS warm: partial. ${failingActiveWatchTargets.length}/${activeWatchTargets.length} active target${activeWatchTargets.length === 1 ? "" : "s"} failed. ${summarizeWarmFailures(failingActiveWatchTargets)}`
          : `Watcher WS warm: failing. ${failingActiveWatchTargets.length}/${activeWatchTargets.length} active target${activeWatchTargets.length === 1 ? "" : "s"} failed. ${summarizeWarmFailures(failingActiveWatchTargets)}`,
      };
    } else if (rateLimitedActiveWatchTargets.length > 0) {
      watchPrewarm = {
        tone: "yellow",
        title: rateLimitedActiveWatchTargets.length < activeWatchTargets.length
          ? `Watcher WS warm: degraded. ${rateLimitedActiveWatchTargets.length}/${activeWatchTargets.length} active target${activeWatchTargets.length === 1 ? "" : "s"} reachable but rate-limited. ${summarizeWarmRateLimits(rateLimitedActiveWatchTargets)}`
          : `Watcher WS warm: rate-limited. ${rateLimitedActiveWatchTargets.length}/${activeWatchTargets.length} active target${activeWatchTargets.length === 1 ? "" : "s"} reachable but rate-limited. ${summarizeWarmRateLimits(rateLimitedActiveWatchTargets)}`,
      };
    } else if (staleActiveWatchTargets.length > 0) {
      watchPrewarm = {
        tone: "yellow",
        title: `Watcher WS warm: waiting. ${staleActiveWatchTargets.length} active target${staleActiveWatchTargets.length === 1 ? "" : "s"} without a fresh success probe (>${Math.round(WARM_TELEMETRY_FRESH_MS / 1000)}s).`,
      };
    } else if (activeWatchTargets.length > 0) {
      const latestSuccess = formatWarmTimestamp(latestWarmTargetSuccess(activeWatchTargets));
      watchPrewarm = {
        tone: "green",
        title: `Watcher WS warm: healthy. ${activeWatchTargets.length} active target${activeWatchTargets.length === 1 ? "" : "s"}. Last success ${latestSuccess}.`,
      };
    } else if (warm.active && healthyWatchTargets.length > 0 && failingWatchTargets.length === 0) {
      const latestSuccess = formatWarmTimestamp(latestWarmTargetSuccess(healthyWatchTargets));
      watchPrewarm = {
        tone: "green",
        title: `Watcher WS warm: healthy. Warm watch routing is enabled and ready. ${healthyWatchTargets.length} target${healthyWatchTargets.length === 1 ? "" : "s"} succeeded. Last success ${latestSuccess}.`,
      };
    } else if (warm.active && healthyWatchTargets.length > 0 && failingWatchTargets.length > 0) {
      watchPrewarm = {
        tone: "yellow",
        title: `Watcher WS warm: partial. Warm watch routing is enabled, but some targets failed. ${summarizeWarmFailures(failingWatchTargets)}`,
      };
    } else {
      const warmReasonText = describeWarmReason(warmReason);
      watchPrewarm = {
        tone: isWarmAutoPausedReason(warmReason) ? "blue" : "yellow",
        title: watchTargets.length > 0
          ? `Watcher WS warm: auto-paused. ${warmReasonText}.`
          : startupWarmInProgress
            ? "Watcher WS warm: starting."
            : isWarmAutoPausedReason(warmReason)
              ? `Watcher WS warm: auto-paused. ${warmReasonText}.`
              : `Watcher WS warm: waiting. ${warmReasonText}.`,
      };
    }
  }

  const watchPath = currentWatchPathSnapshot();
  const componentRows = [
    {
      tone: stateWarm.tone,
      label: "State warm",
      detail: stateWarm.title.replace(/^[^:]+:\s*/, ""),
    },
    {
      tone: endpointPrewarm.tone,
      label: endpointWarmLabel,
      detail: endpointPrewarm.title.replace(/^[^:]+:\s*/, ""),
    },
    {
      tone: watchPrewarm.tone,
      label: "Watcher WS warm",
      detail: watchPrewarm.title.replace(/^[^:]+:\s*/, ""),
    },
    {
      tone: watchPath.tone,
      label: "Watch path",
      detail: watchPath.title.replace(/^[^:]+:\s*/, ""),
    },
  ];
  const connectionRows = useStartupWarmFallback
    ? [
        ...buildStartupWarmTooltipRows(startupWarm.endpointTargets),
        ...buildStartupWarmTooltipRows(startupWarm.watchTargets),
      ]
    : [
        ...buildWarmTooltipRowsFromTargets(activeEndpointTargets.length ? activeEndpointTargets : endpointTargets, nowMs),
        ...buildWarmTooltipRowsFromTargets(activeWatchTargets.length ? activeWatchTargets : watchTargets, nowMs),
      ];
  const stateRows = useStartupWarmFallback
    ? buildStartupWarmTooltipRows(startupWarm.stateTargets)
    : buildWarmTooltipRowsFromTargets(activeStateTargets.length ? activeStateTargets : stateTargets, nowMs);
  const overallTone = strongestIndicatorTone(componentRows.map((row) => row.tone));
  const failingComponents = componentRows.filter((row) => row.tone === "red").length;
  const degradedComponents = componentRows.filter((row) => row.tone === "yellow" || row.tone === "blue").length;
  const healthyComponents = componentRows.filter((row) => row.tone === "green").length;
  let title = "Warm: disabled.";
  if (overallTone === "red") {
    title = `Warm: failing. ${failingComponents}/${componentRows.length} warm checks failing.`;
  } else if (overallTone === "yellow" || overallTone === "blue") {
    title = startupWarmInProgress
      ? "Warm: starting."
      : `Warm: partial. ${degradedComponents}/${componentRows.length} warm checks need attention.`;
  } else if (overallTone === "green") {
    title = `Warm: healthy. ${healthyComponents}/${componentRows.length} warm checks healthy.`;
  } else if (startupWarmInProgress) {
    title = "Warm: starting.";
  }
  return {
    warm: {
      key: "warm",
      label: "Warm",
      tone: overallTone,
      title,
      sections: [
        { title: "Summary", rows: componentRows },
        { title: "Warm connections", rows: connectionRows },
        { title: "State caches", rows: stateRows },
      ].filter((section) => section.rows.length > 0),
    },
  };
}

function formatRpcRequestsPerMinuteLabel() {
  const rt = latestRuntimeStatus && latestRuntimeStatus.rpcTraffic && typeof latestRuntimeStatus.rpcTraffic === "object"
    ? latestRuntimeStatus.rpcTraffic
    : null;
  if (!latestRuntimeStatus || !rt) {
    return {
      text: "\u2014/min",
      title: "Outbound RPC-credit requests in the last 60 seconds. Waiting for runtime status\u2026",
      muted: true,
    };
  }
  if (rt.enabled === false) {
    return {
      text: "off",
      title: "RPC traffic meter disabled (set LAUNCHDECK_RPC_TRAFFIC_METER=1 to enable, or remove the env var).",
      muted: true,
    };
  }
  const raw = rt.requestsLast60s;
  const n = raw == null ? null : Number(raw);
  if (n == null || !Number.isFinite(n)) {
    return {
      text: "\u2014/min",
      title: "Outbound RPC-credit requests in the last 60 seconds. No sample yet.",
      muted: true,
    };
  }
  const rounded = Math.max(0, Math.round(n));
  return {
    text: `${rounded}/min`,
    title: `About ${rounded} outbound RPC-credit requests in the last 60 seconds (JSON-RPC, Helius priority estimates, warm getVersion, wallet balance reads, and other metered RPC calls). Jito-only requests and Sender pings are excluded.`,
    muted: false,
  };
}

function renderPlatformRuntimeIndicators() {
  if (!platformRuntimeIndicators) return;
  const indicatorState = buildPlatformRuntimeIndicatorState();
  const warmIndicator = indicatorState && indicatorState.warm ? indicatorState.warm : {
    label: "Warm",
    tone: "gray",
    title: "Warm: disabled.",
    sections: [],
  };
  const warmTone = ["green", "yellow", "blue", "red", "gray"].includes(warmIndicator.tone) ? warmIndicator.tone : "gray";
  const warmTooltipSections = renderRuntimeIndicatorTooltipSections(warmIndicator.sections);
  const dotsMarkup = `
    <span class="runtime-indicator-popover" tabindex="0" aria-label="${escapeHTML(warmIndicator.title)}">
      <span class="runtime-indicator-dot is-${warmTone}" aria-hidden="true"></span>
      <span class="runtime-indicator-tooltip" role="tooltip">
        <span class="runtime-indicator-tooltip-title">${escapeHTML(warmIndicator.title)}</span>
        ${warmTooltipSections || '<span class="runtime-indicator-tooltip-empty">No active warm targets right now.</span>'}
      </span>
    </span>
  `;
  const rpcLabel = formatRpcRequestsPerMinuteLabel();
  const rpcClass = `runtime-rpc-rate${rpcLabel.muted ? " is-muted" : ""}`;
  const rpcMarkup = `
    <span class="runtime-rpc-rate-popover" tabindex="0" aria-label="${escapeHTML(rpcLabel.title)}">
      <span class="${rpcClass}">${escapeHTML(rpcLabel.text)}</span>
      <span class="runtime-rpc-tooltip" role="tooltip">
        <span class="runtime-rpc-tooltip-title">Requests per minute</span>
        <span class="runtime-rpc-tooltip-body">${escapeHTML(rpcLabel.title)}</span>
      </span>
    </span>
  `;
  const markup = `${dotsMarkup}${rpcMarkup}`;
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "platformRuntimeIndicators", platformRuntimeIndicators, markup);
  } else {
    platformRuntimeIndicators.innerHTML = markup;
  }
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
  const warm = latestRuntimeStatus && latestRuntimeStatus.warm && typeof latestRuntimeStatus.warm === "object"
    ? latestRuntimeStatus.warm
    : null;
  const sharedConfigured = formatBackendRegionValue(shared.configured, "None");
  const sharedEffective = formatBackendRegionValue(shared.effective);
  const providerRows = ["helius-sender", "jito-bundle"].map((provider) => {
    const entry = providers[provider] || {};
    const configured = formatBackendRegionValue(entry.configured, "None");
    const effective = formatBackendRegionValue(entry.effective);
    const metaText = entry.endpointOverrideActive
      ? `Override: ${configured} | endpoint pinned`
      : `Override: ${configured}`;
    return `
      <div class="settings-region-card">
        <div class="settings-region-card-head">
          <strong>${escapeHTML(PROVIDER_LABELS[provider] || provider)}</strong>
          <span class="settings-region-effective">${escapeHTML(effective)}</span>
        </div>
        <div class="settings-region-meta">${escapeHTML(metaText)}</div>
      </div>
    `;
  }).join("");
  const warmCard = warm ? `
      <div class="settings-region-card settings-region-card-shared">
        <div class="settings-region-card-head">
          <strong>Warm</strong>
          <span class="settings-region-effective">${escapeHTML(warm.active ? "active" : "suspended")}</span>
        </div>
        <div class="settings-region-meta">${escapeHTML(String(warm.reason || "--"))}</div>
        <div class="settings-region-meta">Providers: ${escapeHTML(formatWarmProviders(warm.selectedProviders))}</div>
        <div class="settings-region-meta">Last active: ${escapeHTML(formatWarmTimestamp(warm.lastActivityAtMs))}</div>
      </div>
    ` : "";
  const markup = `
    <div class="settings-region-row">
      <div class="settings-region-card settings-region-card-shared">
        <div class="settings-region-card-head">
          <strong>Shared</strong>
          <span class="settings-region-effective">${escapeHTML(sharedEffective)}</span>
        </div>
        <div class="settings-region-meta">Configured: ${escapeHTML(sharedConfigured)}</div>
      </div>
      ${providerRows}
      ${warmCard}
    </div>
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

function setDevBuyPercentDisplay(value) {
  const normalized = normalizeDecimalInput(String(value || ""), 4);
  if (devBuyPercentInput) {
    devBuyPercentInput.value = normalized;
    devBuyPercentInput.placeholder = normalized || "0";
  }
}

function getDevBuyQuoteRequestShape(modeOverride, amountOverride) {
  return {
    launchpad: getLaunchpad(),
    quoteAsset: getQuoteAsset(),
    launchMode: getMode(),
    mode: String(modeOverride || getDevBuyMode() || "sol").trim().toLowerCase(),
    amount: String(amountOverride != null ? amountOverride : (getNamedValue("devBuyAmount") || "")).trim(),
  };
}

function getDevBuyQuoteCacheKey(shape) {
  return [
    shape.launchpad,
    shape.quoteAsset,
    shape.launchMode,
    shape.mode,
    shape.amount,
  ].join("|");
}

function getCachedDevBuyQuote(shape) {
  const key = getDevBuyQuoteCacheKey(shape);
  const entry = devBuyQuoteCache.get(key);
  if (!entry) return null;
  if ((Date.now() - entry.cachedAt) > DEV_BUY_QUOTE_CACHE_TTL_MS) {
    devBuyQuoteCache.delete(key);
    return null;
  }
  return entry.quote || null;
}

function setCachedDevBuyQuote(shape, quote) {
  if (!quote || !shape.amount) return;
  devBuyQuoteCache.set(getDevBuyQuoteCacheKey(shape), {
    quote,
    cachedAt: Date.now(),
  });
}

function renderDevBuyQuoteMessage(quote, mode, { provisional = false } = {}) {
  if (!quoteOutput || !quote) return;
  const quoteLabel = getQuoteAssetLabel(quote.quoteAsset || getQuoteAsset());
  quoteOutput.hidden = false;
  quoteOutput.textContent = mode === "sol"
    ? `${provisional ? "Preview: " : ""}Estimated tokens out: ${quote.estimatedTokens} (${quote.estimatedSupplyPercent}% supply)`
    : `${provisional ? "Preview: " : ""}Estimated ${quoteLabel} required: ${(quote.estimatedQuoteAmount || quote.estimatedSol)} for ${quote.estimatedSupplyPercent}% supply`;
}

function findNearestCachedDevBuyQuote(shape) {
  let nearest = null;
  let nearestDistance = Number.POSITIVE_INFINITY;
  for (const [key, entry] of devBuyQuoteCache.entries()) {
    if ((Date.now() - entry.cachedAt) > DEV_BUY_QUOTE_CACHE_TTL_MS) {
      devBuyQuoteCache.delete(key);
      continue;
    }
    const [launchpad, quoteAsset, launchMode, mode] = key.split("|");
    if (
      launchpad !== shape.launchpad
      || quoteAsset !== shape.quoteAsset
      || launchMode !== shape.launchMode
      || mode !== shape.mode
    ) {
      continue;
    }
    const cachedAmount = Number(entry.quote && entry.quote.input);
    const requestedAmount = Number(shape.amount);
    if (!Number.isFinite(cachedAmount) || !Number.isFinite(requestedAmount) || cachedAmount <= 0 || requestedAmount <= 0) {
      continue;
    }
    const distance = Math.abs(cachedAmount - requestedAmount);
    if (distance < nearestDistance) {
      nearest = entry.quote;
      nearestDistance = distance;
    }
  }
  return nearest;
}

function renderProvisionalDevBuyPreview(shape) {
  if (!quoteOutput || !shape.amount) return;
  if (shape.mode === "tokens") {
    const percent = tokenAmountToPercent(shape.amount);
    if (percent) setDevBuyPercentDisplay(percent);
    quoteOutput.hidden = false;
    quoteOutput.textContent = `Estimating ${getQuoteAssetLabel(shape.quoteAsset)} required${percent ? ` for ${percent}% supply` : ""}...`;
    return;
  }
  const nearest = findNearestCachedDevBuyQuote(shape);
  if (nearest) {
    const requestedAmount = Number(shape.amount);
    const cachedAmount = Number(nearest.input || shape.amount);
    const cachedTokens = Number(nearest.estimatedTokens || "0");
    const cachedPercent = Number(nearest.estimatedSupplyPercent || "0");
    if (
      Number.isFinite(requestedAmount)
      && Number.isFinite(cachedAmount)
      && Number.isFinite(cachedTokens)
      && Number.isFinite(cachedPercent)
      && cachedAmount > 0
    ) {
      const ratio = requestedAmount / cachedAmount;
      const previewQuote = {
        ...nearest,
        input: shape.amount,
        estimatedQuoteAmount: shape.amount,
        estimatedSol: shape.amount,
        estimatedTokens: normalizeDecimalInput(String(cachedTokens * ratio), 6),
        estimatedSupplyPercent: normalizeDecimalInput(String(cachedPercent * ratio), 4),
      };
      setDevBuyPercentDisplay(previewQuote.estimatedSupplyPercent);
      renderDevBuyQuoteMessage(previewQuote, shape.mode, { provisional: true });
      return;
    }
  }
  quoteOutput.hidden = false;
  quoteOutput.textContent = `Estimating ${getQuoteAssetLabel(shape.quoteAsset)} curve position...`;
}

function warmDevBuyQuoteCache(signal) {
  if (!startupWarmState.enabled) return Promise.resolve();
  const baseShape = getDevBuyQuoteRequestShape("sol", "");
  const baseShapes = [
    baseShape,
    { ...baseShape, launchpad: "bonk", launchMode: "regular", quoteAsset: "sol", mode: "sol" },
    { ...baseShape, launchpad: "bonk", launchMode: "regular", quoteAsset: "usd1", mode: "sol" },
    { ...baseShape, launchpad: "bonk", launchMode: "bonkers", quoteAsset: "sol", mode: "sol" },
    { ...baseShape, launchpad: "bonk", launchMode: "bonkers", quoteAsset: "usd1", mode: "sol" },
  ];
  const amounts = Array.from(new Set(
    getQuickDevBuyPresetAmounts()
      .map((value) => normalizeDecimalInput(value, 9))
      .filter(Boolean),
  ));
  const warmRequests = baseShapes.flatMap((base) => amounts.map((amount) => {
    const shape = { ...base, amount };
    if (getCachedDevBuyQuote(shape)) return Promise.resolve();
    const key = getDevBuyQuoteCacheKey(shape);
    if (devBuyQuoteWarmInFlight.has(key)) return Promise.resolve();
    devBuyQuoteWarmInFlight.add(key);
    const url = `/api/quote?launchpad=${encodeURIComponent(shape.launchpad)}&quoteAsset=${encodeURIComponent(shape.quoteAsset)}&launchMode=${encodeURIComponent(shape.launchMode)}&mode=${encodeURIComponent(shape.mode)}&amount=${encodeURIComponent(shape.amount)}`;
    return fetch(url, signal ? { signal } : undefined)
      .then((response) => response.json().then((payload) => ({ response, payload })))
      .then(({ response, payload }) => {
        if (response.ok && payload && payload.ok && payload.quote) {
          setCachedDevBuyQuote(shape, payload.quote);
        }
      })
      .catch(() => {})
      .finally(() => {
        devBuyQuoteWarmInFlight.delete(key);
      });
  }));
  return Promise.allSettled(warmRequests);
}

function readStoredStartupWarmCache() {
  try {
    const raw = window.localStorage.getItem(STARTUP_WARM_CACHE_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    const cachedAtMs = Number(parsed.cachedAtMs);
    if (!Number.isFinite(cachedAtMs)) return null;
    if ((Date.now() - cachedAtMs) > STARTUP_WARM_CACHE_MAX_AGE_MS) return null;
    const payload = parsed.payload && typeof parsed.payload === "object" ? parsed.payload : null;
    if (!payload) return null;
    return { cachedAtMs, payload };
  } catch (_error) {
    return null;
  }
}

function writeStoredStartupWarmCache(payload) {
  try {
    if (!payload || typeof payload !== "object") {
      window.localStorage.removeItem(STARTUP_WARM_CACHE_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(STARTUP_WARM_CACHE_STORAGE_KEY, JSON.stringify({
      cachedAtMs: Date.now(),
      payload,
    }));
  } catch (_error) {
    // Ignore storage failures and keep boot flow functional.
  }
}

function clearStoredStartupWarmCache() {
  try {
    window.localStorage.removeItem(STARTUP_WARM_CACHE_STORAGE_KEY);
  } catch (_error) {
    // Ignore storage failures and keep boot flow functional.
  }
}

function getCurrentThemeMode() {
  return document.documentElement.classList.contains("theme-light") ? "light" : "dark";
}

function isImageLayoutCompactActive() {
  return Boolean(tokenSurfaceSection && tokenSurfaceSection.classList.contains("is-image-compact"));
}

function getLiveSyncControlKey(control) {
  if (!(control instanceof HTMLElement)) return "";
  if (control.id) return `id:${control.id}`;
  const name = control.getAttribute("name");
  if (!name) return "";
  const type = String(control.getAttribute("type") || "").toLowerCase();
  if (type === "radio" || type === "checkbox") {
    return `name:${name}:value:${control.getAttribute("value") || ""}`;
  }
  return `name:${name}`;
}

function getLiveSyncControls() {
  if (!form) return [];
  return Array.from(form.querySelectorAll("input, select, textarea"))
    .filter((control) => {
      if (!(control instanceof HTMLElement)) return false;
      if (String(control.getAttribute("type") || "").toLowerCase() === "file") return false;
      return control.getAttribute("name") !== "vanityPrivateKey";
    });
}

function isRefreshPersistedFormControlKey(key) {
  return /^name:launchpad:value:/.test(key)
    || /^name:mode:value:/.test(key)
    || key === "name:mayhemMode:value:"
    || key === "name:quoteAsset"
    || key === "name:postLaunchStrategy"
    || key === "name:sniperEnabled"
    || key === "name:automaticDevSellEnabled:value:"
    || key === "name:feeSplitEnabled:value:"
    || key === "name:bagsIdentityMode";
}

function filterRefreshPersistedFormControls(formControls) {
  if (!formControls || typeof formControls !== "object") return {};
  return Object.fromEntries(
    Object.entries(formControls).filter(([key]) => isRefreshPersistedFormControlKey(key)),
  );
}

function buildLiveSyncFormControls() {
  return getLiveSyncControls().reduce((accumulator, control) => {
    const key = getLiveSyncControlKey(control);
    if (!key) return accumulator;
    const type = String(control.getAttribute("type") || "").toLowerCase();
    if (type === "checkbox" || type === "radio") {
      accumulator[key] = { checked: Boolean(control.checked) };
    } else {
      accumulator[key] = { value: "value" in control ? String(control.value) : "" };
    }
    return accumulator;
  }, {});
}

function buildLiveSyncPayload() {
  return {
    sourceId: LIVE_SYNC_SOURCE_ID,
    timestampMs: Date.now(),
    themeMode: getCurrentThemeMode(),
    outputVisible: isOutputSectionCurrentlyVisible(),
    reportsVisible: isReportsTerminalCurrentlyVisible(),
    reportsListWidth: getCurrentReportsTerminalListWidth(),
    imageLayoutCompact: isImageLayoutCompactActive(),
    selectedWalletKey: walletSelect ? String(walletSelect.value || "") : "",
    config: cloneConfig(getConfig()),
    walletStatusSnapshot: latestWalletStatus,
    runtimeStatusSnapshot: latestRuntimeStatus,
    startupWarmSnapshot: {
      started: Boolean(startupWarmState.started),
      ready: Boolean(startupWarmState.ready),
      enabled: startupWarmState.enabled !== false,
      backendLoaded: Boolean(startupWarmState.backendLoaded),
      backendPayload: startupWarmState.backendPayload,
      backendError: String(startupWarmState.backendError || ""),
    },
    reportsTerminalSnapshot: {
      ...reportsTerminalState,
      activeLogs: reportsTerminalState.activeLogs,
      activePayload: reportsTerminalState.activePayload,
      activeBenchmarkSnapshot: reportsTerminalState.activeBenchmarkSnapshot,
    },
    followJobsSnapshot: {
      ...followJobsState,
      refreshTimer: null,
    },
    outputSnapshot: {
      statusLabel: currentStatusLabel(),
      metaText: metaNode ? String(metaNode.textContent || "") : "",
      outputText: output ? String(output.textContent || "") : "",
    },
    formControls: buildLiveSyncFormControls(),
  };
}

function buildEarlyBootSnapshot(payload = buildLiveSyncPayload()) {
  return {
    sourceId: payload.sourceId,
    timestampMs: payload.timestampMs,
    themeMode: payload.themeMode,
    outputVisible: payload.outputVisible,
    reportsVisible: payload.reportsVisible,
    reportsListWidth: payload.reportsListWidth,
    imageLayoutCompact: payload.imageLayoutCompact,
    selectedWalletKey: payload.selectedWalletKey,
    config: payload.config || null,
    walletStatusSnapshot: payload.walletStatusSnapshot || null,
    runtimeStatusSnapshot: payload.runtimeStatusSnapshot || null,
    startupWarmSnapshot: payload.startupWarmSnapshot || null,
    formControls: payload.formControls || {},
  };
}

function buildPersistedLiveSyncPayload(payload = buildLiveSyncPayload()) {
  return {
    sourceId: payload.sourceId,
    timestampMs: payload.timestampMs,
    themeMode: payload.themeMode,
    outputVisible: payload.outputVisible,
    reportsVisible: payload.reportsVisible,
    reportsListWidth: payload.reportsListWidth,
    imageLayoutCompact: payload.imageLayoutCompact,
    selectedWalletKey: payload.selectedWalletKey,
    config: payload.config || null,
    walletStatusSnapshot: payload.walletStatusSnapshot || null,
    runtimeStatusSnapshot: payload.runtimeStatusSnapshot || null,
    startupWarmSnapshot: payload.startupWarmSnapshot || null,
    reportsTerminalSnapshot: payload.reportsTerminalSnapshot
      ? {
          allEntries: Array.isArray(payload.reportsTerminalSnapshot.allEntries) ? payload.reportsTerminalSnapshot.allEntries : [],
          entries: Array.isArray(payload.reportsTerminalSnapshot.entries) ? payload.reportsTerminalSnapshot.entries : [],
          launches: Array.isArray(payload.reportsTerminalSnapshot.launches) ? payload.reportsTerminalSnapshot.launches : [],
          activeLogs: payload.reportsTerminalSnapshot.activeLogs || { live: [], errors: [], error: "", updatedAtMs: 0 },
          launchBundles: payload.reportsTerminalSnapshot.launchBundles || {},
          launchMetadataByUri: payload.reportsTerminalSnapshot.launchMetadataByUri || {},
          activeId: payload.reportsTerminalSnapshot.activeId || "",
          activePayload: payload.reportsTerminalSnapshot.activePayload || null,
          activeBenchmarkReportId: payload.reportsTerminalSnapshot.activeBenchmarkReportId || "",
          activeBenchmarkSnapshot: payload.reportsTerminalSnapshot.activeBenchmarkSnapshot || null,
          activeText: payload.reportsTerminalSnapshot.activeText || "",
          activeTab: payload.reportsTerminalSnapshot.activeTab || "overview",
          view: payload.reportsTerminalSnapshot.view || "transactions",
          activeLogsView: payload.reportsTerminalSnapshot.activeLogsView || "live",
          sort: payload.reportsTerminalSnapshot.sort || "newest",
        }
      : null,
    followJobsSnapshot: payload.followJobsSnapshot
      ? {
          configured: Boolean(payload.followJobsSnapshot.configured),
          reachable: Boolean(payload.followJobsSnapshot.reachable),
          jobs: Array.isArray(payload.followJobsSnapshot.jobs) ? payload.followJobsSnapshot.jobs : [],
          health: payload.followJobsSnapshot.health || null,
          error: payload.followJobsSnapshot.error || "",
          loaded: Boolean(payload.followJobsSnapshot.loaded),
          refreshTimer: null,
        }
      : null,
    outputSnapshot: payload.outputSnapshot || null,
    formControls: payload.formControls || {},
  };
}

function readStoredLiveSyncPayload() {
  try {
    const raw = window.localStorage.getItem(LIVE_SYNC_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    const timestampMs = Number(parsed.timestampMs);
    if (!Number.isFinite(timestampMs)) return null;
    if ((Date.now() - timestampMs) > LIVE_SYNC_MAX_AGE_MS) return null;
    return parsed;
  } catch (_error) {
    return null;
  }
}

function readStoredEarlyBootSnapshot() {
  try {
    const raw = window.localStorage.getItem(EARLY_BOOT_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    const timestampMs = Number(parsed.timestampMs);
    if (!Number.isFinite(timestampMs)) return null;
    if ((Date.now() - timestampMs) > LIVE_SYNC_MAX_AGE_MS) return null;
    return parsed;
  } catch (_error) {
    return null;
  }
}

function readSessionEarlyBootSnapshot() {
  try {
    const raw = window.sessionStorage.getItem(EARLY_BOOT_SESSION_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    const timestampMs = Number(parsed.timestampMs);
    if (!Number.isFinite(timestampMs)) return null;
    if ((Date.now() - timestampMs) > LIVE_SYNC_MAX_AGE_MS) return null;
    return parsed;
  } catch (_error) {
    return null;
  }
}

function readSessionLiveSyncPayload() {
  try {
    const raw = window.sessionStorage.getItem(LIVE_SYNC_SESSION_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    const timestampMs = Number(parsed.timestampMs);
    if (!Number.isFinite(timestampMs)) return null;
    if ((Date.now() - timestampMs) > LIVE_SYNC_MAX_AGE_MS) return null;
    return parsed;
  } catch (_error) {
    return null;
  }
}

function readOpenerLiveSyncPayload() {
  try {
    if (!window.opener || window.opener === window) return null;
    const payload = window.opener.__launchdeckLiveSyncSnapshot;
    if (!payload || typeof payload !== "object") return null;
    const timestampMs = Number(payload.timestampMs);
    if (!Number.isFinite(timestampMs)) return null;
    if ((Date.now() - timestampMs) > LIVE_SYNC_MAX_AGE_MS) return null;
    return payload;
  } catch (_error) {
    return null;
  }
}

function dispatchLiveSyncPayload(payload) {
  if (!payload || typeof payload !== "object") return;
  const earlyBootPayload = buildEarlyBootSnapshot(payload);
  const persistedPayload = buildPersistedLiveSyncPayload(payload);
  try {
    window.__launchdeckLiveSyncSnapshot = payload;
  } catch (_error) {
    // Ignore window assignment failures and continue with other sync paths.
  }
  try {
    window.__launchdeckEarlyLiveSyncSnapshot = earlyBootPayload;
  } catch (_error) {
    // Ignore window assignment failures and continue with storage fallbacks.
  }
  try {
    window.sessionStorage.setItem(EARLY_BOOT_SESSION_STORAGE_KEY, JSON.stringify(earlyBootPayload));
  } catch (_error) {
    // Ignore session storage failures and continue with other sync paths.
  }
  try {
    window.sessionStorage.setItem(LIVE_SYNC_SESSION_STORAGE_KEY, JSON.stringify(persistedPayload));
  } catch (_error) {
    // Ignore session storage failures and continue with other sync paths.
  }
  if (liveSyncChannel) {
    try {
      liveSyncChannel.postMessage(payload);
    } catch (_error) {
      // Ignore BroadcastChannel failures and continue with storage fallback.
    }
  }
  try {
    window.localStorage.setItem(EARLY_BOOT_STORAGE_KEY, JSON.stringify(earlyBootPayload));
  } catch (_error) {
    // Ignore storage failures and keep live sync best-effort.
  }
  try {
    window.localStorage.setItem(LIVE_SYNC_STORAGE_KEY, JSON.stringify(persistedPayload));
  } catch (_error) {
    // Ignore storage failures and keep live sync best-effort.
  }
}

function scheduleLiveSyncBroadcast({ immediate = false } = {}) {
  if (!liveSyncReady || isApplyingLiveSync) return;
  if (liveSyncTimer) {
    window.clearTimeout(liveSyncTimer);
    liveSyncTimer = 0;
  }
  if (immediate) {
    dispatchLiveSyncPayload(buildLiveSyncPayload());
    return;
  }
  liveSyncTimer = window.setTimeout(() => {
    liveSyncTimer = 0;
    dispatchLiveSyncPayload(buildLiveSyncPayload());
  }, 60);
}

function applyLiveSyncFormControls(formControls) {
  if (!formControls || typeof formControls !== "object") return;
  const controlsByKey = new Map(
    getLiveSyncControls().map((control) => [getLiveSyncControlKey(control), control]),
  );
  Object.entries(formControls).forEach(([key, snapshot]) => {
    const control = controlsByKey.get(key);
    if (!control || !snapshot || typeof snapshot !== "object") return;
    const type = String(control.getAttribute("type") || "").toLowerCase();
    if (type === "checkbox" || type === "radio") {
      const nextChecked = Boolean(snapshot.checked);
      if (control.checked === nextChecked) return;
      control.checked = nextChecked;
      control.dispatchEvent(new Event("change", { bubbles: true }));
      return;
    }
    const nextValue = snapshot.value == null ? "" : String(snapshot.value);
    if (String(control.value) === nextValue) return;
    control.value = nextValue;
    const eventName = control.tagName === "SELECT" ? "change" : "input";
    control.dispatchEvent(new Event(eventName, { bubbles: true }));
  });
}

function applyIncomingLiveSyncPayload(payload, {
  allowBeforeReady = false,
  skipVisibilityState = false,
  skipDashboardViewState = false,
  skipThemeMode = false,
  skipFormControls = false,
  restorePersistedFormControlsOnly = false,
  restoreOutputFromSync = true,
} = {}) {
  if (!allowBeforeReady && !liveSyncReady) return;
  if (!payload || typeof payload !== "object" || payload.sourceId === LIVE_SYNC_SOURCE_ID) return;
  isApplyingLiveSync = true;
  try {
    if (!skipThemeMode && (payload.themeMode === "light" || payload.themeMode === "dark")) {
      setThemeMode(payload.themeMode);
    }
    if (!skipVisibilityState && typeof payload.outputVisible === "boolean") {
      setOutputSectionVisible(payload.outputVisible);
    }
    if (!skipVisibilityState && typeof payload.reportsVisible === "boolean") {
      setReportsTerminalVisible(payload.reportsVisible);
    }
    if (Number.isFinite(payload.reportsListWidth)) {
      setReportsTerminalListWidth(payload.reportsListWidth);
    }
    if (typeof payload.imageLayoutCompact === "boolean") {
      setImageLayoutCompact(payload.imageLayoutCompact);
    }
    if (payload.config && typeof payload.config === "object") {
      const nextConfig = cloneConfig(payload.config);
      setConfig(nextConfig);
      setPresetEditing(isPresetEditing(nextConfig));
      applyPresetToSettingsInputs(getActivePreset(nextConfig), { syncToMainForm: false });
      queueWarmActivity({ immediate: true });
    }
    if (payload.startupWarmSnapshot && typeof payload.startupWarmSnapshot === "object") {
      startupWarmState = {
        ...startupWarmState,
        started: Boolean(payload.startupWarmSnapshot.started),
        ready: Boolean(payload.startupWarmSnapshot.ready),
        enabled: payload.startupWarmSnapshot.enabled !== false,
        backendLoaded: Boolean(payload.startupWarmSnapshot.backendLoaded),
        backendPayload: payload.startupWarmSnapshot.backendPayload && typeof payload.startupWarmSnapshot.backendPayload === "object"
          ? payload.startupWarmSnapshot.backendPayload
          : null,
        backendError: String(payload.startupWarmSnapshot.backendError || ""),
        promise: null,
      };
      renderPlatformRuntimeIndicators();
    }
    if (payload.walletStatusSnapshot && typeof payload.walletStatusSnapshot === "object") {
      applyWalletStatusPayload(payload.walletStatusSnapshot);
    }
    if (payload.runtimeStatusSnapshot && typeof payload.runtimeStatusSnapshot === "object") {
      applyRuntimeStatusPayload(payload.runtimeStatusSnapshot, { hydrateOnly: true });
    }
    if (payload.followJobsSnapshot && typeof payload.followJobsSnapshot === "object") {
      clearFollowJobsRefreshTimer();
      followJobsState = {
        ...followJobsState,
        ...payload.followJobsSnapshot,
        refreshTimer: null,
      };
      syncFollowStatusChrome();
      renderReportsTerminalList();
      renderReportsTerminalOutput();
    }
    if (payload.reportsTerminalSnapshot && typeof payload.reportsTerminalSnapshot === "object") {
      reportsTerminalState = {
        ...reportsTerminalState,
        ...payload.reportsTerminalSnapshot,
      };
      if (skipDashboardViewState) {
        reportsTerminalState.view = getStoredReportsTerminalView();
        reportsTerminalState.activeLogsView = getStoredActiveLogsView();
      }
      renderReportsTerminalList();
      renderReportsTerminalOutput();
    }
    if (restoreOutputFromSync && payload.outputSnapshot && typeof payload.outputSnapshot === "object") {
      setStatusLabel(payload.outputSnapshot.statusLabel || "");
      if (metaNode) metaNode.textContent = String(payload.outputSnapshot.metaText || "");
      if (output) output.textContent = String(payload.outputSnapshot.outputText || "");
    }
    if (payload.selectedWalletKey && walletSelect && walletSelect.value !== payload.selectedWalletKey) {
      walletSelect.value = payload.selectedWalletKey;
      walletSelect.dispatchEvent(new Event("change", { bubbles: true }));
    }
    if (restorePersistedFormControlsOnly) {
      applyLiveSyncFormControls(filterRefreshPersistedFormControls(payload.formControls));
    } else if (!skipFormControls) {
      applyLiveSyncFormControls(payload.formControls);
    }
  } finally {
    isApplyingLiveSync = false;
    schedulePopoutAutosize();
  }
}

function enableLiveSync() {
  liveSyncReady = true;
  const storedPayload = readStoredLiveSyncPayload();
  if (storedPayload) {
    applyIncomingLiveSyncPayload(storedPayload, {
      skipThemeMode: true,
      skipFormControls: false,
      restorePersistedFormControlsOnly: !isPopoutMode,
      skipVisibilityState: true,
      skipDashboardViewState: true,
      restoreOutputFromSync: isPopoutMode,
    });
  }
  scheduleLiveSyncBroadcast({ immediate: true });
}

function preloadLiveSyncSnapshot() {
  const payload = readOpenerLiveSyncPayload()
    || window.__launchdeckEarlyLiveSyncSnapshot
    || readSessionEarlyBootSnapshot()
    || readStoredEarlyBootSnapshot()
    || readSessionLiveSyncPayload()
    || readStoredLiveSyncPayload();
  if (!payload) return false;
  applyIncomingLiveSyncPayload(payload, {
    allowBeforeReady: true,
    skipThemeMode: true,
    skipFormControls: false,
    restorePersistedFormControlsOnly: !isPopoutMode,
    skipVisibilityState: true,
    skipDashboardViewState: true,
    restoreOutputFromSync: isPopoutMode,
  });
  return true;
}

// Run after this script finishes so `fieldValidators` and other `const` helpers below exist
// (validateFieldByName is invoked from live-sync / auto-sell blur handlers during hydration).
queueMicrotask(() => {
  preloadLiveSyncSnapshot();
});

function beginStartupWarmup() {
  if (!startupWarmState.enabled) {
    startupWarmState.started = true;
    startupWarmState.ready = true;
    startupWarmState.backendLoaded = true;
    startupWarmState.backendPayload = null;
    startupWarmState.backendError = "";
    startupWarmState.promise = Promise.resolve();
    renderPlatformRuntimeIndicators();
    return startupWarmState.promise;
  }
  if (startupWarmState.started) {
    return startupWarmState.promise || Promise.resolve();
  }
  const cachedWarm = readStoredStartupWarmCache();
  if (cachedWarm) {
    startupWarmState.started = true;
    startupWarmState.ready = true;
    startupWarmState.backendLoaded = true;
    startupWarmState.backendPayload = cachedWarm.payload;
    startupWarmState.backendError = "";
    startupWarmState.promise = Promise.resolve(cachedWarm.payload);
    renderPlatformRuntimeIndicators();
    return startupWarmState.promise;
  }
  startupWarmState.started = true;
  renderPlatformRuntimeIndicators();
  const controller = typeof AbortController === "function" ? new AbortController() : null;
  const timeoutId = setTimeout(() => {
    if (controller) controller.abort();
  }, STARTUP_WARM_REQUEST_TIMEOUT_MS);
  const warmRequest = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(currentWarmActivityPayload()),
    ...(controller ? { signal: controller.signal } : {}),
  };
  const backendWarm = fetch("/api/startup-warm", warmRequest)
    .then((response) => response.json().catch(() => ({})).then((payload) => ({ response, payload })))
    .then(({ response, payload }) => {
      startupWarmState.backendLoaded = true;
      startupWarmState.backendPayload = payload && typeof payload === "object" ? payload : null;
      startupWarmState.backendError = response && !response.ok
        ? String(payload && payload.error || "Startup warm failed.")
        : "";
      if (response && response.ok && payload && typeof payload === "object") {
        writeStoredStartupWarmCache(payload);
      } else {
        clearStoredStartupWarmCache();
      }
      renderPlatformRuntimeIndicators();
      return { response, payload };
    })
    .catch((error) => {
      startupWarmState.backendLoaded = true;
      startupWarmState.backendPayload = null;
      startupWarmState.backendError = error && error.message ? error.message : "Startup warm failed.";
      clearStoredStartupWarmCache();
      renderPlatformRuntimeIndicators();
      return { response: null, payload: null };
    });
  const quoteWarm = Promise.resolve(warmDevBuyQuoteCache(controller ? controller.signal : undefined)).catch(() => {});
  startupWarmState.promise = Promise.allSettled([backendWarm, quoteWarm]).finally(() => {
    clearTimeout(timeoutId);
    startupWarmState.ready = true;
    renderPlatformRuntimeIndicators();
  });
  return startupWarmState.promise;
}

async function ensureStartupWarmReady() {
  if (!startupWarmState.enabled) {
    startupWarmState.ready = true;
    return;
  }
  const warmPromise = beginStartupWarmup();
  if (startupWarmState.ready) return;
  metaNode.textContent = "Finalizing startup warmup so the first launch uses hot caches.";
  await Promise.race([
    warmPromise.catch(() => {}),
    new Promise((resolve) => setTimeout(resolve, STARTUP_WARM_WAIT_TIMEOUT_MS)),
  ]);
}

function applyDevBuyQuotePayload(quote, mode) {
  if (!quote) return;
  syncingDevBuyInputs = true;
  if (mode === "sol") {
    setDevBuyPercentDisplay(quote.estimatedSupplyPercent);
  } else {
    if (devBuySolInput) {
      const nextQuoteAmount = quote.estimatedQuoteAmount || quote.estimatedSol || "";
      devBuySolInput.value = nextQuoteAmount;
      devBuySolInput.placeholder = nextQuoteAmount || "0.0";
    }
    setDevBuyPercentDisplay(quote.estimatedSupplyPercent);
  }
  syncingDevBuyInputs = false;
}

function clearDevBuyState() {
  setDevBuyHiddenState("sol", "");
  syncingDevBuyInputs = true;
  if (devBuySolInput) devBuySolInput.value = "";
  if (devBuySolInput) devBuySolInput.placeholder = "0.0";
  setDevBuyPercentDisplay("");
  syncingDevBuyInputs = false;
  if (quoteOutput) {
    quoteOutput.hidden = true;
    quoteOutput.textContent = "No dev buy selected.";
  }
}

function percentToTokenAmount(percentValue) {
  const percentRaw = parseDecimalToBigInt(percentValue, 4);
  const rawTokens = (TOTAL_SUPPLY_TOKENS * (10n ** BigInt(TOKEN_DECIMALS)) * percentRaw) / 1_000_000n;
  return formatBigIntDecimal(rawTokens, TOKEN_DECIMALS, TOKEN_DECIMALS);
}

function tokenAmountToPercent(tokenAmount) {
  try {
    const rawTokens = parseDecimalToBigInt(tokenAmount, TOKEN_DECIMALS);
    if (rawTokens <= 0n) return "";
    const denominator = TOTAL_SUPPLY_TOKENS * (10n ** BigInt(TOKEN_DECIMALS));
    const scaledPercent = (rawTokens * 1_000_000n) / denominator;
    return formatBigIntDecimal(scaledPercent, 4, 4);
  } catch (_error) {
    return "";
  }
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
  renderProvisionalDevBuyPreview(getDevBuyQuoteRequestShape("sol", amount));
  queueQuoteUpdate();
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
    renderProvisionalDevBuyPreview(getDevBuyQuoteRequestShape("tokens", tokenAmount));
    queueQuoteUpdate();
  } catch (_error) {
    setDevBuyHiddenState("tokens", "");
    if (quoteOutput) {
      quoteOutput.hidden = false;
      quoteOutput.textContent = "Enter a valid percentage.";
    }
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
  const target = normalizeLaunchModeForLaunchpad(mode, getLaunchpad());
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

function normalizeQuoteAsset(value) {
  return String(value || "").trim().toLowerCase() === "usd1" ? "usd1" : "sol";
}

function getQuoteAsset() {
  if (getLaunchpad() !== "bonk") return "sol";
  return normalizeQuoteAsset(getNamedValue("quoteAsset"));
}

function getQuoteAssetLabel(asset = getQuoteAsset()) {
  return normalizeQuoteAsset(asset) === "usd1" ? "USD1" : "SOL";
}

function getDevBuyAssetLabel(launchpad = getLaunchpad(), quoteAsset = getQuoteAsset()) {
  return launchpad === "bonk" ? "SOL" : getQuoteAssetLabel(quoteAsset);
}

function getQuoteAssetButtonLabel(asset = getQuoteAsset()) {
  return normalizeQuoteAsset(asset) === "usd1" ? "usd1" : "solana";
}

function syncBonkQuoteAssetUI() {
  const launchpad = getLaunchpad();
  const mode = getMode();
  const visible = launchpad === "bonk" && ["regular", "bonkers"].includes(mode);
  const stored = getStoredBonkQuoteAsset();
  const current = normalizeQuoteAsset(getNamedValue("quoteAsset"));
  const asset = visible ? normalizeQuoteAsset(current || stored || "sol") : "sol";
  if (bonkQuoteAssetInput) bonkQuoteAssetInput.value = asset;
  if (bonkQuoteAssetToggle) bonkQuoteAssetToggle.hidden = !visible;
  if (bonkQuoteAssetToggle) bonkQuoteAssetToggle.disabled = !visible;
  if (bonkQuoteAssetToggle) {
    const nextAsset = asset === "usd1" ? "solana" : "usd1";
    bonkQuoteAssetToggle.title = `Active quote asset: ${getQuoteAssetButtonLabel(asset)}. Click to switch to ${nextAsset}.`;
    bonkQuoteAssetToggle.setAttribute(
      "aria-label",
      `Active quote asset: ${getQuoteAssetButtonLabel(asset)}. Click to switch to ${nextAsset}.`,
    );
  }
  if (bonkQuoteAssetToggleSolIcon) bonkQuoteAssetToggleSolIcon.hidden = asset !== "sol";
  if (bonkQuoteAssetToggleUsd1Icon) bonkQuoteAssetToggleUsd1Icon.hidden = asset !== "usd1";
  if (visible) setStoredBonkQuoteAsset(asset);
  if (devBuyQuotePrefixIcon) devBuyQuotePrefixIcon.hidden = false;
  if (devBuyQuotePrefixText) {
    devBuyQuotePrefixText.hidden = true;
    devBuyQuotePrefixText.textContent = "SOL";
  }
}

function getBagsIdentityMode() {
  return String(bagsIdentityModeInput && bagsIdentityModeInput.value || "wallet-only").trim().toLowerCase() === "linked"
    ? "linked"
    : "wallet-only";
}

function setBagsIdentityStateInputs(nextState = {}) {
  bagsIdentityState = {
    ...bagsIdentityState,
    ...nextState,
  };
  if (bagsIdentityModeInput) bagsIdentityModeInput.value = bagsIdentityState.mode === "linked" ? "linked" : "wallet-only";
  if (bagsAgentUsernameHiddenInput) bagsAgentUsernameHiddenInput.value = bagsIdentityState.agentUsername || "";
  if (bagsAuthTokenInput) bagsAuthTokenInput.value = bagsIdentityState.authToken || "";
  if (bagsIdentityVerifiedWalletInput) bagsIdentityVerifiedWalletInput.value = bagsIdentityState.verifiedWallet || "";
}

function describeBagsIdentity() {
  if (getBagsIdentityMode() !== "linked") return "Wallet Only";
  if (bagsIdentityState.agentUsername) return `Linked Bags Identity (@${bagsIdentityState.agentUsername})`;
  return "Linked Bags Identity";
}

function syncBagsIdentityUI() {
  const visible = getLaunchpad() === "bagsapp";
  if (bagsIdentityButton) bagsIdentityButton.hidden = !visible;
  if (bagsIdentityButtonLabel) {
    bagsIdentityButtonLabel.textContent = getBagsIdentityMode() === "linked"
      ? (bagsIdentityState.agentUsername ? `Linked Identity (@${bagsIdentityState.agentUsername})` : "Linked Identity")
      : "Wallet Only";
  }
  if (bagsIdentityButton) {
    bagsIdentityButton.classList.toggle("active", visible && getBagsIdentityMode() === "linked");
    bagsIdentityButton.title = visible
      ? describeBagsIdentity()
      : "";
  }
}

function setBagsIdentityError(message = "") {
  if (bagsIdentityError) bagsIdentityError.textContent = message;
}

function showBagsIdentityModal() {
  if (!bagsIdentityModal) return;
  if (bagsIdentityCurrent) {
    const message = bagsIdentityState.configuredApiKey
      ? `Configured API key detected. Selected wallet must belong to the same Bags account to keep linked mode enabled.`
      : "No Bags API key is configured yet.";
    bagsIdentityCurrent.hidden = false;
    bagsIdentityCurrent.textContent = message;
  }
  if (bagsAgentUsernameInput) bagsAgentUsernameInput.value = bagsIdentityState.agentUsername || "";
  if (bagsVerificationContent) bagsVerificationContent.value = bagsIdentityState.verificationPostContent || "";
  if (bagsPostIdInput) bagsPostIdInput.value = "";
  setBagsIdentityError("");
  bagsIdentityModal.hidden = false;
}

function hideBagsIdentityModal({ preserveLinked = false } = {}) {
  if (!bagsIdentityModal) return;
  bagsIdentityModal.hidden = true;
  if (!preserveLinked && getBagsIdentityMode() === "linked" && !bagsIdentityState.verified) {
    setBagsIdentityStateInputs({
      mode: "wallet-only",
      agentUsername: "",
      authToken: "",
      verifiedWallet: "",
      publicIdentifier: "",
      secret: "",
      verificationPostContent: "",
      error: "",
    });
    syncBagsIdentityUI();
  }
}

async function refreshBagsIdentityStatus() {
  const walletKey = selectedWalletKey();
  const query = walletKey ? `?wallet=${encodeURIComponent(walletKey)}` : "";
  const response = await fetch(`/api/bags/identity/status${query}`);
  const payload = await response.json();
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to load Bags identity status.");
  }
  setBagsIdentityStateInputs({
    configuredApiKey: Boolean(payload.configuredApiKey),
    verified: Boolean(payload.verified),
    agentUsername: payload.agentUsername || "",
    authToken: payload.authToken || "",
    verifiedWallet: payload.verifiedWallet || "",
    mode: payload.mode === "linked" && payload.verified ? "linked" : getBagsIdentityMode(),
  });
  if (getBagsIdentityMode() === "linked" && !payload.verified) {
    setBagsIdentityStateInputs({ mode: "wallet-only" });
  }
  syncBagsIdentityUI();
  return payload;
}

async function initBagsIdentityVerification() {
  const response = await fetch("/api/bags/identity/init", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      apiKey: bagsApiKeyInput ? bagsApiKeyInput.value.trim() : "",
      saveApiKey: Boolean(bagsApiKeySave && bagsApiKeySave.checked),
      agentUsername: bagsAgentUsernameInput ? bagsAgentUsernameInput.value.trim() : "",
    }),
  });
  const payload = await response.json();
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to initialize Bags identity verification.");
  }
  setBagsIdentityStateInputs({
    configuredApiKey: Boolean(payload.configuredApiKey),
    agentUsername: payload.agentUsername || "",
    publicIdentifier: payload.publicIdentifier || "",
    secret: payload.secret || "",
    verificationPostContent: payload.verificationPostContent || "",
  });
  if (bagsVerificationContent) bagsVerificationContent.value = payload.verificationPostContent || "";
  if (bagsAgentUsernameInput) bagsAgentUsernameInput.value = payload.agentUsername || "";
  syncBagsIdentityUI();
  return payload;
}

async function verifyBagsIdentity() {
  const response = await fetch("/api/bags/identity/verify", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      apiKey: bagsApiKeyInput ? bagsApiKeyInput.value.trim() : "",
      saveApiKey: Boolean(bagsApiKeySave && bagsApiKeySave.checked),
      agentUsername: bagsAgentUsernameInput ? bagsAgentUsernameInput.value.trim() : "",
      publicIdentifier: bagsIdentityState.publicIdentifier || "",
      secret: bagsIdentityState.secret || "",
      postId: bagsPostIdInput ? bagsPostIdInput.value.trim() : "",
      walletEnvKey: selectedWalletKey(),
    }),
  });
  const payload = await response.json();
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to verify Bags identity.");
  }
  setBagsIdentityStateInputs({
    mode: "linked",
    configuredApiKey: Boolean(payload.configuredApiKey),
    verified: Boolean(payload.verified),
    agentUsername: payload.agentUsername || "",
    authToken: payload.authToken || "",
    verifiedWallet: payload.verifiedWallet || "",
  });
  syncBagsIdentityUI();
  hideBagsIdentityModal({ preserveLinked: true });
}

function setLaunchpad(launchpad, { resetMode = false, persistMode = false } = {}) {
  const normalized = normalizeLaunchpad(launchpad);
  const target = document.querySelector(`input[name="launchpad"][value="${CSS.escape(normalized)}"]`)
    || document.querySelector('input[name="launchpad"][value="pump"]');
  if (!target || target.disabled) return;
  target.checked = true;
  if (resetMode) {
    setImportedCreatorFeeState(null);
    const nextMode = defaultLaunchModeForLaunchpad(normalized);
    setMode(nextMode);
    if (persistMode) setStoredLaunchMode(nextMode);
  }
}

function applyImportedLaunchContext(token = {}) {
  const launchpad = normalizeLaunchpad(token.launchpad || getLaunchpad());
  setLaunchpad(launchpad, { resetMode: true, persistMode: false });
  setMode(token.mode || defaultLaunchModeForLaunchpad(launchpad));
  if (launchpad === "bonk") {
    setNamedValue("quoteAsset", normalizeQuoteAsset(token.quoteAsset || "sol"));
  } else {
    setNamedValue("quoteAsset", "sol");
  }
  applyImportedRouteState({
    launchpad,
    feeSharingRecipients: token.routes && Array.isArray(token.routes.feeSharingRecipients)
      ? token.routes.feeSharingRecipients
      : [],
    agentFeeRecipients: token.routes && Array.isArray(token.routes.agentFeeRecipients)
      ? token.routes.agentFeeRecipients
      : [],
    creatorFee: token.routes && token.routes.creatorFee ? token.routes.creatorFee : null,
  });
  syncLaunchpadModeOptions();
  syncBonkQuoteAssetUI();
  syncBagsIdentityUI();
}

function getLaunchpadUiCapabilities(launchpad = getLaunchpad()) {
  const entry = latestLaunchpadRegistry && latestLaunchpadRegistry[launchpad];
  const supportsStrategies = entry && entry.supportsStrategies ? entry.supportsStrategies : {};
  if (launchpad === "pump") {
    return {
      allowedModes: ["regular", "cashback", "agent-custom", "agent-unlocked", "agent-locked"],
      mayhem: true,
      feeSplit: true,
      vanity: true,
      sniper: true,
      autoSell: true,
    };
  }
  if (launchpad === "bonk") {
    return {
      allowedModes: ["regular", "bonkers"],
      mayhem: false,
      feeSplit: false,
      vanity: true,
      sniper: supportsStrategies["snipe-own-launch"] !== false,
      autoSell: supportsStrategies["automatic-dev-sell"] !== false,
    };
  }
  if (launchpad === "bagsapp") {
    return {
      allowedModes: ["bags-2-2", "bags-025-1", "bags-1-025"],
      mayhem: false,
      feeSplit: true,
      vanity: false,
      sniper: supportsStrategies["snipe-own-launch"] !== false,
      autoSell: supportsStrategies["automatic-dev-sell"] !== false,
    };
  }
  return {
    allowedModes: ["regular"],
    mayhem: false,
    feeSplit: false,
    vanity: true,
    sniper: supportsStrategies["snipe-own-launch"] === true,
    autoSell: supportsStrategies["automatic-dev-sell"] === true,
  };
}

function syncLaunchpadModeOptions() {
  const { allowedModes, mayhem, feeSplit, vanity, sniper, autoSell } = getLaunchpadUiCapabilities();
  form.querySelectorAll('input[name="mode"]').forEach((input) => {
    const option = input.closest(".mode-option");
    const visible = allowedModes.includes(input.value);
    if (option) option.hidden = !visible;
    input.disabled = !visible;
  });
  const mayhemInput = form.querySelector('input[name="mayhemMode"]');
  if (mayhemInput) {
    const mayhemOption = mayhemInput.closest(".mode-option");
    if (mayhemOption) mayhemOption.hidden = !mayhem;
    mayhemInput.disabled = !mayhem;
    if (!mayhem) mayhemInput.checked = false;
  }
  if (!allowedModes.includes(getMode())) {
    setMode(allowedModes[0] || "regular");
    setStoredLaunchMode(getMode());
  }
  if (feeSplitPill) {
    feeSplitPill.hidden = !feeSplit;
    if (!feeSplit && feeSplitEnabled) feeSplitEnabled.checked = false;
    if (!feeSplit && feeSplitModal) feeSplitModal.hidden = true;
  }
  if (modeVanityButton) {
    modeVanityButton.hidden = !vanity;
    if (!vanity) hideVanityModal();
  }
  if (modeSniperButton) {
    modeSniperButton.hidden = !sniper;
    if (!sniper) hideSniperModal();
  }
  if (devAutoSellButton) {
    devAutoSellButton.hidden = !autoSell;
    if (!autoSell && devAutoSellPanel) devAutoSellPanel.hidden = true;
    if (!autoSell && autoSellEnabledInput) autoSellEnabledInput.checked = false;
  }
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

function syncAutoFeeButtonState(button, input) {
  if (!button || !input) return;
  button.classList.toggle("active", Boolean(input.checked));
  button.setAttribute("aria-pressed", input.checked ? "true" : "false");
  button.disabled = Boolean(input.disabled);
}

function syncToggleButtonState(button, input) {
  if (!button || !input) return;
  button.classList.toggle("active", Boolean(input.checked));
  button.setAttribute("aria-pressed", input.checked ? "true" : "false");
  button.disabled = Boolean(input.disabled);
}

function syncAutoFeeButtons() {
  syncAutoFeeButtonState(creationAutoFeeButton, creationAutoFeeInput);
  syncAutoFeeButtonState(buyAutoFeeButton, buyAutoFeeInput);
  syncAutoFeeButtonState(sellAutoFeeButton, sellAutoFeeInput);
}

function isHelloMoonProvider(provider) {
  return String(provider || "").trim().toLowerCase() === "hellomoon";
}

function defaultMevModeForProvider(provider) {
  return isHelloMoonProvider(provider) ? "reduced" : "off";
}

function normalizeMevMode(value, fallback = "off") {
  if (typeof value === "boolean") return value ? "reduced" : "off";
  const normalized = String(value || "").trim().toLowerCase();
  return normalized === "reduced" || normalized === "secure" || normalized === "off"
    ? normalized
    : fallback;
}

function normalizeSelectableMevMode(provider, value, fallback = "off") {
  return normalizeMevMode(value, fallback);
}

function setMevModeOptionAvailability(select, provider) {
  if (!select) return;
  const secureOption = Array.from(select.options).find((option) => option.value === "secure");
  if (!secureOption) return;
  void provider;
  secureOption.disabled = false;
  secureOption.textContent = "Secure";
  secureOption.title = "";
}

function setMevModeSelectValue(select, value, fallback = "off", provider = "") {
  if (!select) return;
  setMevModeOptionAvailability(select, provider);
  select.value = normalizeSelectableMevMode(provider, value, fallback);
}

function setFieldVisibility(input, visible) {
  if (!input) return;
  const label = input.closest("label");
  if (label) label.hidden = !visible;
}

function isStandardRpcProvider(provider) {
  return String(provider || "").trim().toLowerCase() === "standard-rpc";
}

function parseNumericSettingValue(value) {
  const normalized = String(value || "").trim().replace(",", ".");
  if (!normalized) return null;
  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}

function ensureStandardRpcSlippageDefault(input, provider) {
  if (!input || !isStandardRpcProvider(provider)) return false;
  const parsed = parseNumericSettingValue(input.value);
  if (parsed == null || parsed === 90) {
    if (input.value.trim() === STANDARD_RPC_SLIPPAGE_DEFAULT) return false;
    input.value = STANDARD_RPC_SLIPPAGE_DEFAULT;
    return true;
  }
  return false;
}

function standardRpcSlippageWarningText(sideLabel, input) {
  const parsed = parseNumericSettingValue(input && input.value);
  const overrideText = parsed != null && parsed > Number(STANDARD_RPC_SLIPPAGE_DEFAULT)
    ? " Values above 20% should only be used intentionally for edge cases."
    : " Default slippage is 20%."
  return `Standard RPC ${sideLabel}: higher MEV and slippage risk.${overrideText}`;
}

function hellomoonMevWarningText(sideLabel) {
  return `Hello Moon ${sideLabel}: Off mode uses QUIC without MEV protection and carries higher MEV risk than Reduced.`;
}

function syncStandardRpcWarnings() {
  const buyIsStandardRpc = isStandardRpcProvider(getBuyProvider());
  if (buyStandardRpcWarning) {
    buyStandardRpcWarning.hidden = !buyIsStandardRpc;
    buyStandardRpcWarning.textContent = buyIsStandardRpc
      ? standardRpcSlippageWarningText("buys", buySlippageInput)
      : "";
  }
  const sellIsStandardRpc = isStandardRpcProvider(getSellProvider());
  if (sellStandardRpcWarning) {
    sellStandardRpcWarning.hidden = !sellIsStandardRpc;
    sellStandardRpcWarning.textContent = sellIsStandardRpc
      ? standardRpcSlippageWarningText("sells", sellSlippageInput)
      : "";
  }
}

function syncHelloMoonMevWarnings() {
  const buyHasHelloMoonOff = isHelloMoonProvider(getBuyProvider())
    && normalizeMevMode(buyMevModeSelect ? buyMevModeSelect.value : "off") === "off";
  if (buyHelloMoonMevWarning) {
    buyHelloMoonMevWarning.hidden = !buyHasHelloMoonOff;
    buyHelloMoonMevWarning.textContent = buyHasHelloMoonOff
      ? hellomoonMevWarningText("buys")
      : "";
  }
  const sellHasHelloMoonOff = isHelloMoonProvider(getSellProvider())
    && normalizeMevMode(sellMevModeSelect ? sellMevModeSelect.value : "off") === "off";
  if (sellHelloMoonMevWarning) {
    sellHelloMoonMevWarning.hidden = !sellHasHelloMoonOff;
    sellHelloMoonMevWarning.textContent = sellHasHelloMoonOff
      ? hellomoonMevWarningText("sells")
      : "";
  }
}

function syncAutoFeeControls() {
  const editing = isPresetEditing(getConfig());
  const creationAuto = Boolean(creationAutoFeeInput && creationAutoFeeInput.checked);
  const buyAuto = Boolean(buyAutoFeeInput && buyAutoFeeInput.checked);
  const sellAuto = Boolean(sellAutoFeeInput && sellAutoFeeInput.checked);
  const creationCapabilities = getRouteCapabilities(getProvider(), "creation");
  const buyCapabilities = getRouteCapabilities(getBuyProvider(), "buy");
  const sellCapabilities = getRouteCapabilities(getSellProvider(), "sell");

  setFieldEnabled(creationPriorityInput, editing && creationCapabilities.priority && !creationAuto);
  setFieldEnabled(creationTipInput, editing && creationCapabilities.tip && !creationAuto);
  setFieldEnabled(creationMaxFeeInput, editing && (creationCapabilities.priority || creationCapabilities.tip) && creationAuto);

  setFieldEnabled(buyPriorityFeeInput, editing && buyCapabilities.priority && !buyAuto);
  setFieldEnabled(buyTipInput, editing && buyCapabilities.tip && !buyAuto);
  setFieldEnabled(buyMaxFeeInput, editing && (buyCapabilities.priority || buyCapabilities.tip) && buyAuto);

  setFieldEnabled(sellPriorityFeeInput, editing && sellCapabilities.priority && !sellAuto);
  setFieldEnabled(sellTipInput, editing && sellCapabilities.tip && !sellAuto);
  setFieldEnabled(sellMaxFeeInput, editing && (sellCapabilities.priority || sellCapabilities.tip) && sellAuto);
  syncAutoFeeButtons();
}

function syncSettingsCapabilities() {
  const editing = isPresetEditing(getConfig());
  const creationProvider = getProvider();
  const buyProvider = getBuyProvider();
  const sellProvider = getSellProvider();
  const creationCapabilities = getRouteCapabilities(getProvider(), "creation");
  const buyCapabilities = getRouteCapabilities(getBuyProvider(), "buy");
  const sellCapabilities = getRouteCapabilities(getSellProvider(), "sell");

  if (providerSelect) providerSelect.disabled = !editing;
  if (buyProviderSelect) buyProviderSelect.disabled = !editing;
  if (sellProviderSelect) sellProviderSelect.disabled = !editing;
  setFieldVisibility(creationTipInput, creationCapabilities.tip);
  setFieldVisibility(creationPriorityInput, creationCapabilities.priority);
  setFieldVisibility(creationMevModeSelect, isHelloMoonProvider(creationProvider));
  setFieldVisibility(buyPriorityFeeInput, buyCapabilities.priority);
  setFieldVisibility(buyTipInput, buyCapabilities.tip);
  setFieldVisibility(buySlippageInput, buyCapabilities.slippage);
  setFieldVisibility(buyMevModeSelect, isHelloMoonProvider(buyProvider));
  setFieldVisibility(sellPriorityFeeInput, sellCapabilities.priority);
  setFieldVisibility(sellTipInput, sellCapabilities.tip);
  setFieldVisibility(sellSlippageInput, sellCapabilities.slippage);
  setFieldVisibility(sellMevModeSelect, isHelloMoonProvider(sellProvider));
  setMevModeOptionAvailability(creationMevModeSelect, creationProvider);
  setMevModeOptionAvailability(buyMevModeSelect, buyProvider);
  setMevModeOptionAvailability(sellMevModeSelect, sellProvider);
  setFieldEnabled(creationAutoFeeInput, editing && (creationCapabilities.priority || creationCapabilities.tip));
  setFieldEnabled(buyAutoFeeInput, editing && (buyCapabilities.priority || buyCapabilities.tip));
  setFieldEnabled(sellAutoFeeInput, editing && (sellCapabilities.priority || sellCapabilities.tip));
  setFieldEnabled(creationMevModeSelect, editing && isHelloMoonProvider(creationProvider));
  setFieldEnabled(buyMevModeSelect, editing && isHelloMoonProvider(buyProvider));
  setFieldEnabled(sellMevModeSelect, editing && isHelloMoonProvider(sellProvider));
  setFieldEnabled(buySlippageInput, editing && buyCapabilities.slippage);
  setFieldEnabled(sellSlippageInput, editing && sellCapabilities.slippage);
  syncAutoFeeControls();
  syncStandardRpcWarnings();
  syncHelloMoonMevWarnings();
}

function applyPresetToSettingsInputs(preset, options = {}) {
  if (!preset) return;
  const { syncToMainForm = true } = options;
  syncingPresetInputs = true;
  if (providerSelect) providerSelect.value = preset.creationSettings.provider || "helius-sender";
  if (creationTipInput) creationTipInput.value = preset.creationSettings.tipSol || "";
  if (creationPriorityInput) creationPriorityInput.value = preset.creationSettings.priorityFeeSol || "";
  setMevModeSelectValue(
    creationMevModeSelect,
    preset.creationSettings.mevMode,
    defaultMevModeForProvider(preset.creationSettings.provider),
    preset.creationSettings.provider
  );
  if (creationAutoFeeInput) creationAutoFeeInput.checked = Boolean(preset.creationSettings.autoFee);
  if (creationMaxFeeInput) creationMaxFeeInput.value = preset.creationSettings.maxFeeSol || "";
  if (buyProviderSelect) buyProviderSelect.value = preset.buySettings.provider || "helius-sender";
  if (buyPriorityFeeInput) buyPriorityFeeInput.value = preset.buySettings.priorityFeeSol || "";
  if (buyTipInput) buyTipInput.value = preset.buySettings.tipSol || "";
  if (buySlippageInput) buySlippageInput.value = preset.buySettings.slippagePercent || "";
  setMevModeSelectValue(
    buyMevModeSelect,
    preset.buySettings.mevMode ?? preset.buySettings.mevProtect,
    defaultMevModeForProvider(preset.buySettings.provider),
    preset.buySettings.provider
  );
  if (buyAutoFeeInput) buyAutoFeeInput.checked = Boolean(preset.buySettings.autoFee);
  if (buyMaxFeeInput) buyMaxFeeInput.value = preset.buySettings.maxFeeSol || "";
  if (sellProviderSelect) sellProviderSelect.value = preset.sellSettings.provider || "helius-sender";
  if (sellPriorityFeeInput) sellPriorityFeeInput.value = preset.sellSettings.priorityFeeSol || "";
  if (sellTipInput) sellTipInput.value = preset.sellSettings.tipSol || "";
  if (sellSlippageInput) sellSlippageInput.value = preset.sellSettings.slippagePercent || "";
  setMevModeSelectValue(
    sellMevModeSelect,
    preset.sellSettings.mevMode ?? preset.sellSettings.mevProtect,
    defaultMevModeForProvider(preset.sellSettings.provider),
    preset.sellSettings.provider
  );
  if (sellAutoFeeInput) sellAutoFeeInput.checked = Boolean(preset.sellSettings.autoFee);
  if (sellMaxFeeInput) sellMaxFeeInput.value = preset.sellSettings.maxFeeSol || "";
  syncingPresetInputs = false;
  const standardizedDefaultsApplied =
    ensureStandardRpcSlippageDefault(buySlippageInput, getBuyProvider())
    || ensureStandardRpcSlippageDefault(sellSlippageInput, getSellProvider());

  if (syncToMainForm) {
    clearDevBuyState();
  }

  syncDevAutoSellUI();
  syncSettingsCapabilities();
  if (standardizedDefaultsApplied) {
    syncActivePresetFromInputs();
  }
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
    mevMode: normalizeMevMode(creationMevModeSelect ? creationMevModeSelect.value : "off"),
    autoFee: Boolean(creationAutoFeeInput && creationAutoFeeInput.checked),
    maxFeeSol: normalizeAutoFeeCapValue(creationMaxFeeInput ? creationMaxFeeInput.value : ""),
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
    mevMode: normalizeMevMode(buyMevModeSelect ? buyMevModeSelect.value : "off"),
    autoFee: Boolean(buyAutoFeeInput && buyAutoFeeInput.checked),
    maxFeeSol: normalizeAutoFeeCapValue(buyMaxFeeInput ? buyMaxFeeInput.value : ""),
  };
  activePreset.sellSettings = {
    ...activePreset.sellSettings,
    provider: getSellProvider(),
    priorityFeeSol: sellPriorityFeeInput ? sellPriorityFeeInput.value.trim() : "",
    tipSol: sellTipInput ? sellTipInput.value.trim() : "",
    slippagePercent: sellSlippageInput ? sellSlippageInput.value.trim() : "",
    mevMode: normalizeMevMode(sellMevModeSelect ? sellMevModeSelect.value : "off"),
    autoFee: Boolean(sellAutoFeeInput && sellAutoFeeInput.checked),
    maxFeeSol: normalizeAutoFeeCapValue(sellMaxFeeInput ? sellMaxFeeInput.value : ""),
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
  // Document-level click handler runs in capture phase before this runs; it would post stale routes.
  queueWarmActivity({ immediate: true });
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
    creationAutoFeeInput,
    creationMaxFeeInput,
    buyProviderSelect,
    buyPriorityFeeInput,
    buyTipInput,
    buySlippageInput,
    buyMevModeSelect,
    buyAutoFeeInput,
    buyMaxFeeInput,
    sellProviderSelect,
    sellPriorityFeeInput,
    sellTipInput,
    sellSlippageInput,
    sellMevModeSelect,
    sellAutoFeeInput,
    sellMaxFeeInput,
  ];
  inputs.forEach((input) => {
    if (!input) return;
    input.disabled = !editing;
  });
  syncSettingsCapabilities();
}


function renderVanityButtonState() {
  if (!modeVanityButton) return;
  const hasVanityKey = Boolean(getNamedValue("vanityPrivateKey").trim());
  modeVanityButton.classList.toggle("active", hasVanityKey);
  if (!vanityDerivedAddressPill && modeVanityButton.parentElement) {
    vanityDerivedAddressPill = document.createElement("div");
    vanityDerivedAddressPill.id = "mode-vanity-address";
    vanityDerivedAddressPill.className = "vanity-derived-address";
    vanityDerivedAddressPill.hidden = true;
    modeVanityButton.parentElement.insertAdjacentElement("afterend", vanityDerivedAddressPill);
  }
  if (!vanityDerivedAddressPill) return;
  if (!hasVanityKey || !vanityDerivedPublicKey || modeVanityButton.hidden) {
    vanityDerivedAddressPill.hidden = true;
    vanityDerivedAddressPill.textContent = "";
    vanityDerivedAddressPill.removeAttribute("title");
    return;
  }
  vanityDerivedAddressPill.hidden = false;
  vanityDerivedAddressPill.textContent = `Vanity CA: ${vanityDerivedPublicKey}`;
  vanityDerivedAddressPill.title = vanityDerivedPublicKey;
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
  if (vampAutoImportTimer) {
    window.clearTimeout(vampAutoImportTimer);
    vampAutoImportTimer = null;
  }
  if (vampModal) vampModal.hidden = true;
}

function setVampStatus(message = "") {
  if (!vampStatus) return;
  vampStatus.hidden = !message;
  vampStatus.textContent = message;
}

function looksLikeSolanaAddress(value) {
  return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(String(value || "").trim());
}

function scheduleVampAutoImport() {
  if (vampAutoImportTimer) {
    window.clearTimeout(vampAutoImportTimer);
    vampAutoImportTimer = null;
  }
  if (!vampModal || vampModal.hidden || !vampContractInput) return;
  const contractAddress = vampContractInput.value.trim();
  if (!looksLikeSolanaAddress(contractAddress)) return;
  vampAutoImportTimer = window.setTimeout(() => {
    vampAutoImportTimer = null;
    if (!vampModal || vampModal.hidden || !vampContractInput) return;
    if (vampContractInput.value.trim() !== contractAddress) return;
    if (contractAddress === vampInFlightAddress) return;
    if (vampImport && vampImport.disabled) return;
    importVampToken().catch(() => {});
  }, 150);
}

async function importVampToken() {
  const contractAddress = vampContractInput ? vampContractInput.value.trim() : "";
  if (!contractAddress) {
    if (vampError) vampError.textContent = "Contract address is required.";
    return;
  }
  if (vampError) vampError.textContent = "";
  setVampStatus("Importing token metadata...");
  vampInFlightAddress = contractAddress;
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
    applyImportedLaunchContext(payload.token || {});
    clearMetadataUploadCache({ clearInput: true });
    updateTokenFieldCounts();

    if (payload.image) {
      const importedPreviewUrl = String(payload.image.previewUrl || "").trim()
        || (payload.image.fileName ? `/uploads/${encodeURIComponent(payload.image.fileName)}` : "");
      imageLibraryState.activeImageId = payload.image.id || "";
      setSelectedImage(payload.image);
      if (importedPreviewUrl) {
        setImagePreview(importedPreviewUrl);
      }
      try {
        await fetchImageLibrary();
        const refreshedImportedImage = imageLibraryState.images.find((entry) => entry.id === imageLibraryState.activeImageId);
        if (refreshedImportedImage) {
          setSelectedImage(refreshedImportedImage);
        } else if (importedPreviewUrl) {
          setImagePreview(importedPreviewUrl);
        }
      } catch (_error) {
        // Keep the imported image selected even if the library refresh fails.
        if (importedPreviewUrl) {
          setImagePreview(importedPreviewUrl);
        }
      }
    }

    const detectionNotes = payload.token && payload.token.detection && Array.isArray(payload.token.detection.notes)
      ? payload.token.detection.notes.filter(Boolean)
      : [];
    imageStatus.textContent = [
      payload.image ? "Token image imported to library." : "",
      payload.warning || "",
      detectionNotes.join(" "),
    ].filter(Boolean).join(" ");
    imagePath.textContent = "";
    hideVampModal();
  } catch (error) {
    if (vampError) vampError.textContent = error.message;
    setVampStatus("");
  } finally {
    if (vampInFlightAddress === contractAddress) vampInFlightAddress = "";
    if (vampImport) vampImport.disabled = false;
    if (vampCancel) vampCancel.disabled = false;
    if (vampClose) vampClose.disabled = false;
  }
}

function applyVanityValue(rawValue, options = {}) {
  const nextValue = String(rawValue || "").trim();
  if (vanityPrivateKeyInput) vanityPrivateKeyInput.value = nextValue;
  if (!nextValue) {
    vanityDerivedPublicKey = "";
  } else if (options && typeof options.publicKey === "string" && options.publicKey.trim()) {
    vanityDerivedPublicKey = options.publicKey.trim();
  }
  renderVanityButtonState();
}

async function validateVanityPrivateKey(rawValue) {
  const nextValue = String(rawValue || "").trim();
  if (!nextValue) return { ok: true, normalizedPrivateKey: "" };
  const response = await fetch("/api/vanity/validate", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ privateKey: nextValue }),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Invalid vanity private key.");
  }
  return payload;
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
  setStatusLabel("Loading");
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

function getStoredWalletStatusLastRefreshAtMs() {
  try {
    const raw = window.localStorage.getItem(WALLET_STATUS_LAST_REFRESH_STORAGE_KEY);
    const numeric = Number(raw);
    return Number.isFinite(numeric) && numeric > 0 ? numeric : 0;
  } catch (_error) {
    return 0;
  }
}

function setStoredWalletStatusLastRefreshAtMs(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return;
  try {
    window.localStorage.setItem(WALLET_STATUS_LAST_REFRESH_STORAGE_KEY, String(Math.round(numeric)));
  } catch (_error) {
    // Ignore storage access failures and keep the UI functional.
  }
}

function walletStatusRefreshDelayMs(referenceMs = Date.now()) {
  const lastRefreshAtMs = getStoredWalletStatusLastRefreshAtMs();
  if (!lastRefreshAtMs) return walletStatusRefreshIntervalMs;
  return Math.max(0, walletStatusRefreshIntervalMs - Math.max(0, referenceMs - lastRefreshAtMs));
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

function normalizeVisibleWallets(wallets) {
  const seen = new Set();
  return (Array.isArray(wallets) ? wallets : [])
    .filter((wallet) => wallet && typeof wallet === "object")
    .filter((wallet) => {
      const envKey = String(wallet.envKey || "").trim();
      const publicKey = String(wallet.publicKey || "").trim();
      if (!envKey || !publicKey || seen.has(envKey)) return false;
      seen.add(envKey);
      return true;
    })
    .map((wallet) => ({
      ...wallet,
      envKey: String(wallet.envKey || "").trim(),
      publicKey: String(wallet.publicKey || "").trim(),
    }));
}

function resolveVisibleSelectedWalletKey(selectedKey, wallets) {
  const normalizedKey = String(selectedKey || "").trim();
  const visibleWallets = normalizeVisibleWallets(wallets);
  if (normalizedKey && visibleWallets.some((wallet) => wallet.envKey === normalizedKey)) {
    return normalizedKey;
  }
  return visibleWallets[0] ? visibleWallets[0].envKey : "";
}

function formatWalletHistoryLabel(envKey) {
  const normalizedKey = String(envKey || "").trim();
  if (!normalizedKey) return "";
  const index = walletIndexFromEnvKey(normalizedKey);
  const wallets = latestWalletStatus && Array.isArray(latestWalletStatus.wallets) ? latestWalletStatus.wallets : [];
  const wallet = wallets.find((entry) => entry && entry.envKey === normalizedKey) || null;
  const customName = wallet && wallet.customName ? String(wallet.customName).trim() : "";
  return customName ? `Wallet #${index} ${customName}` : `Wallet #${index}`;
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

function formatWalletDropdownAmount(value) {
  if (value == null || Number.isNaN(Number(value))) return "--";
  return Number(value).toFixed(4);
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
          <span class="wallet-option-sol">
            <img src="/solana-mark.png" alt="SOL" class="wallet-balance-icon">
            <span>${escapeHTML(formatWalletDropdownAmount(solValue))}</span>
          </span>
          <span class="wallet-option-usd">
            <img src="/usd1-mark.png" alt="USD1" class="wallet-balance-icon wallet-balance-icon-usd1">
            <span>${escapeHTML(formatWalletDropdownAmount(usdValue))}</span>
          </span>
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

function formatLegendRecipientLabel(type, value, fallback = "wallet") {
  const normalized = String(value || "").trim();
  if (!normalized) {
    return { full: fallback, short: fallback };
  }
  if (type === "github") {
    const full = `@${normalized.replace(/^@+/, "")}`;
    return {
      full,
      short: full.length > 14 ? `${full.slice(0, 7)}...${full.slice(-4)}` : full,
    };
  }
  return {
    full: normalized,
    short: shortenAddress(normalized, 3) || fallback,
  };
}

function setFeeSplitModalError(message = "") {
  if (feeSplitModalError) feeSplitModalError.textContent = message;
}

function getDeployerFeeSplitAddress() {
  return String(latestWalletStatus && latestWalletStatus.wallet || "").trim();
}

function hasMeaningfulFeeSplitRecipients(recipients) {
  const entries = Array.isArray(recipients) ? recipients.filter(Boolean) : [];
  if (entries.length === 0) return false;
  if (entries.length !== 1) return true;
  const [entry] = entries;
  if (!entry || entry.type !== "wallet") return true;
  const deployerAddress = getDeployerFeeSplitAddress();
  if (!deployerAddress) return true;
  return String(entry.address || "").trim() !== deployerAddress || Number(entry.shareBps || 0) !== 10_000;
}

function hasMeaningfulFeeSplitConfiguration() {
  return hasMeaningfulFeeSplitRecipients(collectFeeSplitRecipients());
}

function hasMeaningfulAgentSplitRecipients(recipients) {
  const entries = Array.isArray(recipients) ? recipients.filter(Boolean) : [];
  if (entries.length === 0) return false;
  const positiveAgentShare = entries
    .filter((entry) => entry.type === "agent")
    .reduce((sum, entry) => sum + Math.max(0, Number(entry.shareBps || 0)), 0);
  if (positiveAgentShare > 0) return true;
  const positiveNonAgentEntries = entries.filter((entry) => entry.type !== "agent" && Number(entry.shareBps || 0) > 0);
  if (positiveNonAgentEntries.length === 0) return false;
  if (positiveNonAgentEntries.length !== 1) return true;
  const [entry] = positiveNonAgentEntries;
  if (!entry || entry.type !== "wallet") return true;
  const deployerAddress = getDeployerFeeSplitAddress();
  if (!deployerAddress) return true;
  return String(entry.address || "").trim() !== deployerAddress || Number(entry.shareBps || 0) !== 10_000;
}

function hasMeaningfulAgentSplitConfiguration() {
  return hasMeaningfulAgentSplitRecipients(collectAgentSplitRecipients());
}

function finalizeFeeSplitDraftForMode() {
  const draft = normalizeFeeSplitDraft(serializeFeeSplitDraft());
  const mode = getMode();
  if (mode === "regular" && !hasMeaningfulFeeSplitConfiguration()) {
    return normalizeFeeSplitDraft({ enabled: false, rows: [] });
  }
  if (mode === "regular") {
    draft.enabled = true;
  }
  return draft;
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
  if (row.dataset.type === "github") {
    const parsedGithubTarget = parseGithubRecipientTarget(entry.value || "");
    const githubUserId = String(entry.githubUserId || parsedGithubTarget.githubUserId || "").trim();
    if (githubUserId) row.dataset.githubUserId = githubUserId;
  }
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
  if (type !== "github") delete row.dataset.githubUserId;
  row.querySelectorAll(".recipient-type-tab").forEach((button) => {
    button.classList.toggle("active", button.dataset.type === type);
  });
  const target = row.querySelector(".recipient-target");
  target.placeholder = type === "github" ? "GitHub username or user id" : "Wallet address";
}

function setRecipientTargetLocked(row, locked) {
  if (!row || row.dataset.locked === "true") return;
  const target = row.querySelector(".recipient-target");
  const toggle = row.querySelector(".recipient-lock-toggle");
  if (!target || !toggle) return;
  target.setCustomValidity("");

  if (locked) {
    if (!target.value.trim()) {
      target.focus();
      return;
    }
    if (row.dataset.type === "github" && looksLikeSolanaAddress(target.value)) {
      target.setCustomValidity("GitHub recipients must use a GitHub username or numeric user id, not a Solana address.");
      target.reportValidity();
      target.focus();
      return;
    }
    row.dataset.targetLocked = "true";
    target.readOnly = true;
    target.title = target.value.trim();
    toggle.textContent = "Edit";
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
  const hasNonDefaultRows = getFeeSplitRows().some((row) => row.dataset.defaultReceiver !== "true");
  if (feeSplitList.dataset.suppressDefaultRow === "true") {
    if (hasNonDefaultRows) return;
    delete feeSplitList.dataset.suppressDefaultRow;
  }
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
    const label = formatLegendRecipientLabel(
      row.dataset.type === "github" ? "github" : "wallet",
      targetValue,
      row.dataset.type === "github" ? "@github" : "wallet"
    );
    if (share > 0) {
      const start = running;
      running += share;
      gradientStops.push(`${color} ${start}%`, `${color} ${running}%`);
      legendItems.push(
        `<span class="legend-chip" title="${escapeHTML(label.full)}"><span class="legend-dot" style="background:${color}"></span><span class="legend-chip-label">${escapeHTML(label.short)}</span><span class="legend-chip-share">${share.toFixed(2).replace(/\.00$/, "")}%</span></span>`
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
  const isBagsMode = mode.startsWith("bags-");
  const active = mode === "agent-custom"
    || (mode === "regular" && feeSplitEnabled.checked && hasMeaningfulFeeSplitConfiguration())
    || (isBagsMode && hasMeaningfulFeeSplitConfiguration());
  feeSplitPill.classList.toggle("active", active);
  feeSplitPill.disabled = mode !== "regular" && mode !== "agent-custom" && !isBagsMode;
  if ((mode === "regular" && feeSplitEnabled.checked) || isBagsMode) ensureFeeSplitDefaultRow();
  if (mode !== "regular" && !isBagsMode && feeSplitModal) feeSplitModal.hidden = true;
  syncFeeSplitTotals();
  syncSettingsCapabilities();
}

function showFeeSplitModal() {
  const mode = getMode();
  if (mode === "regular" || mode.startsWith("bags-")) {
    feeSplitModalSnapshot = normalizeFeeSplitDraft(serializeFeeSplitDraft());
    clearFeeSplitRestoreState();
    feeSplitEnabled.checked = true;
    updateFeeSplitVisibility();
    ensureFeeSplitDefaultRow();
    setFeeSplitModalError("");
    if (feeSplitModal) feeSplitModal.hidden = false;
    return;
  }
  if (mode === "agent-custom") {
    showAgentSplitModal();
  }
}

function hideFeeSplitModal() {
  setFeeSplitModalError("");
  clearFeeSplitRestoreState();
  if (feeSplitModal) feeSplitModal.hidden = true;
}

function attemptCloseFeeSplitModal() {
  const errors = validateFeeSplit();
  if (errors.length > 0) {
    setFeeSplitModalError(errors[0]);
    return false;
  }
  const nextDraft = finalizeFeeSplitDraftForMode();
  applyFeeSplitDraft(nextDraft, { persist: false });
  updateFeeSplitVisibility();
  setStoredFeeSplitDraft(nextDraft);
  syncAgentSplitDraftFromFeeSplitDraft(nextDraft);
  feeSplitModalSnapshot = nextDraft;
  hideFeeSplitModal();
  return true;
}

function cancelFeeSplitModal() {
  applyFeeSplitDraft(feeSplitModalSnapshot, { persist: false });
  updateFeeSplitVisibility();
  setStoredFeeSplitDraft(normalizeFeeSplitDraft(serializeFeeSplitDraft()));
  feeSplitModalSnapshot = null;
  hideFeeSplitModal();
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
      ? { full: "Agent Buyback", short: "Agent" }
      : formatLegendRecipientLabel(
        row.dataset.type === "github" ? "github" : "wallet",
        targetValue,
        "wallet"
      );
    if (share > 0) {
      const start = running;
      running += share;
      gradientStops.push(`${color} ${start}%`, `${color} ${running}%`);
      legendItems.push(
        `<span class="legend-chip" title="${escapeHTML(label.full)}"><span class="legend-dot" style="background:${color}"></span><span class="legend-chip-label">${escapeHTML(label.short)}</span><span class="legend-chip-share">${share.toFixed(2).replace(/\.00$/, "")}%</span></span>`
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
  clearAgentSplitRestoreState();
  syncAgentSplitTotals();
  if (agentSplitModalError) agentSplitModalError.textContent = "";
  if (agentSplitModal) agentSplitModal.hidden = false;
}

function hideAgentSplitModal() {
  clearAgentSplitRestoreState();
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
  const nextDraft = normalizeAgentSplitDraft(serializeAgentSplitDraft());
  setStoredAgentSplitDraft(nextDraft);
  syncFeeSplitDraftFromAgentSplitDraft(nextDraft);
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
    const parsedGithubTarget = parseGithubRecipientTarget(value);
    const sharePercent = row.querySelector(".recipient-share").value.trim();
    if (!value && !sharePercent) return null;
    const numericShare = Number(sharePercent);
    return {
      type,
      address: type === "wallet" ? value : "",
      githubUsername: type === "github" ? parsedGithubTarget.githubUsername : "",
      githubUserId: type === "github" ? parsedGithubTarget.githubUserId : "",
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
  syncLaunchpadModeOptions();
  syncBonkQuoteAssetUI();
  syncBagsIdentityUI();
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
      label.removeAttribute("title");
    }
    if (titleNode) {
      const baseLabel = input.value === "bagsapp"
        ? "Bagsapp"
        : input.value.charAt(0).toUpperCase() + input.value.slice(1);
      titleNode.textContent = (entry && entry.label) || baseLabel;
    }
  });

  const checked = document.querySelector('input[name="launchpad"]:checked');
  if (!checked || checked.disabled) {
    const fallback = launchpadInputs.find((input) => !input.disabled);
    if (fallback) fallback.checked = true;
  }

  applyLaunchpadTokenMetadata();
  updateModeVisibility();
}

function applyPersistentDefaults(config) {
  if (!config || defaultsApplied) return;
  const defaults = config.defaults || {};
  setImportedCreatorFeeState(null);
  const defaultMode = defaults.mode || "regular";
  const storedSniperDraft = getStoredSniperDraft();
  const storedMode = getStoredLaunchMode();
  const storedLaunchpad = getStoredLaunchpad();
  const storedBonkQuoteAsset = getStoredBonkQuoteAsset();
  const storedFeeSplitDraft = isPopoutMode ? getStoredFeeSplitDraft() : null;
  const storedAgentSplitDraft = isPopoutMode ? getStoredAgentSplitDraft() : null;
  const storedAutoSellDraft = getStoredAutoSellDraft();
  const resolvedMode = storedMode || defaultMode;
  const resolvedFeeSplitDraft = storedFeeSplitDraft || (defaults.misc && defaults.misc.feeSplitDraft) || null;
  const resolvedAgentSplitDraft = storedAgentSplitDraft || (defaults.misc && defaults.misc.agentSplitDraft) || null;
  setLaunchpad(storedLaunchpad || defaults.launchpad || "pump");
  if (bonkQuoteAssetInput) bonkQuoteAssetInput.value = normalizeQuoteAsset(storedBonkQuoteAsset || "sol");
  setConfig(config);
  applyPresetToSettingsInputs(getActivePreset(config));
  warmDevBuyQuoteCache();
  if (storedAutoSellDraft) {
    applyAutoSellDraft(storedAutoSellDraft, { persist: false });
  } else if (defaults.automaticDevSell) {
    applyAutoSellDraft({
      enabled: defaults.automaticDevSell.enabled,
      percent: defaults.automaticDevSell.enabled
        ? Math.max(1, Number(defaults.automaticDevSell.percent || 100))
        : Number(defaults.automaticDevSell.percent || 100),
      triggerFamily: defaults.automaticDevSell.triggerFamily
        || ((Boolean(defaults.automaticDevSell.marketCapEnabled) || Boolean(defaults.automaticDevSell.marketCapThreshold))
          ? "market-cap"
          : "time"),
      triggerMode: defaults.automaticDevSell.triggerMode
        || (Number(defaults.automaticDevSell.delaySeconds || 0) > 0 ? "submit-delay" : "block-offset"),
      delayMs: defaults.automaticDevSell.delayMs != null
        ? defaults.automaticDevSell.delayMs
        : Number(defaults.automaticDevSell.delaySeconds || 0) * 1000,
      blockOffset: defaults.automaticDevSell.targetBlockOffset || 0,
      marketCapEnabled: Boolean(defaults.automaticDevSell.marketCapEnabled)
        || Boolean(defaults.automaticDevSell.marketCapThreshold),
      marketCapThreshold: defaults.automaticDevSell.marketCapThreshold || "",
      marketCapScanTimeoutSeconds: defaults.automaticDevSell.marketCapScanTimeoutSeconds != null
        ? defaults.automaticDevSell.marketCapScanTimeoutSeconds
        : (defaults.automaticDevSell.marketCapScanTimeoutMinutes != null
          ? (defaults.automaticDevSell.marketCapScanTimeoutMinutes * 60)
          : 30),
      marketCapTimeoutAction: defaults.automaticDevSell.marketCapTimeoutAction || "stop",
    }, { persist: false });
  }
  applyFeeSplitDraft(resolvedFeeSplitDraft, { persist: false });
  applyAgentSplitDraft(resolvedAgentSplitDraft, { persist: false });
  if (resolvedMode === "agent-custom" && resolvedAgentSplitDraft) {
    syncFeeSplitDraftFromAgentSplitDraft(resolvedAgentSplitDraft);
  } else if (resolvedFeeSplitDraft) {
    syncAgentSplitDraftFromFeeSplitDraft(resolvedFeeSplitDraft);
  }
  if (defaults.misc && defaults.misc.bagsIdentity) {
    setBagsIdentityStateInputs({
      mode: String(defaults.misc.bagsIdentity.mode || "wallet-only").trim().toLowerCase() === "linked"
        ? "linked"
        : "wallet-only",
      agentUsername: defaults.misc.bagsIdentity.agentUsername || "",
    });
  }
  setMode(resolvedMode);
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
      const githubUserId = String(row.dataset.githubUserId || "").trim();
      const parsedGithubTarget = parseGithubRecipientTarget(value);
      const sharePercent = row.querySelector(".recipient-share").value.trim();
      if (!value && !sharePercent) return null;
      const numericShare = Number(sharePercent);
      return {
        type,
        address: type === "wallet" ? value : "",
        githubUsername: type === "github" ? parsedGithubTarget.githubUsername : "",
        githubUserId: type === "github" ? (githubUserId || parsedGithubTarget.githubUserId) : "",
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
  const autoSellRequested = isNamedChecked("automaticDevSellEnabled");
  const automaticDevSellEnabled = autoSellRequested && Boolean(devBuyAmount);
  const rawAgentSplitRecipients = mode === "agent-custom" ? collectAgentSplitRecipients() : [];
  const agentSplitRecipients = mode === "agent-custom" && hasMeaningfulAgentSplitRecipients(rawAgentSplitRecipients)
    ? rawAgentSplitRecipients
    : [];
  const agentBuyback = rawAgentSplitRecipients.find((entry) => entry.type === "agent");
  const meaningfulFeeSplitEnabled = mode === "regular"
    ? Boolean(feeSplitEnabled && feeSplitEnabled.checked && hasMeaningfulFeeSplitConfiguration())
    : mode.startsWith("bags-");
  let sniperWallets = [];
  try {
    const parsed = JSON.parse(getNamedValue("sniperConfigJson") || "[]");
    if (Array.isArray(parsed)) {
      sniperWallets = parsed.filter((entry) => entry && entry.envKey);
    }
  } catch (_error) {
    sniperWallets = [];
  }

  const creationMaxFeeSol = normalizeAutoFeeCapValue(getNamedValue("creationMaxFeeSol"));
  const buyMaxFeeSol = normalizeAutoFeeCapValue(getNamedValue("buyMaxFeeSol"));
  const sellMaxFeeSol = normalizeAutoFeeCapValue(getNamedValue("sellMaxFeeSol"));

  return {
    selectedWalletKey: selectedWalletKey(),
    launchpad: getLaunchpad(),
    quoteAsset: getQuoteAsset(),
    provider: getProvider(),
    buyProvider: getBuyProvider(),
    sellProvider: getSellProvider(),
    creationMevMode: normalizeMevMode(getNamedValue("creationMevMode"), "off"),
    buyMevMode: normalizeMevMode(getNamedValue("buyMevMode"), "off"),
    sellMevMode: normalizeMevMode(getNamedValue("sellMevMode"), "off"),
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
    autoGas: isNamedChecked("creationAutoFeeEnabled"),
    buyAutoGas: isNamedChecked("buyAutoFeeEnabled"),
    sellAutoGas: isNamedChecked("sellAutoFeeEnabled"),
    creationAutoFeeEnabled: isNamedChecked("creationAutoFeeEnabled"),
    buyAutoFeeEnabled: isNamedChecked("buyAutoFeeEnabled"),
    sellAutoFeeEnabled: isNamedChecked("sellAutoFeeEnabled"),
    priorityFeeSol: creationCapabilities.priority ? (getNamedValue("creationPriorityFeeSol") || "") : "",
    creationTipSol: creationCapabilities.tip ? (getNamedValue("creationTipSol") || "") : "",
    creationMaxFeeSol,
    maxPriorityFeeSol: isNamedChecked("creationAutoFeeEnabled") ? creationMaxFeeSol : (creationCapabilities.priority ? (getNamedValue("creationPriorityFeeSol") || "") : ""),
    maxTipSol: isNamedChecked("creationAutoFeeEnabled") ? creationMaxFeeSol : (creationCapabilities.tip ? (getNamedValue("creationTipSol") || "") : ""),
    buyPriorityFeeSol: buyCapabilities.priority ? (getNamedValue("buyPriorityFeeSol") || "") : "",
    buyTipSol: buyCapabilities.tip ? (getNamedValue("buyTipSol") || "") : "",
    buySlippagePercent: getNamedValue("buySlippagePercent") || "",
    buyMaxFeeSol,
    buyMaxPriorityFeeSol: isNamedChecked("buyAutoFeeEnabled") ? buyMaxFeeSol : (buyCapabilities.priority ? (getNamedValue("buyPriorityFeeSol") || "") : ""),
    buyMaxTipSol: isNamedChecked("buyAutoFeeEnabled") ? buyMaxFeeSol : (buyCapabilities.tip ? (getNamedValue("buyTipSol") || "") : ""),
    sellPriorityFeeSol: sellCapabilities.priority ? (getNamedValue("sellPriorityFeeSol") || "") : "",
    sellTipSol: sellCapabilities.tip ? (getNamedValue("sellTipSol") || "") : "",
    sellSlippagePercent: getNamedValue("sellSlippagePercent") || "",
    sellMaxFeeSol,
    sellMaxPriorityFeeSol: isNamedChecked("sellAutoFeeEnabled") ? sellMaxFeeSol : (sellCapabilities.priority ? (getNamedValue("sellPriorityFeeSol") || "") : ""),
    sellMaxTipSol: isNamedChecked("sellAutoFeeEnabled") ? sellMaxFeeSol : (sellCapabilities.tip ? (getNamedValue("sellTipSol") || "") : ""),
    enableJito: getProvider() === "jito-bundle" || Number(getNamedValue("creationTipSol") || 0) > 0,
    jitoTipSol: creationCapabilities.tip ? (getNamedValue("creationTipSol") || "") : "",
    skipPreflight: getNamedValue("skipPreflight") === "true",
    trackSendBlockHeight: isTrackSendBlockHeightEnabled(),
    feeSplitEnabled: meaningfulFeeSplitEnabled,
    feeSplitRecipients: mode === "regular"
      ? (meaningfulFeeSplitEnabled ? collectFeeSplitRecipients() : [])
      : (mode.startsWith("bags-") ? collectFeeSplitRecipients() : []),
    creatorFeeMode: importedCreatorFeeState.mode || "",
    creatorFeeAddress: importedCreatorFeeState.address || "",
    creatorFeeGithubUsername: importedCreatorFeeState.githubUsername || "",
    creatorFeeGithubUserId: importedCreatorFeeState.githubUserId || "",
    postLaunchStrategy: getNamedValue("postLaunchStrategy") || "none",
    snipeBuyAmountSol: getNamedValue("snipeBuyAmountSol") || "",
    sniperEnabled: getNamedValue("sniperEnabled") === "true",
    sniperWallets,
    sniperConfigJson: getNamedValue("sniperConfigJson") || "[]",
    automaticDevSellEnabled,
    automaticDevSellPercent: getNamedValue("automaticDevSellPercent") || "0",
    automaticDevSellTriggerFamily: getAutoSellTriggerFamily(),
    automaticDevSellTriggerMode: getAutoSellTriggerMode(),
    automaticDevSellDelayMs: String(getAutoSellDelayMs()),
    automaticDevSellBlockOffset: String(getAutoSellBlockOffset()),
    automaticDevSellMarketCapEnabled: getAutoSellTriggerFamily() === "market-cap",
    automaticDevSellMarketCapThreshold: getNamedValue("automaticDevSellMarketCapThreshold") || "",
    automaticDevSellMarketCapScanTimeoutSeconds: getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds")
      || getNamedValue("automaticDevSellMarketCapScanTimeoutMinutes")
      || "30",
    automaticDevSellMarketCapTimeoutAction: getNamedValue("automaticDevSellMarketCapTimeoutAction") || "stop",
    vanityPrivateKey: getNamedValue("vanityPrivateKey") || "",
    imageFileName: uploadedImage ? uploadedImage.fileName : "",
    metadataUri: metadataUri.value || "",
    bagsIdentityMode: getBagsIdentityMode(),
    bagsAgentUsername: bagsIdentityState.agentUsername || "",
    bagsAuthToken: bagsIdentityState.authToken || "",
    bagsIdentityVerifiedWallet: bagsIdentityState.verifiedWallet || "",
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
  metadataUploadState.lastAlertedWarning = "";
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
  metadataUploadState.lastAlertedWarning = "";
  if (metadataUri) {
    metadataUri.value = "";
  }
}

function currentMetadataRetryDelayMs() {
  return metadataUploadState.autoRetryFailures >= 2
    ? METADATA_PREUPLOAD_DEBOUNCE_MS * 2
    : METADATA_PREUPLOAD_DEBOUNCE_MS;
}

function surfaceMetadataWarning(warning) {
  const message = String(warning || "").trim();
  if (!message) return;
  imageStatus.textContent = message;
  if (metadataUploadState.lastAlertedWarning === message) return;
  metadataUploadState.lastAlertedWarning = message;
  window.alert(message);
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
        imageStatus.textContent = payload.metadataWarning ? payload.metadataWarning : "Metadata ready.";
      } else {
        metadataUploadState.staleWhileUploading = true;
      }
      surfaceMetadataWarning(payload.metadataWarning);
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
  metaNode.textContent = "";
  updateLockedModeFields();
}

async function refreshWalletStatus(preserveSelection = true, force = false) {
  try {
    const wallet = preserveSelection ? selectedWalletKey() : "";
    const query = new URLSearchParams();
    if (wallet) query.set("wallet", wallet);
    if (force) query.set("refresh", String(Date.now()));
    const url = query.size ? `/api/wallet-status?${query.toString()}` : "/api/wallet-status";
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("wallet-status", url, {
        cache: force ? "no-store" : "default",
      }, requestStates.walletStatus)
      : null;
    if (result && result.aborted) return;
    const response = result
      ? result.response
      : await fetch(url, { cache: force ? "no-store" : "default" });
    const payload = result ? result.payload : await response.json();
    if (result && !result.isLatest) return;
    walletStatusRequestSerial += 1;
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to load wallet status.");
    }
    applyWalletStatusPayload(payload);
    setStoredWalletStatusLastRefreshAtMs(Date.now());
  } catch (error) {
    if (walletBalance && !latestWalletStatus) walletBalance.textContent = "-";
    metaNode.textContent = error.message;
  } finally {
    scheduleWalletStatusRefresh();
  }
}

function applyBootstrapFastPayload(payload) {
  startupWarmState.enabled = payload && payload.startupWarm
    ? payload.startupWarm.enabled !== false
    : true;
  const configuredWalletStatusRefreshIntervalMs = Number(
    payload && payload.uiRefresh && payload.uiRefresh.walletStatusIntervalMs,
  );
  if (Number.isFinite(configuredWalletStatusRefreshIntervalMs) && configuredWalletStatusRefreshIntervalMs > 0) {
    walletStatusRefreshIntervalMs = Math.max(1000, Math.round(configuredWalletStatusRefreshIntervalMs));
  }
  renderPlatformRuntimeIndicators();
  const previousWalletStatus = latestWalletStatus || null;
  const previousWallets = previousWalletStatus && Array.isArray(previousWalletStatus.wallets)
    ? normalizeVisibleWallets(previousWalletStatus.wallets)
    : [];
  const nextWallets = Array.isArray(payload.wallets)
    ? normalizeVisibleWallets(payload.wallets).map((wallet) => {
        const previous = previousWallets.find((entry) => entry && entry.envKey === (wallet && wallet.envKey));
        return {
          ...(previous || {}),
          ...(wallet || {}),
          balanceLamports: wallet && wallet.balanceLamports == null
            ? (previous && previous.balanceLamports != null ? previous.balanceLamports : null)
            : wallet.balanceLamports,
          balanceSol: wallet && wallet.balanceSol == null
            ? (previous && previous.balanceSol != null ? previous.balanceSol : null)
            : wallet.balanceSol,
          usd1Balance: wallet && wallet.usd1Balance == null
            ? (previous && previous.usd1Balance != null ? previous.usd1Balance : null)
            : wallet.usd1Balance,
        };
      })
    : previousWallets;
  const selectedWalletKeyValue = resolveVisibleSelectedWalletKey(
    payload.selectedWalletKey || (previousWalletStatus && previousWalletStatus.selectedWalletKey) || "",
    nextWallets,
  );
  const selectedWalletRecord = nextWallets.find((wallet) => wallet.envKey === selectedWalletKeyValue) || null;
  latestWalletStatus = {
    ...(previousWalletStatus || {}),
    selectedWalletKey: selectedWalletKeyValue,
    wallets: nextWallets,
    wallet: selectedWalletRecord ? selectedWalletRecord.publicKey : null,
    connected: Boolean(selectedWalletRecord && selectedWalletRecord.publicKey),
    balanceLamports: selectedWalletRecord && selectedWalletRecord.balanceLamports != null
      ? selectedWalletRecord.balanceLamports
      : null,
    balanceSol: selectedWalletRecord && selectedWalletRecord.balanceSol != null
      ? selectedWalletRecord.balanceSol
      : null,
    usd1Balance: selectedWalletRecord && selectedWalletRecord.usd1Balance != null
      ? selectedWalletRecord.usd1Balance
      : null,
    config: payload.config || (previousWalletStatus && previousWalletStatus.config) || null,
    regionRouting: payload.regionRouting || (previousWalletStatus && previousWalletStatus.regionRouting) || null,
    providers: payload.providers || (previousWalletStatus && previousWalletStatus.providers) || {},
    launchpads: payload.launchpads || (previousWalletStatus && previousWalletStatus.launchpads) || {},
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

function applyRuntimeStatusPayload(payload, { hydrateOnly = false } = {}) {
  latestRuntimeStatus = payload;
  markBootstrapState({ runtimeLoaded: true });
  renderBackendRegionSummary();
  syncFollowStatusChrome();
  syncWalletStatusRefreshLoop();
  if (!hydrateOnly) {
    refreshFollowJobs({ silent: true }).catch(() => {});
  }
}

function currentWarmActivityPayload() {
  const preset = getActivePreset(getConfig()) || {};
  const creationSettings = preset && preset.creationSettings && typeof preset.creationSettings === "object"
    ? preset.creationSettings
    : {};
  const buySettings = preset && preset.buySettings && typeof preset.buySettings === "object"
    ? preset.buySettings
    : {};
  const sellSettings = preset && preset.sellSettings && typeof preset.sellSettings === "object"
    ? preset.sellSettings
    : {};
  return {
    creationProvider: providerSelect ? providerSelect.value : "",
    creationEndpointProfile: String(creationSettings.endpointProfile || "").trim(),
    creationMevMode: normalizeMevMode(creationMevModeSelect ? creationMevModeSelect.value : "off", "off"),
    buyProvider: buyProviderSelect ? buyProviderSelect.value : "",
    buyEndpointProfile: String(buySettings.endpointProfile || "").trim(),
    buyMevMode: normalizeMevMode(buyMevModeSelect ? buyMevModeSelect.value : "off", "off"),
    sellProvider: sellProviderSelect ? sellProviderSelect.value : "",
    sellEndpointProfile: String(sellSettings.endpointProfile || "").trim(),
    sellMevMode: normalizeMevMode(sellMevModeSelect ? sellMevModeSelect.value : "off", "off"),
  };
}

async function flushWarmActivity() {
  if (warmActivityState.debounceTimer) {
    window.clearTimeout(warmActivityState.debounceTimer);
    warmActivityState.debounceTimer = null;
  }
  if (warmActivityState.inFlightPromise) {
    warmActivityState.pendingFlush = true;
    return warmActivityState.inFlightPromise;
  }
  warmActivityState.lastSentAtMs = Date.now();
  warmActivityState.inFlightPromise = fetch("/api/warm/activity", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(currentWarmActivityPayload()),
  })
    .then((response) => response.json().catch(() => ({})).then((payload) => ({ response, payload })))
    .then(({ response, payload }) => {
      if (!response.ok || !payload.ok || !payload.warm) return;
      latestRuntimeStatus = {
        ...(latestRuntimeStatus || {}),
        warm: payload.warm,
        ...(payload.rpcTraffic && typeof payload.rpcTraffic === "object"
          ? { rpcTraffic: payload.rpcTraffic }
          : {}),
      };
      renderBackendRegionSummary();
      syncFollowStatusChrome();
      syncWalletStatusRefreshLoop({ immediateResume: true });
    })
    .catch(() => {})
    .finally(() => {
      warmActivityState.inFlightPromise = null;
      if (warmActivityState.pendingFlush) {
        warmActivityState.pendingFlush = false;
        flushWarmActivity().catch(() => {});
      }
    });
  return warmActivityState.inFlightPromise;
}

function queueWarmActivity({ immediate = false } = {}) {
  const now = Date.now();
  if (immediate || now - warmActivityState.lastSentAtMs >= WARM_ACTIVITY_DEBOUNCE_MS) {
    flushWarmActivity().catch(() => {});
    return;
  }
  if (warmActivityState.debounceTimer) window.clearTimeout(warmActivityState.debounceTimer);
  warmActivityState.debounceTimer = window.setTimeout(() => {
    flushWarmActivity().catch(() => {});
  }, WARM_ACTIVITY_DEBOUNCE_MS);
}

function startRuntimeStatusRefreshLoop() {
  if (runtimeStatusRefreshTimer) window.clearInterval(runtimeStatusRefreshTimer);
  runtimeStatusRefreshTimer = window.setInterval(() => {
    refreshRuntimeStatus().catch(() => {});
    if (reportsTerminalSection && !reportsTerminalSection.hidden && normalizeReportsTerminalView(reportsTerminalState.view) === "active-logs") {
      refreshActiveLogs({ showLoading: false }).catch(() => {});
    }
  }, RUNTIME_STATUS_REFRESH_INTERVAL_MS);
}

function scheduleWalletStatusRefresh() {
  clearWalletStatusRefreshTimer();
  if (walletRefreshPausedByIdleSuspend()) return;
  const delayMs = walletStatusRefreshIntervalMs;
  walletStatusRefreshTimer = window.setTimeout(() => {
    walletStatusRefreshTimer = null;
    refreshWalletStatus(true, true).catch(() => {});
  }, delayMs);
}

function runtimeFollowDaemonStatus() {
  return latestRuntimeStatus && latestRuntimeStatus.followDaemon && typeof latestRuntimeStatus.followDaemon === "object"
    ? latestRuntimeStatus.followDaemon
    : null;
}

function clearFollowJobsRefreshTimer() {
  if (!followJobsState.refreshTimer) return;
  window.clearTimeout(followJobsState.refreshTimer);
  followJobsState.refreshTimer = null;
}

function countFollowJobStates(jobs = []) {
  return jobs.reduce((accumulator, job) => {
    const state = String(job && job.state || "").trim().toLowerCase();
    if (state === "reserved") accumulator.reserved += 1;
    else if (state === "armed") accumulator.armed += 1;
    else if (state === "running") accumulator.running += 1;
    if (!isTerminalFollowJobState(state)) {
      accumulator.active += 1;
      if (job && job.lastError) accumulator.issues += 1;
    }
    return accumulator;
  }, {
    active: 0,
    reserved: 0,
    armed: 0,
    running: 0,
    issues: 0,
  });
}

function followStatusSnapshot() {
  const runtimeFollow = runtimeFollowDaemonStatus();
  const configured = Boolean(runtimeFollow && runtimeFollow.configured);
  const reachable = Boolean(runtimeFollow && runtimeFollow.reachable);
  const health = followJobsState.health || (runtimeFollow && runtimeFollow.health) || null;
  const counts = followJobsState.loaded
    ? countFollowJobStates(followJobsState.jobs)
    : {
        active: Number(health && health.activeJobs || 0),
        reserved: 0,
        armed: 0,
        running: 0,
        issues: 0,
      };
  return {
    configured,
    reachable,
    health,
    counts,
    canCancelAll: followJobsState.loaded && counts.active > 0,
    offline: configured && !reachable,
  };
}

function buildFollowJobsSummaryText(snapshot = followStatusSnapshot()) {
  if (!snapshot.configured) return "Follow disabled";
  if (snapshot.offline) return "Follow offline";
  if (snapshot.counts.active > 0) {
    const parts = [`${snapshot.counts.active} active`];
    if (snapshot.counts.reserved > 0) parts.push(`${snapshot.counts.reserved} reserved`);
    if (snapshot.counts.armed > 0) parts.push(`${snapshot.counts.armed} armed`);
    if (snapshot.counts.running > 0) parts.push(`${snapshot.counts.running} running`);
    if (snapshot.counts.issues > 0) parts.push(`${snapshot.counts.issues} issue${snapshot.counts.issues === 1 ? "" : "s"}`);
    return parts.join(" | ");
  }
  return "Follow idle";
}

function syncFollowStatusChrome() {
  const snapshot = followStatusSnapshot();
  if (toggleReportsButton) {
    toggleReportsButton.title = snapshot.offline
      ? "Dashboard (follow daemon offline)"
      : snapshot.counts.active > 0
        ? `Dashboard (${snapshot.counts.active} active follow launch${snapshot.counts.active === 1 ? "" : "es"})`
        : "Dashboard";
  }
  renderPlatformRuntimeIndicators();
}

function scheduleFollowJobsRefresh() {
  clearFollowJobsRefreshTimer();
  const snapshot = followStatusSnapshot();
  const shouldRefresh = snapshot.offline || snapshot.counts.active > 0 || Boolean(reportsTerminalSection && !reportsTerminalSection.hidden);
  if (!shouldRefresh) return;
  const delayMs = snapshot.offline ? FOLLOW_JOBS_OFFLINE_RETRY_MS : FOLLOW_JOBS_REFRESH_INTERVAL_MS;
  followJobsState.refreshTimer = window.setTimeout(() => {
    refreshFollowJobs({ silent: true }).catch(() => {});
  }, delayMs);
}

async function refreshFollowJobs({ silent = false } = {}) {
  clearFollowJobsRefreshTimer();
  const runtimeFollow = runtimeFollowDaemonStatus();
  if (runtimeFollow && runtimeFollow.configured === false) {
    followJobsState = {
      ...followJobsState,
      configured: false,
      reachable: false,
      jobs: [],
      health: null,
      error: "",
      loaded: true,
    };
    syncFollowStatusChrome();
    return;
  }
  try {
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("follow-jobs", "/api/follow/jobs", {}, requestStates.followJobs)
      : null;
    if (result && result.aborted) return;
    const response = result ? result.response : await fetch("/api/follow/jobs");
    const payload = result ? result.payload : await response.json();
    if (result && !result.isLatest) return;
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to load follow launch status.");
    }
    followJobsState = {
      ...followJobsState,
      configured: true,
      reachable: true,
      jobs: Array.isArray(payload.jobs) ? payload.jobs : [],
      health: payload.health && typeof payload.health === "object" ? payload.health : null,
      error: "",
      loaded: true,
    };
  } catch (error) {
    followJobsState = {
      ...followJobsState,
      configured: Boolean(runtimeFollow && runtimeFollow.configured),
      reachable: false,
      jobs: [],
      health: runtimeFollow && runtimeFollow.health && typeof runtimeFollow.health === "object" ? runtimeFollow.health : null,
      error: error && error.message ? error.message : "Failed to load follow launch status.",
      loaded: true,
    };
    if (!silent && reportsTerminalOutput && ["launches", "active-jobs"].includes(normalizeReportsTerminalView(reportsTerminalState.view))) {
      reportsTerminalState.activeText = followJobsState.error;
    }
  }
  syncFollowStatusChrome();
  if (["launches", "active-jobs"].includes(normalizeReportsTerminalView(reportsTerminalState.view)) && reportsTerminalOutput) {
    renderReportsTerminalOutput();
  }
  scheduleFollowJobsRefresh();
}

async function cancelFollowJob(traceId, { note = "" } = {}) {
  const response = await fetch("/api/follow/cancel", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      traceId,
      note,
    }),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to cancel follow launch.");
  }
  followJobsState = {
    ...followJobsState,
    jobs: Array.isArray(payload.jobs) ? payload.jobs : followJobsState.jobs,
    health: payload.health && typeof payload.health === "object" ? payload.health : followJobsState.health,
    reachable: true,
    configured: true,
    loaded: true,
    error: "",
  };
  syncFollowStatusChrome();
  scheduleFollowJobsRefresh();
}

async function cancelAllFollowJobs() {
  const response = await fetch("/api/follow/stop-all", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      note: "Cancelled from History panel",
    }),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to cancel active follow launches.");
  }
  followJobsState = {
    ...followJobsState,
    jobs: Array.isArray(payload.jobs) ? payload.jobs : [],
    health: payload.health && typeof payload.health === "object" ? payload.health : followJobsState.health,
    reachable: true,
    configured: true,
    loaded: true,
    error: "",
  };
  syncFollowStatusChrome();
  scheduleFollowJobsRefresh();
}

function activeFollowJobForTraceId(traceId) {
  const normalized = String(traceId || "").trim();
  if (!normalized) return null;
  const job = followJobsState.jobs.find((entry) => String(entry && entry.traceId || "").trim() === normalized);
  if (!job || isTerminalFollowJobState(job.state)) return null;
  return job;
}

function applyWalletStatusPayload(payload) {
  const normalizedWallets = normalizeVisibleWallets(payload.wallets || []);
  const selectedWalletKeyValue = resolveVisibleSelectedWalletKey(
    payload.selectedWalletKey || (latestWalletStatus && latestWalletStatus.selectedWalletKey) || "",
    normalizedWallets,
  );
  const selectedWalletRecord = normalizedWallets.find((wallet) => wallet.envKey === selectedWalletKeyValue) || null;
  latestWalletStatus = {
    ...(latestWalletStatus || {}),
    ...payload,
    selectedWalletKey: selectedWalletKeyValue,
    wallets: normalizedWallets,
    wallet: selectedWalletRecord ? selectedWalletRecord.publicKey : null,
    connected: Boolean(selectedWalletRecord && selectedWalletRecord.publicKey),
    balanceLamports: selectedWalletRecord && selectedWalletRecord.balanceLamports != null
      ? selectedWalletRecord.balanceLamports
      : null,
    balanceSol: selectedWalletRecord && selectedWalletRecord.balanceSol != null
      ? selectedWalletRecord.balanceSol
      : null,
    usd1Balance: selectedWalletRecord && selectedWalletRecord.usd1Balance != null
      ? selectedWalletRecord.usd1Balance
      : null,
    config: payload.config || (latestWalletStatus && latestWalletStatus.config) || null,
    regionRouting: payload.regionRouting || (latestWalletStatus && latestWalletStatus.regionRouting) || null,
    providers: payload.providers || (latestWalletStatus && latestWalletStatus.providers) || {},
    launchpads: payload.launchpads || (latestWalletStatus && latestWalletStatus.launchpads) || {},
  };
  const wallets = latestWalletStatus.wallets || [];
  renderWalletOptions(wallets, latestWalletStatus.selectedWalletKey || "", latestWalletStatus.balanceSol);
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
  metaNode.textContent = "";
  updateLockedModeFields();
  schedulePopoutAutosize();
}

async function bootstrapApp() {
  markBootstrapState({ started: true });
  setSettingsLoadingState(true);
  setBootOverlayMessage(null, "Connecting to engine…");
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
  setBootOverlayMessage(null, "Syncing wallets, runtime status, and warm caches…");
  const startupWarmPromise = beginStartupWarmup().catch(() => {});
  const runtimeHydrationPromise = refreshRuntimeStatus().catch(() => {});
  const walletStatusPromise = refreshWalletStatus(true, true).catch(() => {});
  refreshBagsIdentityStatus().catch(() => {});
  await Promise.allSettled([
    startupWarmPromise,
    runtimeHydrationPromise,
    walletStatusPromise,
  ]);
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
  const shape = getDevBuyQuoteRequestShape();
  const buyAmount = shape.amount;
  if (!buyAmount) {
    if (quoteOutput) {
      quoteOutput.hidden = true;
      quoteOutput.textContent = "No dev buy selected.";
    }
    if (!syncingDevBuyInputs) {
      syncingDevBuyInputs = true;
      if (devBuyPercentInput) devBuyPercentInput.value = "";
      if (lastDevBuyEditSource !== "percent" && devBuySolInput) devBuySolInput.value = "";
      syncingDevBuyInputs = false;
    }
    return;
  }

  try {
    const mode = shape.mode;
    const cachedQuote = getCachedDevBuyQuote(shape);
    if (cachedQuote) {
      applyDevBuyQuotePayload(cachedQuote, mode);
      renderDevBuyQuoteMessage(cachedQuote, mode);
      return;
    }
    const url = `/api/quote?launchpad=${encodeURIComponent(shape.launchpad)}&quoteAsset=${encodeURIComponent(shape.quoteAsset)}&launchMode=${encodeURIComponent(shape.launchMode)}&mode=${encodeURIComponent(mode)}&amount=${encodeURIComponent(buyAmount)}`;
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
      if (quoteOutput) {
        quoteOutput.hidden = false;
        quoteOutput.textContent = "Enter a valid dev buy amount.";
      }
      return;
    }
    setCachedDevBuyQuote(shape, payload.quote);
    applyDevBuyQuotePayload(payload.quote, mode);
    renderDevBuyQuoteMessage(payload.quote, mode);
  } catch (error) {
    if (quoteOutput) {
      quoteOutput.hidden = false;
      quoteOutput.textContent = error.message;
    }
  }
}

function queueQuoteUpdate() {
  if (RequestUtils.scheduleDebounced) {
    RequestUtils.scheduleDebounced(requestStates.quote, DEV_BUY_QUOTE_DEBOUNCE_MS, () => {
      updateQuote().catch((error) => {
        if (quoteOutput) quoteOutput.textContent = error.message;
      });
    });
    return;
  }
  clearTimeout(quoteTimer);
  quoteTimer = setTimeout(updateQuote, DEV_BUY_QUOTE_DEBOUNCE_MS);
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

  if (!hasAttachedImage()) {
    try {
      const reusedExisting = await ensureTestImageSelected();
      if (reusedExisting) {
        return;
      }
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
    const provider = getProvider();
    if (isNamedChecked("creationAutoFeeEnabled") || !providerMinimumTipSol(provider)) {
      return validateNonNegativeSolField(v);
    }
    return validateRequiredTipField(v, provider);
  },
  creationPriorityFeeSol(v) {
    const provider = getProvider();
    if (isNamedChecked("creationAutoFeeEnabled") || !providerRequiresPriorityFee(provider)) {
      return validateNonNegativeSolField(v);
    }
    return validateRequiredPriorityFeeField(v, provider);
  },
  creationMaxFeeSol(v) {
    return isNamedChecked("creationAutoFeeEnabled")
      ? validateOptionalAutoFeeCapField(v, getProvider())
      : validateNonNegativeSolField(v);
  },
  buyPriorityFeeSol(v) {
    const provider = getBuyProvider();
    if (isNamedChecked("buyAutoFeeEnabled") || !providerRequiresPriorityFee(provider)) {
      return validateNonNegativeSolField(v);
    }
    return validateRequiredPriorityFeeField(v, provider);
  },
  buyTipSol(v) {
    const provider = getBuyProvider();
    if (isNamedChecked("buyAutoFeeEnabled") || !providerMinimumTipSol(provider)) {
      return validateNonNegativeSolField(v);
    }
    return validateRequiredTipField(v, provider);
  },
  buySlippagePercent(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  buyMaxFeeSol(v) {
    return isNamedChecked("buyAutoFeeEnabled")
      ? validateOptionalAutoFeeCapField(v, getBuyProvider())
      : validateNonNegativeSolField(v);
  },
  sellPriorityFeeSol(v) {
    const provider = getSellProvider();
    if (isNamedChecked("sellAutoFeeEnabled") || !providerRequiresPriorityFee(provider)) {
      return validateNonNegativeSolField(v);
    }
    return validateRequiredPriorityFeeField(v, provider);
  },
  sellTipSol(v) {
    const provider = getSellProvider();
    if (isNamedChecked("sellAutoFeeEnabled") || !providerMinimumTipSol(provider)) {
      return validateNonNegativeSolField(v);
    }
    return validateRequiredTipField(v, provider);
  },
  sellSlippagePercent(v) {
    if (!v) return "";
    const n = Number(v);
    if (isNaN(n) || n < 0) return "Must be a valid number";
    return "";
  },
  sellMaxFeeSol(v) {
    return isNamedChecked("sellAutoFeeEnabled")
      ? validateOptionalAutoFeeCapField(v, getSellProvider())
      : validateNonNegativeSolField(v);
  },
  automaticDevSellPercent(v) {
    if (!isNamedChecked("automaticDevSellEnabled")) return "";
    const n = Number(v);
    if (isNaN(n) || n <= 0 || n > 100) return "Must be between 1 and 100";
    return "";
  },
  automaticDevSellDelayMs(v) {
    if (!isNamedChecked("automaticDevSellEnabled")
      || getAutoSellTriggerFamily() !== "time"
      || getAutoSellTriggerMode() !== "submit-delay") return "";
    const n = Number(v);
    if (isNaN(n) || n < 0 || n > 1500) return "Must be between 0 and 1500";
    return "";
  },
  automaticDevSellBlockOffset(v) {
    if (!isNamedChecked("automaticDevSellEnabled")
      || getAutoSellTriggerFamily() !== "time"
      || getAutoSellTriggerMode() !== "block-offset") return "";
    const n = Number(v);
    if (isNaN(n) || n < 0 || n > 23) return "Must be between 0 and 23";
    return "";
  },
  automaticDevSellMarketCapThreshold(v) {
    if (!isNamedChecked("automaticDevSellEnabled") || getAutoSellTriggerFamily() !== "market-cap") return "";
    if (!String(v || "").trim()) return "USD market cap is required";
    const normalized = parseAutoSellMarketCapThreshold(v);
    if (!Number.isFinite(normalized) || normalized <= 0) return "Use a positive USD amount like 100000 or 100k";
    return "";
  },
  automaticDevSellMarketCapScanTimeoutSeconds(v) {
    if (!isNamedChecked("automaticDevSellEnabled") || getAutoSellTriggerFamily() !== "market-cap") return "";
    const n = Number(v);
    if (isNaN(n) || n < 1 || n > 86400) return "Must be between 1 and 86400";
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

function focusFirstInvalidInlineField() {
  const input = form.querySelector(".input-error");
  if (!input || typeof input.focus !== "function") return;
  input.focus();
  if (typeof input.select === "function" && input.tagName === "INPUT" && input.type !== "checkbox") {
    input.select();
  }
}

function validateSettingsModalBeforeSave() {
  const errors = validateAllInlineFields();
  if (!errors.length) return [];
  focusFirstInvalidInlineField();
  return errors;
}

function validateProviderFeeFields(scope) {
  const names = scope === "creation"
    ? ["creationPriorityFeeSol", "creationTipSol", "creationMaxFeeSol"]
    : scope === "buy"
      ? ["buyPriorityFeeSol", "buyTipSol", "buyMaxFeeSol"]
      : scope === "sell"
        ? ["sellPriorityFeeSol", "sellTipSol", "sellMaxFeeSol"]
        : [];
  names.forEach((name) => validateFieldByName(name));
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
    if (entry.type === "github" && looksLikeSolanaAddress(entry.githubUsername || entry.githubUserId)) {
      errors.push(`Agent split recipient ${index + 1} cannot use a Solana address while GitHub is selected.`);
    }
    if (entry.type === "github" && !entry.githubUsername && !entry.githubUserId) {
      errors.push(`Agent split recipient ${index + 1} is missing a GitHub username or user id.`);
    }
  });

  return errors;
}

function validateFeeSplit() {
  const errors = [];
  const mode = getMode();
  const isBagsMode = mode.startsWith("bags-");
  if (mode !== "regular" && !isBagsMode) return errors;
  if (!isBagsMode && !feeSplitEnabled.checked) return errors;
  const recipients = collectFeeSplitRecipients();
  if (recipients.length > MAX_FEE_SPLIT_RECIPIENTS) {
    errors.push(`Fee split supports at most ${MAX_FEE_SPLIT_RECIPIENTS} recipients.`);
  }
  recipients.forEach((entry, index) => {
    if (!Number.isFinite(entry.shareBps) || entry.shareBps <= 0) {
      errors.push(`Fee split recipient ${index + 1} has an invalid %.`);
      return;
    }
    if (entry.type === "wallet" && !entry.address) {
      errors.push(`Fee split recipient ${index + 1} is missing a wallet address.`);
      return;
    }
    if (entry.type === "github" && !entry.githubUsername && !entry.githubUserId) {
      errors.push(`Fee split recipient ${index + 1} is missing a GitHub username or user id.`);
      return;
    }
    if (entry.type === "github" && looksLikeSolanaAddress(entry.githubUsername || entry.githubUserId)) {
      errors.push(`Fee split recipient ${index + 1} cannot use a Solana address while GitHub is selected.`);
    }
  });
  const total = recipients.reduce((sum, entry) => sum + (Number(entry.shareBps) || 0), 0);
  if (recipients.length > 0 && total !== 10_000) {
    errors.push("Fee split must total 100%.");
  }
  return errors;
}

function validateForm() {
  const errors = [];
  const f = readForm();
  if (!f.name.trim()) errors.push("Token name is required.");
  if (!f.symbol.trim()) errors.push("Ticker is required.");
  if (!hasAttachedImage()) errors.push("Token image is required.");
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
    ? `${f.devBuyAmount} ${f.devBuyMode === "tokens" ? "tokens" : getDevBuyAssetLabel(f.launchpad, f.quoteAsset)}`
    : "None";

  const quoteText = quoteOutput ? (quoteOutput.textContent || "") : "";

  const modeLabels = {
    regular: "Regular",
    bonkers: "Bonkers",
    cashback: "Cashback",
    "agent-custom": "Agent Custom",
    "agent-unlocked": "Agent Unlocked",
    "agent-locked": "Agent Locked",
    "bags-2-2": "2% / 2%",
    "bags-025-1": "0.25% / 1%",
    "bags-1-025": "1% / 0.25%",
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

  const creationAutoFeeCap = normalizeAutoFeeCapValue(getNamedValue("creationMaxFeeSol"));
  let txSettingsText = isNamedChecked("creationAutoFeeEnabled")
    ? `Auto fee${creationAutoFeeCap ? ` <= ${creationAutoFeeCap} SOL` : ""}`
    : (usesBundledJito()
      ? "Priority: custom"
      : `Priority: ${f.priorityFeeSol || "off"}`);
  if (!isNamedChecked("creationAutoFeeEnabled") && f.jitoTipSol) txSettingsText += ` | Tip: ${f.jitoTipSol || "default"} SOL`;
  if (f.skipPreflight) txSettingsText += " | Preflight off";

  const buybackText = f.mode === "agent-locked" ? "100%"
    : (f.mode === "agent-custom" || f.mode === "agent-unlocked") ? `${f.buybackPercent || "50"}%`
    : "-";
  const sniperText = f.sniperEnabled
    ? (f.sniperWallets.length
      ? f.sniperWallets.map((entry) => `#${walletIndexFromEnvKey(entry.envKey)} ${entry.amountSol} ${getQuoteAssetLabel(f.quoteAsset)} @ ${entry.targetBlockOffset != null ? `b${entry.targetBlockOffset}` : (entry.submitWithLaunch ? "same-time" : getSniperTriggerSummary(entry).toLowerCase())}`).join(" | ")
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
    ...(f.launchpad === "bagsapp" ? [{ label: "Identity", value: describeBagsIdentity(), cls: "secondary" }] : []),
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

function buildOutputMetaTextFromReport(report) {
  const normalizedReport = report && typeof report === "object" ? report : {};
  const execution = normalizedReport.execution && typeof normalizedReport.execution === "object"
    ? normalizedReport.execution
    : {};
  const followMeta = normalizedReport.followDaemon && normalizedReport.followDaemon.enabled
    ? ` | Follow: ${normalizedReport.followDaemon.job && normalizedReport.followDaemon.job.state ? normalizedReport.followDaemon.job.state : "armed"}`
    : "";
  return `${normalizedReport.launchpad || "pump"} | ${execution.resolvedProvider || execution.provider || "helius-sender"} | Mint: ${shortAddress(normalizedReport.mint)}${followMeta}`;
}

function isTerminalFollowJobState(state) {
  const normalized = String(state || "").trim().toLowerCase();
  return ["completed", "completed-with-failures", "cancelled", "failed"].includes(normalized);
}

function stopOutputFollowRefresh() {
  outputFollowRefreshState.serial += 1;
  if (outputFollowRefreshState.timer) {
    window.clearTimeout(outputFollowRefreshState.timer);
  }
  outputFollowRefreshState.timer = null;
  outputFollowRefreshState.reportId = "";
  outputFollowRefreshState.startedAtMs = 0;
}

function updateReportsTerminalSummaryEntry(entry) {
  if (!entry || !entry.id) return;
  const normalizedId = String(entry.id).trim();
  if (!normalizedId) return;
  let didUpdate = false;
  reportsTerminalState.entries = reportsTerminalState.entries.map((item) => {
    if (item && item.id === normalizedId) {
      didUpdate = true;
      return entry;
    }
    return item;
  });
  reportsTerminalState.allEntries = reportsTerminalState.allEntries.map((item) => {
    if (item && item.id === normalizedId) {
      return entry;
    }
    return item;
  });
  reportsTerminalState.launches = reportsTerminalState.launches.map((launch) => {
    if (launch && launch.id === normalizedId) {
      return { ...launch, entry };
    }
    return launch;
  });
  if (!didUpdate) return;
  renderReportsTerminalList();
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "launches") {
    renderReportsTerminalOutput();
  }
}

function applyLiveReportSnapshotToOutput(payload) {
  const entryId = payload && payload.entry && payload.entry.id ? payload.entry.id : "";
  const rawBundle = payload && payload.payload && typeof payload.payload === "object" ? payload.payload : null;
  const bundle = applyFrozenBenchmarkSnapshot(entryId, rawBundle);
  const report = bundle && bundle.report && typeof bundle.report === "object" ? bundle.report : null;
  if (payload && payload.entry && typeof payload.entry === "object") {
    updateReportsTerminalSummaryEntry(payload.entry);
  }
  // Main output stays the last Build/Simulate/Deploy result; follow refreshes only update the reports panel.
  if (entryId && outputFollowRefreshState.reportId === entryId) {
    reportsTerminalState.activeId = entryId;
  }
  if (entryId && reportsTerminalState.activeId === entryId && normalizeReportsTerminalView(reportsTerminalState.view) === "transactions") {
    reportsTerminalState.activePayload = bundle;
    reportsTerminalState.activeText = payload.text || reportsTerminalState.activeText;
    renderReportsTerminalOutput();
  }
}

async function pollOutputFollowReport(reportId, refreshSerial) {
  if (!reportId || refreshSerial !== outputFollowRefreshState.serial) return;
  try {
    const response = await fetch(`/api/reports/view?id=${encodeURIComponent(reportId)}`);
    const payload = await response.json();
    if (refreshSerial !== outputFollowRefreshState.serial) return;
    if (response.ok && payload.ok) {
      applyLiveReportSnapshotToOutput(payload);
      const report = payload.payload && payload.payload.report && typeof payload.payload.report === "object"
        ? payload.payload.report
        : null;
      const followState = report && report.followDaemon && report.followDaemon.job
        ? report.followDaemon.job.state
        : "";
      const elapsedMs = Date.now() - outputFollowRefreshState.startedAtMs;
      if (isTerminalFollowJobState(followState) || elapsedMs >= OUTPUT_FOLLOW_REFRESH_TIMEOUT_MS) {
        stopOutputFollowRefresh();
        return;
      }
    }
  } catch (_error) {
    if (refreshSerial !== outputFollowRefreshState.serial) return;
    const elapsedMs = Date.now() - outputFollowRefreshState.startedAtMs;
    if (elapsedMs >= OUTPUT_FOLLOW_REFRESH_TIMEOUT_MS) {
      stopOutputFollowRefresh();
      return;
    }
  }
  if (refreshSerial !== outputFollowRefreshState.serial) return;
  outputFollowRefreshState.timer = window.setTimeout(() => {
    pollOutputFollowReport(reportId, refreshSerial);
  }, OUTPUT_FOLLOW_REFRESH_INTERVAL_MS);
}

function startOutputFollowRefresh(reportId) {
  stopOutputFollowRefresh();
  if (!reportId) return;
  outputFollowRefreshState.reportId = reportId;
  outputFollowRefreshState.startedAtMs = Date.now();
  const refreshSerial = outputFollowRefreshState.serial;
  outputFollowRefreshState.timer = window.setTimeout(() => {
    pollOutputFollowReport(reportId, refreshSerial);
  }, OUTPUT_FOLLOW_REFRESH_INTERVAL_MS);
}

async function run(action) {
  if (!ensureInteractiveBootstrapReady()) return;
  const actualAction = action === "deploy" ? "send" : action;
  const label = action === "deploy" ? "Deploying..." : action === "simulate" ? "Simulating..." : "Building...";
  setBusy(true, label);
  output.textContent = "Working...";
  stopOutputFollowRefresh();

  try {
    await new Promise((resolve) => requestAnimationFrame(() => resolve()));
    await ensureStartupWarmReady();
    const clientActionStartedAt = performance.now();
    await ensureMetadataReadyForAction(actualAction);
    const requestPayloadStartedAt = performance.now();
    const formPayload = readForm();
    const prepareRequestPayloadMs = Math.max(0, Math.round(performance.now() - requestPayloadStartedAt));
    const clientPreRequestMs = Math.max(0, Math.round(performance.now() - clientActionStartedAt));
    const response = await fetch("/api/run", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        action: actualAction,
        form: formPayload,
        clientPreRequestMs,
        prepareRequestPayloadMs,
      }),
    });
    const payload = await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Request failed.");
    }

    setStatusLabel(action === "deploy" ? "Deployed" : action === "simulate" ? "Simulated" : "Built");
    metaNode.textContent = buildOutputMetaTextFromReport(payload.report);
    metadataUri.value = payload.metadataUri || "";
    if (payload.metadataUri) {
      metadataUploadState.completedFingerprint = metadataFingerprintFromForm(readForm());
    }
    surfaceMetadataWarning(payload.metadataWarning);
    output.textContent = payload.text;
    setBusy(false, currentStatusLabel());
    if (payload.sendLogPath) {
      const reportId = extractReportIdFromPath(payload.sendLogPath);
      if (reportId && normalizeReportsTerminalView(reportsTerminalState.view) === "transactions") {
        reportsTerminalState.activeId = reportId;
        reportsTerminalState.activePayload = null;
        reportsTerminalState.activeText = "Loading latest report...";
        renderReportsTerminalOutput();
        renderReportsTerminalList();
      }
      refreshReportsTerminal({
        preserveSelection: false,
        preferId: reportId,
      }).catch((error) => {
        if (reportsTerminalOutput && reportsTerminalSection && !reportsTerminalSection.hidden) {
          reportsTerminalState.activePayload = null;
          reportsTerminalState.activeText = error.message || "Failed to refresh reports.";
          renderReportsTerminalOutput();
        }
      });
      if (actualAction === "send" && payload.report && payload.report.followDaemon && payload.report.followDaemon.enabled) {
        refreshFollowJobs({ silent: true }).catch(() => {});
        startOutputFollowRefresh(reportId);
      }
    }
    if (actualAction === "send") {
      if (formPayload && String(formPayload.vanityPrivateKey || "").trim()) {
        if (vanityPrivateKeyText) vanityPrivateKeyText.value = "";
        if (vanityModalError) vanityModalError.textContent = "";
        applyVanityValue("", { publicKey: "" });
      }
      refreshWalletStatus(true, true).catch(() => {});
    }
  } catch (error) {
    setStatusLabel("Error");
    output.textContent = error.message;
  } finally {
    if (buttons.some((button) => button.disabled)) {
      setBusy(false, currentStatusLabel());
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
    presetEditing: false,
    misc: {
      ...(base.defaults && base.defaults.misc ? base.defaults.misc : {}),
      sniperDraft: normalizeSniperDraftState(sniperFeature.getState()),
      feeSplitDraft: normalizeFeeSplitDraft(serializeFeeSplitDraft()),
      agentSplitDraft: normalizeAgentSplitDraft(serializeAgentSplitDraft()),
      bagsIdentity: {
        mode: getBagsIdentityMode(),
        agentUsername: bagsIdentityState.agentUsername || "",
      },
    },
    automaticDevSell: {
      enabled: isNamedChecked("automaticDevSellEnabled"),
      percent: Number(f.automaticDevSellPercent || 100),
      triggerFamily: normalizeAutoSellTriggerFamily(f.automaticDevSellTriggerFamily),
      triggerMode: normalizeAutoSellTriggerMode(f.automaticDevSellTriggerMode),
      delayMs: Number(f.automaticDevSellDelayMs || 0),
      targetBlockOffset: Number(f.automaticDevSellBlockOffset || 0),
      marketCapEnabled: normalizeAutoSellTriggerFamily(f.automaticDevSellTriggerFamily) === "market-cap",
      marketCapThreshold: f.automaticDevSellMarketCapThreshold || "",
      marketCapScanTimeoutSeconds: Number(
        f.automaticDevSellMarketCapScanTimeoutSeconds
          || ((Number(f.automaticDevSellMarketCapScanTimeoutMinutes || 0) || 0) * 60)
      ) || 30,
      marketCapTimeoutAction: f.automaticDevSellMarketCapTimeoutAction || "stop",
    },
  };

  return base;
}

async function saveSettings() {
  if (!hasBootstrapConfig()) {
    setStatusLabel("Loading");
    metaNode.textContent = "Settings are still loading from the backend.";
    return;
  }
  const inlineErrors = validateSettingsModalBeforeSave();
  if (inlineErrors.length) {
    setStatusLabel("Error");
    output.textContent = inlineErrors[0] || "Please fix the highlighted settings fields.";
    return;
  }
  setBusy(true, "Saving defaults...");
  try {
    syncActivePresetFromInputs();
    const configToSave = buildSavedConfigFromForm();
    const result = RequestUtils.fetchJsonLatest
      ? await RequestUtils.fetchJsonLatest("settings-save", "/api/settings", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          config: configToSave,
        }),
      })
      : null;
    const response = result ? result.response : await fetch("/api/settings", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        config: configToSave,
      }),
    });
    const payload = result ? result.payload : await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(payload.error || "Failed to save settings.");
    }
    const savedConfig = cloneConfig(payload.config || configToSave);
    if (!savedConfig.defaults) savedConfig.defaults = {};
    savedConfig.defaults.presetEditing = false;
    setStatusLabel("Defaults saved");
    setRegionRouting(payload.regionRouting || (latestWalletStatus && latestWalletStatus.regionRouting));
    setConfig(savedConfig);
    metaNode.textContent = "Launch defaults and selected presets saved.";
    renderQuickDevBuyButtons(savedConfig);
    populateDevBuyPresetEditor(savedConfig);
    renderBackendRegionSummary(payload.regionRouting);
    queueWarmActivity({ immediate: true });
    hideSettingsModal("save");
  } catch (error) {
    setStatusLabel("Error");
    output.textContent = error.message;
  } finally {
    setBusy(false, currentStatusLabel());
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
  settingsModalInitialConfig = cloneConfig(getConfig());
  if (settingsModal) settingsModal.hidden = false;
}

function hideSettingsModal(reason = "dismiss") {
  if (!settingsModal) return false;
  if (reason === "cancel") {
    if (settingsModalInitialConfig) {
      const restoredConfig = cloneConfig(settingsModalInitialConfig);
      if (!restoredConfig.defaults) restoredConfig.defaults = {};
      restoredConfig.defaults.presetEditing = false;
      setConfig(restoredConfig);
      applyPresetToSettingsInputs(getActivePreset(restoredConfig), { syncToMainForm: false });
      renderBackendRegionSummary();
    }
  } else if (reason === "save") {
    setPresetEditing(false);
  } else {
    return false;
  }
  settingsModal.hidden = true;
  settingsModalInitialConfig = null;
  return true;
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
  syncReportsTerminalLayoutHeight();
  schedulePopoutAutosize();
  scheduleLiveSyncBroadcast({ immediate: true });
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

function getStoredReportsTerminalView() {
  try {
    return normalizeReportsTerminalView(window.localStorage.getItem(REPORTS_TERMINAL_VIEW_KEY));
  } catch (_error) {
    return "transactions";
  }
}

function setStoredReportsTerminalView(view) {
  try {
    window.localStorage.setItem(REPORTS_TERMINAL_VIEW_KEY, normalizeReportsTerminalView(view));
  } catch (_error) {
    // Ignore storage failures and keep the UI functional.
  }
}

function getStoredActiveLogsView() {
  try {
    return normalizeActiveLogsView(window.localStorage.getItem(REPORTS_ACTIVE_LOGS_VIEW_KEY));
  } catch (_error) {
    return "live";
  }
}

function setStoredActiveLogsView(view) {
  try {
    window.localStorage.setItem(REPORTS_ACTIVE_LOGS_VIEW_KEY, normalizeActiveLogsView(view));
  } catch (_error) {
    // Ignore storage failures and keep the UI functional.
  }
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
  scheduleLiveSyncBroadcast();
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

function normalizeReportsTerminalView(view) {
  const normalized = String(view || "").trim().toLowerCase();
  if (normalized === "launches") return "launches";
  if (normalized === "active-jobs") return "active-jobs";
  if (normalized === "active-logs") return "active-logs";
  return "transactions";
}

function normalizeActiveLogsView(view) {
  return String(view || "").trim().toLowerCase() === "errors" ? "errors" : "live";
}

function reportsTerminalMetaText(view = reportsTerminalState.view) {
  const normalized = normalizeReportsTerminalView(view);
  if (normalized === "launches") return "Latest 25 launches.";
  if (normalized === "active-jobs") {
    const snapshot = followStatusSnapshot();
    if (snapshot.offline) return "Follow daemon offline.";
    if (!snapshot.configured) return "Follow daemon disabled.";
    if (snapshot.counts.active > 0) {
      return `${snapshot.counts.active} live follow job${snapshot.counts.active === 1 ? "" : "s"}.`;
    }
    return "Live follow-daemon jobs.";
  }
  if (normalized === "active-logs") {
    return normalizeActiveLogsView(reportsTerminalState.activeLogsView) === "errors"
      ? "Persisted historic backend errors."
      : "Latest 100 in-memory backend activity logs.";
  }
  return "Latest 25 transactions.";
}

function syncReportsTerminalChrome() {
  const view = normalizeReportsTerminalView(reportsTerminalState.view);
  reportsTerminalState.view = view;
  if (reportsTerminalSection) {
    reportsTerminalSection.classList.toggle("is-launches-view", view === "launches");
    reportsTerminalSection.classList.toggle("is-active-jobs-view", view === "active-jobs");
    reportsTerminalSection.classList.toggle("is-active-logs-view", view === "active-logs");
  }
  if (reportsTransactionsButton) reportsTransactionsButton.classList.toggle("active", view === "transactions");
  if (reportsLaunchesButton) reportsLaunchesButton.classList.toggle("active", view === "launches");
  if (reportsActiveJobsButton) reportsActiveJobsButton.classList.toggle("active", view === "active-jobs");
  if (reportsActiveLogsButton) reportsActiveLogsButton.classList.toggle("active", view === "active-logs");
  if (reportsTerminalMeta) reportsTerminalMeta.textContent = reportsTerminalMetaText(view);
  syncReportsTerminalLayoutHeight();
  syncFollowStatusChrome();
}

function syncReportsTerminalLayoutHeight() {
  if (!reportsTerminalSection || !launchSurfaceCard) return;
  const launchSurfaceHeight = Math.round(launchSurfaceCard.getBoundingClientRect().height || 0);
  const outputVisible = Boolean(outputSection && !outputSection.hidden);
  const outputHeight = outputVisible
    ? Math.round(outputSection.getBoundingClientRect().height || 0)
    : 0;
  const measuredHeight = Math.max(0, launchSurfaceHeight - outputHeight);
  if (measuredHeight <= 0) {
    reportsTerminalSection.style.removeProperty("--reports-terminal-match-height");
    return;
  }
  reportsTerminalSection.style.setProperty("--reports-terminal-match-height", `${measuredHeight}px`);
}

function metadataUriToGatewayUrl(uri) {
  const raw = String(uri || "").trim();
  if (!raw) return "";
  if (/^ipfs:\/\//i.test(raw)) {
    const normalized = raw.replace(/^ipfs:\/\//i, "").replace(/^ipfs\//i, "");
    return `https://ipfs.io/ipfs/${normalized}`;
  }
  return raw;
}

function parseDevBuyDescription(value) {
  const raw = String(value || "").trim();
  if (!raw || raw === "none") return { mode: "sol", amount: "" };
  const [kind, amount] = raw.split(":");
  const normalizedKind = String(kind || "").trim().toLowerCase();
  return {
    mode: normalizedKind === "tokens" ? "tokens" : "sol",
    amount: String(amount || "").trim(),
  };
}

function launchHistoryTitle(metadata, report) {
  const symbol = String(metadata && metadata.symbol || "").trim();
  const name = String(metadata && metadata.name || "").trim();
  if (name) return name;
  if (symbol) return symbol;
  return String(report && report.mint || "Unknown launch");
}

function launchHistorySymbol(metadata, report) {
  const symbol = String(metadata && metadata.symbol || "").trim();
  if (symbol) return symbol;
  const mode = String(report && report.mode || "").trim();
  return mode ? mode.toUpperCase() : "LAUNCH";
}

function launchHistoryImageUrl(metadata) {
  const raw = String(metadata && metadata.image || "").trim();
  return metadataUriToGatewayUrl(raw);
}

async function fetchLaunchMetadataSummary(metadataUriValue) {
  const metadataUriValueNormalized = String(metadataUriValue || "").trim();
  if (!metadataUriValueNormalized) return null;
  if (Object.prototype.hasOwnProperty.call(reportsTerminalState.launchMetadataByUri, metadataUriValueNormalized)) {
    return reportsTerminalState.launchMetadataByUri[metadataUriValueNormalized];
  }
  const url = metadataUriToGatewayUrl(metadataUriValueNormalized);
  if (!url) {
    reportsTerminalState.launchMetadataByUri[metadataUriValueNormalized] = null;
    return null;
  }
  try {
    const response = await fetch(url, { cache: "force-cache" });
    if (!response.ok) throw new Error("metadata fetch failed");
    const payload = await response.json();
    const metadata = payload && typeof payload === "object" ? payload : null;
    reportsTerminalState.launchMetadataByUri[metadataUriValueNormalized] = metadata;
    return metadata;
  } catch (_error) {
    reportsTerminalState.launchMetadataByUri[metadataUriValueNormalized] = null;
    return null;
  }
}

async function fetchReportBundleForLaunch(id) {
  const normalizedId = String(id || "").trim();
  if (!normalizedId) return null;
  if (reportsTerminalState.launchBundles[normalizedId]) return reportsTerminalState.launchBundles[normalizedId];
  const response = await fetch(`/api/reports/view?id=${encodeURIComponent(normalizedId)}`);
  const payload = await response.json();
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to load report.");
  }
  reportsTerminalState.launchBundles[normalizedId] = payload;
  return payload;
}

function getLaunchHistoryEntry(id) {
  const normalizedId = String(id || "").trim();
  if (!normalizedId) return null;
  return reportsTerminalState.launches.find((entry) => entry.id === normalizedId) || null;
}

function buildLaunchHistoryEntry(entry, bundle, metadata) {
  const payload = bundle && bundle.payload && typeof bundle.payload === "object" ? bundle.payload : {};
  const report = payload.report && typeof payload.report === "object" ? payload.report : {};
  const execution = report.execution && typeof report.execution === "object" ? report.execution : {};
  const followDaemon = report.followDaemon && typeof report.followDaemon === "object" ? report.followDaemon : {};
  const followJob = followDaemon.job && typeof followDaemon.job === "object" ? followDaemon.job : {};
  const savedFollowLaunch = report.savedFollowLaunch && typeof report.savedFollowLaunch === "object" ? report.savedFollowLaunch : null;
  const savedBags = report.savedBags && typeof report.savedBags === "object" ? report.savedBags : null;
  const savedFeeSharingRecipients = Array.isArray(report.savedFeeSharingRecipients) ? report.savedFeeSharingRecipients : [];
  const savedAgentFeeRecipients = Array.isArray(report.savedAgentFeeRecipients) ? report.savedAgentFeeRecipients : [];
  const savedCreatorFee = report.savedCreatorFee && typeof report.savedCreatorFee === "object" ? report.savedCreatorFee : null;
  const followLaunch = savedFollowLaunch || (followJob.followLaunch && typeof followJob.followLaunch === "object" ? followJob.followLaunch : {});
  const devBuy = parseDevBuyDescription(report.devBuyDescription);
  return {
    id: entry.id,
    traceId: String(entry && entry.traceId || followJob.traceId || "").trim(),
    entry,
    payload,
    report,
    execution,
    followJob,
    followLaunch,
    selectedWalletKey: String(report.savedSelectedWalletKey || followJob.selectedWalletKey || "").trim(),
    quoteAsset: String(report.savedQuoteAsset || followJob.quoteAsset || "sol").trim(),
    metadata: metadata || null,
    title: launchHistoryTitle(metadata, report),
    symbol: launchHistorySymbol(metadata, report),
    imageUrl: launchHistoryImageUrl(metadata),
    metadataUri: String(report.metadataUri || "").trim(),
    devBuy,
    bags: savedBags,
    feeSharingRecipients: savedFeeSharingRecipients,
    agentFeeRecipients: savedAgentFeeRecipients,
    creatorFee: savedCreatorFee,
  };
}

async function loadReportsTerminalLaunches() {
  const sourceEntries = reportsTerminalState.allEntries
    .filter((entry) => String(entry && entry.action || "").trim().toLowerCase() === "send")
    .slice(0, REPORTS_TERMINAL_ITEM_LIMIT);
  const launches = await Promise.all(sourceEntries.map(async (entry) => {
    try {
      const bundle = await fetchReportBundleForLaunch(entry.id);
      const payload = bundle && bundle.payload && typeof bundle.payload === "object" ? bundle.payload : {};
      const report = payload.report && typeof payload.report === "object" ? payload.report : {};
      const metadata = await fetchLaunchMetadataSummary(report.metadataUri || "");
      return buildLaunchHistoryEntry(entry, bundle, metadata);
    } catch (_error) {
      return buildLaunchHistoryEntry(entry, null, null);
    }
  }));
  reportsTerminalState.launches = launches;
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

function cloneReportValue(value) {
  if (value == null) return value;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (_error) {
    return value;
  }
}

function hasReportObjectFields(value) {
  return Boolean(value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).length);
}

function captureFrozenBenchmarkSnapshot(reportId, payload) {
  const normalizedId = String(reportId || "").trim();
  if (!normalizedId || !payload || typeof payload !== "object") return;
  const report = payload.report && typeof payload.report === "object" ? payload.report : null;
  if (!report) return;
  const benchmark = report.benchmark && typeof report.benchmark === "object" ? report.benchmark : null;
  const executionTimings = report.execution && report.execution.timings && typeof report.execution.timings === "object"
    ? report.execution.timings
    : null;
  const hasBenchmark = hasReportObjectFields(benchmark);
  const hasExecutionTimings = hasReportObjectFields(executionTimings);
  if (!hasBenchmark && !hasExecutionTimings) return;
  const previous = reportsTerminalState.activeBenchmarkReportId === normalizedId
    ? reportsTerminalState.activeBenchmarkSnapshot
    : null;
  reportsTerminalState.activeBenchmarkReportId = normalizedId;
  reportsTerminalState.activeBenchmarkSnapshot = {
    benchmark: hasBenchmark ? cloneReportValue(benchmark) : cloneReportValue(previous && previous.benchmark),
    executionTimings: hasExecutionTimings
      ? cloneReportValue(executionTimings)
      : cloneReportValue(previous && previous.executionTimings),
  };
}

function applyFrozenBenchmarkSnapshot(reportId, payload) {
  const normalizedId = String(reportId || "").trim();
  if (!normalizedId || !payload || typeof payload !== "object") return payload;
  if (reportsTerminalState.activeBenchmarkReportId !== normalizedId) return payload;
  const snapshot = reportsTerminalState.activeBenchmarkSnapshot;
  if (!snapshot) return payload;
  const report = payload.report && typeof payload.report === "object" ? payload.report : null;
  if (!report) return payload;
  const nextPayload = cloneReportValue(payload);
  const nextReport = nextPayload.report && typeof nextPayload.report === "object" ? nextPayload.report : null;
  if (!nextReport) return payload;
  if (!hasReportObjectFields(nextReport.benchmark) && hasReportObjectFields(snapshot.benchmark)) {
    nextReport.benchmark = cloneReportValue(snapshot.benchmark);
  }
  if (nextReport.execution && typeof nextReport.execution === "object") {
    if (
      !hasReportObjectFields(nextReport.execution.timings)
      && hasReportObjectFields(snapshot.executionTimings)
    ) {
      nextReport.execution.timings = cloneReportValue(snapshot.executionTimings);
    }
  }
  return nextPayload;
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

function formatReportMetric(value, suffix = "", fallback = "--", digits = 0) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return `${numeric.toFixed(digits)}${suffix}`;
}

function parseReportMetricNumber(value) {
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : null;
}

function buildTimingMetricItem(label, value, detail = "", { hideZero = false, tone = "" } = {}) {
  const numeric = parseReportMetricNumber(value);
  if (numeric == null || (hideZero && numeric === 0)) return null;
  return {
    label,
    value: formatReportMetric(numeric, "ms"),
    detail,
    tone,
  };
}

function deriveRemainingTiming(totalValue, childValues = []) {
  const total = parseReportMetricNumber(totalValue);
  if (total == null) return null;
  let hasChild = false;
  const consumed = childValues.reduce((sum, value) => {
    const numeric = parseReportMetricNumber(value);
    if (numeric == null) return sum;
    hasChild = true;
    return sum + numeric;
  }, 0);
  if (!hasChild) return null;
  return Math.max(0, total - consumed);
}

function buildLegacyBenchmarkTimingSections(timings = {}) {
  const compileTotal = parseReportMetricNumber(timings.compileTransactionsMs);
  const compileAltLoad = parseReportMetricNumber(timings.compileAltLoadMs);
  const compileBlockhash = parseReportMetricNumber(timings.compileBlockhashFetchMs);
  const compileGlobal = parseReportMetricNumber(timings.compileGlobalFetchMs);
  const compileFollowUp = parseReportMetricNumber(timings.compileFollowUpPrepMs);
  const compileSerialize = parseReportMetricNumber(timings.compileTxSerializeMs);
  const compileOther = deriveRemainingTiming(compileTotal, [
    compileAltLoad,
    compileBlockhash,
    compileGlobal,
    compileFollowUp,
    compileSerialize,
  ]);

  const sendTotal = parseReportMetricNumber(timings.sendMs);
  const submitTotal = parseReportMetricNumber(timings.sendSubmitMs);
  const confirmTotal = parseReportMetricNumber(timings.sendConfirmMs);
  const bagsSetupSubmit = parseReportMetricNumber(timings.bagsSetupSubmitMs);
  const bagsSetupConfirm = parseReportMetricNumber(timings.bagsSetupConfirmMs);
  const launchSubmit = bagsSetupSubmit != null ? deriveRemainingTiming(submitTotal, [bagsSetupSubmit]) : null;
  const launchConfirm = bagsSetupConfirm != null ? deriveRemainingTiming(confirmTotal, [bagsSetupConfirm]) : null;
  const sendOther = deriveRemainingTiming(sendTotal, [submitTotal, confirmTotal]);

  return {
    topLevel: [
      buildTimingMetricItem("End-to-end", timings.totalElapsedMs, "client + backend"),
      buildTimingMetricItem("Client overhead", timings.clientPreRequestMs, "before engine work starts"),
      buildTimingMetricItem("Backend total", timings.backendTotalElapsedMs, "all engine work"),
      buildTimingMetricItem("Compile total", timings.compileTransactionsMs, "inclusive stage total"),
      buildTimingMetricItem("Send total", timings.sendMs, "inclusive of submit + confirm"),
      buildTimingMetricItem("Persist report", timings.persistReportMs, "final report write"),
    ],
    prep: [
      buildTimingMetricItem("Form -> Raw", timings.formToRawConfigMs, "UI payload to engine config"),
      buildTimingMetricItem("Normalize", timings.normalizeConfigMs, "config validation + normalization"),
      buildTimingMetricItem("Wallet load", timings.walletLoadMs, "wallet/env hydration"),
      buildTimingMetricItem("Report build", timings.reportBuildMs, "initial report assembly"),
    ],
    compile: [
      buildTimingMetricItem("Compile total", timings.compileTransactionsMs, "inclusive stage total"),
      buildTimingMetricItem("ALT load", timings.compileAltLoadMs, "lookup table fetch"),
      buildTimingMetricItem("Blockhash", timings.compileBlockhashFetchMs, "latest blockhash fetch"),
      buildTimingMetricItem("Global fetch", timings.compileGlobalFetchMs, "shared launch context"),
      buildTimingMetricItem("Follow-up prep", timings.compileFollowUpPrepMs, "follow action planning"),
      buildTimingMetricItem("Serialize tx", timings.compileTxSerializeMs, "tx serialization only"),
      buildTimingMetricItem("Compile other", compileOther, "remaining compile work", { hideZero: true }),
    ],
    send: [
      buildTimingMetricItem("Send total", timings.sendMs, "inclusive stage total"),
      buildTimingMetricItem("Submit total", timings.sendSubmitMs, "all transaction submissions"),
      buildTimingMetricItem("Confirm total", timings.sendConfirmMs, "all confirmation waits"),
      buildTimingMetricItem("Launch submit", launchSubmit, "launch tx only", { hideZero: true }),
      buildTimingMetricItem("Setup submit", timings.bagsSetupSubmitMs, "setup tx submit", { hideZero: true }),
      buildTimingMetricItem("Launch confirm", launchConfirm, "launch tx only", { hideZero: true }),
      buildTimingMetricItem("Setup confirm", timings.bagsSetupConfirmMs, "setup tx confirm", { hideZero: true }),
      buildTimingMetricItem("Send other", sendOther, "remaining transport overhead", { hideZero: true }),
    ],
  };
}

function benchmarkModeLabel(mode) {
  const normalized = String(mode || "").trim().toLowerCase();
  if (!normalized) return "";
  if (normalized === "off") return "Off";
  if (normalized === "light" || normalized === "basic") return "Light";
  if (normalized === "full") return "Full";
  return String(mode || "").trim();
}

function benchmarkMetricCardFromGroupItem(item) {
  if (!item || typeof item !== "object") return null;
  const numeric = parseReportMetricNumber(item.valueMs != null ? item.valueMs : item.value);
  if (numeric == null) return null;
  const detailParts = [];
  if (item.detail) detailParts.push(String(item.detail));
  if (item.inclusive) detailParts.push("Inclusive total");
  if (item.remainder) detailParts.push("Remainder");
  return {
    label: String(item.label || item.key || "--"),
    value: formatReportMetric(numeric, "ms"),
    detail: detailParts.join(" | "),
  };
}

function benchmarkTimingGroupsFromPayload(benchmark = {}, execution = {}) {
  const groups = Array.isArray(benchmark.timingGroups) ? benchmark.timingGroups : [];
  if (groups.length) {
    return groups.map((group) => ({
      key: String(group.key || ""),
      label: String(group.label || group.key || "Timings"),
      items: Array.isArray(group.items) ? group.items.map(benchmarkMetricCardFromGroupItem).filter(Boolean) : [],
    }));
  }
  const timings = benchmark.timings || execution.timings || {};
  const legacy = buildLegacyBenchmarkTimingSections(timings);
  return [
    { key: "topLevel", label: "Top-Level Timings", items: legacy.topLevel },
    { key: "prep", label: "Preparation", items: legacy.prep },
    { key: "compile", label: "Compile Breakdown", items: legacy.compile },
    { key: "send", label: "Send Breakdown", items: legacy.send },
  ];
}

function benchmarkTimingGroupByKey(groups, key) {
  return Array.isArray(groups) ? groups.find((group) => group && group.key === key) : null;
}

function currentReportsTerminalAutoFee() {
  const execution = currentReportsTerminalExecution();
  return execution && execution.autoFee && typeof execution.autoFee === "object"
    ? execution.autoFee
    : null;
}

function formatLamportsForReport(value) {
  const numeric = parseReportMetricNumber(value);
  if (numeric == null) return "--";
  return numeric.toLocaleString();
}

function formatSolForReport(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return "--";
  if (numeric === 0) return "0";
  const fixed = numeric.toFixed(9).replace(/\.?0+$/, "");
  return fixed === "-0" ? "0" : fixed;
}

function formatPriorityPriceForReport(value) {
  const numeric = parseReportMetricNumber(value);
  if (numeric == null) return "--";
  const solEquivalent = formatSolForReport(numeric / 1_000_000_000);
  return `${numeric.toLocaleString()} micro-lamports/CU (~${solEquivalent} SOL @ 1M CU)`;
}

function formatTipLamportsForReport(value) {
  const numeric = parseReportMetricNumber(value);
  if (numeric == null) return "--";
  const solEquivalent = formatSolForReport(numeric / 1_000_000_000);
  return `${numeric.toLocaleString()} lamports (${solEquivalent} SOL)`;
}

function autoFeeActionSignature(action) {
  if (!action || typeof action !== "object" || !action.enabled) return "";
  return JSON.stringify([
    action.provider || "",
    action.prioritySource || "",
    parseReportMetricNumber(action.priorityEstimateLamports),
    parseReportMetricNumber(action.resolvedPriorityLamports),
    action.tipSource || "",
    parseReportMetricNumber(action.tipEstimateLamports),
    parseReportMetricNumber(action.resolvedTipLamports),
    parseReportMetricNumber(action.capLamports),
  ]);
}

function groupAutoFeeActions(autoFee) {
  const entries = [
    { label: "Creation", action: autoFee && autoFee.creation },
    { label: "Buy", action: autoFee && autoFee.buy },
    { label: "Sell", action: autoFee && autoFee.sell },
  ].filter((entry) => entry.action && typeof entry.action === "object" && entry.action.enabled);
  const grouped = [];
  const indexesBySignature = new Map();
  entries.forEach((entry) => {
    const signature = autoFeeActionSignature(entry.action);
    if (!signature) return;
    if (indexesBySignature.has(signature)) {
      grouped[indexesBySignature.get(signature)].labels.push(entry.label);
      return;
    }
    indexesBySignature.set(signature, grouped.length);
    grouped.push({ labels: [entry.label], action: entry.action });
  });
  return grouped;
}

function buildAutoFeeActionCards(label, action) {
  if (!action || typeof action !== "object" || !action.enabled) return [];
  return [
    { label: `${label} Provider`, value: action.provider || "--" },
    { label: `${label} Priority Source`, value: action.prioritySource || "--", detail: action.priorityEstimateLamports != null ? `${formatPriorityPriceForReport(action.priorityEstimateLamports)} est` : "" },
    { label: `${label} Priority Used`, value: action.resolvedPriorityLamports != null ? formatPriorityPriceForReport(action.resolvedPriorityLamports) : "--" },
    { label: `${label} Tip Source`, value: action.tipSource || "--", detail: action.tipEstimateLamports != null ? `${formatTipLamportsForReport(action.tipEstimateLamports)} est` : "" },
    { label: `${label} Tip Used`, value: action.resolvedTipLamports != null ? formatTipLamportsForReport(action.resolvedTipLamports) : "--" },
    { label: `${label} Max Auto Fee`, value: action.capLamports != null ? formatTipLamportsForReport(action.capLamports) : "--" },
  ];
}

function buildAutoFeeBenchmarkSection(autoFee, benchmarkMode) {
  if (!autoFee || benchmarkMode !== "Full") return "";
  const jitoTipPercentile = String(autoFee.jitoTipPercentile || "p99").trim() || "p99";
  const snapshot = autoFee.snapshot && typeof autoFee.snapshot === "object" ? autoFee.snapshot : {};
  const snapshotCards = [
    { label: "Warm Launch Template Estimate", value: snapshot.helius_launch_priority_lamports != null ? formatPriorityPriceForReport(snapshot.helius_launch_priority_lamports) : "--" },
    { label: `Warm Jito ${jitoTipPercentile} Tip`, value: snapshot.jito_tip_p99_lamports != null ? formatTipLamportsForReport(snapshot.jito_tip_p99_lamports) : "--" },
  ].filter((card) => card.value !== "--");
  const actionCards = groupAutoFeeActions(autoFee)
    .flatMap(({ labels, action }) => buildAutoFeeActionCards(labels.join(" / "), action));
  return `
    <section class="reports-panel-section">
      <div class="reports-panel-title">Auto-Fee Sources</div>
      <div class="reports-panel-note">Full benchmark mode only. Shows the final per-action auto-fee values that were actually used.</div>
      ${renderReportMetricGrid(actionCards)}
      ${snapshotCards.length ? `
        <details class="reports-active-log-details">
          <summary>Auto-Fee Debug Snapshot</summary>
          ${renderReportMetricGrid(snapshotCards)}
        </details>
      ` : ""}
    </section>
  `;
}

function sumMetricNumbers(values = []) {
  let total = 0;
  let hasAny = false;
  values.forEach((value) => {
    const numeric = parseReportMetricNumber(value);
    if (numeric == null) return;
    total += numeric;
    hasAny = true;
  });
  return hasAny ? total : null;
}

function deriveBenchmarkRollup(timings = {}) {
  const totalElapsed = parseReportMetricNumber(timings.totalElapsedMs);
  const clientPreRequest = parseReportMetricNumber(timings.clientPreRequestMs);
  const prepareRequestPayload = parseReportMetricNumber(timings.prepareRequestPayloadMs);
  const backendTotal = parseReportMetricNumber(timings.backendTotalElapsedMs);
  const backendPrep = sumMetricNumbers([
    timings.formToRawConfigMs,
    timings.normalizeConfigMs,
    timings.walletLoadMs,
    timings.reportBuildMs,
  ]);
  const backendOrchestration = sumMetricNumbers([
    timings.transportPlanBuildMs,
    timings.autoFeeResolveMs,
    timings.sameTimeFeeGuardMs,
    timings.followDaemonReadyMs,
    timings.followDaemonReserveMs,
    timings.followDaemonArmMs,
    timings.followDaemonStatusRefreshMs,
  ]);
  const compileTotal = parseReportMetricNumber(timings.compileTransactionsMs);
  const simulateTotal = parseReportMetricNumber(timings.simulateMs);
  const sendTotal = parseReportMetricNumber(timings.sendMs);
  const reportingOverhead = sumMetricNumbers([
    timings.reportingOverheadMs,
  ]) ?? sumMetricNumbers([
    timings.persistInitialSnapshotMs,
    timings.persistFinalReportUpdateMs,
    timings.followSnapshotFlushMs,
    timings.reportRenderMs,
    timings.reportListRefreshMs,
  ]);
  const clientRemainder = deriveRemainingTiming(clientPreRequest, [prepareRequestPayload]);
  const backendMeasured = sumMetricNumbers([
    backendPrep,
    backendOrchestration,
    compileTotal,
    simulateTotal,
    sendTotal,
    reportingOverhead,
  ]);
  const backendRemainder = deriveRemainingTiming(backendTotal, [backendMeasured]);
  const executionDerived = backendTotal != null
    ? Math.max(0, backendTotal - (reportingOverhead || 0))
    : parseReportMetricNumber(timings.executionTotalMs);
  const endToEndRemainder = deriveRemainingTiming(totalElapsed, [clientPreRequest, backendTotal]);
  return {
    totalElapsed,
    clientPreRequest,
    prepareRequestPayload,
    clientRemainder,
    backendTotal,
    backendPrep,
    backendOrchestration,
    compileTotal,
    simulateTotal,
    sendTotal,
    reportingOverhead,
    backendRemainder,
    executionDerived,
    endToEndRemainder,
  };
}

function buildBenchmarkHeadlineCards(timings = {}) {
  const rollup = deriveBenchmarkRollup(timings);
  const submitTotal = parseReportMetricNumber(timings.sendSubmitMs);
  const confirmWait = parseReportMetricNumber(timings.sendConfirmMs);
  const submittedTotal = sumMetricNumbers([
    rollup.clientPreRequest,
    rollup.backendPrep,
    rollup.backendOrchestration,
    rollup.compileTotal,
    rollup.simulateTotal,
    submitTotal,
  ]);
  return [
    buildTimingMetricItem("Submitted", submittedTotal, "client + backend through provider acceptance", { tone: "primary" }),
    buildTimingMetricItem("Confirmed", rollup.totalElapsed, "full path including confirmation"),
    buildTimingMetricItem("Confirm wait", confirmWait, "provider/RPC confirmation latency", { tone: "muted" }),
  ].filter(Boolean);
}

function buildBenchmarkReconciliationSections(timings = {}, benchmarkMode = "") {
  const rollup = deriveBenchmarkRollup(timings);
  const modeLabel = benchmarkModeLabel(benchmarkMode || timings.benchmarkMode);
  return {
    topLevel: [
      buildTimingMetricItem("End-to-end", rollup.totalElapsed, "top-level wall time for this report"),
      buildTimingMetricItem("Client overhead", rollup.clientPreRequest, "before backend work starts"),
      buildTimingMetricItem("Backend total", rollup.backendTotal, "all backend-observed work"),
      buildTimingMetricItem("End-to-end remainder", rollup.endToEndRemainder, "time not explained by client + backend totals", { hideZero: true }),
    ],
    client: [
      buildTimingMetricItem("Client overhead", rollup.clientPreRequest, "inclusive client-side total"),
      buildTimingMetricItem("Prepare request payload", rollup.prepareRequestPayload, "form serialization before the POST"),
      buildTimingMetricItem("Client remainder", rollup.clientRemainder, "client time not broken into smaller steps yet", { hideZero: true }),
    ],
    backend: [
      buildTimingMetricItem("Backend total", rollup.backendTotal, "inclusive backend total"),
      buildTimingMetricItem("Execution total", rollup.executionDerived, "backend total minus reporting overhead"),
      buildTimingMetricItem("Prep subtotal", rollup.backendPrep, "normalize + wallet + routing + fee/follow setup + initial report"),
      buildTimingMetricItem("Orchestration subtotal", rollup.backendOrchestration, "transport planning, auto-fee work, and follow daemon control calls"),
      buildTimingMetricItem("Compile total", rollup.compileTotal, "inclusive compile stage"),
      buildTimingMetricItem("Simulate total", rollup.simulateTotal, "inclusive simulate stage"),
      buildTimingMetricItem("Send total", rollup.sendTotal, "inclusive send stage"),
      buildTimingMetricItem("Reporting overhead", rollup.reportingOverhead, "persist/render/report-sync work kept out of core execution"),
      buildTimingMetricItem("Backend remainder", rollup.backendRemainder, modeLabel === "Light"
        ? "backend time not broken into smaller steps in light mode"
        : "backend time not yet broken into smaller steps", { hideZero: true }),
    ],
  };
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
  if (["confirmed", "completed", "success", "healthy", "stopped"].includes(normalized)) return "is-good";
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
  if (action.marketCap && String(action.marketCap.threshold || "").trim()) {
    const timeoutSeconds = action.marketCap.scanTimeoutSeconds != null
      ? Number(action.marketCap.scanTimeoutSeconds)
      : (action.marketCap.scanTimeoutMinutes != null ? Number(action.marketCap.scanTimeoutMinutes) * 60 : null);
    const timeoutAction = String(action.marketCap.timeoutAction || "").trim();
    return `Market Cap ${action.marketCap.threshold}${Number.isFinite(timeoutSeconds) && timeoutSeconds > 0
      ? ` (${timeoutSeconds}s${timeoutAction ? `, ${timeoutAction}` : ""})`
      : ""}`;
  }
  if (action.requireConfirmation) return "After confirmation";
  if (action.targetBlockOffset != null) return `On Confirmed Block + ${action.targetBlockOffset}`;
  if (Number(action.submitDelayMs || 0) > 0) return `Submit + ${action.submitDelayMs}ms`;
  if (action.submitDelayMs != null) return "On Submit";
  if (Number(action.delayMs || 0) > 0) return `Delay ${action.delayMs}ms`;
  return "Immediate";
}

function describeFollowActionSize(action) {
  if (!action || typeof action !== "object") return "--";
  const quoteLabel = getQuoteAssetLabel(
    action.quoteAsset
      || (action.followJob && action.followJob.quoteAsset)
      || (action.parentQuoteAsset)
      || "sol",
  );
  if (action.buyAmountSol) return `${action.buyAmountSol} ${quoteLabel}`;
  if (action.sellPercent != null) return `${action.sellPercent}%`;
  return "--";
}

function describeFollowActionWallet(action) {
  if (!action || !action.walletEnvKey) return "--";
  return `Wallet #${walletIndexFromEnvKey(action.walletEnvKey)}`;
}

function formatProviderLabel(provider) {
  const normalized = String(provider || "").trim();
  if (!normalized) return "--";
  return PROVIDER_LABELS[normalized] || normalized;
}

function formatLaunchTransactionLabel(label) {
  const normalized = String(label || "").trim();
  if (!normalized) return "transaction";
  if (normalized === "follow-up") return "fee-sharing setup";
  if (normalized === "agent-setup") return "agent fee setup";
  return normalized;
}

function formatTransportLabel(transportType) {
  const normalized = String(transportType || "").trim().toLowerCase();
  if (!normalized) return "--";
  if (normalized === "helius-sender") return "Helius Sender";
  if (normalized === "hellomoon-quic") return "Hello Moon QUIC";
  if (normalized === "hellomoon-bundle") return "Hello Moon Bundle";
  if (normalized.startsWith("standard-rpc")) return "Standard RPC";
  if (normalized === "jito-bundle") return "Jito Bundle";
  return String(transportType || "").trim();
}

function buildBagsLaunchPhaseSummary(report, execution = {}) {
  const launchpad = String(report && report.launchpad || "").trim().toLowerCase();
  if (launchpad !== "bagsapp") return null;
  const sent = Array.isArray(execution.sent) ? execution.sent : [];
  const launchItems = sent.filter((item) => String(item && item.label || "").trim().toLowerCase() === "launch");
  const setupItems = sent.filter((item) => !launchItems.includes(item));
  const uniqueSetupTransports = Array.from(new Set(
    setupItems.map((item) => formatTransportLabel(item && item.transportType)).filter((value) => value && value !== "--"),
  ));
  const uniqueLaunchTransports = Array.from(new Set(
    launchItems.map((item) => formatTransportLabel(item && item.transportType)).filter((value) => value && value !== "--"),
  ));
  return {
    cards: [
      {
        label: "Launch Phases",
        value: setupItems.length ? "Setup + launch" : "Launch only",
        detail: setupItems.length
          ? `${setupItems.length} setup tx before final token creation`
          : "Single tracked launch phase",
      },
      {
        label: "Setup Phase",
        value: setupItems.length ? `${setupItems.length} tx` : "--",
        detail: uniqueSetupTransports.length ? uniqueSetupTransports.join(" | ") : "No setup transactions recorded",
      },
      {
        label: "Final Launch",
        value: launchItems.length ? `${launchItems.length} tx` : "--",
        detail: uniqueLaunchTransports.length ? uniqueLaunchTransports.join(" | ") : "Final launch transport unavailable",
      },
    ],
    note: "Bags launches are slower because they first submit and confirm setup/config transactions before the final token creation transaction. Those extra setup transactions are intentionally included in the report and benchmark path.",
  };
}

function followActionRouteDetails(action, followJob) {
  const kind = String(action && action.kind || "").trim().toLowerCase();
  const execution = followJob && followJob.execution && typeof followJob.execution === "object"
    ? followJob.execution
    : {};
  const isBuy = kind === "sniper-buy";
  const isSell = kind === "dev-auto-sell" || kind === "sniper-sell";
  return {
    provider: String(
      (action && action.provider)
      || (isBuy ? execution.buyProvider : "")
      || (isSell ? execution.sellProvider : "")
      || execution.provider
      || "",
    ).trim(),
    endpointProfile: String(
      (action && action.endpointProfile)
      || (isBuy ? execution.buyEndpointProfile : "")
      || (isSell ? execution.sellEndpointProfile : "")
      || execution.endpointProfile
      || "",
    ).trim(),
    transportType: String(action && action.transportType || "").trim(),
  };
}

function describeFollowActionRoute(action, followJob) {
  const route = followActionRouteDetails(action, followJob);
  const parts = [];
  if (route.provider) parts.push(formatProviderLabel(route.provider));
  if (route.transportType && route.transportType !== route.provider) parts.push(route.transportType);
  return parts.join(" | ");
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

function formatReportEndpointList(endpoints = []) {
  const normalized = Array.isArray(endpoints)
    ? endpoints
      .map((value) => String(value || "").trim())
      .filter(Boolean)
    : [];
  if (!normalized.length) return "--";
  return Array.from(new Set(normalized))
    .map((value) => shortenReportEndpoint(value))
    .join(" | ");
}

function formatWatcherModeLabel(mode) {
  const normalized = String(mode || "").trim().toLowerCase();
  if (!normalized) return "";
  if (normalized === "helius-transaction-subscribe") return "Helius transactionSubscribe";
  if (normalized === "standard-ws") return "Standard websocket";
  if (normalized === "rpc-polling") return "RPC polling";
  return String(mode || "").trim();
}

function buildWatcherDetail(mode, fallbackReason) {
  const parts = [];
  const modeLabel = formatWatcherModeLabel(mode);
  if (modeLabel) parts.push(`Mode: ${modeLabel}`);
  const note = String(fallbackReason || "").trim();
  if (note) parts.push(note);
  return parts.join(" | ");
}

function buildCombinedFollowWatcherCard(actions = [], health = null) {
  const relevantActions = actions.filter((action) => {
    const kind = String(action && action.kind || "").trim().toLowerCase();
    return ["sniper-buy", "sniper-sell", "dev-auto-sell"].includes(kind);
  });
  const actionModes = Array.from(new Set(
    relevantActions
      .map((action) => formatWatcherModeLabel(action && action.watcherMode))
      .filter(Boolean),
  ));
  const healthModes = Array.from(new Set(
    [
      health && health.slotWatcherMode,
      health && health.signatureWatcherMode,
      health && health.marketWatcherMode,
    ]
      .map((mode) => formatWatcherModeLabel(mode))
      .filter(Boolean),
  ));
  const modes = actionModes.length ? actionModes : healthModes;
  const endpointLabel = shortenReportEndpoint(health && health.watchEndpoint);
  if (!modes.length && (!endpointLabel || endpointLabel === "--")) return null;
  const detailParts = [];
  if (modes.length === 1) {
    detailParts.push(`Mode: ${modes[0]}`);
  } else if (modes.length > 1) {
    detailParts.push(`Modes: ${modes.join(" | ")}`);
  }
  return {
    label: "Follow Watcher WS",
    value: endpointLabel && endpointLabel !== "--"
      ? endpointLabel
      : (modes.length === 1 ? modes[0] : "Mixed"),
    detail: detailParts.join(" | "),
  };
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
  const route = followActionRouteDetails(action, followJob);
  const metrics = [
    { label: "Provider", value: formatProviderLabel(route.provider) },
    { label: "Transport", value: route.transportType || "--" },
    { label: "Endpoint Profile", value: route.endpointProfile || "--" },
    { label: "Watcher", value: formatWatcherModeLabel(action && action.watcherMode) || "--", detail: action && action.watcherFallbackReason ? String(action.watcherFallbackReason) : "" },
    { label: "Wallet", value: describeFollowActionWallet(action) },
    { label: "Trigger", value: describeFollowActionTrigger(action) },
    {
      label: "Size",
      value: describeFollowActionSize({
        ...action,
        parentQuoteAsset: followJob && followJob.quoteAsset,
      }),
    },
    { label: "Send Block Height", value: action && action.sendObservedBlockHeight != null ? String(action.sendObservedBlockHeight) : isBuy && followJob && followJob.sendObservedBlockHeight != null ? `launch ${followJob.sendObservedBlockHeight}` : "--" },
    { label: "Confirm Block Height", value: action && action.confirmedObservedBlockHeight != null ? String(action.confirmedObservedBlockHeight) : "--" },
    { label: "Blocks To Confirm", value: action && action.blocksToConfirm != null ? String(action.blocksToConfirm) : "--" },
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
        <div class="reports-metric-card${item.tone ? ` is-${escapeHTML(String(item.tone))}` : ""}">
          <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
          <strong class="reports-metric-value">${item.renderValue ? item.renderValue : escapeHTML(String(item.value))}</strong>
          ${item.detail ? `<span class="reports-metric-note">${escapeHTML(String(item.detail))}</span>` : ""}
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
  const benchmarkGroups = benchmarkTimingGroupsFromPayload(benchmark, execution);
  const topLevelGroup = benchmarkTimingGroupByKey(benchmarkGroups, "topLevel") || benchmarkGroups[0] || { items: [] };
  const health = report && report.followDaemon && report.followDaemon.health ? report.followDaemon.health : null;
  const job = currentReportsTerminalFollowJob();
  const actions = currentReportsTerminalFollowActions();
  const benchmarkMode = benchmarkModeLabel(benchmark.mode || (timings && timings.benchmarkMode));
  const launchSpeedCards = buildBenchmarkHeadlineCards(timings);
  const bagsLaunchPhaseSummary = buildBagsLaunchPhaseSummary(report, execution);
  const providerCardLabel = job ? "Launch Provider" : "Provider";
  const transportCardLabel = job ? "Launch Transport" : "Transport";
  const problemCount = actions.filter((action) => ["failed", "cancelled", "expired"].includes(String(action.state || "").toLowerCase())).length;
  const runningCount = actions.filter((action) => ["running", "eligible", "armed", "queued", "sent"].includes(String(action.state || "").toLowerCase())).length;
  const combinedWatcherCard = buildCombinedFollowWatcherCard(actions, health);
  const formatDaemonCapacityValue = (available, max) => {
    if (max == null) return "Uncapped";
    if (available == null) return "--";
    return String(available);
  };
  const overviewCards = [
    { label: "Action", value: entry && entry.action ? entry.action : payload && payload.action ? payload.action : "--" },
    {
      label: "Mint",
      value: entry && entry.mint ? shortenAddress(entry.mint, 6) : report && report.mint ? shortenAddress(report.mint, 6) : "--",
      renderValue: entry && entry.mint
        ? renderCopyableHash(entry.mint, "Copy mint")
        : (report && report.mint ? renderCopyableHash(report.mint, "Copy mint") : "--"),
    },
    { label: providerCardLabel, value: execution.resolvedProvider || execution.provider || "--" },
    { label: transportCardLabel, value: execution.transportType || (entry && entry.transportType) || "--" },
    { label: "Signatures", value: entry ? String(entry.signatureCount || 0) : String(Array.isArray(payload && payload.signatures) ? payload.signatures.length : 0) },
    { label: "Follow", value: job ? (job.state || "armed") : "Off" },
    { label: "Selected Wallet", value: job && job.selectedWalletKey ? `Wallet #${walletIndexFromEnvKey(job.selectedWalletKey)}` : "--" },
    { label: "Follow Actions", value: actions.length ? `${actions.length} total` : "0" },
    { label: "Problems", value: String(problemCount) },
    { label: "Running", value: String(runningCount) },
  ].concat(combinedWatcherCard ? [combinedWatcherCard] : []);
  const watcherCards = health
    ? [
      { label: "Slot Watcher", value: health.slotWatcher || "--", detail: buildWatcherDetail(health.slotWatcherMode) },
      { label: "Signature Watcher", value: health.signatureWatcher || "--", detail: buildWatcherDetail(health.signatureWatcherMode) },
      { label: "Market Watcher", value: health.marketWatcher || "--", detail: buildWatcherDetail(health.marketWatcherMode) },
      { label: "Queue Depth", value: String(health.queueDepth != null ? health.queueDepth : "--") },
      { label: "Compile Slots", value: formatDaemonCapacityValue(health.availableCompileSlots, health.maxConcurrentCompiles) },
      { label: "Send Slots", value: formatDaemonCapacityValue(health.availableSendSlots, health.maxConcurrentSends) },
    ]
    : [];
  return `
    <div class="reports-panel-stack">
      <section class="reports-panel-section">
        <div class="reports-panel-title">Overview</div>
        ${renderReportMetricGrid(overviewCards)}
      </section>
      ${bagsLaunchPhaseSummary ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Bags Launch Phases</div>
          ${renderReportMetricGrid(bagsLaunchPhaseSummary.cards)}
          <div class="reports-callout is-bad">${escapeHTML(bagsLaunchPhaseSummary.note)}</div>
        </section>
      ` : ""}
      ${launchSpeedCards.length ? `
        <section class="reports-panel-section reports-panel-section-launch-speed">
          <div class="reports-panel-title">Launch Speed</div>
          <div class="reports-panel-note">Submission is the steadier execution metric. Confirmation varies more between runs because it depends on provider/RPC observation latency.</div>
          <div class="reports-metric-grid reports-metric-grid-launch-speed">
            ${launchSpeedCards.map((item) => `
              <div class="reports-metric-card${item.tone ? ` is-${escapeHTML(String(item.tone))}` : ""}">
                <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
                <strong class="reports-metric-value">${escapeHTML(String(item.value))}</strong>
                ${item.detail ? `<span class="reports-metric-note">${escapeHTML(String(item.detail))}</span>` : ""}
              </div>
            `).join("")}
          </div>
        </section>
      ` : ""}
      <section class="reports-panel-section">
        <div class="reports-panel-title">Stage Totals</div>
        <div class="reports-panel-note">
          ${benchmarkMode ? `Benchmark mode: ${escapeHTML(benchmarkMode)}. ` : ""}Totals are inclusive. Child timings and remainders are broken out on the Benchmarks tab.
        </div>
        ${renderReportMetricGrid(topLevelGroup.items || [])}
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
  const report = currentReportsTerminalReport();
  const execution = currentReportsTerminalExecution() || {};
  const followJob = currentReportsTerminalFollowJob();
  const actions = currentReportsTerminalFollowActions();
  const launchSends = Array.isArray(execution.sent) ? execution.sent : [];
  const bagsLaunchPhaseSummary = buildBagsLaunchPhaseSummary(report, execution);
  if (!launchSends.length && !actions.length) {
    return '<div class="reports-terminal-empty">No action data available in this report.</div>';
  }
  return `
    <div class="reports-panel-stack">
      ${bagsLaunchPhaseSummary ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Bags Launch Phases</div>
          <div class="reports-callout is-bad">${escapeHTML(bagsLaunchPhaseSummary.note)}</div>
        </section>
      ` : ""}
      ${launchSends.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Launch Send</div>
          <div class="reports-action-list">
            ${launchSends.map((sent) => `
              <article class="reports-action-card">
                <div class="reports-action-head">
                  <div>
                    <strong>${escapeHTML(formatLaunchTransactionLabel(sent.label || "launch"))}</strong>
                    <div class="reports-action-subtitle">${escapeHTML([
                      execution.resolvedProvider || execution.provider || execution.transportType || "--",
                      sent.endpoint ? `winning ${shortenReportEndpoint(sent.endpoint)}` : "",
                    ].filter(Boolean).join(" | "))}</div>
                  </div>
                  <span class="reports-state-badge ${reportStateClass(sent.confirmationStatus)}">${escapeHTML(sent.confirmationStatus || "sent")}</span>
                </div>
                ${renderReportMetricGrid([
                  { label: "Winning Endpoint", value: shortenReportEndpoint(sent.endpoint) },
                  { label: "Attempted Endpoints", value: formatReportEndpointList(sent.attemptedEndpoints), detail: Array.isArray(sent.attemptedEndpoints) && sent.attemptedEndpoints.length > 1 ? `${sent.attemptedEndpoints.length} attempted` : "" },
                  { label: "Send Block Height", value: sent.sendObservedBlockHeight != null ? String(sent.sendObservedBlockHeight) : "--" },
                  { label: "Confirm Block Height", value: sent.confirmedObservedBlockHeight != null ? String(sent.confirmedObservedBlockHeight) : "--" },
                  { label: "Blocks To Confirm", value: sent.confirmedObservedBlockHeight != null && sent.sendObservedBlockHeight != null ? String(sent.confirmedObservedBlockHeight - sent.sendObservedBlockHeight) : "--" },
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
              const subtitleParts = [
                describeFollowActionRoute(action, followJob),
                describeFollowActionWallet(action),
                describeFollowActionTrigger(action),
                describeFollowActionSize({ ...action, parentQuoteAsset: followJob && followJob.quoteAsset }),
              ].filter((part) => part && part !== "--");
              return `
                <article class="reports-action-card">
                  <div class="reports-action-head">
                    <div>
                      <strong>${escapeHTML(action.kind || action.actionId || "action")}</strong>
                      <div class="reports-action-subtitle">${escapeHTML(subtitleParts.join(" | "))}</div>
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
  const autoFee = currentReportsTerminalAutoFee();
  const timings = benchmark.timings || execution.timings || {};
  const benchmarkGroups = benchmarkTimingGroupsFromPayload(benchmark, execution);
  const sent = Array.isArray(benchmark.sent) && benchmark.sent.length ? benchmark.sent : (Array.isArray(execution.sent) ? execution.sent : []);
  const benchmarkMode = benchmarkModeLabel(benchmark.mode || (timings && timings.benchmarkMode));
  const launchSpeedCards = buildBenchmarkHeadlineCards(timings);
  const reconciliation = buildBenchmarkReconciliationSections(timings, benchmark.mode || (timings && timings.benchmarkMode));
  const timingSectionsMarkup = benchmarkGroups.length
    ? benchmarkGroups.map((group) => `
      <section class="reports-panel-section">
        <div class="reports-panel-title">${escapeHTML(group.label || "Timings")}</div>
        ${group.key === "topLevel"
          ? '<div class="reports-panel-note">Inclusive totals and remainder buckets are separated so the path can be audited.</div>'
          : ""}
        ${renderReportMetricGrid(group.items || [])}
      </section>
    `).join("")
    : '<section class="reports-panel-section"><div class="reports-terminal-empty">No benchmark timing groups are available for this report.</div></section>';
  return `
    <div class="reports-panel-stack">
      ${benchmarkMode ? `<section class="reports-panel-section"><div class="reports-panel-title">Benchmark Mode</div><div class="reports-panel-note">${escapeHTML(benchmarkMode)} benchmark collection is active for this report.</div></section>` : ""}
      ${buildAutoFeeBenchmarkSection(autoFee, benchmarkMode)}
      ${launchSpeedCards.length ? `
        <section class="reports-panel-section reports-panel-section-launch-speed">
          <div class="reports-panel-title">Launch Speed</div>
          <div class="reports-panel-note">Submitted is the steadier execution metric. Confirmed includes the variable provider/RPC confirmation wait.</div>
          <div class="reports-metric-grid reports-metric-grid-launch-speed">
            ${launchSpeedCards.map((item) => `
              <div class="reports-metric-card${item.tone ? ` is-${escapeHTML(String(item.tone))}` : ""}">
                <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
                <strong class="reports-metric-value">${escapeHTML(String(item.value))}</strong>
                ${item.detail ? `<span class="reports-metric-note">${escapeHTML(String(item.detail))}</span>` : ""}
              </div>
            `).join("")}
          </div>
        </section>
      ` : ""}
      <section class="reports-panel-section">
        <div class="reports-panel-title">End-to-End Composition</div>
        <div class="reports-panel-note">This section shows exactly what the top-line benchmark consists of before you scroll into the lower-level groups.</div>
        ${renderReportMetricGrid(reconciliation.topLevel)}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Client Composition</div>
        ${renderReportMetricGrid(reconciliation.client)}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Backend Composition</div>
        <div class="reports-panel-note">If a remainder is non-zero, that is measured time inside the parent total that this report or benchmark mode has not broken into smaller named steps yet.</div>
        ${renderReportMetricGrid(reconciliation.backend)}
      </section>
      ${timingSectionsMarkup}
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
                  { label: "Send Block Height", value: item.sendBlockHeight != null ? String(item.sendBlockHeight) : item.sendObservedBlockHeight != null ? String(item.sendObservedBlockHeight) : "--" },
                  { label: "Confirm Block Height", value: item.confirmedBlockHeight != null ? String(item.confirmedBlockHeight) : item.confirmedObservedBlockHeight != null ? String(item.confirmedObservedBlockHeight) : "--" },
                  { label: "Blocks To Confirm", value: item.blocksToConfirm != null ? String(item.blocksToConfirm) : "--" },
                  { label: "Confirmed Slot", value: item.confirmedSlot != null ? String(item.confirmedSlot) : "--" },
                ])}
              </article>
            `).join("")}
          </div>
        ` : '<div class="reports-terminal-empty">No chain benchmark entries recorded.</div>'}
      </section>
    </div>
  `;
}

function buildBenchmarksPopoutTitle() {
  return "Benchmark Popout";
}

function renderBenchmarksPopoutModal() {
  if (!benchmarksPopoutModal || benchmarksPopoutModal.hidden || !benchmarksPopoutBody) return;
  if (benchmarksPopoutTitle) {
    benchmarksPopoutTitle.textContent = buildBenchmarksPopoutTitle();
  }
  const payload = currentReportsTerminalPayload();
  benchmarksPopoutBody.innerHTML = payload
    ? `<div class="benchmarks-popout-content">${buildReportsBenchmarksMarkup()}</div>`
    : '<div class="reports-callout">Structured benchmark data is unavailable for this report.</div>';
}

function showBenchmarksPopoutModal() {
  if (!benchmarksPopoutModal || !benchmarksPopoutBody) return;
  benchmarksPopoutModal.hidden = false;
  renderBenchmarksPopoutModal();
}

function hideBenchmarksPopoutModal() {
  if (!benchmarksPopoutModal) return;
  benchmarksPopoutModal.hidden = true;
}

function buildReportsRawMarkup() {
  return `<pre class="console reports-console">${escapeHTML(reportsTerminalState.activeText || "Report is empty.")}</pre>`;
}

function buildLaunchHistorySettingsText(launch) {
  const settings = [
    launch.report && launch.report.launchpad ? launch.report.launchpad : "",
    launch.report && launch.report.mode ? launch.report.mode : "",
    launch.quoteAsset || "",
    launch.bags && launch.bags.identityMode === "linked"
      ? (launch.bags.agentUsername ? `identity @${launch.bags.agentUsername}` : "identity linked")
      : "",
    launch.execution && launch.execution.activePresetId ? launch.execution.activePresetId : "",
    launch.selectedWalletKey ? formatWalletHistoryLabel(launch.selectedWalletKey) : "",
    launch.execution && launch.execution.provider ? launch.execution.provider : "",
    launch.execution && launch.execution.buyProvider ? `buy ${launch.execution.buyProvider}` : "",
    launch.execution && launch.execution.sellProvider ? `sell ${launch.execution.sellProvider}` : "",
  ].filter(Boolean);
  return settings.join(" | ");
}

function parseLaunchBuyWalletLabel(rawLabel) {
  const label = String(rawLabel || "").trim();
  const match = label.match(/wallet-(.+)$/i);
  if (!match) return "";
  const suffix = String(match[1] || "").trim();
  if (!suffix || suffix === "primary") return "Wallet #1";
  if (/^\d+$/.test(suffix)) return `Wallet #${suffix}`;
  return `Wallet ${suffix}`;
}

function buildLaunchHistorySnipeText(launch) {
  const quoteAsset = launch.quoteAsset || "sol";
  const quoteLabel = getQuoteAssetLabel(quoteAsset);
  const snipes = launch.followLaunch && Array.isArray(launch.followLaunch.snipes)
    ? launch.followLaunch.snipes.filter((entry) => entry && entry.enabled !== false)
    : [];
  if (!snipes.length) return "";
  return snipes.map((entry) => {
    const walletLabel = formatWalletHistoryLabel(entry.walletEnvKey || entry.envKey || "");
    const amount = entry.buyAmountSol ? `${entry.buyAmountSol} ${quoteLabel}` : `-- ${quoteLabel}`;
    const trigger = entry.targetBlockOffset != null
      ? `b${entry.targetBlockOffset}`
      : (entry.submitWithLaunch ? "same-time" : getSniperTriggerSummary(entry).toLowerCase());
    const retry = entry.retryOnFailure ? " | retry once" : "";
    return `${walletLabel || "Wallet"} ${amount} @ ${trigger}${retry}`;
  }).join(" | ");
}

function buildLaunchHistoryLaunchBuyFallbackText(launch) {
  const sent = launch.execution && Array.isArray(launch.execution.sent) ? launch.execution.sent : [];
  const labels = sent
    .map((entry) => parseLaunchBuyWalletLabel(entry && entry.label))
    .filter(Boolean);
  if (!labels.length) return "";
  return `launch buys ${labels.join(" | ")}`;
}

function buildLaunchHistoryAutoSellText(launch) {
  const devAutoSell = launch.followLaunch && launch.followLaunch.devAutoSell && launch.followLaunch.devAutoSell.enabled;
  if (!devAutoSell) return "";
  const autoSell = launch.followLaunch.devAutoSell;
  const percent = autoSell.percent != null ? autoSell.percent : 100;
  const parts = [`${percent}%`];
  if (autoSell.marketCap && autoSell.marketCap.threshold) {
    const trigger = autoSell.marketCap;
    parts.push(
      `market ${trigger.threshold}${
        (trigger.scanTimeoutSeconds != null || trigger.scanTimeoutMinutes != null)
          ? ` (${trigger.scanTimeoutSeconds != null ? trigger.scanTimeoutSeconds : trigger.scanTimeoutMinutes * 60}s${trigger.timeoutAction ? `, ${trigger.timeoutAction}` : ""})`
          : ""
      }`
    );
  } else if (autoSell.targetBlockOffset != null) {
    parts.push(`confirmed + ${autoSell.targetBlockOffset}`);
  } else if (autoSell.requireConfirmation) {
    parts.push("after confirmation");
  } else {
    const delayMs = autoSell.delayMs != null ? autoSell.delayMs : 0;
    parts.push(delayMs > 0 ? `submit + ${delayMs}ms` : "on submit");
  }
  return parts.join(" | ");
}

function buildLaunchHistoryBuyAmountItems(launch) {
  const items = [];
  if (launch.devBuy && launch.devBuy.amount) {
    items.push({
      label: "Dev Buy",
      value: `${launch.devBuy.amount} ${launch.devBuy.mode === "tokens" ? "%" : getDevBuyAssetLabel(launch.launchpad || "pump", launch.quoteAsset || "sol")}`,
    });
  }
  const snipeText = buildLaunchHistorySnipeText(launch);
  if (snipeText) {
    items.push({ label: "Auto Snipe", value: snipeText });
  } else {
    const fallbackSnipeText = buildLaunchHistoryLaunchBuyFallbackText(launch);
    if (fallbackSnipeText) items.push({ label: "Launch Buys", value: fallbackSnipeText });
  }
  const autoSellText = buildLaunchHistoryAutoSellText(launch);
  if (autoSellText) {
    items.push({ label: "Auto Sell", value: autoSellText });
  }
  return items;
}

function buildLaunchHistoryBuyAmountsMarkup(launch) {
  const items = buildLaunchHistoryBuyAmountItems(launch);
  if (!items.length) {
    return '<div class="reports-launch-card-copy">No buy actions recorded.</div>';
  }
  return `
    <div class="reports-launch-card-detail-list">
      ${items.map((item) => `
        <div class="reports-launch-card-detail-row">
          <div class="reports-launch-card-detail-key">${escapeHTML(item.label)}</div>
          <div class="reports-launch-card-detail-value">${escapeHTML(item.value)}</div>
        </div>
      `).join("")}
    </div>
  `;
}

function formatLaunchHistoryDisplayTime(entry) {
  const writtenAtMs = entry && entry.writtenAtMs != null ? entry.writtenAtMs : 0;
  const numeric = Number(writtenAtMs);
  if (Number.isFinite(numeric) && numeric > 0) {
    try {
      return new Date(numeric).toLocaleString([], {
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
      });
    } catch (_error) {
      // Fall back below.
    }
  }
  return entry && entry.displayTime ? String(entry.displayTime) : "";
}

function launchPrimarySignature(launch) {
  return launch.payload && Array.isArray(launch.payload.signatures) && launch.payload.signatures[0]
    ? String(launch.payload.signatures[0]).trim()
    : "";
}

function launchSolscanUrl(launch) {
  const signature = launchPrimarySignature(launch);
  return signature ? `https://solscan.io/tx/${encodeURIComponent(signature)}` : "";
}

function formatFollowStateLabel(value) {
  const normalized = String(value || "").trim();
  if (!normalized) return "Unknown";
  return normalized
    .split("-")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function followStateBadgeTone(state) {
  const normalized = String(state || "").trim().toLowerCase();
  if (["running", "sent", "confirmed", "completed", "stopped"].includes(normalized)) return "is-good";
  if (["failed", "cancelled", "completed-with-failures", "expired"].includes(normalized)) return "is-bad";
  if (["armed", "eligible", "reserved"].includes(normalized)) return "is-warn";
  return "";
}

function formatCompactDateTime(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return "";
  try {
    return new Date(numeric).toLocaleString([], {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  } catch (_error) {
    return "";
  }
}

function summarizeFollowJobProgress(job) {
  const actions = Array.isArray(job && job.actions) ? job.actions : [];
  if (!actions.length) {
    return job && job.cancelRequested
      ? "Cancel requested."
      : "Waiting for follow actions.";
  }
  const counts = actions.reduce((accumulator, action) => {
    const state = String(action && action.state || "").trim().toLowerCase();
    if (state) {
      accumulator[state] = (accumulator[state] || 0) + 1;
    }
    return accumulator;
  }, {});
  const doneCount = (counts.confirmed || 0) + (counts.sent || 0) + (counts.stopped || 0);
  const activeCount = (counts.running || 0) + (counts.eligible || 0);
  const queuedCount = (counts.queued || 0) + (counts.armed || 0);
  const stoppedCount = counts.stopped || 0;
  const failedCount = counts.failed || 0;
  const cancelledCount = counts.cancelled || 0;
  const expiredCount = counts.expired || 0;
  const parts = [`${doneCount}/${actions.length} done`];
  if (activeCount > 0) parts.push(`${activeCount} active`);
  if (queuedCount > 0) parts.push(`${queuedCount} queued`);
  if (stoppedCount > 0) parts.push(`${stoppedCount} stopped`);
  if (failedCount > 0) parts.push(`${failedCount} failed`);
  if (cancelledCount > 0) parts.push(`${cancelledCount} cancelled`);
  if (expiredCount > 0) parts.push(`${expiredCount} expired`);
  if (job && job.cancelRequested) parts.push("cancel requested");
  return parts.join(" | ");
}

function buildFollowActionSubtitle(action) {
  const parts = [];
  if (action && action.walletEnvKey) parts.push(`W${walletIndexFromEnvKey(action.walletEnvKey)}`);
  if (action && action.buyAmountSol) parts.push(`${action.buyAmountSol} SOL`);
  if (action && action.sellPercent != null) parts.push(`${action.sellPercent}% sell`);
  if (action && action.targetBlockOffset != null) parts.push(`+${action.targetBlockOffset} blocks`);
  if (action && action.submitDelayMs != null && Number(action.submitDelayMs) > 0) parts.push(`${action.submitDelayMs}ms delay`);
  if (action && action.watcherMode) parts.push(formatWatcherModeLabel(action.watcherMode));
  if (action && action.signature) parts.push(shortAddress(action.signature));
  return parts.join(" | ");
}

function buildActiveJobActionRouteMarkup(action, followJob) {
  const route = followActionRouteDetails(action, followJob);
  const rows = [
    { label: "Provider", value: formatProviderLabel(route.provider) },
    { label: "Transport", value: route.transportType || "--" },
    { label: "Profile", value: route.endpointProfile || "--" },
  ];
  if (action && action.watcherMode) {
    rows.push({ label: "Watcher", value: formatWatcherModeLabel(action.watcherMode) });
  }
  return `
    <div class="reports-active-job-action-meta">
      ${rows.map((row) => `
        <span class="reports-active-job-action-meta-pill">
          <strong>${escapeHTML(row.label)}</strong>
          <span>${escapeHTML(row.value)}</span>
        </span>
      `).join("")}
    </div>
  `;
}

function buildActiveJobLaunchRouteMarkup(job) {
  const plan = job && job.transportPlan && typeof job.transportPlan === "object"
    ? job.transportPlan
    : {};
  const execution = job && job.execution && typeof job.execution === "object"
    ? job.execution
    : {};
  const rows = [
    { label: "Launch Provider", value: formatProviderLabel(plan.resolvedProvider || execution.provider || "") },
    { label: "Launch Transport", value: String(plan.transportType || "--").trim() || "--" },
    { label: "Launch Profile", value: String(plan.resolvedEndpointProfile || execution.endpointProfile || "--").trim() || "--" },
  ];
  return `
    <div class="reports-active-job-route-meta">
      ${rows.map((row) => `
        <span class="reports-active-job-action-meta-pill">
          <strong>${escapeHTML(row.label)}</strong>
          <span>${escapeHTML(row.value)}</span>
        </span>
      `).join("")}
    </div>
  `;
}

function buildReportsActiveJobsMarkup() {
  const snapshot = followStatusSnapshot();
  const activeJobs = followJobsState.jobs.filter((job) => !isTerminalFollowJobState(job && job.state));
  const summaryClassNames = [
    "reports-follow-summary",
    snapshot.offline ? "is-offline" : "",
    snapshot.counts.active > 0 ? "is-active" : "",
    snapshot.counts.issues > 0 ? "is-issues" : "",
  ].filter(Boolean).join(" ");
  if (snapshot.offline) {
    return `
      <div class="reports-panel-stack">
        <div class="reports-active-jobs-header">
          <div class="reports-active-jobs-heading">
            <strong>Jobs</strong>
            <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
          </div>
          <button
            type="button"
            class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger"
            data-follow-cancel-all="1"
            disabled
          >Cancel all</button>
        </div>
        <div class="reports-callout is-bad">${escapeHTML(followJobsState.error || "Follow daemon is offline. Live active jobs are temporarily unavailable.")}</div>
      </div>
    `;
  }
  if (!snapshot.configured) {
    return `
      <div class="reports-panel-stack">
        <div class="reports-active-jobs-header">
          <div class="reports-active-jobs-heading">
            <strong>Jobs</strong>
            <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
          </div>
        </div>
        <div class="reports-terminal-empty">Follow daemon is not enabled for this workspace.</div>
      </div>
    `;
  }
  if (!activeJobs.length) {
    return `
      <div class="reports-panel-stack">
        <div class="reports-active-jobs-header">
          <div class="reports-active-jobs-heading">
            <strong>Jobs</strong>
            <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
          </div>
        </div>
        <div class="reports-terminal-empty">No active follow jobs right now.</div>
      </div>
    `;
  }
  return `
    <div class="reports-panel-stack">
      <div class="reports-active-jobs-header">
        <div class="reports-active-jobs-heading">
          <strong>Jobs</strong>
          <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
        </div>
        <button
          type="button"
          class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger"
          data-follow-cancel-all="1"
          ${snapshot.canCancelAll && !snapshot.offline ? "" : "disabled"}
        >Cancel all</button>
      </div>
      <div class="reports-active-jobs-grid">
        ${activeJobs.map((job) => {
          const createdLabel = formatCompactDateTime(job.createdAtMs || job.updatedAtMs);
          const launchUrl = job.launchSignature ? `https://solscan.io/tx/${encodeURIComponent(job.launchSignature)}` : "";
          return `
            <article class="reports-launch-card reports-active-job-card">
              <div class="reports-action-head">
                <div>
                  <strong class="reports-launch-card-title">${escapeHTML(`${job.launchpad || "launch"} follow job`)}</strong>
                  <div class="reports-launch-card-subtitle">${escapeHTML(createdLabel ? `Created ${createdLabel}` : `Trace ${shortAddress(job.traceId || "")}`)}</div>
                </div>
                <span class="reports-state-badge ${followStateBadgeTone(job.state)}">${escapeHTML(formatFollowStateLabel(job.state))}</span>
              </div>
              <div class="reports-launch-card-chip-row">
                <span class="reports-launch-card-chip">${escapeHTML(job.launchpad || "launch")}</span>
                <span class="reports-launch-card-chip">${escapeHTML(job.quoteAsset || "sol")}</span>
                ${job.cancelRequested ? '<span class="reports-launch-card-chip">cancel requested</span>' : ""}
              </div>
              ${buildActiveJobLaunchRouteMarkup(job)}
              <div class="reports-active-job-meta-grid">
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Wallet</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.selectedWalletKey || "-")}</span>
                </div>
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Mint</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.mint || "-")}</span>
                </div>
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Trace</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.traceId || "-")}</span>
                </div>
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Launch</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.launchSignature || "-")}</span>
                </div>
              </div>
              <div class="reports-launch-card-section">
                <div class="reports-launch-card-label">Progress</div>
                <div class="reports-launch-card-copy">${escapeHTML(summarizeFollowJobProgress(job))}</div>
                <div class="reports-active-job-action-list">
                  ${(Array.isArray(job.actions) ? job.actions : []).map((action) => `
                    <div class="reports-active-job-action">
                      <div class="reports-active-job-action-copy">
                        <strong>${escapeHTML(formatFollowStateLabel(action.kind || "action"))}</strong>
                        <span>${escapeHTML(buildFollowActionSubtitle(action) || "No extra details.")}</span>
                        ${action && action.watcherFallbackReason ? `<span>${escapeHTML(String(action.watcherFallbackReason))}</span>` : ""}
                        ${buildActiveJobActionRouteMarkup(action, job)}
                      </div>
                      <span class="reports-state-badge ${followStateBadgeTone(action.state)}">${escapeHTML(formatFollowStateLabel(action.state))}</span>
                    </div>
                  `).join("")}
                </div>
              </div>
              ${job.lastError ? `<div class="reports-callout is-bad">${escapeHTML(job.lastError)}</div>` : ""}
              <div class="reports-launch-card-footer">
                ${launchUrl ? `<a class="reports-inline-link" href="${escapeHTML(launchUrl)}" target="_blank" rel="noreferrer">Open launch tx</a>` : '<span class="reports-launch-card-copy">Launch signature not available yet.</span>'}
                <div class="reports-launch-card-actions">
                  <button
                    type="button"
                    class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger"
                    data-follow-cancel-trace-id="${escapeHTML(job.traceId || "")}"
                    ${job.cancelRequested || snapshot.offline ? "disabled" : ""}
                  >Cancel</button>
                </div>
              </div>
            </article>
          `;
        }).join("")}
      </div>
    </div>
  `;
}

function logLevelBadgeTone(level) {
  const normalized = String(level || "").trim().toLowerCase();
  if (normalized === "error") return "is-bad";
  if (normalized === "warn" || normalized === "warning") return "is-warn";
  return "is-good";
}

function formatActiveLogLevel(level) {
  const normalized = String(level || "").trim().toLowerCase();
  if (!normalized) return "INFO";
  return normalized.toUpperCase();
}

function stringifyActiveLogContext(context) {
  if (context == null) return "";
  try {
    return JSON.stringify(context, null, 2);
  } catch (_error) {
    return String(context);
  }
}

function summarizeActiveLogContext(context) {
  if (context == null || typeof context !== "object") return "";
  const entries = Object.entries(context)
    .filter(([, value]) => value == null || ["string", "number", "boolean"].includes(typeof value))
    .slice(0, 3)
    .map(([key, value]) => `${key}: ${String(value)}`);
  return entries.join(" | ");
}

function buildActiveLogsMarkup() {
  const activeLogsView = normalizeActiveLogsView(reportsTerminalState.activeLogsView);
  const logsState = reportsTerminalState.activeLogs && typeof reportsTerminalState.activeLogs === "object"
    ? reportsTerminalState.activeLogs
    : { live: [], errors: [], error: "", updatedAtMs: 0 };
  const logs = Array.isArray(logsState[activeLogsView]) ? logsState[activeLogsView] : [];
  const updatedLabel = logsState.updatedAtMs ? formatCompactDateTime(logsState.updatedAtMs) : "";
  return `
    <div class="reports-panel-stack">
      <div class="reports-active-jobs-header">
        <div class="reports-active-jobs-heading">
          <strong>Logs</strong>
          <span class="reports-follow-summary ${logsState.error ? "is-issues" : logs.length ? "is-active" : ""}">
            ${escapeHTML(
              logsState.error
                ? logsState.error
                : `${logs.length} ${activeLogsView === "errors" ? "saved error" : "live log"} entr${logs.length === 1 ? "y" : "ies"}${updatedLabel ? ` | Updated ${updatedLabel}` : ""}`
            )}
          </span>
        </div>
        <div class="reports-terminal-tabs reports-active-logs-tabs">
          <button
            type="button"
            class="reports-terminal-tab${activeLogsView === "live" ? " active" : ""}"
            data-active-logs-view="live"
          >Live Logs</button>
          <button
            type="button"
            class="reports-terminal-tab${activeLogsView === "errors" ? " active" : ""}"
            data-active-logs-view="errors"
          >Errors</button>
        </div>
      </div>
      ${logsState.error ? `<div class="reports-callout is-bad">${escapeHTML(logsState.error)}</div>` : ""}
      ${logs.length ? `
        <div class="reports-active-logs-list">
          ${logs.map((entry) => {
            const timestamp = formatCompactDateTime(entry && entry.timestampMs);
            const level = formatActiveLogLevel(entry && entry.level);
            const source = String(entry && entry.source || "engine").trim() || "engine";
            const context = stringifyActiveLogContext(entry && entry.context);
            const contextSummary = summarizeActiveLogContext(entry && entry.context);
            const message = String(entry && entry.message || "No message recorded.");
            return `
              <article class="reports-active-log-entry">
                <div class="reports-active-log-row">
                  <span class="reports-state-badge ${logLevelBadgeTone(level)}">${escapeHTML(level)}</span>
                  <span class="reports-active-log-time">${escapeHTML(timestamp || "Unknown time")}</span>
                  <strong class="reports-active-log-source">${escapeHTML(source)}</strong>
                  <span class="reports-active-log-message">${escapeHTML(message)}</span>
                  ${contextSummary ? `<span class="reports-active-log-context-summary">${escapeHTML(contextSummary)}</span>` : ""}
                  ${entry && entry.persisted ? '<span class="reports-launch-card-chip">saved</span>' : ""}
                </div>
                ${context ? `
                  <details class="reports-active-log-details">
                    <summary>View raw details</summary>
                    <pre class="reports-active-log-context">${escapeHTML(context)}</pre>
                  </details>
                ` : ""}
              </article>
            `;
          }).join("")}
        </div>
      ` : '<div class="reports-terminal-empty">No log entries recorded yet.</div>'}
    </div>
  `;
}

function buildReportsLaunchesMarkup() {
  if (!reportsTerminalState.launches.length) {
    return '<div class="reports-terminal-empty">No deployed launches found yet.</div>';
  }
  return `
    <div class="reports-launches-grid">
      ${reportsTerminalState.launches.map((launch) => {
        const title = launch.title || "Unknown launch";
        const symbol = launch.symbol || "LAUNCH";
        const activeFollowJob = activeFollowJobForTraceId(launch.traceId);
        const followState = activeFollowJob && activeFollowJob.state
          ? String(activeFollowJob.state)
          : String(launch.followJob && launch.followJob.state || "").trim();
        const imageMarkup = launch.imageUrl
          ? `<img src="${escapeHTML(launch.imageUrl)}" alt="${escapeHTML(title)}" class="reports-launch-card-image">`
          : `<span class="reports-launch-card-image-fallback">${escapeHTML(symbol.slice(0, 4).toUpperCase())}</span>`;
        const solscanUrl = launchSolscanUrl(launch);
        return `
          <article class="reports-launch-card">
            <div class="reports-launch-card-head">
              ${imageMarkup}
              <div>
                <strong class="reports-launch-card-title">${escapeHTML(title)}</strong>
                <span class="reports-launch-card-subtitle">${escapeHTML(symbol)}</span>
                <span class="reports-launch-card-subtitle">${escapeHTML(formatLaunchHistoryDisplayTime(launch.entry) || "Unknown time")}</span>
              </div>
            </div>
            ${launch.report && launch.report.mint ? `
              <button
                type="button"
                class="reports-launch-card-ca"
                data-copy-value="${escapeHTML(launch.report.mint)}"
                title="Copy contract address"
              >
                <span class="reports-launch-card-ca-label">CA</span>
                <span class="reports-launch-card-ca-value">${escapeHTML(launch.report.mint)}</span>
              </button>
            ` : ""}
            <div class="reports-launch-card-chip-row">
              <span class="reports-launch-card-chip">${escapeHTML(launch.report && launch.report.launchpad ? launch.report.launchpad : "launch")}</span>
              <span class="reports-launch-card-chip">${escapeHTML(launch.report && launch.report.mode ? launch.report.mode : "regular")}</span>
              ${launch.followJob && launch.followJob.quoteAsset ? `<span class="reports-launch-card-chip">${escapeHTML(launch.followJob.quoteAsset)}</span>` : ""}
              ${followState ? `<span class="reports-launch-card-chip">${escapeHTML(`follow ${followState}`)}</span>` : ""}
            </div>
            <div class="reports-launch-card-section">
              <div class="reports-launch-card-label">Settings</div>
              <div class="reports-launch-card-copy">${escapeHTML(buildLaunchHistorySettingsText(launch) || "No settings recorded.")}</div>
            </div>
            <div class="reports-launch-card-section">
              <div class="reports-launch-card-label">Buy Amounts</div>
              ${buildLaunchHistoryBuyAmountsMarkup(launch)}
            </div>
            <div class="reports-launch-card-footer">
              ${solscanUrl ? `<a class="reports-inline-link" href="${escapeHTML(solscanUrl)}" target="_blank" rel="noreferrer">Open in Solscan</a>` : '<span class="reports-launch-card-copy">No signature recorded.</span>'}
              <div class="reports-launch-card-actions">
                ${activeFollowJob ? `<button type="button" class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger" data-follow-cancel-trace-id="${escapeHTML(launch.traceId)}">Cancel</button>` : ""}
                <button type="button" class="preset-chip compact reports-terminal-chip" data-report-reuse-id="${escapeHTML(launch.id)}">Reuse</button>
                <button type="button" class="preset-chip compact reports-terminal-chip active" data-report-relaunch-id="${escapeHTML(launch.id)}">Relaunch</button>
              </div>
            </div>
          </article>
        `;
      }).join("")}
    </div>
  `;
}

function buildReportsTerminalOutputMarkup() {
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "launches") {
    return `<div class="reports-terminal-content">${buildReportsLaunchesMarkup()}</div>`;
  }
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "active-jobs") {
    return `<div class="reports-terminal-content">${buildReportsActiveJobsMarkup()}</div>`;
  }
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "active-logs") {
    return `<div class="reports-terminal-content">${buildActiveLogsMarkup()}</div>`;
  }
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
      <span class="reports-terminal-tabs-spacer"></span>
      <button
        type="button"
        class="reports-terminal-tab reports-terminal-tab-icon"
        data-benchmark-popout="1"
        title="Open benchmark popout"
        aria-label="Open benchmark popout"
        ${payload ? "" : "disabled"}
      >&#x29C9;</button>
    </div>
    <div class="reports-terminal-content">${content}</div>
  `;
}

function renderReportsTerminalOutput() {
  if (!reportsTerminalOutput) return;
  syncReportsTerminalChrome();
  const markup = buildReportsTerminalOutputMarkup();
  if (RenderUtils.setCachedHTML) {
    RenderUtils.setCachedHTML(renderCache, "reportsOutput", reportsTerminalOutput, markup);
  } else {
    reportsTerminalOutput.innerHTML = markup;
  }
  renderBenchmarksPopoutModal();
}

function renderReportsTerminalList() {
  if (!reportsTerminalList) return;
  syncReportsTerminalChrome();
  if (["launches", "active-jobs", "active-logs"].includes(normalizeReportsTerminalView(reportsTerminalState.view))) {
    if (RenderUtils.setCachedHTML) {
      RenderUtils.setCachedHTML(renderCache, "reportsList", reportsTerminalList, "");
    } else {
      reportsTerminalList.innerHTML = "";
    }
    return;
  }
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

async function loadReportsTerminalEntry(id, { showLoading = true } = {}) {
  if (!id || !reportsTerminalOutput) return;
  if (normalizeReportsTerminalView(reportsTerminalState.view) !== "transactions") return;
  const loadSerial = ++reportsTerminalLoadSerial;
  reportsTerminalState.activeId = id;
  if (showLoading) {
    reportsTerminalState.activePayload = null;
    reportsTerminalState.activeText = "Loading report...";
    renderReportsTerminalList();
    renderReportsTerminalOutput();
  } else {
    renderReportsTerminalList();
  }
  const url = `/api/reports/view?id=${encodeURIComponent(id)}`;
  const result = RequestUtils.fetchJsonLatest
    ? await RequestUtils.fetchJsonLatest("report-view", url, {}, requestStates.reportView)
    : null;
  if (loadSerial !== reportsTerminalLoadSerial) return;
  if (result && result.aborted) return;
  const response = result ? result.response : await fetch(url);
  const payload = result ? result.payload : await response.json();
  if (loadSerial !== reportsTerminalLoadSerial) return;
  if (result && !result.isLatest) return;
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to load report.");
  }
  const nextId = payload.entry && payload.entry.id ? payload.entry.id : id;
  const rawActivePayload = payload.payload && typeof payload.payload === "object" ? payload.payload : null;
  reportsTerminalState.activeId = nextId;
  captureFrozenBenchmarkSnapshot(nextId, rawActivePayload);
  reportsTerminalState.activePayload = applyFrozenBenchmarkSnapshot(nextId, rawActivePayload);
  reportsTerminalState.activeText = payload.text || "Report is empty.";
  renderReportsTerminalOutput();
  renderReportsTerminalList();
}

async function refreshActiveLogs({ showLoading = true } = {}) {
  if (!reportsTerminalOutput) return;
  const activeLogsView = normalizeActiveLogsView(reportsTerminalState.activeLogsView);
  if (showLoading) {
    reportsTerminalState.activeLogs.error = "";
    renderReportsTerminalOutput();
  }
  const url = `/api/logs?view=${encodeURIComponent(activeLogsView)}&limit=${encodeURIComponent(activeLogsView === "errors" ? "250" : "100")}`;
  const result = RequestUtils.fetchJsonLatest
    ? await RequestUtils.fetchJsonLatest("logs", url, {}, requestStates.logs)
    : null;
  if (result && result.aborted) return;
  const response = result ? result.response : await fetch(url);
  const payload = result ? result.payload : await response.json();
  if (result && !result.isLatest) return;
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || "Failed to load active logs.");
  }
  reportsTerminalState.activeLogs[activeLogsView] = Array.isArray(payload.logs) ? payload.logs : [];
  reportsTerminalState.activeLogs.error = "";
  reportsTerminalState.activeLogs.updatedAtMs = Date.now();
  renderReportsTerminalOutput();
}

async function refreshReportsTerminal({ preserveSelection = true, preferId = "", showLoading = true } = {}) {
  if (!reportsTerminalList || !reportsTerminalOutput) return;
  syncReportsTerminalChrome();
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "active-logs") {
    reportsTerminalState.activePayload = null;
    reportsTerminalState.activeBenchmarkReportId = "";
    reportsTerminalState.activeBenchmarkSnapshot = null;
    reportsTerminalState.activeText = "";
    renderReportsTerminalList();
    try {
      await refreshActiveLogs({ showLoading });
    } catch (error) {
      reportsTerminalState.activeLogs.error = error && error.message ? error.message : "Failed to load active logs.";
      renderReportsTerminalOutput();
    }
    return;
  }
  if (showLoading) {
    if (RenderUtils.setCachedHTML) {
      RenderUtils.setCachedHTML(renderCache, "reportsList", reportsTerminalList, '<div class="reports-terminal-empty">Loading reports...</div>');
    } else {
      reportsTerminalList.innerHTML = '<div class="reports-terminal-empty">Loading reports...</div>';
    }
  }
  const url = "/api/reports?sort=newest";
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
  reportsTerminalState.allEntries = Array.isArray(payload.reports) ? payload.reports : [];
  reportsTerminalState.entries = reportsTerminalState.allEntries
    .filter((entry) => ["send", "simulate", "build"].includes(String(entry && entry.action || "").trim().toLowerCase()))
    .slice(0, REPORTS_TERMINAL_ITEM_LIMIT);
  reportsTerminalState.sort = "newest";
  await refreshFollowJobs({ silent: true }).catch(() => {});
  const availableIds = new Set(reportsTerminalState.entries.map((entry) => entry.id));
  const nextId = preferId && availableIds.has(preferId)
    ? preferId
    : preserveSelection && reportsTerminalState.activeId && availableIds.has(reportsTerminalState.activeId)
      ? reportsTerminalState.activeId
      : reportsTerminalState.entries[0] && reportsTerminalState.entries[0].id
        ? reportsTerminalState.entries[0].id
        : "";
  reportsTerminalState.activeId = nextId;
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "launches") {
    await loadReportsTerminalLaunches();
    reportsTerminalState.activePayload = null;
    reportsTerminalState.activeBenchmarkReportId = "";
    reportsTerminalState.activeBenchmarkSnapshot = null;
    reportsTerminalState.activeText = "";
    renderReportsTerminalList();
    renderReportsTerminalOutput();
    return;
  }
  if (normalizeReportsTerminalView(reportsTerminalState.view) === "active-jobs") {
    reportsTerminalState.activePayload = null;
    reportsTerminalState.activeBenchmarkReportId = "";
    reportsTerminalState.activeBenchmarkSnapshot = null;
    reportsTerminalState.activeText = "";
    renderReportsTerminalList();
    renderReportsTerminalOutput();
    return;
  }
  renderReportsTerminalList();
  if (!nextId) {
    reportsTerminalState.activePayload = null;
    reportsTerminalState.activeBenchmarkReportId = "";
    reportsTerminalState.activeBenchmarkSnapshot = null;
    reportsTerminalState.activeText = "Run Build, Simulate, or Deploy to create persisted reports.";
    renderReportsTerminalOutput();
    return;
  }
  await loadReportsTerminalEntry(nextId, { showLoading });
}

function applyWalletSelectionFromLaunch(walletKey) {
  const nextKey = String(walletKey || "").trim();
  if (!nextKey || !latestWalletStatus || !Array.isArray(latestWalletStatus.wallets)) return;
  const exists = latestWalletStatus.wallets.some((wallet) => wallet.envKey === nextKey);
  if (!exists) return;
  setStoredSelectedWalletKey(nextKey);
  applySelectedWalletLocally(nextKey);
  refreshWalletStatus(true).catch(() => {});
}

function applyProvidersFromLaunch(launch) {
  const savedExecution = launch.followJob && launch.followJob.execution && typeof launch.followJob.execution === "object"
    ? launch.followJob.execution
    : {};
  if (providerSelect) providerSelect.value = launch.execution && launch.execution.provider ? launch.execution.provider : "helius-sender";
  if (buyProviderSelect) buyProviderSelect.value = savedExecution.buyProvider || (launch.execution && launch.execution.buyProvider) || "helius-sender";
  if (sellProviderSelect) sellProviderSelect.value = savedExecution.sellProvider || (launch.execution && launch.execution.sellProvider) || "helius-sender";
  if (creationPriorityInput) creationPriorityInput.value = savedExecution.priorityFeeSol || "";
  if (creationTipInput) creationTipInput.value = savedExecution.tipSol || "";
  setMevModeSelectValue(
    creationMevModeSelect,
    savedExecution.mevMode ?? savedExecution.mevProtect,
    defaultMevModeForProvider(providerSelect ? providerSelect.value : ""),
    providerSelect ? providerSelect.value : ""
  );
  if (creationAutoFeeInput) creationAutoFeeInput.checked = Boolean(savedExecution.autoGas);
  if (creationMaxFeeInput) creationMaxFeeInput.value = savedExecution.maxPriorityFeeSol || savedExecution.maxTipSol || "";
  if (buyPriorityFeeInput) buyPriorityFeeInput.value = savedExecution.buyPriorityFeeSol || "";
  if (buyTipInput) buyTipInput.value = savedExecution.buyTipSol || "";
  if (buySlippageInput) buySlippageInput.value = savedExecution.buySlippagePercent || "";
  setMevModeSelectValue(
    buyMevModeSelect,
    savedExecution.buyMevMode ?? savedExecution.buyMevProtect,
    defaultMevModeForProvider(buyProviderSelect ? buyProviderSelect.value : ""),
    buyProviderSelect ? buyProviderSelect.value : ""
  );
  if (buyAutoFeeInput) buyAutoFeeInput.checked = Boolean(savedExecution.buyAutoGas);
  if (buyMaxFeeInput) buyMaxFeeInput.value = savedExecution.buyMaxPriorityFeeSol || savedExecution.buyMaxTipSol || "";
  if (sellPriorityFeeInput) sellPriorityFeeInput.value = savedExecution.sellPriorityFeeSol || "";
  if (sellTipInput) sellTipInput.value = savedExecution.sellTipSol || "";
  if (sellSlippageInput) sellSlippageInput.value = savedExecution.sellSlippagePercent || "";
  setMevModeSelectValue(
    sellMevModeSelect,
    savedExecution.sellMevMode ?? savedExecution.sellMevProtect,
    defaultMevModeForProvider(sellProviderSelect ? sellProviderSelect.value : ""),
    sellProviderSelect ? sellProviderSelect.value : ""
  );
  if (sellAutoFeeInput) sellAutoFeeInput.checked = Boolean(savedExecution.sellAutoGas);
  if (sellMaxFeeInput) sellMaxFeeInput.value = savedExecution.sellMaxPriorityFeeSol || savedExecution.sellMaxTipSol || "";
  syncSettingsCapabilities();
  syncActivePresetFromInputs();
}

function buildSniperWalletStateFromLaunch(launch) {
  const snipes = launch.followLaunch && Array.isArray(launch.followLaunch.snipes) ? launch.followLaunch.snipes : [];
  return snipes.reduce((accumulator, entry) => {
    if (!entry || !entry.walletEnvKey) return accumulator;
    accumulator[entry.walletEnvKey] = {
      selected: entry.enabled !== false,
      amountSol: entry.buyAmountSol || "",
      triggerMode: entry.targetBlockOffset != null
        ? "block-offset"
        : (entry.submitWithLaunch ? "same-time" : "on-submit"),
      submitDelayMs: entry.submitDelayMs || 0,
      targetBlockOffset: entry.targetBlockOffset,
      retryOnce: Boolean(entry.retryOnFailure),
    };
    return accumulator;
  }, {});
}

async function applyLaunchHistoryEntryToForm(id) {
  const launch = getLaunchHistoryEntry(id);
  if (!launch) return null;
  const metadata = launch.metadata || await fetchLaunchMetadataSummary(launch.metadataUri);
  applyWalletSelectionFromLaunch(launch.selectedWalletKey);
  const nextLaunchpad = launch.report && launch.report.launchpad ? launch.report.launchpad : "pump";
  applyImportedLaunchContext({
    launchpad: nextLaunchpad,
    mode: launch.report && launch.report.mode ? launch.report.mode : defaultLaunchModeForLaunchpad(nextLaunchpad),
    quoteAsset: launch.quoteAsset || "sol",
    routes: {
      feeSharingRecipients: launch.feeSharingRecipients || [],
      agentFeeRecipients: launch.agentFeeRecipients || [],
      creatorFee: launch.creatorFee || null,
    },
  });
  if (launch.report && launch.report.launchpad === "bagsapp" && launch.bags) {
    setBagsIdentityStateInputs({
      mode: launch.bags.identityMode === "linked" ? "linked" : "wallet-only",
      agentUsername: launch.bags.agentUsername || "",
      verifiedWallet: launch.bags.identityVerifiedWallet || "",
      verified: launch.bags.identityMode === "linked" && Boolean(launch.bags.identityVerifiedWallet),
    });
  } else {
    setBagsIdentityStateInputs({
      mode: "wallet-only",
      agentUsername: "",
      authToken: "",
      verifiedWallet: "",
      verified: false,
    });
  }
  syncLaunchpadModeOptions();
  syncBonkQuoteAssetUI();
  syncBagsIdentityUI();
  if (launch.report && launch.report.launchpad === "bagsapp") {
    refreshBagsIdentityStatus().catch(() => {});
  }

  if (nameInput) nameInput.value = metadata && metadata.name ? metadata.name : (launch.title || "");
  if (symbolInput) {
    syncingTickerFromName = true;
    symbolInput.value = formatTickerValue(metadata && metadata.symbol ? metadata.symbol : (launch.symbol || ""));
    syncingTickerFromName = false;
    tickerManuallyEdited = Boolean(String(metadata && metadata.symbol ? metadata.symbol : "").trim());
    tickerClearedForManualEntry = false;
  }
  if (descriptionInput) {
    descriptionInput.value = metadata && metadata.description ? metadata.description : "";
    toggleDescriptionDisclosure(Boolean(descriptionInput.value.trim()));
    updateDescriptionDisclosure();
  }
  if (websiteInput) websiteInput.value = metadata && metadata.website ? metadata.website : "";
  if (twitterInput) twitterInput.value = metadata && metadata.twitter ? metadata.twitter : "";
  if (telegramInput) telegramInput.value = metadata && metadata.telegram ? metadata.telegram : "";

  uploadedImage = null;
  imageLibraryState.activeImageId = "";
  clearMetadataUploadCache({ clearInput: true });
  if (metadataUri) metadataUri.value = launch.metadataUri || "";
  setImagePreview(launch.imageUrl || "");
  imageStatus.textContent = launch.metadataUri ? "Restored image from saved launch metadata." : "";
  imagePath.textContent = "";

  const devBuy = launch.devBuy || { mode: "sol", amount: "" };
  setDevBuyHiddenState(devBuy.mode, devBuy.amount);
  syncingDevBuyInputs = true;
  if (devBuySolInput) devBuySolInput.value = devBuy.mode === "sol" ? devBuy.amount : "";
  if (devBuyPercentInput) {
    devBuyPercentInput.value = devBuy.mode === "tokens"
      ? tokenAmountToPercent(devBuy.amount)
      : "";
  }
  syncingDevBuyInputs = false;

  const sniperWallets = buildSniperWalletStateFromLaunch(launch);
  sniperFeature.setState({
    enabled: Object.values(sniperWallets).some((entry) => entry && entry.selected),
    wallets: sniperWallets,
  });
  applySniperStateToForm();
  renderSniperUI();

  const devAutoSell = launch.followLaunch && launch.followLaunch.devAutoSell ? launch.followLaunch.devAutoSell : null;
  if (autoSellEnabledInput) autoSellEnabledInput.checked = Boolean(devAutoSell && devAutoSell.enabled);
  setNamedValue(
    "automaticDevSellTriggerFamily",
    devAutoSell && devAutoSell.marketCap && devAutoSell.marketCap.threshold ? "market-cap" : "time"
  );
  setNamedValue("automaticDevSellPercent", String(devAutoSell && devAutoSell.percent != null ? devAutoSell.percent : 100));
  setNamedValue("automaticDevSellTriggerMode", devAutoSell && devAutoSell.targetBlockOffset != null
    ? "block-offset"
    : (devAutoSell && (devAutoSell.delayMs || 0) > 0 ? "submit-delay" : "block-offset"));
  setNamedValue("automaticDevSellDelayMs", String(devAutoSell && devAutoSell.delayMs != null ? devAutoSell.delayMs : 0));
  setNamedValue("automaticDevSellBlockOffset", String(devAutoSell && devAutoSell.targetBlockOffset != null ? devAutoSell.targetBlockOffset : 0));
  setNamedChecked(
    "automaticDevSellMarketCapEnabled",
    Boolean(devAutoSell && devAutoSell.marketCap && devAutoSell.marketCap.threshold)
  );
  setNamedValue(
    "automaticDevSellMarketCapThreshold",
    String(devAutoSell && devAutoSell.marketCap && devAutoSell.marketCap.threshold ? devAutoSell.marketCap.threshold : "")
  );
  setNamedValue(
    "automaticDevSellMarketCapScanTimeoutSeconds",
    String(
      devAutoSell
        && devAutoSell.marketCap
        && (devAutoSell.marketCap.scanTimeoutSeconds != null || devAutoSell.marketCap.scanTimeoutMinutes != null)
        ? (devAutoSell.marketCap.scanTimeoutSeconds != null
          ? devAutoSell.marketCap.scanTimeoutSeconds
          : devAutoSell.marketCap.scanTimeoutMinutes * 60)
        : 30
    )
  );
  setNamedValue(
    "automaticDevSellMarketCapTimeoutAction",
    String(devAutoSell && devAutoSell.marketCap && devAutoSell.marketCap.timeoutAction ? devAutoSell.marketCap.timeoutAction : "stop")
  );
  syncDevAutoSellUI();

  applyProvidersFromLaunch(launch);
  updateTokenFieldCounts();
  clearValidationErrors();
  Object.keys(fieldValidators).forEach((name) => setFieldError(name, ""));
  queueQuoteUpdate();
  return launch;
}

async function reuseFromHistory(id) {
  const launch = await applyLaunchHistoryEntryToForm(id);
  if (!launch) return;
  setStatusLabel("Launch loaded");
  metaNode.textContent = `Restored ${launch.title || "saved launch"} into the form.`;
}

async function relaunchFromHistory(id) {
  const launch = await applyLaunchHistoryEntryToForm(id);
  if (!launch) return;
  setStatusLabel("Relaunching");
  metaNode.textContent = `Re-launching ${launch.title || "saved launch"} with saved settings.`;
  await run("deploy");
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
    if (themeToggleSunIcon) themeToggleSunIcon.hidden = normalized === "light";
    if (themeToggleMoonIcon) themeToggleMoonIcon.hidden = normalized !== "light";
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
  scheduleLiveSyncBroadcast({ immediate: true });
}

function setBootOverlayMessage(title, note) {
  if (!bootOverlay) return;
  if (title != null && title !== "") {
    const titleNode = bootOverlay.querySelector(".boot-overlay-title");
    if (titleNode) titleNode.textContent = title;
  }
  if (note != null) {
    const noteNode = bootOverlay.querySelector(".boot-overlay-note");
    if (noteNode) noteNode.textContent = note;
  }
}

function completeInitialBoot() {
  if (window.__launchdeckBootFallback) {
    window.clearTimeout(window.__launchdeckBootFallback);
    window.__launchdeckBootFallback = null;
  }
  setBootOverlayMessage("LaunchDeck", "Preparing wallets, settings, caches, and runtime status.");
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
  popoutUrl.searchParams.delete("popout");
  popoutUrl.searchParams.delete("output");
  popoutUrl.searchParams.delete("reports");
  dispatchLiveSyncPayload(buildLiveSyncPayload());
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
    POPOUT_WINDOW_NAME,
    `popup=yes,width=${width},height=${height},menubar=no,toolbar=no,location=no,status=no,resizable=yes,scrollbars=yes`,
  );
}

form.querySelectorAll('input[name="mode"]').forEach((node) => {
  node.addEventListener("change", () => {
    setStoredLaunchMode(getMode());
    updateModeVisibility();
    warmDevBuyQuoteCache();
  });
});
launchpadInputs.forEach((input) => {
  input.addEventListener("change", () => {
    if (!input.checked) return;
    setStoredLaunchpad(input.value);
    setLaunchpad(input.value, { resetMode: true, persistMode: true });
    applyLaunchpadTokenMetadata();
    warmDevBuyQuoteCache();
    if (input.value === "bagsapp") {
      refreshBagsIdentityStatus().catch(() => {});
    }
  });
});
if (bonkQuoteAssetToggle) {
  bonkQuoteAssetToggle.addEventListener("click", () => {
    const asset = getQuoteAsset() === "usd1" ? "sol" : "usd1";
    if (bonkQuoteAssetInput) bonkQuoteAssetInput.value = asset;
    setStoredBonkQuoteAsset(asset);
    syncBonkQuoteAssetUI();
    warmDevBuyQuoteCache();
    queueQuoteUpdate();
  });
}
if (bagsIdentityButton) {
  bagsIdentityButton.addEventListener("click", async () => {
    if (getLaunchpad() !== "bagsapp") return;
    if (getBagsIdentityMode() === "linked") {
      try {
        await fetch("/api/bags/identity/clear", { method: "POST" });
      } catch (_error) {
        // Ignore backend clear errors and still reset the frontend state.
      }
      setBagsIdentityStateInputs({
        mode: "wallet-only",
        verified: false,
        agentUsername: "",
        authToken: "",
        verifiedWallet: "",
      });
      syncBagsIdentityUI();
      return;
    }
    showBagsIdentityModal();
  });
}
if (bagsIdentityClose) {
  bagsIdentityClose.addEventListener("click", () => hideBagsIdentityModal());
}
if (bagsIdentityCancel) {
  bagsIdentityCancel.addEventListener("click", () => hideBagsIdentityModal());
}
if (bagsIdentityInitButton) {
  bagsIdentityInitButton.addEventListener("click", async () => {
    setBagsIdentityError("");
    try {
      await initBagsIdentityVerification();
    } catch (error) {
      setBagsIdentityError(error.message || "Failed to initialize Bags identity.");
    }
  });
}
if (bagsIdentityVerifyButton) {
  bagsIdentityVerifyButton.addEventListener("click", async () => {
    setBagsIdentityError("");
    try {
      await verifyBagsIdentity();
    } catch (error) {
      setBagsIdentityError(error.message || "Failed to verify Bags identity.");
    }
  });
}
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
  if (isHelloMoonProvider(getProvider()) && creationMevModeSelect) {
    setMevModeSelectValue(creationMevModeSelect, "reduced", "reduced", getProvider());
  }
  syncActivePresetFromInputs();
  updateJitoVisibility();
  validateProviderFeeFields("creation");
});
if (buyProviderSelect) buyProviderSelect.addEventListener("change", () => {
  if (isHelloMoonProvider(getBuyProvider()) && buyMevModeSelect) {
    setMevModeSelectValue(buyMevModeSelect, "reduced", "reduced", getBuyProvider());
  }
  ensureStandardRpcSlippageDefault(buySlippageInput, getBuyProvider());
  syncActivePresetFromInputs();
  validateProviderFeeFields("buy");
});
if (sellProviderSelect) sellProviderSelect.addEventListener("change", () => {
  if (isHelloMoonProvider(getSellProvider()) && sellMevModeSelect) {
    setMevModeSelectValue(sellMevModeSelect, "reduced", "reduced", getSellProvider());
  }
  ensureStandardRpcSlippageDefault(sellSlippageInput, getSellProvider());
  syncActivePresetFromInputs();
  validateProviderFeeFields("sell");
});
feeSplitPill.addEventListener("click", () => {
  const mode = getMode();
  if (mode !== "regular" && mode !== "agent-custom" && !mode.startsWith("bags-")) return;
  if (!mode.startsWith("bags-") && !feeSplitEnabled.checked) {
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
  walletRefreshButton.addEventListener("click", async (event) => {
    event.preventDefault();
    event.stopPropagation();
    walletRefreshButton.disabled = true;
    try {
      await refreshWalletStatus(true, true);
      if (getLaunchpad() === "bagsapp") {
        await refreshBagsIdentityStatus().catch(() => {});
      }
    } finally {
      walletRefreshButton.disabled = false;
    }
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
    if (getLaunchpad() === "bagsapp") refreshBagsIdentityStatus().catch(() => {});
  });
}
walletSelect.addEventListener("change", () => {
  const nextKey = selectedWalletKey();
  setStoredSelectedWalletKey(nextKey);
  applySelectedWalletLocally(nextKey);
  refreshWalletStatus(true);
  if (getLaunchpad() === "bagsapp") refreshBagsIdentityStatus().catch(() => {});
});
document.addEventListener("click", (event) => {
  if (!walletDropdown || walletDropdown.hidden) return;
  const target = event.target;
  if (walletBox && walletBox.contains(target)) return;
  setWalletDropdownOpen(false);
});

feeSplitAdd.addEventListener("click", () => {
  clearFeeSplitRestoreState();
  if (getFeeSplitRows().length >= MAX_FEE_SPLIT_RECIPIENTS) return;
  feeSplitList.appendChild(createFeeSplitRow({ type: "wallet", sharePercent: "" }));
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
  setFeeSplitModalError("");
});

feeSplitReset.addEventListener("click", () => {
  clearFeeSplitRestoreState();
  getFeeSplitRows().forEach((row) => {
    row.querySelector(".recipient-share").value = "0";
    row.querySelector(".recipient-slider").value = "0";
  });
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
  setFeeSplitModalError("");
});

feeSplitEven.addEventListener("click", () => {
  clearFeeSplitRestoreState();
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
  setFeeSplitModalError("");
});

if (feeSplitClearAll) {
  feeSplitClearAll.addEventListener("click", () => {
    if (feeSplitClearAllRestoreSnapshot) {
      applyFeeSplitDraft(feeSplitClearAllRestoreSnapshot, { persist: false });
      syncFeeSplitTotals();
      setStoredFeeSplitDraft(serializeFeeSplitDraft());
      clearFeeSplitRestoreState();
      setFeeSplitModalError("");
      return;
    }
    feeSplitClearAllRestoreSnapshot = normalizeFeeSplitDraft(serializeFeeSplitDraft());
    applyFeeSplitDraft(feeSplitClearAllDraft(), { persist: false });
    syncFeeSplitTotals();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    updateFeeSplitClearAllButton();
    setFeeSplitModalError("");
  });
}

feeSplitList.addEventListener("click", (event) => {
  const lockToggle = event.target.closest(".recipient-lock-toggle");
  if (lockToggle) {
    clearFeeSplitRestoreState();
    const row = lockToggle.closest(".fee-split-row");
    setRecipientTargetLocked(row, row.dataset.targetLocked !== "true");
    syncFeeSplitTotals();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    setFeeSplitModalError("");
    return;
  }
  const tab = event.target.closest(".recipient-type-tab");
  if (tab) {
    clearFeeSplitRestoreState();
    updateFeeSplitRowType(tab.closest(".fee-split-row"), tab.dataset.type);
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    setFeeSplitModalError("");
    return;
  }
  const removeButton = event.target.closest(".recipient-remove");
  if (removeButton) {
    clearFeeSplitRestoreState();
    removeButton.closest(".fee-split-row").remove();
    ensureFeeSplitDefaultRow();
    syncFeeSplitTotals();
    setStoredFeeSplitDraft(serializeFeeSplitDraft());
    setFeeSplitModalError("");
  }
});

feeSplitList.addEventListener("input", (event) => {
  const row = event.target.closest(".fee-split-row");
  if (!row) return;
  clearFeeSplitRestoreState();
  if (event.target.classList.contains("recipient-target")) {
    event.target.setCustomValidity("");
  }
  if (event.target.classList.contains("recipient-target") && row.dataset.type === "github") {
    delete row.dataset.githubUserId;
  }
  if (event.target.classList.contains("recipient-slider")) {
    row.querySelector(".recipient-share").value = event.target.value;
  }
  if (event.target.classList.contains("recipient-share")) {
    row.querySelector(".recipient-slider").value = event.target.value || "0";
  }
  syncFeeSplitTotals();
  setStoredFeeSplitDraft(serializeFeeSplitDraft());
  setFeeSplitModalError("");
});

agentSplitAdd.addEventListener("click", () => {
  clearAgentSplitRestoreState();
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
  clearAgentSplitRestoreState();
  getAgentSplitRows().forEach((row) => {
    row.querySelector(".recipient-share").value = "0";
    row.querySelector(".recipient-slider").value = "0";
  });
  syncAgentSplitTotals();
  setStoredAgentSplitDraft(serializeAgentSplitDraft());
  setAgentSplitModalError("");
});

agentSplitEven.addEventListener("click", () => {
  clearAgentSplitRestoreState();
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

if (agentSplitClearAll) {
  agentSplitClearAll.addEventListener("click", () => {
    if (agentSplitClearAllRestoreSnapshot) {
      applyAgentSplitDraft(agentSplitClearAllRestoreSnapshot, { persist: false });
      syncAgentSplitTotals();
      setStoredAgentSplitDraft(serializeAgentSplitDraft());
      clearAgentSplitRestoreState();
      setAgentSplitModalError("");
      return;
    }
    agentSplitClearAllRestoreSnapshot = normalizeAgentSplitDraft(serializeAgentSplitDraft());
    applyAgentSplitDraft(agentSplitClearAllDraft(), { persist: false });
    syncAgentSplitTotals();
    setStoredAgentSplitDraft(serializeAgentSplitDraft());
    updateAgentSplitClearAllButton();
    setAgentSplitModalError("");
  });
}

agentSplitList.addEventListener("click", (event) => {
  const lockToggle = event.target.closest(".recipient-lock-toggle");
  if (lockToggle) {
    clearAgentSplitRestoreState();
    const row = lockToggle.closest(".fee-split-row");
    setRecipientTargetLocked(row, row.dataset.targetLocked !== "true");
    syncAgentSplitTotals();
    setStoredAgentSplitDraft(serializeAgentSplitDraft());
    setAgentSplitModalError("");
    return;
  }
  const tab = event.target.closest(".recipient-type-tab");
  if (tab && tab.dataset.type) {
    clearAgentSplitRestoreState();
    updateFeeSplitRowType(tab.closest(".fee-split-row"), tab.dataset.type);
    syncAgentSplitTotals();
    setStoredAgentSplitDraft(serializeAgentSplitDraft());
    setAgentSplitModalError("");
    return;
  }
  const removeButton = event.target.closest(".recipient-remove");
  if (removeButton) {
    clearAgentSplitRestoreState();
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
  clearAgentSplitRestoreState();
  if (event.target.classList.contains("recipient-target")) {
    event.target.setCustomValidity("");
  }
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
if (benchmarksPopoutClose) benchmarksPopoutClose.addEventListener("click", hideBenchmarksPopoutModal);
if (benchmarksPopoutModal) {
  benchmarksPopoutModal.addEventListener("click", (event) => {
    if (event.target === benchmarksPopoutModal) {
      hideBenchmarksPopoutModal();
    }
  });
}
if (settingsCancel) settingsCancel.addEventListener("click", () => hideSettingsModal("cancel"));
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
  [creationAutoFeeButton, creationAutoFeeInput],
  [buyAutoFeeButton, buyAutoFeeInput],
  [sellAutoFeeButton, sellAutoFeeInput],
].forEach(([button, input]) => {
  if (!button || !input) return;
  button.addEventListener("click", () => {
    if (button.disabled) return;
    input.checked = !input.checked;
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
});
[
  creationTipInput,
  creationPriorityInput,
  creationMevModeSelect,
  creationAutoFeeInput,
  creationMaxFeeInput,
  buyPriorityFeeInput,
  buyTipInput,
  buySlippageInput,
  buyMevModeSelect,
  buyAutoFeeInput,
  buyMaxFeeInput,
  sellPriorityFeeInput,
  sellTipInput,
  sellSlippageInput,
  sellMevModeSelect,
  sellAutoFeeInput,
  sellMaxFeeInput,
].forEach((input) => {
  if (!input) return;
  const eventName = input.tagName === "SELECT" || input.type === "checkbox" ? "change" : "input";
  input.addEventListener(eventName, () => {
    syncActivePresetFromInputs();
    syncSettingsCapabilities();
    if (input.name) validateFieldByName(input.name);
    if (input === creationAutoFeeInput) validateProviderFeeFields("creation");
    if (input === buyAutoFeeInput) validateProviderFeeFields("buy");
    if (input === sellAutoFeeInput) validateProviderFeeFields("sell");
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
if (feeSplitClose) feeSplitClose.addEventListener("click", attemptCloseFeeSplitModal);
if (feeSplitSave) feeSplitSave.addEventListener("click", attemptCloseFeeSplitModal);
if (feeSplitDisable) {
  feeSplitDisable.addEventListener("click", cancelFeeSplitModal);
}
if (feeSplitModal) {
  feeSplitModal.addEventListener("click", (event) => {
    if (event.target === feeSplitModal) attemptCloseFeeSplitModal();
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
  vanitySave.addEventListener("click", async () => {
    const nextValue = vanityPrivateKeyText ? vanityPrivateKeyText.value.trim() : "";
    if (vanityModalError) vanityModalError.textContent = "";
    try {
      const payload = await validateVanityPrivateKey(nextValue);
      applyVanityValue(
        payload && payload.normalizedPrivateKey ? payload.normalizedPrivateKey : nextValue,
        { publicKey: payload && payload.publicKey ? payload.publicKey : "" },
      );
      hideVanityModal();
    } catch (error) {
      if (vanityModalError) {
        vanityModalError.textContent = error && error.message ? error.message : "Invalid vanity private key.";
      }
    }
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
  vampContractInput.addEventListener("input", () => {
    if (vampError) vampError.textContent = "";
    scheduleVampAutoImport();
  });
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
hydrateDevAutoSellState();
hydrateModeActionState();
updateTokenFieldCounts();
updateDescriptionDisclosure();
setSettingsLoadingState(true);
renderBackendRegionSummary(null);
renderSniperUI();
renderReportsTerminalOutput();
Promise.resolve(bootstrapApp())
  .then(() => {
    startRuntimeStatusRefreshLoop();
    enableLiveSync();
    if (isReportsTerminalCurrentlyVisible()) {
      refreshReportsTerminal({
        preserveSelection: true,
        preferId: reportsTerminalState.activeId,
        showLoading: false,
      }).catch(() => {});
    }
    completeInitialBoot();
  })
  .catch((error) => {
    if (walletBalance) walletBalance.textContent = "-";
    metaNode.textContent = error.message;
    if (bootOverlay) {
      const titleNode = bootOverlay.querySelector(".boot-overlay-title");
      const noteNode = bootOverlay.querySelector(".boot-overlay-note");
      if (titleNode) titleNode.textContent = "LaunchDeck failed to load";
      if (noteNode) noteNode.textContent = error.message || "Refresh the page and check the backend runtime.";
    }
  });

document.addEventListener("input", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  // Ignore programmatic sync/update events so they do not keep idle warm alive.
  if (!event.isTrusted) return;
  if (target.matches("input, textarea, select") || target.isContentEditable) {
    queueWarmActivity();
    scheduleLiveSyncBroadcast();
  }
}, true);

document.addEventListener("change", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  // Only count real operator edits as warm activity.
  if (!event.isTrusted) return;
  if (target.matches("input, textarea, select")) {
    queueWarmActivity({ immediate: true });
    scheduleLiveSyncBroadcast({ immediate: true });
  }
}, true);

document.addEventListener("click", (event) => {
  const target = event.target instanceof Element
    ? event.target.closest("button, .button, [data-action]")
    : null;
  if (target) queueWarmActivity({ immediate: true });
}, true);

window.addEventListener("focus", () => {
  queueWarmActivity({ immediate: true });
});

window.addEventListener("pagehide", () => {
  dispatchLiveSyncPayload(buildLiveSyncPayload());
});

document.addEventListener("visibilitychange", () => {
  if (document.visibilityState !== "visible") return;
  queueWarmActivity({ immediate: true });
  refreshRuntimeStatus().catch(() => {});
});

if (liveSyncChannel) {
  liveSyncChannel.addEventListener("message", (event) => {
    applyIncomingLiveSyncPayload(event && event.data);
  });
}

window.addEventListener("storage", (event) => {
  if (event.key !== LIVE_SYNC_STORAGE_KEY || !event.newValue) return;
  try {
    applyIncomingLiveSyncPayload(JSON.parse(event.newValue));
  } catch (_error) {
    // Ignore malformed storage sync payloads.
  }
});
