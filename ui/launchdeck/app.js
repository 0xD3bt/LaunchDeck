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
const feeSplitPillTitle = document.getElementById("fee-split-pill-title");
const feeSplitPillProgress = document.getElementById("fee-split-pill-progress");
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
const agentSplitTitle = document.getElementById("agent-split-title");
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
const feeSplitTitle = document.getElementById("fee-split-title");
const feeSplitIntro = document.getElementById("fee-split-intro");
const feeSplitRecipientsCopy = document.getElementById("fee-split-recipients-copy");
const bagsFeeSplitSummary = document.getElementById("bags-fee-split-summary");
const feeSplitSummaryPrimaryLabel = document.getElementById("fee-split-summary-primary-label");
const feeSplitSummarySecondaryLabel = document.getElementById("fee-split-summary-secondary-label");
const bagsFeeSplitCreatorShare = document.getElementById("bags-fee-split-creator-share");
const bagsFeeSplitSharedShare = document.getElementById("bags-fee-split-shared-share");
const bagsFeeSplitValidationCopy = document.getElementById("bags-fee-split-validation-copy");
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
const modeSniperProgress = document.getElementById("mode-sniper-progress");
const modeVanityButton = document.getElementById("mode-vanity-button");
let vanityDerivedAddressPill = document.getElementById("mode-vanity-address");
let vanityDerivedPublicKey = "";
const devAutoSellButton = document.getElementById("dev-auto-sell-button");
const devAutoSellPopover = devAutoSellButton ? devAutoSellButton.closest(".auto-sell-popover") : null;
const autoSellButtonProgress = document.getElementById("auto-sell-button-progress");
const devAutoSellPanel = document.getElementById("dev-auto-sell-panel");
const autoSellEnabledInput = document.getElementById("auto-sell-enabled-input");
const autoSellToggleState = document.getElementById("auto-sell-toggle-state");
const autoSellTriggerFamilyValue = document.getElementById("auto-sell-trigger-family-value");
const autoSellTriggerValue = document.getElementById("auto-sell-trigger-value");
const autoSellTimeSettings = document.getElementById("auto-sell-time-settings");
const autoSellTriggerFamilyButtons = Array.from(document.querySelectorAll("[data-auto-sell-trigger-family]"));
const autoSellDelaySlider = document.getElementById("auto-sell-delay-slider");
const autoSellDelayInput = document.getElementById("auto-sell-delay-input");
const autoSellDelayControl = document.getElementById("auto-sell-delay-control");
const autoSellPercentSlider = document.getElementById("auto-sell-percent-slider");
const autoSellPercentInput = document.getElementById("auto-sell-percent-input");
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
const autoSellSniperEnabledInput = document.getElementById("auto-sell-sniper-enabled-input");
const autoSellSniperToggleState = document.getElementById("auto-sell-sniper-toggle-state");
const autoSellSniperWalletList = document.getElementById("auto-sell-sniper-wallet-list");
const launchSurfaceModeSection = form ? form.querySelector(".launch-surface-mode") : null;
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
const SNIPER_DRAFT_STORAGE_PREFIX = "launchdeck.sniperDraft";
const IMAGE_LAYOUT_COMPACT_STORAGE_KEY = "launchdeck.imageLayoutCompact";
const SELECTED_MODE_STORAGE_KEY = "launchdeck.selectedMode";
const SELECTED_BONK_QUOTE_ASSET_STORAGE_KEY = "launchdeck.bonkQuoteAsset";
const FEE_SPLIT_DRAFT_STORAGE_KEY = "launchdeck.feeSplitDraft.v1";
const AGENT_SPLIT_DRAFT_STORAGE_KEY = "launchdeck.agentSplitDraft.v1";
const AUTO_SELL_DRAFT_STORAGE_KEY = "launchdeck.autoSellDraft.v1";
const AUTO_SELL_DRAFT_STORAGE_PREFIX = "launchdeck.autoSellDraft";
const TICKER_CAPS_STORAGE_KEY = "launchdeck.tickerCapsEnabled";
const POPOUT_FORM_WIDTH = 532;
const POPOUT_REPORTS_WIDTH = 560;
const POPOUT_WORKSPACE_GAP = 12;
const POPOUT_WINDOW_NAME = "launchdeck-popout";
const pageSearchParams = new URLSearchParams(window.location.search);
const hasLegacyPopoutQuery = pageSearchParams.get("popout") === "1";
const isPopoutMode = window.name === POPOUT_WINDOW_NAME || hasLegacyPopoutQuery;
let popoutAutosizeFrame = 0;
const LIVE_SYNC_SOURCE_ID = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
const RequestUtils = window.LaunchDeckRequestUtils || {};
const RenderUtils = window.LaunchDeckRenderUtils || {};
const FormDomainModule = window.LaunchDeckFormDomain || {};
const QuotePreviewDomainModule = window.LaunchDeckQuotePreviewDomain || {};
const RuntimeActionsModule = window.LaunchDeckRuntimeActions || {};
const FeeRoutingModule = window.LaunchDeckFeeRouting || {};
const SplitEditorsDomainModule = window.LaunchDeckSplitEditorsDomain || {};
const SettingsDomainModule = window.LaunchDeckSettingsDomain || {};
const WalletRuntimeDomainModule = window.LaunchDeckWalletRuntimeDomain || {};
const LiveSyncModule = window.LaunchDeckLiveSync || {};
const LocalBindersModule = window.LaunchDeckLocalBinders || {};
const ImageMetadataDomainModule = window.LaunchDeckImageMetadataDomain || {};
let settingsDomain = null;
let splitEditorsDomain = null;
let walletRuntimeDomain = null;
let imageMetadataDomain = null;
let quotePreviewDomain = null;
let reportsPresenters = null;
let reportsHistory = null;
let reportsFeature = null;
let liveSyncSupport = null;
const DEFAULT_LAUNCHPAD_TOKEN_METADATA = Object.freeze({
  nameMaxLength: 32,
  symbolMaxLength: 10,
});
const STANDARD_RPC_SLIPPAGE_DEFAULT = "20";

function readEarlyLiveSyncSnapshot() {
  return typeof LiveSyncModule.readEarlyLiveSyncSnapshot === "function"
    ? LiveSyncModule.readEarlyLiveSyncSnapshot()
    : null;
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
  document.title = "Trench.Tools - LaunchDeck Popout";
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
let importedCreatorFeeState = {
  mode: "",
  address: "",
  githubUsername: "",
  githubUserId: "",
};
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
const STARTUP_WARM_CACHE_STORAGE_KEY = "launchdeck.startupWarmCache.v2";
const LEGACY_STARTUP_WARM_CACHE_STORAGE_KEY = "launchdeck.startupWarmCache.v1";
const STARTUP_WARM_CACHE_SCHEMA_VERSION = 2;
const PREVIEW_INPUTS_STORAGE_KEY = "launchdeck.previewInputs.v1";
const PREVIEW_INPUTS_SCHEMA_VERSION = 1;
const WALLET_STATUS_LAST_REFRESH_STORAGE_KEY = "launchdeck.walletStatusLastRefreshAtMs";
let walletStatusRefreshIntervalMs = 30000;
const RUNTIME_STATUS_REFRESH_INTERVAL_MS = 15000;
const STARTUP_WARM_CACHE_MAX_AGE_MS = RUNTIME_STATUS_REFRESH_INTERVAL_MS;
const WARM_ACTIVITY_DEBOUNCE_MS = 1000;
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
  bagsFeeRecipientLookup: RequestUtils.createLatestRequestState ? RequestUtils.createLatestRequestState() : { serial: 0, controller: null, debounceTimer: null },
};
const renderCache = {
  walletDropdown: "",
  platformRuntimeIndicators: "",
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
let lastDevBuyEditSource = "sol";
let previewInputsState = null;
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
const BAGS_FEE_SPLIT_VISIBLE_CARD_COUNT = 5;
const BAGS_FEE_SPLIT_VIEWPORT_BUFFER_PX = 4;
const SNIPER_EXECUTION_RESERVE_SOL = 0.005;
const SNIPER_BALANCE_PRESETS = [
  { label: "Max", ratio: 1 },
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
settingsDomain = SettingsDomainModule.create ? SettingsDomainModule.create({
  elements: {
    topPresetChipBar,
    settingsPresetChipBar,
    presetEditToggle,
    devBuyQuickButtons,
    changeDevBuyPresetsButton,
    cancelDevBuyPresetsButton,
    saveDevBuyPresetsButton,
    providerSelect,
    creationTipInput,
    creationPriorityInput,
    creationMevModeSelect,
    creationAutoFeeInput,
    creationAutoFeeButton,
    creationMaxFeeInput,
    buyProviderSelect,
    buyPriorityFeeInput,
    buyTipInput,
    buySlippageInput,
    buyMevModeSelect,
    buyAutoFeeInput,
    buyAutoFeeButton,
    buyMaxFeeInput,
    buyHelloMoonMevWarning,
    buyStandardRpcWarning,
    sellProviderSelect,
    sellPriorityFeeInput,
    sellTipInput,
    sellSlippageInput,
    sellMevModeSelect,
    sellAutoFeeInput,
    sellAutoFeeButton,
    sellMaxFeeInput,
    sellHelloMoonMevWarning,
    sellStandardRpcWarning,
    settingsBackendRegionSummary,
    settingsModal,
    settingsClose,
    settingsCancel,
    output,
  },
  constants: {
    defaultQuickDevBuyAmounts: DEFAULT_QUICK_DEV_BUY_AMOUNTS,
    defaultPresetId: DEFAULT_PRESET_ID,
    standardRpcSlippageDefault: STANDARD_RPC_SLIPPAGE_DEFAULT,
    providerLabels: PROVIDER_LABELS,
    routeCapabilities: ROUTE_CAPABILITIES,
    providerFeeRequirements: PROVIDER_FEE_REQUIREMENTS,
  },
  renderCache,
  renderUtils: RenderUtils,
  state: {
    getLatestWalletStatus: () => latestWalletStatus,
    setLatestWalletStatus: (value) => {
      latestWalletStatus = value;
    },
    getLatestRuntimeStatus: () => latestRuntimeStatus,
  },
  helpers: {
    escapeHTML,
    getNamedValue,
    isNamedChecked,
    validateFieldByName,
    validateAllInlineFields,
    focusFirstInvalidInlineField,
    hasBootstrapConfig,
    setStatusLabel,
  },
  actions: {
    scheduleLiveSyncBroadcast: (options) => scheduleLiveSyncBroadcast(options),
    queueWarmActivity: (options) => queueWarmActivity(options),
    syncDevAutoSellUI: () => syncDevAutoSellUI(),
    clearDevBuyState: () => clearDevBuyState(),
  },
}) : null;
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

function selectedModeStorageKeyForLaunchpad(launchpad = getLaunchpad()) {
  return `${SELECTED_MODE_STORAGE_KEY}.${normalizeLaunchpad(launchpad)}`;
}

function getStoredLaunchMode(launchpad = getLaunchpad()) {
  try {
    const scoped = window.localStorage.getItem(selectedModeStorageKeyForLaunchpad(launchpad));
    if (scoped) return normalizeLaunchMode(scoped);
    const legacy = window.localStorage.getItem(SELECTED_MODE_STORAGE_KEY);
    return legacy ? normalizeLaunchMode(legacy) : "";
  } catch (_error) {
    return "";
  }
}

function setStoredLaunchMode(mode, launchpad = getLaunchpad()) {
  try {
    const normalizedLaunchpad = normalizeLaunchpad(launchpad);
    const normalizedMode = normalizeLaunchModeForLaunchpad(mode, normalizedLaunchpad);
    window.localStorage.setItem(selectedModeStorageKeyForLaunchpad(normalizedLaunchpad), normalizedMode);
    window.localStorage.setItem(SELECTED_MODE_STORAGE_KEY, normalizedMode);
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

function scopedLaunchpadDraftKey(prefix, launchpad = getLaunchpad()) {
  const scopedLaunchpad = normalizeLaunchpad(launchpad) || "pump";
  return `${prefix}.${scopedLaunchpad}.v2`;
}

function knownDraftScopeLaunchpads() {
  const values = new Set(["pump", "bonk", "bagsapp"]);
  document.querySelectorAll('input[name="launchpad"]').forEach((input) => {
    const value = normalizeLaunchpad(input && input.value);
    if (value) values.add(value);
  });
  return Array.from(values);
}

function readStoredScopedDraft(prefix, legacyStorageKey, launchpad = getLaunchpad()) {
  const normalizedLaunchpad = normalizeLaunchpad(launchpad) || "pump";
  const parseScopedValue = (raw, { hydrateLaunchpad = false } = {}) => {
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return parsed;
    const parsedLaunchpad = normalizeLaunchpad(parsed.launchpad || normalizedLaunchpad) || normalizedLaunchpad;
    if (!hydrateLaunchpad && parsed.launchpad && parsedLaunchpad !== normalizedLaunchpad) {
      return null;
    }
    return {
      ...parsed,
      launchpad: normalizedLaunchpad,
    };
  };
  try {
    const scopedRaw = window.localStorage.getItem(scopedLaunchpadDraftKey(prefix, normalizedLaunchpad));
    if (scopedRaw) return parseScopedValue(scopedRaw);
    if (!legacyStorageKey) return null;
    return parseScopedValue(
      window.localStorage.getItem(legacyStorageKey),
      { hydrateLaunchpad: true },
    );
  } catch (_error) {
    return null;
  }
}

function setStoredScopedDraft(prefix, legacyStorageKey, value, launchpad = getLaunchpad()) {
  const normalizedLaunchpad = normalizeLaunchpad(launchpad) || "pump";
  const scopedKey = scopedLaunchpadDraftKey(prefix, normalizedLaunchpad);
  try {
    if (!value) {
      window.localStorage.removeItem(scopedKey);
      if (legacyStorageKey) window.localStorage.removeItem(legacyStorageKey);
      return;
    }
    const payload = value && typeof value === "object" && !Array.isArray(value)
      ? { ...value, launchpad: normalizedLaunchpad }
      : value;
    window.localStorage.setItem(scopedKey, JSON.stringify(payload));
    if (legacyStorageKey) window.localStorage.removeItem(legacyStorageKey);
  } catch (_error) {
    // Ignore storage failures and keep local drafts best-effort.
  }
}

function readStoredGlobalDraft(prefix, legacyStorageKey) {
  const parseValue = (raw) => {
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return parsed;
    const { launchpad: _launchpad, ...rest } = parsed;
    return rest;
  };
  try {
    if (legacyStorageKey) {
      const legacyRaw = window.localStorage.getItem(legacyStorageKey);
      if (legacyRaw) return parseValue(legacyRaw);
    }
    for (const launchpad of knownDraftScopeLaunchpads()) {
      const scopedRaw = window.localStorage.getItem(scopedLaunchpadDraftKey(prefix, launchpad));
      if (!scopedRaw) continue;
      const parsed = parseValue(scopedRaw);
      if (parsed && legacyStorageKey) {
        window.localStorage.setItem(legacyStorageKey, JSON.stringify(parsed));
      }
      return parsed;
    }
    return null;
  } catch (_error) {
    return null;
  }
}

function setStoredGlobalDraft(prefix, legacyStorageKey, value) {
  try {
    if (!value) {
      if (legacyStorageKey) window.localStorage.removeItem(legacyStorageKey);
      knownDraftScopeLaunchpads().forEach((launchpad) => {
        window.localStorage.removeItem(scopedLaunchpadDraftKey(prefix, launchpad));
      });
      return;
    }
    if (legacyStorageKey) {
      window.localStorage.setItem(legacyStorageKey, JSON.stringify(value));
    }
    knownDraftScopeLaunchpads().forEach((launchpad) => {
      window.localStorage.removeItem(scopedLaunchpadDraftKey(prefix, launchpad));
    });
  } catch (_error) {
    // Ignore storage failures and keep local drafts best-effort.
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

function getStoredTickerCapsEnabled() {
  try {
    return window.localStorage.getItem(TICKER_CAPS_STORAGE_KEY) === "true";
  } catch (_error) {
    return false;
  }
}

function setStoredTickerCapsEnabled(enabled) {
  try {
    window.localStorage.setItem(TICKER_CAPS_STORAGE_KEY, String(Boolean(enabled)));
  } catch (_error) {
    // Ignore storage failures and keep the toggle functional.
  }
}

function serializeFeeSplitDraft(...args) {
  return splitEditorsDomain.serializeFeeSplitDraft(...args);
}

function normalizeRecipientType(type, { allowAgent = false } = {}) {
  return feeRoutingSupport.normalizeRecipientType(type, { allowAgent });
}

function launchpadSupportsExtendedSocialRecipients(launchpad = getLaunchpad()) {
  return feeRoutingSupport.launchpadSupportsExtendedSocialRecipients(launchpad);
}

function supportedRecipientTypesForLaunchpad(launchpad = getLaunchpad(), { allowAgent = false } = {}) {
  return feeRoutingSupport.supportedRecipientTypesForLaunchpad(launchpad, { allowAgent });
}

function isRecipientTypeSupportedForLaunchpad(type, launchpad = getLaunchpad(), { allowAgent = false } = {}) {
  return feeRoutingSupport.isRecipientTypeSupportedForLaunchpad(type, launchpad, { allowAgent });
}

function recipientTypeTabsMarkup() {
  return feeRoutingSupport.recipientTypeTabsMarkup();
}

function syncRecipientTypeTabVisibility(row) {
  feeRoutingSupport.syncRecipientTypeTabVisibility(row);
}

function isSocialRecipientType(type) {
  return feeRoutingSupport.isSocialRecipientType(type);
}

function recipientTypeLabel(type) {
  return feeRoutingSupport.recipientTypeLabel(type);
}

function recipientTypeIconSrc(type) {
  return feeRoutingSupport.recipientTypeIconSrc(type);
}

function recipientTypeIconMarkup(type) {
  return feeRoutingSupport.recipientTypeIconMarkup(type);
}

function recipientTargetPlaceholder(type) {
  return feeRoutingSupport.recipientTargetPlaceholder(type);
}

function recipientDisplayValueFromEntry(entry) {
  return feeRoutingSupport.recipientDisplayValueFromEntry(entry);
}

function isBagsFeeSplitLaunchpad(...args) {
  return splitEditorsDomain.isBagsFeeSplitLaunchpad(...args);
}

function isPumpFeeSplitLaunchpad(...args) {
  return splitEditorsDomain.isPumpFeeSplitLaunchpad(...args);
}

function usesImplicitBagsCreatorShareMode(...args) {
  return splitEditorsDomain.usesImplicitBagsCreatorShareMode(...args);
}

function usesImplicitPumpCreatorShareMode(...args) {
  return splitEditorsDomain.usesImplicitPumpCreatorShareMode(...args);
}

function usesImplicitCreatorShareMode(...args) {
  return splitEditorsDomain.usesImplicitCreatorShareMode(...args);
}

function nextFeeSplitRowId(...args) {
  return splitEditorsDomain.nextFeeSplitRowId(...args);
}

function getFeeSplitRowState(...args) {
  return splitEditorsDomain.getFeeSplitRowState(...args);
}

function clearFeeSplitRowLookupTimer(...args) {
  return splitEditorsDomain.clearFeeSplitRowLookupTimer(...args);
}

function clearFeeSplitRowState(...args) {
  return splitEditorsDomain.clearFeeSplitRowState(...args);
}

function normalizeFeeSplitLookupState(...args) {
  return splitEditorsDomain.normalizeFeeSplitLookupState(...args);
}

function serializeFeeSplitLookupState(...args) {
  return splitEditorsDomain.serializeFeeSplitLookupState(...args);
}

function restoreFeeSplitLookupState(...args) {
  return splitEditorsDomain.restoreFeeSplitLookupState(...args);
}

function validateFeeSplitSocialTarget(...args) {
  return splitEditorsDomain.validateFeeSplitSocialTarget(...args);
}

function buildFeeSplitLookupDescriptor(...args) {
  return splitEditorsDomain.buildFeeSplitLookupDescriptor(...args);
}

function describeFeeSplitLookupFailure(...args) {
  return splitEditorsDomain.describeFeeSplitLookupFailure(...args);
}

function syncBagsFeeSplitSummary(...args) {
  return splitEditorsDomain.syncBagsFeeSplitSummary(...args);
}

function removeImplicitCreatorRows(...args) {
  return splitEditorsDomain.removeImplicitCreatorRows(...args);
}

function syncFeeSplitModalPresentation(...args) {
  return splitEditorsDomain.syncFeeSplitModalPresentation(...args);
}

function syncCompactFeeSplitListViewport(...args) {
  return splitEditorsDomain.syncCompactFeeSplitListViewport(...args);
}

function syncAgentSplitListViewport(...args) {
  return splitEditorsDomain.syncAgentSplitListViewport(...args);
}

function copyBagsResolvedWallet(...args) {
  return splitEditorsDomain.copyBagsResolvedWallet(...args);
}

function updateFeeSplitRowValidationUi(...args) {
  return splitEditorsDomain.updateFeeSplitRowValidationUi(...args);
}

function runFeeSplitLookup(...args) {
  return splitEditorsDomain.runFeeSplitLookup(...args);
}

function scheduleFeeSplitLookup(...args) {
  return splitEditorsDomain.scheduleFeeSplitLookup(...args);
}

function normalizeFeeSplitDraft(...args) {
  return splitEditorsDomain.normalizeFeeSplitDraft(...args);
}

function stripImplicitCreatorRowsFromFeeSplitDraftRows(...args) {
  return splitEditorsDomain.stripImplicitCreatorRowsFromFeeSplitDraftRows(...args);
}

function normalizeFeeSplitDraftForLaunchpad(...args) {
  return splitEditorsDomain.normalizeFeeSplitDraftForLaunchpad(...args);
}

function currentFeeSplitDraftLaunchpad(...args) {
  return splitEditorsDomain.currentFeeSplitDraftLaunchpad(...args);
}

function feeSplitDraftStorageKey(...args) {
  return splitEditorsDomain.feeSplitDraftStorageKey(...args);
}

function feeSplitDraftSessionStorageKey(...args) {
  return splitEditorsDomain.feeSplitDraftSessionStorageKey(...args);
}

function getStoredFeeSplitDraft(...args) {
  return splitEditorsDomain.getStoredFeeSplitDraft(...args);
}

function setStoredFeeSplitDraft(...args) {
  return splitEditorsDomain.setStoredFeeSplitDraft(...args);
}

function withSuspendedFeeSplitDraftPersistence(...args) {
  return splitEditorsDomain.withSuspendedFeeSplitDraftPersistence(...args);
}

function restoreFeeSplitDraftForLaunchpad(...args) {
  return splitEditorsDomain.restoreFeeSplitDraftForLaunchpad(...args);
}

function applyFeeSplitDraft(...args) {
  return splitEditorsDomain.applyFeeSplitDraft(...args);
}

function feeSplitClearAllDraft(...args) {
  return splitEditorsDomain.feeSplitClearAllDraft(...args);
}

function updateFeeSplitClearAllButton(...args) {
  return splitEditorsDomain.updateFeeSplitClearAllButton(...args);
}

function clearFeeSplitRestoreState(...args) {
  return splitEditorsDomain.clearFeeSplitRestoreState(...args);
}

function agentSplitClearAllDraft(...args) {
  return splitEditorsDomain.agentSplitClearAllDraft(...args);
}

function updateAgentSplitClearAllButton(...args) {
  return splitEditorsDomain.updateAgentSplitClearAllButton(...args);
}

function clearAgentSplitRestoreState(...args) {
  return splitEditorsDomain.clearAgentSplitRestoreState(...args);
}

function serializeAgentSplitDraft(...args) {
  return splitEditorsDomain.serializeAgentSplitDraft(...args);
}

function normalizeAgentSplitDraft(...args) {
  return splitEditorsDomain.normalizeAgentSplitDraft(...args);
}

function buildAgentSplitDraftFromFeeSplitDraft(...args) {
  return splitEditorsDomain.buildAgentSplitDraftFromFeeSplitDraft(...args);
}

function buildFeeSplitDraftFromAgentSplitDraft(...args) {
  return splitEditorsDomain.buildFeeSplitDraftFromAgentSplitDraft(...args);
}

function getStoredAgentSplitDraft(...args) {
  return splitEditorsDomain.getStoredAgentSplitDraft(...args);
}

function setStoredAgentSplitDraft(...args) {
  return splitEditorsDomain.setStoredAgentSplitDraft(...args);
}

function normalizeAutoSellDraft(value) {
  if (!value || typeof value !== "object") return null;
  const triggerFamily = normalizeAutoSellTriggerFamily(
    value.triggerFamily || ((Boolean(value.marketCapEnabled) || String(value.marketCapThreshold || "").trim()) ? "market-cap" : "time")
  );
  const hasExplicitTimeoutSeconds = Object.prototype.hasOwnProperty.call(value, "marketCapScanTimeoutSeconds");
  const legacyTimeoutMinutesRaw = String(value.marketCapScanTimeoutMinutes || "").trim();
  const explicitTimeoutSeconds = hasExplicitTimeoutSeconds
    ? String(value.marketCapScanTimeoutSeconds ?? "").trim()
    : "";
  const timeoutSeconds = hasExplicitTimeoutSeconds
    ? (explicitTimeoutSeconds
      ? Math.max(1, Math.min(86400, Math.round(Number(explicitTimeoutSeconds || 30) || 30)))
      : "")
    : (legacyTimeoutMinutesRaw
      ? Math.max(1, Math.min(86400, Math.round((Number(legacyTimeoutMinutesRaw || 15) || 15) * 60)))
      : 30);
  return {
    enabled: Boolean(value.enabled),
    sniperEnabled: Boolean(value.sniperEnabled),
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

function buildAutoSellDraftFromDefaults(value) {
  if (!value || typeof value !== "object") return null;
  return {
    enabled: value.enabled,
    sniperEnabled: Boolean(value.sniperEnabled),
    percent: value.enabled
      ? Math.max(1, Number(value.percent || 100))
      : Number(value.percent || 100),
    triggerFamily: value.triggerFamily
      || ((Boolean(value.marketCapEnabled) || Boolean(value.marketCapThreshold))
        ? "market-cap"
        : "time"),
    triggerMode: value.triggerMode
      || (Number(value.delaySeconds || 0) > 0 ? "submit-delay" : "block-offset"),
    delayMs: value.delayMs != null
      ? value.delayMs
      : Number(value.delaySeconds || 0) * 1000,
    blockOffset: value.targetBlockOffset || 0,
    marketCapEnabled: Boolean(value.marketCapEnabled)
      || Boolean(value.marketCapThreshold),
    marketCapThreshold: value.marketCapThreshold || "",
    marketCapScanTimeoutSeconds: value.marketCapScanTimeoutSeconds != null
      ? value.marketCapScanTimeoutSeconds
      : (value.marketCapScanTimeoutMinutes != null
        ? (value.marketCapScanTimeoutMinutes * 60)
        : 30),
    marketCapTimeoutAction: value.marketCapTimeoutAction || "stop",
  };
}

function getStoredAutoSellDraft() {
  return normalizeAutoSellDraft(
    readStoredGlobalDraft(AUTO_SELL_DRAFT_STORAGE_PREFIX, AUTO_SELL_DRAFT_STORAGE_KEY),
  );
}

function setStoredAutoSellDraft(value, { scheduleSync = true } = {}) {
  const normalized = normalizeAutoSellDraft(value);
  setStoredGlobalDraft(
    AUTO_SELL_DRAFT_STORAGE_PREFIX,
    AUTO_SELL_DRAFT_STORAGE_KEY,
    normalized,
  );
  if (scheduleSync) scheduleLiveSyncBroadcast({ immediate: true });
}

function getSavedGlobalDefaults(config = getConfig()) {
  const defaults = config && config.defaults && typeof config.defaults === "object"
    ? config.defaults
    : {};
  const misc = defaults.misc && typeof defaults.misc === "object"
    ? defaults.misc
    : {};
  return {
    defaults,
    misc,
  };
}

function getSavedGlobalSniperDraft(config = getConfig()) {
  const { misc } = getSavedGlobalDefaults(config);
  if (misc.sniperDraft) return misc.sniperDraft;
  if (misc.sniperDraftsByLaunchpad && typeof misc.sniperDraftsByLaunchpad === "object") {
    const firstScopedDraft = Object.values(misc.sniperDraftsByLaunchpad).find((entry) => entry && typeof entry === "object");
    if (firstScopedDraft) return firstScopedDraft;
  }
  return null;
}

function getSavedGlobalAutoSellDraft(config = getConfig()) {
  const { defaults, misc } = getSavedGlobalDefaults(config);
  if (defaults.automaticDevSell) return defaults.automaticDevSell;
  if (misc.autoSellDraftsByLaunchpad && typeof misc.autoSellDraftsByLaunchpad === "object") {
    const firstScopedDraft = Object.values(misc.autoSellDraftsByLaunchpad).find((entry) => entry && typeof entry === "object");
    if (firstScopedDraft) return firstScopedDraft;
  }
  return null;
}

function applyAutoSellDraft(value, { persist = false } = {}) {
  const draft = normalizeAutoSellDraft(value);
  if (!draft) return;
  if (autoSellEnabledInput) autoSellEnabledInput.checked = Boolean(draft.enabled);
  if (autoSellSniperEnabledInput) autoSellSniperEnabledInput.checked = Boolean(draft.sniperEnabled);
  setNamedValue("automaticDevSellTriggerFamily", draft.triggerFamily);
  setNamedValue("automaticDevSellPercent", String(draft.percent));
  setNamedValue("automaticDevSellTriggerMode", draft.triggerMode);
  setNamedValue("automaticDevSellDelayMs", String(draft.delayMs));
  setNamedValue("automaticDevSellBlockOffset", String(draft.blockOffset));
  setNamedChecked("automaticDevSellMarketCapEnabled", draft.triggerFamily === "market-cap");
  setNamedValue("automaticDevSellMarketCapThreshold", draft.marketCapThreshold);
  setNamedValue("automaticDevSellMarketCapScanTimeoutSeconds", String(draft.marketCapScanTimeoutSeconds));
  setNamedValue("automaticDevSellMarketCapTimeoutAction", draft.marketCapTimeoutAction);
  setNamedChecked("automaticSniperSellEnabled", Boolean(draft.sniperEnabled));
  syncDevAutoSellUI();
  if (persist) setStoredAutoSellDraft(draft);
}

function applyAgentSplitDraft(...args) {
  return splitEditorsDomain.applyAgentSplitDraft(...args);
}

function syncAgentSplitDraftFromFeeSplitDraft(...args) {
  return splitEditorsDomain.syncAgentSplitDraftFromFeeSplitDraft(...args);
}

function syncFeeSplitDraftFromAgentSplitDraft(...args) {
  return splitEditorsDomain.syncFeeSplitDraftFromAgentSplitDraft(...args);
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

function matchesImportedCreatorFeeRecipient(entry, creatorFee = importedCreatorFeeState) {
  if (!entry) return false;
  const normalizedCreatorFee = normalizeImportedCreatorFeeState(creatorFee);
  const entryType = normalizeRecipientType(entry.type);
  const entryValue = String(entry.value || "").trim();
  if (!normalizedCreatorFee.mode) return false;
  if (normalizedCreatorFee.mode === "deployer") {
    const deployerAddress = getDeployerFeeSplitAddress();
    return entryType === "wallet" && Boolean(deployerAddress) && entryValue === deployerAddress;
  }
  if (normalizedCreatorFee.mode === "wallet") {
    return entryType === "wallet" && Boolean(normalizedCreatorFee.address) && entryValue === normalizedCreatorFee.address;
  }
  if (normalizedCreatorFee.mode === "github" && entryType === "github") {
    const parsedTarget = parseGithubRecipientTarget(entryValue);
    const entryGithubUserId = String(entry.githubUserId || parsedTarget.githubUserId || "").trim();
    const entryGithubUsername = String(parsedTarget.githubUsername || "").trim();
    return (
      (normalizedCreatorFee.githubUserId && normalizedCreatorFee.githubUserId === entryGithubUserId)
      || (normalizedCreatorFee.githubUsername && normalizedCreatorFee.githubUsername === entryGithubUsername)
    );
  }
  return false;
}

function buildFeeSplitDraftFromRecipients(recipients, { enabled = false, launchpad = getLaunchpad(), creatorFee = importedCreatorFeeState } = {}) {
  const normalizedLaunchpad = normalizeLaunchpad(launchpad);
  const rows = Array.isArray(recipients)
    ? recipients.map((entry) => ({
      type: normalizeRecipientType(entry && entry.type),
      value: recipientDisplayValueFromEntry(entry),
      githubUserId: String(entry && entry.githubUserId || "").trim(),
      sharePercent: formatShareBpsAsPercent(entry && entry.shareBps),
      defaultReceiver: false,
      targetLocked: false,
    })).filter((entry) => entry.value || entry.sharePercent)
    : [];
  let importedCreatorCollapsed = false;
  const filteredRows = normalizedLaunchpad === "pump"
    ? rows.filter((entry) => {
      if (!importedCreatorCollapsed && matchesImportedCreatorFeeRecipient(entry, creatorFee)) {
        importedCreatorCollapsed = true;
        return false;
      }
      return true;
    })
    : rows;
  return normalizeFeeSplitDraftForLaunchpad({
    enabled: normalizedLaunchpad === "pump" ? (enabled && filteredRows.length > 0) : enabled,
    suppressDefaultRow: filteredRows.length > 0,
    rows: filteredRows,
  }, launchpad);
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
          type: normalizeRecipientType(type),
          value: recipientDisplayValueFromEntry(entry),
          githubUserId: String(entry && entry.githubUserId || "").trim(),
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
  setImportedCreatorFeeState(creatorFee || null);
  applyFeeSplitDraft(
    buildFeeSplitDraftFromRecipients(feeRecipients, {
      enabled: nextLaunchpad === "bagsapp" || feeRecipients.length > 0,
      launchpad: nextLaunchpad,
      creatorFee,
    }),
    { persist: false },
  );
  applyAgentSplitDraft(buildAgentSplitDraftFromRecipients(agentRecipients), { persist: false });
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

walletRuntimeDomain = WalletRuntimeDomainModule.create({
  elements: {
    walletDropdown,
    walletDropdownList,
    walletTriggerButton,
    walletSummarySol,
    walletSummaryUsd,
    walletSelect,
    walletBalance,
    metaNode,
    platformRuntimeIndicators,
    toggleReportsButton,
    reportsTerminalSection,
    reportsTerminalOutput,
    providerSelect,
    creationMevModeSelect,
    buyProviderSelect,
    buyMevModeSelect,
    sellProviderSelect,
    sellMevModeSelect,
  },
  storageKeys: {
    selectedWallet: SELECTED_WALLET_STORAGE_KEY,
    walletStatusLastRefreshAtMs: WALLET_STATUS_LAST_REFRESH_STORAGE_KEY,
  },
  constants: {
    runtimeStatusRefreshIntervalMs: RUNTIME_STATUS_REFRESH_INTERVAL_MS,
    followJobsRefreshIntervalMs: FOLLOW_JOBS_REFRESH_INTERVAL_MS,
    followJobsOfflineRetryMs: FOLLOW_JOBS_OFFLINE_RETRY_MS,
    warmActivityDebounceMs: WARM_ACTIVITY_DEBOUNCE_MS,
  },
  requestUtils: RequestUtils,
  renderUtils: RenderUtils,
  renderCache,
  requestStates,
  state: {
    getLatestWalletStatus: () => latestWalletStatus,
    setLatestWalletStatus: (value) => {
      latestWalletStatus = value;
    },
    getLatestRuntimeStatus: () => latestRuntimeStatus,
    setLatestRuntimeStatus: (value) => {
      latestRuntimeStatus = value;
    },
    getStartupWarmState: () => startupWarmState,
    setStartupWarmState: (value) => {
      startupWarmState = value;
    },
    getAppBootstrapState: () => appBootstrapState,
    getFollowJobsState: () => followJobsState,
    setFollowJobsState: (value) => {
      followJobsState = value;
    },
    getWarmActivityState: () => warmActivityState,
    setWarmActivityState: (value) => {
      warmActivityState = value;
    },
    getWalletStatusRefreshTimer: () => walletStatusRefreshTimer,
    setWalletStatusRefreshTimer: (value) => {
      walletStatusRefreshTimer = value;
    },
    getRuntimeStatusRefreshTimer: () => runtimeStatusRefreshTimer,
    setRuntimeStatusRefreshTimer: (value) => {
      runtimeStatusRefreshTimer = value;
    },
    getWalletStatusRefreshIntervalMs: () => walletStatusRefreshIntervalMs,
    setWalletStatusRefreshIntervalMs: (value) => {
      walletStatusRefreshIntervalMs = value;
    },
    getReportsTerminalState: () => reportsTerminalState,
  },
  helpers: {
    selectedWalletKey,
    getConfig,
    getActivePreset,
    normalizeMevMode,
    normalizeReportsTerminalView,
    isTerminalFollowJobState,
    escapeHTML,
    shortenAddress,
    shortAddress,
    shortenReportEndpoint,
    walletIndexFromEnvKey,
  },
  actions: {
    renderSniperUI,
    updateLockedModeFields,
    renderBackendRegionSummary,
    applyPersistentDefaults,
    applyProviderAvailability,
    applyLaunchpadAvailability,
    renderQuickDevBuyButtons,
    populateDevBuyPresetEditor,
    updateQuote: () => updateQuote(),
    markBootstrapState,
    hasBootstrapConfig,
    setSettingsLoadingState,
    schedulePopoutAutosize,
    refreshActiveLogs: (options) => refreshActiveLogs(options),
    renderReportsTerminalOutput: () => renderReportsTerminalOutput(),
    loadRuntimeStatus: () => refreshRuntimeStatus(),
  },
});

reportsPresenters = window.LaunchDeckReportsPresenters.create({
  elements: {
    launchSurfaceCard,
    outputSection,
    reportsTerminalSection,
    reportsTerminalList,
    reportsTerminalOutput,
    reportsTerminalMeta,
    reportsTransactionsButton,
    reportsLaunchesButton,
    reportsActiveJobsButton,
    reportsActiveLogsButton,
    benchmarksPopoutModal,
    benchmarksPopoutTitle,
    benchmarksPopoutBody,
  },
  renderCache,
  setCachedHTML: RenderUtils.setCachedHTML,
  getState: () => reportsTerminalState,
  getFollowJobsState: () => followJobsState,
  getFollowStatusSnapshot: () => followStatusSnapshot(),
  syncFollowStatusChrome: () => syncFollowStatusChrome(),
  activeFollowJobForTraceId,
  isTerminalFollowJobState,
  escapeHTML,
  shortenAddress,
  shortAddress,
  walletIndexFromEnvKey,
  formatWalletHistoryLabel,
  getQuoteAssetLabel,
  getDevBuyAssetLabel,
  getSniperTriggerSummary,
  providerLabels: PROVIDER_LABELS,
});

reportsHistory = window.LaunchDeckReportsHistory.create({
  getState: () => reportsTerminalState,
  reportLimit: REPORTS_TERMINAL_ITEM_LIMIT,
  normalizeSavedFollowLaunchForUi,
});

reportsFeature = window.ReportsFeature.create({
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
  getActiveTab: () => reportsTerminalState.activeTab,
  setActiveTab: (value) => {
    reportsTerminalState.activeTab = value;
  },
  reuseEntry: (id) => reuseFromHistory(id),
  relaunchEntry: (id) => relaunchFromHistory(id),
  normalizeTab: (tab) => normalizeReportsTerminalTab(tab),
  normalizeActiveLogsView: (view) => normalizeActiveLogsView(view),
  setActiveLogsView: (value) => setReportsActiveLogsView(value),
  refreshActiveLogs: (options) => refreshActiveLogs(options),
  showBenchmarksPopout: () => showBenchmarksPopoutModal(),
  cancelAllJobs: () => cancelAllFollowJobs(),
  cancelJob: (traceId, options) => cancelFollowJob(traceId, options),
  setMetaText: (value) => {
    if (reportsTerminalMeta) {
      reportsTerminalMeta.textContent = value || "";
      return;
    }
    if (metaNode) metaNode.textContent = value || "";
  },
  scheduleLiveSyncBroadcast: () => scheduleLiveSyncBroadcast(),
  shortenAddress,
  openPopoutWindow,
});

reportsFeature.bindEvents();

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
  syncCompactFeeSplitListViewport();
  syncAgentSplitListViewport();
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
  if (imageMetadataDomain && typeof imageMetadataDomain.setSelectedImage === "function") {
    imageMetadataDomain.setSelectedImage(image);
    return;
  }
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
  return imageMetadataDomain
    ? imageMetadataDomain.hasAttachedImage()
    : Boolean(
        uploadedImage
        || (metadataUri && metadataUri.value)
        || (imagePreview && !imagePreview.hidden && imagePreview.src),
      );
}

async function ensureTestImageSelected() {
  if (!imageMetadataDomain) return false;
  return imageMetadataDomain.ensureTestImageSelected();
}

async function selectImportedImage(image) {
  if (!imageMetadataDomain) return;
  await imageMetadataDomain.selectImportedImage(image);
}

function restoreLaunchHistoryImage(launch) {
  if (!imageMetadataDomain) return;
  imageMetadataDomain.restoreLaunchHistoryImage(launch);
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
    autoSellButtonProgress,
    devAutoSellPanel,
    autoSellEnabledInput,
    autoSellToggleState,
    autoSellTriggerFamilyValue,
    autoSellTriggerValue,
    autoSellTimeSettings,
    autoSellTriggerFamilyButtons,
    autoSellDelaySlider,
    autoSellDelayInput,
    autoSellDelayControl,
    autoSellPercentSlider,
    autoSellPercentInput,
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
    autoSellSniperEnabledInput,
    autoSellSniperToggleState,
    autoSellSniperWalletList,
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
  getSniperAutosellRows,
  updateSniperAutosellWallet,
  persistDraft: () => setStoredAutoSellDraft({
    enabled: isNamedChecked("automaticDevSellEnabled"),
    sniperEnabled: isNamedChecked("automaticSniperSellEnabled"),
    percent: getNamedValue("automaticDevSellPercent") || "100",
    triggerFamily: getNamedValue("automaticDevSellTriggerFamily") || "time",
    triggerMode: getNamedValue("automaticDevSellTriggerMode") || "block-offset",
    delayMs: getNamedValue("automaticDevSellDelayMs") || "0",
    blockOffset: getNamedValue("automaticDevSellBlockOffset") || "0",
    marketCapEnabled: (getNamedValue("automaticDevSellTriggerFamily") || "time") === "market-cap",
    marketCapThreshold: getNamedValue("automaticDevSellMarketCapThreshold") || "",
    marketCapScanTimeoutSeconds: String(getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds") || "").trim(),
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

function formatSavedMarketCapThresholdForUi(value) {
  const trimmed = String(value || "").trim();
  if (!/^\d+$/.test(trimmed)) return trimmed;
  try {
    const micros = BigInt(trimmed);
    if (micros < 1000000n) return trimmed;
    const wholeUsd = micros / 1000000n;
    const fractionalMicros = micros % 1000000n;
    const formatWithSuffix = (suffixValue, suffixLabel) => {
      const whole = wholeUsd / suffixValue;
      const remainder = wholeUsd % suffixValue;
      if (whole >= 100n || remainder === 0n) return `${whole.toString()}${suffixLabel}`;
      const decimal = (remainder * 10n) / suffixValue;
      if (decimal === 0n) return `${whole.toString()}${suffixLabel}`;
      return `${whole.toString()}.${decimal.toString()}${suffixLabel}`;
    };
    if (fractionalMicros === 0n) {
      if (wholeUsd >= 1000000000000n) return formatWithSuffix(1000000000000n, "t");
      if (wholeUsd >= 1000000000n) return formatWithSuffix(1000000000n, "b");
      if (wholeUsd >= 1000000n) return formatWithSuffix(1000000n, "m");
      if (wholeUsd >= 1000n) return formatWithSuffix(1000n, "k");
      return wholeUsd.toString();
    }
    const fractionalText = fractionalMicros.toString().padStart(6, "0").replace(/0+$/, "");
    return fractionalText ? `${wholeUsd.toString()}.${fractionalText}` : wholeUsd.toString();
  } catch (_error) {
    return trimmed;
  }
}

function normalizeSavedFollowLaunchForUi(value) {
  if (!value || typeof value !== "object") return {};
  const followLaunch = { ...value };
  if (followLaunch.devAutoSell && typeof followLaunch.devAutoSell === "object") {
    followLaunch.devAutoSell = { ...followLaunch.devAutoSell };
    if (followLaunch.devAutoSell.marketCap && typeof followLaunch.devAutoSell.marketCap === "object") {
      followLaunch.devAutoSell.marketCap = {
        ...followLaunch.devAutoSell.marketCap,
        threshold: formatSavedMarketCapThresholdForUi(followLaunch.devAutoSell.marketCap.threshold),
      };
    }
  }
  return followLaunch;
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

function hydrateDevAutoSellState({ preferExistingFormFallback = true } = {}) {
  const storedDraft = getStoredAutoSellDraft();
  if (storedDraft) {
    applyAutoSellDraft(storedDraft, { persist: false });
    return;
  }
  const savedDraft = getSavedGlobalAutoSellDraft();
  if (savedDraft) {
    applyAutoSellDraft(buildAutoSellDraftFromDefaults(savedDraft), { persist: false });
    return;
  }
  if (!preferExistingFormFallback) {
    applyAutoSellDraft({
      enabled: false,
      sniperEnabled: false,
      percent: 100,
      triggerFamily: "time",
      triggerMode: "block-offset",
      delayMs: 0,
      blockOffset: 0,
      marketCapEnabled: false,
      marketCapThreshold: "",
      marketCapScanTimeoutSeconds: 30,
      marketCapTimeoutAction: "stop",
    }, { persist: false });
    return;
  }
  setNamedChecked("automaticSniperSellEnabled", isNamedChecked("automaticSniperSellEnabled"));
  syncDevAutoSellUI();
}

function toggleDevAutoSellPanel(forceOpen) {
  autoSellFeature.togglePanel(forceOpen);
}

const sniperFeature = window.SniperFeature.create({
  storageKey: SNIPER_DRAFT_STORAGE_KEY,
  readStoredDraft: () => getStoredSniperDraft(),
  writeStoredDraft: (draft) => setStoredSniperDraft(draft),
  renderCache,
  balancePresets: SNIPER_BALANCE_PRESETS,
  executionReserveSol: SNIPER_EXECUTION_RESERVE_SOL,
  elements: {
    postLaunchStrategyInput,
    snipeBuyAmountInput,
    sniperEnabledInput,
    sniperConfigJsonInput,
    modeSniperButton,
    modeSniperProgress,
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
  isNamedChecked,
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
  onStateChange: () => {
    syncDevAutoSellUI();
  },
});

sniperFeature.bindEvents();

const feeRoutingSupport = FeeRoutingModule.createFeeRouting({
  getLaunchpad,
  normalizeLaunchpad,
  escapeHTML,
});

splitEditorsDomain = SplitEditorsDomainModule.create ? SplitEditorsDomainModule.create({
  elements: {
    feeSplitPill,
    feeSplitPillTitle,
    feeSplitPillProgress,
    agentUnlockedAuthority,
    agentSplitList,
    agentSplitAdd,
    agentSplitReset,
    agentSplitEven,
    agentSplitClearAll,
    agentSplitTotal,
    agentSplitBar,
    agentSplitLegendList,
    agentSplitModal,
    agentSplitModalError,
    agentSplitTitle,
    feeSplitEnabled,
    feeSplitList,
    feeSplitAdd,
    feeSplitReset,
    feeSplitEven,
    feeSplitClearAll,
    feeSplitTotal,
    feeSplitBar,
    feeSplitLegendList,
    feeSplitModal,
    feeSplitModalError,
    feeSplitTitle,
    feeSplitIntro,
    feeSplitRecipientsCopy,
    bagsFeeSplitSummary,
    feeSplitSummaryPrimaryLabel,
    feeSplitSummarySecondaryLabel,
    bagsFeeSplitCreatorShare,
    bagsFeeSplitSharedShare,
    bagsFeeSplitValidationCopy,
  },
  constants: {
    legacyFeeSplitDraftStorageKey: FEE_SPLIT_DRAFT_STORAGE_KEY,
    agentSplitDraftStorageKey: AGENT_SPLIT_DRAFT_STORAGE_KEY,
    initialFeeSplitDraftLaunchpad: "pump",
    maxFeeSplitRecipients: MAX_FEE_SPLIT_RECIPIENTS,
    splitColors: SPLIT_COLORS,
    bagsFeeSplitVisibleCardCount: BAGS_FEE_SPLIT_VISIBLE_CARD_COUNT,
    bagsFeeSplitViewportBufferPx: BAGS_FEE_SPLIT_VIEWPORT_BUFFER_PX,
  },
  helpers: {
    feeRouting: feeRoutingSupport,
    getLaunchpad,
    getMode,
    normalizeLaunchpad,
    normalizeLaunchMode,
    normalizeDecimalInput,
    escapeHTML,
    shortenAddress,
    formatPercentNumber,
    parseGithubRecipientTarget,
    looksLikeSolanaAddress,
    getDeployerFeeSplitAddress,
    syncSettingsCapabilities,
    scheduleLiveSyncBroadcast: (options) => scheduleLiveSyncBroadcast(options),
  },
}) : null;

const formDomain = FormDomainModule.createFormDomain({
  form,
  metadataUri,
  feeSplitEnabled,
  getRouteCapabilities,
  getProvider,
  getBuyProvider,
  getSellProvider,
  isNamedChecked,
  getNamedValue,
  selectedWalletKey,
  getLaunchpad,
  getQuoteAsset,
  normalizeMevMode,
  getActivePresetId,
  collectAgentSplitRecipients,
  hasMeaningfulAgentSplitRecipients,
  hasMeaningfulFeeSplitConfiguration,
  getDevBuyMode,
  normalizeAutoFeeCapValue,
  isTrackSendBlockHeightEnabled,
  collectSubmittedFeeSplitRecipients,
  getImportedCreatorFeeState: () => importedCreatorFeeState,
  getLaunchpadUiCapabilities,
  getAutoSellTriggerFamily,
  getAutoSellTriggerMode,
  getAutoSellDelayMs,
  getAutoSellBlockOffset,
  getUploadedImage: () => uploadedImage,
  getMetadataUploadState: () => metadataUploadState,
  cloneConfig,
  getConfig,
  createFallbackConfig,
  defaultPresetId: DEFAULT_PRESET_ID,
  normalizeSniperDraftState,
  getSniperState: () => sniperFeature.getState(),
  normalizeFeeSplitDraft,
  serializeFeeSplitDraft,
  normalizeAgentSplitDraft,
  serializeAgentSplitDraft,
  normalizeAutoSellTriggerFamily,
  normalizeAutoSellTriggerMode,
});

imageMetadataDomain = ImageMetadataDomainModule.create ? ImageMetadataDomainModule.create({
  elements: {
    imagePreview,
    imageEmpty,
    imageStatus,
    imagePath,
    metadataUri,
  },
  constants: {
    metadataPreuploadDebounceMs: METADATA_PREUPLOAD_DEBOUNCE_MS,
  },
  state: {
    getUploadedImage: () => uploadedImage,
    getImageLibraryState: () => imageLibraryState,
    getMetadataUploadState: () => metadataUploadState,
  },
  helpers: {
    readForm: (...args) => formDomain.readForm(...args),
    canPreuploadMetadata: (...args) => formDomain.canPreuploadMetadata(...args),
    hasFreshPreuploadedMetadata: (...args) => formDomain.hasFreshPreuploadedMetadata(...args),
    launchpadHandlesOwnMetadata: (...args) => formDomain.launchpadHandlesOwnMetadata(...args),
    metadataFingerprintFromForm: (...args) => formDomain.metadataFingerprintFromForm(...args),
  },
  actions: {
    fetchImageLibrary,
    showImageDetailsModal,
    setSelectedImageInFeature: (image) => imagesFeature.setSelectedImage(image),
  },
}) : null;

liveSyncSupport = LiveSyncModule.createLiveSyncSupport({
  liveSyncSourceId: LIVE_SYNC_SOURCE_ID,
  getCurrentThemeMode,
  isOutputSectionCurrentlyVisible,
  isReportsTerminalCurrentlyVisible,
  getCurrentReportsTerminalListWidth,
  isImageLayoutCompactActive,
  getSelectedWalletValue: () => walletSelect ? String(walletSelect.value || "") : "",
  cloneConfig,
  getConfig,
  getLatestWalletStatus: () => latestWalletStatus,
  getLatestRuntimeStatus: () => latestRuntimeStatus,
  getStartupWarmState: () => startupWarmState,
  setStartupWarmState: (value) => {
    startupWarmState = value;
  },
  getReportsTerminalState: () => reportsTerminalState,
  setReportsTerminalState: (value) => {
    reportsTerminalState = value;
  },
  getFollowJobsState: () => followJobsState,
  setFollowJobsState: (value) => {
    followJobsState = value;
  },
  currentStatusLabel,
  metaNode,
  output,
  normalizeFeeSplitDraft,
  serializeFeeSplitDraft,
  buildLiveSyncFormControls,
  getLiveSyncControls,
  getLiveSyncControlKey,
  filterRefreshPersistedFormControls,
  setThemeMode,
  setOutputSectionVisible,
  setReportsTerminalVisible,
  setReportsTerminalListWidth,
  setImageLayoutCompact,
  setConfig,
  setPresetEditing,
  isPresetEditing,
  getActivePreset,
  applyPresetToSettingsInputs,
  queueWarmActivity,
  renderPlatformRuntimeIndicators,
  applyWalletStatusPayload,
  applyRuntimeStatusPayload,
  clearFollowJobsRefreshTimer,
  syncFollowStatusChrome,
  renderReportsTerminalList,
  renderReportsTerminalOutput,
  getStoredReportsTerminalView,
  getStoredActiveLogsView,
  setStatusLabel,
  walletSelect,
  applyFeeSplitDraft,
  setStoredFeeSplitDraft,
  schedulePopoutAutosize,
  isPopoutMode,
});

const runtimeActions = RuntimeActionsModule.createRuntimeActions({
  requestUtils: RequestUtils,
  requestStates,
  markBootstrapState,
  setSettingsLoadingState,
  setBootOverlayMessage,
  getStoredSelectedWalletKey,
  applyBootstrapFastPayload,
  beginStartupWarmup,
  flushWarmActivity,
  refreshRuntimeStatus: (...args) => refreshRuntimeStatus(...args),
  refreshWalletStatus,
  ensureInteractiveBootstrapReady,
  setBusy,
  output,
  stopOutputFollowRefresh,
  ensureStartupWarmReady,
  ensureMetadataReadyForAction,
  readForm: (...args) => readForm(...args),
  buildOutputMetaTextFromReport,
  metaNode,
  metadataUri,
  getMetadataUploadState: () => metadataUploadState,
  metadataFingerprintFromForm: (...args) => metadataFingerprintFromForm(...args),
  surfaceMetadataWarning,
  currentStatusLabel,
  setStatusLabel,
  getReportsTerminalState: () => reportsTerminalState,
  normalizeReportsTerminalView,
  renderReportsTerminalOutput,
  renderReportsTerminalList,
  extractReportIdFromPath,
  refreshReportsTerminal,
  refreshFollowJobs,
  startOutputFollowRefresh,
  vanityPrivateKeyText,
  vanityModalError,
  applyVanityValue,
  buttons,
  hasBootstrapConfig,
  validateSettingsModalBeforeSave,
  syncActivePresetFromInputs,
  buildSavedConfigFromForm: (...args) => buildSavedConfigFromForm(...args),
  cloneConfig,
  capturePreviewInputsFromRunReport: (report) => persistPreviewInputsFromRunReport(report),
  setRegionRouting,
  getLatestWalletStatus: () => latestWalletStatus,
  setConfig,
  renderQuickDevBuyButtons,
  populateDevBuyPresetEditor,
  renderBackendRegionSummary,
  queueWarmActivity,
  hideSettingsModal,
  applyRuntimeStatusPayload,
  reportsTerminalOutput,
  reportsTerminalSection,
});

function normalizeSniperDraftState(value) {
  return sniperFeature.normalizeDraftState(value);
}

function getStoredSniperDraft() {
  const storedDraft = readStoredGlobalDraft(
    SNIPER_DRAFT_STORAGE_PREFIX,
    SNIPER_DRAFT_STORAGE_KEY,
  );
  return storedDraft ? normalizeSniperDraftState(storedDraft) : null;
}

function setStoredSniperDraft(value, { scheduleSync = true } = {}) {
  const normalized = normalizeSniperDraftState(value);
  const hasWalletState = normalized && Object.keys(normalized.wallets || {}).length > 0;
  setStoredGlobalDraft(
    SNIPER_DRAFT_STORAGE_PREFIX,
    SNIPER_DRAFT_STORAGE_KEY,
    normalized && (normalized.enabled || hasWalletState) ? normalized : null,
  );
  if (scheduleSync) scheduleLiveSyncBroadcast({ immediate: true });
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

function getSniperAutosellRows() {
  const state = normalizeSniperDraftState(sniperFeature.getState());
  const wallets = latestWalletStatus && Array.isArray(latestWalletStatus.wallets)
    ? latestWalletStatus.wallets
    : [];
  return Object.entries(state.wallets || {})
    .filter(([, entry]) => entry && entry.selected)
    .map(([envKey, entry]) => {
      const wallet = wallets.find((candidate) => candidate && candidate.envKey === envKey) || null;
      return {
        envKey,
        walletLabel: wallet ? walletDisplayName(wallet) : `#${walletIndexFromEnvKey(envKey)}`,
        buyAmountValue: entry.amountSol ? String(entry.amountSol) : "",
        buyAmountAssetLabel: getQuoteAssetLabel(),
        sellEnabled: Boolean(entry.sellEnabled),
        sellPercent: entry.sellPercent || "",
        sellTriggerMode: entry.sellTriggerMode || "block-offset",
        sellTargetBlockOffset: entry.sellTargetBlockOffset != null ? Number(entry.sellTargetBlockOffset) : 0,
        sellMarketCapThreshold: entry.sellMarketCapThreshold || "",
        sellMarketCapTimeoutSeconds: String(entry.sellMarketCapTimeoutSeconds ?? "").trim(),
        sellMarketCapTimeoutAction: String(entry.sellMarketCapTimeoutAction || "").trim().toLowerCase() === "sell"
          ? "sell"
          : "stop",
      };
    });
}

function updateSniperAutosellWallet(envKey, patch = {}) {
  const state = normalizeSniperDraftState(sniperFeature.getState());
  const current = state.wallets && state.wallets[envKey] ? state.wallets[envKey] : {};
  if (!state.wallets) state.wallets = {};
  state.wallets[envKey] = {
    ...current,
    selected: true,
    ...patch,
  };
  sniperFeature.setState(state);
  sniperFeature.applyStateToForm();
  sniperFeature.renderUI();
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

quotePreviewDomain = QuotePreviewDomainModule.create ? QuotePreviewDomainModule.create({
  state: {
    getStartupWarmPayload: () => startupWarmState.backendPayload,
    getStoredPreviewInputs: () => previewInputsState,
  },
  helpers: {
    formatBigIntDecimal,
  },
}) : null;
previewInputsState = readStoredPreviewInputs();

function getQuickDevBuyPresetAmounts(config = latestWalletStatus && latestWalletStatus.config) {
  return settingsDomain.getQuickDevBuyPresetAmounts(config);
}

function renderQuickDevBuyButtons(config = latestWalletStatus && latestWalletStatus.config) {
  return settingsDomain.renderQuickDevBuyButtons(config);
}

function populateDevBuyPresetEditor(config = latestWalletStatus && latestWalletStatus.config) {
  return settingsDomain.populateDevBuyPresetEditor(config);
}

function setDevBuyPresetEditorOpen(isOpen) {
  return settingsDomain.setDevBuyPresetEditorOpen(isOpen);
}

function isDevBuyPresetEditorOpen() {
  return settingsDomain.isDevBuyPresetEditorOpen();
}

async function saveDevBuyPresetEditor() {
  return settingsDomain.saveDevBuyPresetEditor();
}

function cloneConfig(value) {
  return settingsDomain.cloneConfig(value);
}

function createFallbackConfig() {
  return settingsDomain.createFallbackConfig();
}

function getConfig() {
  return settingsDomain.getConfig();
}

function isTrackSendBlockHeightEnabled(config = getConfig()) {
  return settingsDomain.isTrackSendBlockHeightEnabled(config);
}

function getPresetItems(config = getConfig()) {
  return settingsDomain.getPresetItems(config);
}

function getActivePresetId(config = getConfig()) {
  return settingsDomain.getActivePresetId(config);
}

function getActivePreset(config = getConfig()) {
  return settingsDomain.getActivePreset(config);
}

function getPresetDisplayLabel(preset, index = 0) {
  return settingsDomain.getPresetDisplayLabel(preset, index);
}

function isPresetEditing(config = getConfig()) {
  return settingsDomain.isPresetEditing(config);
}

function setConfig(nextConfig) {
  return settingsDomain.setConfig(nextConfig);
}

function normalizeAutoFeeCapValue(value) {
  return settingsDomain.normalizeAutoFeeCapValue(value);
}

function setRegionRouting(nextRegionRouting) {
  return settingsDomain.setRegionRouting(nextRegionRouting);
}

function providerMinimumTipSol(provider) {
  return settingsDomain.providerMinimumTipSol(provider);
}

function providerRequiresPriorityFee(provider) {
  return settingsDomain.providerRequiresPriorityFee(provider);
}

function validateNonNegativeSolField(value) {
  return settingsDomain.validateNonNegativeSolField(value);
}

function validateRequiredPriorityFeeField(value, provider) {
  return settingsDomain.validateRequiredPriorityFeeField(value, provider);
}

function validateRequiredTipField(value, provider) {
  return settingsDomain.validateRequiredTipField(value, provider);
}

function validateOptionalAutoFeeCapField(value, provider) {
  return settingsDomain.validateOptionalAutoFeeCapField(value, provider);
}

function renderPlatformRuntimeIndicators() {
  walletRuntimeDomain.renderPlatformRuntimeIndicators();
}

function renderBackendRegionSummary(regionRouting = latestWalletStatus && latestWalletStatus.regionRouting) {
  return settingsDomain.renderBackendRegionSummary(regionRouting);
}

function renderPresetChips() {
  return settingsDomain.renderPresetChips();
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

function resolveLocalDevBuyQuote(shape) {
  if (!quotePreviewDomain || typeof quotePreviewDomain.computeLocalQuote !== "function") {
    return {
      quote: null,
      placeholder: "Preview unavailable until local preview data is ready.",
    };
  }
  return quotePreviewDomain.computeLocalQuote(shape) || {
    quote: null,
    placeholder: "Preview unavailable until local preview data is ready.",
  };
}

function renderDevBuyQuoteMessage(quote, mode, { provisional = false } = {}) {
  if (!quoteOutput || !quote) return;
  const quoteLabel = getQuoteAssetLabel(quote.quoteAsset || getQuoteAsset());
  const previewOnlyNote = quote.previewOnly ? " [preview only]" : "";
  quoteOutput.hidden = false;
  quoteOutput.textContent = mode === "sol"
    ? `${provisional ? "Preview: " : ""}Estimated tokens out: ${quote.estimatedTokens} (${quote.estimatedSupplyPercent}% supply)${previewOnlyNote}`
    : `${provisional ? "Preview: " : ""}Estimated ${quoteLabel} required: ${(quote.estimatedQuoteAmount || quote.estimatedSol)} for ${quote.estimatedSupplyPercent}% supply${previewOnlyNote}`;
}

function applyDevBuyPreviewPlaceholder(shape, message) {
  syncingDevBuyInputs = true;
  if (shape.mode === "sol") {
    setDevBuyPercentDisplay("");
  } else if (devBuySolInput) {
    devBuySolInput.value = "";
    devBuySolInput.placeholder = "0.0";
  }
  syncingDevBuyInputs = false;
  if (quoteOutput) {
    quoteOutput.hidden = false;
    quoteOutput.textContent = message;
  }
}

function renderProvisionalDevBuyPreview(shape) {
  if (!quoteOutput || !shape.amount) return;
  try {
    const result = resolveLocalDevBuyQuote(shape);
    if (result && result.quote) {
      applyDevBuyQuotePayload(result.quote, shape.mode);
      renderDevBuyQuoteMessage(result.quote, shape.mode, { provisional: true });
      return;
    }
    if (shape.mode === "tokens") {
      const percent = tokenAmountToPercent(shape.amount, shape.launchpad);
      if (percent) setDevBuyPercentDisplay(percent);
    }
    applyDevBuyPreviewPlaceholder(
      shape,
      result && result.placeholder ? result.placeholder : "Preview unavailable until local preview data is ready.",
    );
  } catch (error) {
    applyDevBuyPreviewPlaceholder(shape, error.message || "Preview unavailable.");
  }
}

function startupWarmPayloadHasPreviewBasis(payload) {
  if (!payload || typeof payload !== "object") return false;
  const pumpPreviewBasis = payload.pumpGlobal
    && typeof payload.pumpGlobal === "object"
    && payload.pumpGlobal.previewBasis
    && typeof payload.pumpGlobal.previewBasis === "object";
  const bonkLaunchDefaults = payload.bonkState
    && typeof payload.bonkState === "object"
    && payload.bonkState.previewBasis
    && typeof payload.bonkState.previewBasis === "object"
    && Array.isArray(payload.bonkState.previewBasis.launchDefaults);
  return Boolean(pumpPreviewBasis || bonkLaunchDefaults);
}

function readStoredPreviewInputs() {
  try {
    const raw = window.localStorage.getItem(PREVIEW_INPUTS_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    if (Number(parsed.schemaVersion) !== PREVIEW_INPUTS_SCHEMA_VERSION) return null;
    const payload = parsed.payload && typeof parsed.payload === "object" ? parsed.payload : null;
    if (!payload) return null;
    return quotePreviewDomain && typeof quotePreviewDomain.normalizePreviewInputs === "function"
      ? quotePreviewDomain.normalizePreviewInputs(payload)
      : payload;
  } catch (_error) {
    return null;
  }
}

function writeStoredPreviewInputs(payload) {
  try {
    if (!payload || typeof payload !== "object") {
      window.localStorage.removeItem(PREVIEW_INPUTS_STORAGE_KEY);
      return;
    }
    const normalized = quotePreviewDomain && typeof quotePreviewDomain.normalizePreviewInputs === "function"
      ? quotePreviewDomain.normalizePreviewInputs(payload)
      : payload;
    window.localStorage.setItem(PREVIEW_INPUTS_STORAGE_KEY, JSON.stringify({
      schemaVersion: PREVIEW_INPUTS_SCHEMA_VERSION,
      cachedAtMs: Date.now(),
      payload: normalized,
    }));
  } catch (_error) {
    // Ignore storage failures and keep boot flow functional.
  }
}

function mergePreviewInputs(nextPreviewInputs) {
  if (!quotePreviewDomain || typeof quotePreviewDomain.mergePreviewInputs !== "function") {
    return;
  }
  previewInputsState = quotePreviewDomain.mergePreviewInputs(previewInputsState, nextPreviewInputs);
  writeStoredPreviewInputs(previewInputsState);
}

function persistPreviewInputsFromStartupWarmPayload(payload) {
  if (!quotePreviewDomain || typeof quotePreviewDomain.extractPreviewInputsFromStartupWarm !== "function") {
    return;
  }
  mergePreviewInputs(quotePreviewDomain.extractPreviewInputsFromStartupWarm(payload));
}

function persistPreviewInputsFromRunReport(report) {
  if (!quotePreviewDomain || typeof quotePreviewDomain.captureBonkUsd1ApproxFromReport !== "function") {
    return;
  }
  const update = quotePreviewDomain.captureBonkUsd1ApproxFromReport(report);
  if (update) mergePreviewInputs(update);
}

function readStoredStartupWarmCache() {
  try {
    const raw = window.localStorage.getItem(STARTUP_WARM_CACHE_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    if (Number(parsed.schemaVersion) !== STARTUP_WARM_CACHE_SCHEMA_VERSION) return null;
    const cachedAtMs = Number(parsed.cachedAtMs);
    if (!Number.isFinite(cachedAtMs)) return null;
    if ((Date.now() - cachedAtMs) > STARTUP_WARM_CACHE_MAX_AGE_MS) return null;
    const payload = parsed.payload && typeof parsed.payload === "object" ? parsed.payload : null;
    if (!payload) return null;
    if (!startupWarmPayloadHasPreviewBasis(payload)) return null;
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
      schemaVersion: STARTUP_WARM_CACHE_SCHEMA_VERSION,
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
    window.localStorage.removeItem(LEGACY_STARTUP_WARM_CACHE_STORAGE_KEY);
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
  return key === "name:mayhemMode:value:";
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
  return liveSyncSupport.buildLiveSyncPayload();
}

function buildEarlyBootSnapshot(payload = buildLiveSyncPayload()) {
  return liveSyncSupport.buildEarlyBootSnapshot(payload);
}

function buildPersistedLiveSyncPayload(payload = buildLiveSyncPayload()) {
  return liveSyncSupport.buildPersistedLiveSyncPayload(payload);
}

function readStoredLiveSyncPayload() {
  return liveSyncSupport.readStoredLiveSyncPayload();
}

function readStoredEarlyBootSnapshot() {
  return liveSyncSupport.readStoredEarlyBootSnapshot();
}

function readSessionEarlyBootSnapshot() {
  return liveSyncSupport.readSessionEarlyBootSnapshot();
}

function readSessionLiveSyncPayload() {
  return liveSyncSupport.readSessionLiveSyncPayload();
}

function readOpenerLiveSyncPayload() {
  return liveSyncSupport.readOpenerLiveSyncPayload();
}

function dispatchLiveSyncPayload(payload) {
  liveSyncSupport.dispatchLiveSyncPayload(payload);
}

function scheduleLiveSyncBroadcast({ immediate = false } = {}) {
  if (!liveSyncSupport) return;
  liveSyncSupport.scheduleLiveSyncBroadcast({ immediate });
}

function applyLiveSyncFormControls(formControls) {
  liveSyncSupport.applyLiveSyncFormControls(formControls);
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
  return liveSyncSupport.applyIncomingLiveSyncPayload(payload, {
    allowBeforeReady,
    skipVisibilityState,
    skipDashboardViewState,
    skipThemeMode,
    skipFormControls,
    restorePersistedFormControlsOnly,
    restoreOutputFromSync,
  });
}

function enableLiveSync() {
  liveSyncSupport.enableLiveSync();
}

function preloadLiveSyncSnapshot() {
  return liveSyncSupport.preloadLiveSyncSnapshot();
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
  startupWarmState.started = true;
  const cachedWarm = readStoredStartupWarmCache();
  if (cachedWarm) {
    startupWarmState.backendLoaded = true;
    startupWarmState.backendPayload = cachedWarm.payload;
    startupWarmState.backendError = "";
    persistPreviewInputsFromStartupWarmPayload(cachedWarm.payload);
    queueQuoteUpdate();
  }
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
      if (response && response.ok && payload && typeof payload === "object" && startupWarmPayloadHasPreviewBasis(payload)) {
        writeStoredStartupWarmCache(payload);
        persistPreviewInputsFromStartupWarmPayload(payload);
      } else {
        clearStoredStartupWarmCache();
      }
      renderPlatformRuntimeIndicators();
      queueQuoteUpdate();
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
  startupWarmState.promise = Promise.allSettled([backendWarm]).finally(() => {
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

function getLaunchpadTokenDecimals(launchpad = getLaunchpad()) {
  return launchpad === "bagsapp" ? 9 : TOKEN_DECIMALS;
}

function percentToTokenAmount(percentValue, launchpad = getLaunchpad()) {
  const tokenDecimals = getLaunchpadTokenDecimals(launchpad);
  const percentRaw = parseDecimalToBigInt(percentValue, 4);
  const rawTokens = (TOTAL_SUPPLY_TOKENS * (10n ** BigInt(tokenDecimals)) * percentRaw) / 1_000_000n;
  return formatBigIntDecimal(rawTokens, tokenDecimals, tokenDecimals);
}

function tokenAmountToPercent(tokenAmount, launchpad = getLaunchpad()) {
  try {
    const tokenDecimals = getLaunchpadTokenDecimals(launchpad);
    const rawTokens = parseDecimalToBigInt(tokenAmount, tokenDecimals);
    if (rawTokens <= 0n) return "";
    const denominator = TOTAL_SUPPLY_TOKENS * (10n ** BigInt(tokenDecimals));
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
    const tokenAmount = percentToTokenAmount(percent, getLaunchpad());
    if (!tokenAmount || Number(percent) <= 0) {
      clearDevBuyState();
      return;
    }
    setDevBuyHiddenState("tokens", tokenAmount);
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

function setTickerCapsEnabled(enabled, { persist = false } = {}) {
  if (!tickerCapsToggle) return;
  const nextEnabled = Boolean(enabled);
  tickerCapsToggle.classList.toggle("active", nextEnabled);
  tickerCapsToggle.setAttribute("aria-pressed", nextEnabled ? "true" : "false");
  if (persist) {
    setStoredTickerCapsEnabled(nextEnabled);
    scheduleLiveSyncBroadcast({ immediate: true });
  }
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

function setMode(mode, { persist = false } = {}) {
  const target = normalizeLaunchModeForLaunchpad(mode, getLaunchpad());
  const next = form.querySelector(`input[name="mode"][value="${CSS.escape(target)}"]`)
    || form.querySelector('input[name="mode"][value="regular"]');
  if (!next) return;
  form.querySelectorAll('input[name="mode"]').forEach((input) => {
    input.checked = false;
  });
  next.checked = true;
  if (persist) setStoredLaunchMode(next.value);
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
  const rawCurrent = String(getNamedValue("quoteAsset") || "").trim().toLowerCase();
  const current = rawCurrent === "usd1" || rawCurrent === "sol" ? rawCurrent : "";
  const asset = visible
    ? normalizeQuoteAsset(stored || current || "sol")
    : normalizeQuoteAsset(stored || current || "sol");
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
  return "wallet-only";
}

function describeBagsIdentity() {
  return "Wallet Only";
}

function syncBagsIdentityUI() {
  // Bags launches are intentionally fixed to the connected wallet path.
}

function setBagsIdentityStateInputs(_value = {}) {
  // Bags identity is fixed to wallet-only now; keep a no-op shim for automation.
}

function setLaunchpad(launchpad, {
  resetMode = false,
  persistMode = false,
  persistLaunchpad = false,
  restoreScopedActions = false,
} = {}) {
  const normalized = normalizeLaunchpad(launchpad);
  const target = document.querySelector(`input[name="launchpad"][value="${CSS.escape(normalized)}"]`)
    || document.querySelector('input[name="launchpad"][value="pump"]');
  if (!target || target.disabled) return;
  document.querySelectorAll('input[name="launchpad"]').forEach((input) => {
    input.checked = false;
  });
  target.checked = true;
  if (persistLaunchpad) setStoredLaunchpad(normalized);
  splitEditorsDomain.setActiveFeeSplitDraftLaunchpad(normalized);
  getFeeSplitRows().forEach((row) => updateFeeSplitRowType(row, row.dataset.type || "wallet"));
  getAgentSplitRows().forEach((row) => {
    if (row.dataset.locked === "true") return;
    updateFeeSplitRowType(row, row.dataset.type || "wallet");
  });
  if (resetMode) {
    setImportedCreatorFeeState(null);
    const nextMode = getStoredLaunchMode(normalized) || defaultLaunchModeForLaunchpad(normalized);
    setMode(nextMode, { persist: persistMode });
  }
  if (restoreScopedActions) {
    hydrateModeActionState({ preferExistingFormFallback: false, launchpad: normalized });
    hydrateDevAutoSellState({ preferExistingFormFallback: false, launchpad: normalized });
  }
}

function applyImportedLaunchContext(token = {}) {
  const launchpad = normalizeLaunchpad(token.launchpad || getLaunchpad());
  withSuspendedFeeSplitDraftPersistence(() => {
    setLaunchpad(launchpad, { resetMode: true, persistMode: false });
  });
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
  const launchpad = getLaunchpad();
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
    const mode = getMode();
    const feeSplitVisible = feeSplit && (
      launchpad === "bagsapp"
      || (launchpad === "pump" && (mode === "regular" || mode === "agent-custom"))
    );
    feeSplitPill.hidden = !feeSplitVisible;
    if (!feeSplit && feeSplitEnabled) feeSplitEnabled.checked = false;
    if (!feeSplitVisible && feeSplitModal) feeSplitModal.hidden = true;
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
  if (devAutoSellPopover) devAutoSellPopover.hidden = !autoSell;
  triggerLaunchpadLayoutAnimation();
}

let lastLaunchpadLayoutAnimationKey = "";
let launchpadLayoutAnimationTimer = 0;

function triggerLaunchpadLayoutAnimation(nextLaunchpad = getLaunchpad()) {
  if (!launchSurfaceModeSection) return;
  const normalized = normalizeLaunchpad(nextLaunchpad);
  if (!normalized) return;
  if (!lastLaunchpadLayoutAnimationKey) {
    lastLaunchpadLayoutAnimationKey = normalized;
    return;
  }
  if (lastLaunchpadLayoutAnimationKey === normalized) return;
  lastLaunchpadLayoutAnimationKey = normalized;
  if (launchpadLayoutAnimationTimer) {
    window.clearTimeout(launchpadLayoutAnimationTimer);
    launchpadLayoutAnimationTimer = 0;
  }
  launchSurfaceModeSection.classList.remove("platform-switching");
  void launchSurfaceModeSection.offsetWidth;
  launchSurfaceModeSection.classList.add("platform-switching");
  launchpadLayoutAnimationTimer = window.setTimeout(() => {
    launchSurfaceModeSection.classList.remove("platform-switching");
    launchpadLayoutAnimationTimer = 0;
  }, 260);
}

function getProvider() {
  return settingsDomain.getProvider();
}

function getBuyProvider() {
  return settingsDomain.getBuyProvider();
}

function getSellProvider() {
  return settingsDomain.getSellProvider();
}

function getRouteCapabilities(route, rowType) {
  return settingsDomain.getRouteCapabilities(route, rowType);
}

function isHelloMoonProvider(provider) {
  return settingsDomain.isHelloMoonProvider(provider);
}

function defaultMevModeForProvider(provider) {
  return settingsDomain.defaultMevModeForProvider(provider);
}

function normalizeMevMode(value, fallback = "off") {
  return settingsDomain.normalizeMevMode(value, fallback);
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
  return settingsDomain.setMevModeSelectValue(select, value, fallback, provider);
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
  return settingsDomain.ensureStandardRpcSlippageDefault(input, provider);
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
  return settingsDomain.syncSettingsCapabilities();
}

function applyPresetToSettingsInputs(preset, options = {}) {
  return settingsDomain.applyPresetToSettingsInputs(preset, options);
}

function syncActivePresetFromInputs() {
  return settingsDomain.syncActivePresetFromInputs();
}

function setActivePreset(presetId, options = {}) {
  return settingsDomain.setActivePreset(presetId, options);
}

function setPresetEditing(editing) {
  return settingsDomain.setPresetEditing(editing);
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
      await selectImportedImage(payload.image);
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

function hydrateModeActionState({ preferExistingFormFallback = true } = {}) {
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
  const savedDraft = getSavedGlobalSniperDraft();
  if (savedDraft) {
    sniperFeature.setState(normalizeSniperDraftState(savedDraft));
    applySniperStateToForm();
    renderSniperUI();
    renderVanityButtonState();
    return;
  }
  if (!preferExistingFormFallback) {
    sniperFeature.setState({ enabled: false, wallets: {} });
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
          sellEnabled: Boolean(entry.sellEnabled),
          sellPercent: entry.sellPercent != null ? String(entry.sellPercent) : "",
          sellTriggerMode: String(entry.sellTriggerMode || "").trim().toLowerCase() === "market-cap"
            ? "market-cap"
            : "block-offset",
          sellTargetBlockOffset: entry.sellTargetBlockOffset != null ? Number(entry.sellTargetBlockOffset) : 0,
          sellMarketCapThreshold: entry.sellMarketCapThreshold || "",
          sellMarketCapTimeoutSeconds: entry.sellMarketCapTimeoutSeconds != null
            ? Number(entry.sellMarketCapTimeoutSeconds)
            : 30,
          sellMarketCapTimeoutAction: String(entry.sellMarketCapTimeoutAction || "").trim().toLowerCase() === "sell" ? "sell" : "stop",
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
  if (imageMetadataDomain && typeof imageMetadataDomain.setImagePreview === "function") {
    imageMetadataDomain.setImagePreview(previewUrl);
    return;
  }
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
  return settingsDomain.setSettingsLoadingState(isLoading);
}

function getStoredSelectedWalletKey() {
  return walletRuntimeDomain.getStoredSelectedWalletKey();
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
  walletRuntimeDomain.setStoredSelectedWalletKey(walletKey);
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

function walletDisplayName(wallet) {
  if (!wallet) return "No wallet";
  if (wallet.customName && String(wallet.customName).trim()) {
    return String(wallet.customName).trim();
  }
  const index = walletIndexFromEnvKey(wallet.envKey);
  return `#${index}`;
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

async function copyWalletDropdownAddress(button) {
  return walletRuntimeDomain.copyWalletDropdownAddress(button);
}

function setWalletDropdownOpen(isOpen) {
  walletRuntimeDomain.setWalletDropdownOpen(isOpen);
}

function toggleWalletDropdown() {
  walletRuntimeDomain.toggleWalletDropdown();
}

function connectedWalletText() {
  return latestWalletStatus && latestWalletStatus.wallet ? latestWalletStatus.wallet : "Connected wallet";
}

function shortenAddress(addr, chars = 6) {
  if (!addr || addr.length <= chars * 2 + 3) return addr;
  return addr.slice(0, chars) + "..." + addr.slice(-chars);
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
    if (host.includes("jito")) return host;
    const label = host.replace(/^www\./, "");
    return label.length > 24 ? shortenAddress(label, 10) : label;
  } catch (_error) {
    return raw.length > 24 ? shortenAddress(raw, 10) : raw;
  }
}

function formatLegendRecipientLabel(...args) {
  return splitEditorsDomain.formatLegendRecipientLabel(...args);
}

function setFeeSplitModalError(...args) {
  return splitEditorsDomain.setFeeSplitModalError(...args);
}

function formatFeeSplitTotalLabel(...args) {
  return splitEditorsDomain.formatFeeSplitTotalLabel(...args);
}

function formatPercentNumber(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return "0";
  return numeric.toFixed(2).replace(/\.00$/, "").replace(/(\.\d*[1-9])0$/, "$1");
}

function getDeployerFeeSplitAddress() {
  return String(latestWalletStatus && latestWalletStatus.wallet || "").trim();
}

function hasMeaningfulFeeSplitRecipients(...args) {
  return splitEditorsDomain.hasMeaningfulFeeSplitRecipients(...args);
}

function hasMeaningfulFeeSplitConfiguration(...args) {
  return splitEditorsDomain.hasMeaningfulFeeSplitConfiguration(...args);
}

function hasMeaningfulAgentSplitRecipients(...args) {
  return splitEditorsDomain.hasMeaningfulAgentSplitRecipients(...args);
}

function hasMeaningfulAgentSplitConfiguration(...args) {
  return splitEditorsDomain.hasMeaningfulAgentSplitConfiguration(...args);
}

function finalizeFeeSplitDraftForMode(...args) {
  return splitEditorsDomain.finalizeFeeSplitDraftForMode(...args);
}

function connectedWalletShort() {
  return latestWalletStatus && latestWalletStatus.wallet
    ? shortenAddress(latestWalletStatus.wallet)
    : "Connected wallet";
}

function getFeeSplitRows(...args) {
  return splitEditorsDomain.getFeeSplitRows(...args);
}

function createFeeSplitRow(...args) {
  return splitEditorsDomain.createFeeSplitRow(...args);
}

function updateFeeSplitRowType(...args) {
  return splitEditorsDomain.updateFeeSplitRowType(...args);
}

function setRecipientTargetLocked(...args) {
  return splitEditorsDomain.setRecipientTargetLocked(...args);
}

function ensureFeeSplitDefaultRow(...args) {
  return splitEditorsDomain.ensureFeeSplitDefaultRow(...args);
}

function formatFeeSplitRecipientProgress(...args) {
  return splitEditorsDomain.formatFeeSplitRecipientProgress(...args);
}

function syncFeeSplitPillSummary(...args) {
  return splitEditorsDomain.syncFeeSplitPillSummary(...args);
}

function syncFeeSplitTotals(...args) {
  return splitEditorsDomain.syncFeeSplitTotals(...args);
}

function updateFeeSplitVisibility(...args) {
  return splitEditorsDomain.updateFeeSplitVisibility(...args);
}

function showFeeSplitModal(...args) {
  return splitEditorsDomain.showFeeSplitModal(...args);
}

function hideFeeSplitModal(...args) {
  return splitEditorsDomain.hideFeeSplitModal(...args);
}

function attemptCloseFeeSplitModal(...args) {
  return splitEditorsDomain.attemptCloseFeeSplitModal(...args);
}

function cancelFeeSplitModal(...args) {
  return splitEditorsDomain.cancelFeeSplitModal(...args);
}

function createAgentSplitRow(...args) {
  return splitEditorsDomain.createAgentSplitRow(...args);
}

function getAgentSplitRows(...args) {
  return splitEditorsDomain.getAgentSplitRows(...args);
}

function syncAgentSplitTotals(...args) {
  return splitEditorsDomain.syncAgentSplitTotals(...args);
}

function initAgentSplitIfEmpty(...args) {
  return splitEditorsDomain.initAgentSplitIfEmpty(...args);
}

function showAgentSplitModal(...args) {
  return splitEditorsDomain.showAgentSplitModal(...args);
}

function hideAgentSplitModal(...args) {
  return splitEditorsDomain.hideAgentSplitModal(...args);
}

function cancelAgentSplitModal(...args) {
  return splitEditorsDomain.cancelAgentSplitModal(...args);
}

function setAgentSplitModalError(...args) {
  return splitEditorsDomain.setAgentSplitModalError(...args);
}

function seedAgentSplitFromFeeSplit(...args) {
  return splitEditorsDomain.seedAgentSplitFromFeeSplit(...args);
}

function resetAgentSplitToDefault(...args) {
  return splitEditorsDomain.resetAgentSplitToDefault(...args);
}

function attemptCloseAgentSplitModal(...args) {
  return splitEditorsDomain.attemptCloseAgentSplitModal(...args);
}

function normalizeAgentSplitStructure(...args) {
  return splitEditorsDomain.normalizeAgentSplitStructure(...args);
}

function collectAgentSplitRecipients(...args) {
  return splitEditorsDomain.collectAgentSplitRecipients(...args);
}

function updateLockedModeFields(...args) {
  return splitEditorsDomain.updateLockedModeFields(...args);
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
  return settingsDomain.applyProviderAvailability(providers);
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
  setStoredLaunchpad(getLaunchpad());
  setStoredLaunchMode(getMode());
}

function applyPersistentDefaults(config) {
  if (defaultsApplied) return;
  const hydratedConfig = latestWalletStatus && latestWalletStatus.config
    ? JSON.parse(JSON.stringify(latestWalletStatus.config))
    : null;
  const resolvedConfig = hydratedConfig || config;
  if (!resolvedConfig) return;
  const defaults = resolvedConfig.defaults || {};
  const defaultsMisc = defaults.misc && typeof defaults.misc === "object" ? defaults.misc : {};
  setImportedCreatorFeeState(null);
  const defaultMode = defaults.mode || "regular";
  const storedLaunchpad = getStoredLaunchpad();
  const storedBonkQuoteAsset = getStoredBonkQuoteAsset();
  const resolvedLaunchpad = storedLaunchpad || defaults.launchpad || "pump";
  const storedSniperDraft = getStoredSniperDraft();
  const storedMode = getStoredLaunchMode(resolvedLaunchpad);
  const storedFeeSplitDraft = getStoredFeeSplitDraft(resolvedLaunchpad);
  const storedAgentSplitDraft = getStoredAgentSplitDraft(resolvedLaunchpad);
  const storedAutoSellDraft = getStoredAutoSellDraft();
  const resolvedMode = storedMode || defaultMode;
  const defaultsLaunchpad = normalizeLaunchpad(defaults.launchpad || resolvedLaunchpad);
  const defaultScopedFeeSplitDraft =
    defaultsMisc.feeSplitDraftsByLaunchpad
      && typeof defaultsMisc.feeSplitDraftsByLaunchpad === "object"
      && defaultsMisc.feeSplitDraftsByLaunchpad[resolvedLaunchpad]
      ? defaultsMisc.feeSplitDraftsByLaunchpad[resolvedLaunchpad]
      : (defaultsLaunchpad === resolvedLaunchpad && defaultsMisc.feeSplitDraft
        ? defaultsMisc.feeSplitDraft
        : null);
  const defaultScopedAgentSplitDraft =
    defaultsMisc.agentSplitDraftsByLaunchpad
      && typeof defaultsMisc.agentSplitDraftsByLaunchpad === "object"
      && defaultsMisc.agentSplitDraftsByLaunchpad[resolvedLaunchpad]
      ? defaultsMisc.agentSplitDraftsByLaunchpad[resolvedLaunchpad]
      : (defaultsLaunchpad === resolvedLaunchpad && defaultsMisc.agentSplitDraft
        ? defaultsMisc.agentSplitDraft
        : null);
  const defaultScopedSniperDraft = getSavedGlobalSniperDraft(resolvedConfig);
  const defaultScopedAutoSellDraft = getSavedGlobalAutoSellDraft(resolvedConfig);
  const resolvedFeeSplitDraft = storedFeeSplitDraft || defaultScopedFeeSplitDraft || null;
  const resolvedAgentSplitDraft = storedAgentSplitDraft || defaultScopedAgentSplitDraft || null;
  setLaunchpad(resolvedLaunchpad, { persistLaunchpad: true });
  if (bonkQuoteAssetInput) bonkQuoteAssetInput.value = normalizeQuoteAsset(storedBonkQuoteAsset || "sol");
  setConfig(resolvedConfig);
  applyPresetToSettingsInputs(getActivePreset(resolvedConfig));
  if (storedAutoSellDraft) {
    applyAutoSellDraft(storedAutoSellDraft, { persist: false });
  } else if (defaultScopedAutoSellDraft) {
    applyAutoSellDraft(buildAutoSellDraftFromDefaults(defaultScopedAutoSellDraft), { persist: false });
  }
  setMode(resolvedMode, { persist: true });
  if (resolvedLaunchpad === "bonk" && bonkQuoteAssetInput && storedBonkQuoteAsset) {
    bonkQuoteAssetInput.value = normalizeQuoteAsset(storedBonkQuoteAsset);
    syncBonkQuoteAssetUI();
  }
  setStoredLaunchpad(getLaunchpad());
  setStoredLaunchMode(getMode());
  applyFeeSplitDraft(resolvedFeeSplitDraft, { persist: false });
  applyAgentSplitDraft(resolvedAgentSplitDraft, { persist: false });
  setPresetEditing(Boolean(defaults.presetEditing));
  if (!storedSniperDraft && defaultScopedSniperDraft) {
    sniperFeature.setState(normalizeSniperDraftState(defaultScopedSniperDraft));
    applySniperStateToForm();
  }
  renderQuickDevBuyButtons(resolvedConfig);
  populateDevBuyPresetEditor(resolvedConfig);
  defaultsApplied = true;
}

function collectFeeSplitRecipients(...args) {
  return splitEditorsDomain.collectFeeSplitRecipients(...args);
}

function collectSubmittedFeeSplitRecipients(...args) {
  return splitEditorsDomain.collectSubmittedFeeSplitRecipients(...args);
}

function readForm() {
  return formDomain.readForm();
}

function metadataFingerprintFromForm(formValues = readForm()) {
  return imageMetadataDomain
    ? imageMetadataDomain.metadataFingerprintFromForm(formValues)
    : formDomain.metadataFingerprintFromForm(formValues);
}

function canPreuploadMetadata(formValues = readForm()) {
  return imageMetadataDomain
    ? imageMetadataDomain.canPreuploadMetadata(formValues)
    : formDomain.canPreuploadMetadata(formValues);
}

function hasFreshPreuploadedMetadata(formValues = readForm()) {
  return imageMetadataDomain
    ? imageMetadataDomain.hasFreshPreuploadedMetadata(formValues)
    : formDomain.hasFreshPreuploadedMetadata(formValues);
}

function clearMetadataUploadCache({ clearInput = false } = {}) {
  if (!imageMetadataDomain) return;
  imageMetadataDomain.clearMetadataUploadCache({ clearInput });
}

function markMetadataUploadDirty() {
  if (!imageMetadataDomain) return;
  imageMetadataDomain.markMetadataUploadDirty();
}

function surfaceMetadataWarning(warning) {
  if (!imageMetadataDomain) return;
  imageMetadataDomain.surfaceMetadataWarning(warning);
}

function scheduleMetadataPreupload({ immediate = false } = {}) {
  if (!imageMetadataDomain) return;
  imageMetadataDomain.scheduleMetadataPreupload({ immediate });
}

async function ensureMetadataReadyForAction(action) {
  if (!imageMetadataDomain) return;
  return imageMetadataDomain.ensureMetadataReadyForAction(action);
}

function applySelectedWalletLocally(nextKey) {
  walletRuntimeDomain.applySelectedWalletLocally(nextKey);
}

async function refreshWalletStatus(preserveSelection = true, force = false) {
  return walletRuntimeDomain.refreshWalletStatus(preserveSelection, force);
}

function applyBootstrapFastPayload(payload) {
  walletRuntimeDomain.applyBootstrapFastPayload(payload);
}

function applyRuntimeStatusPayload(payload, { hydrateOnly = false } = {}) {
  walletRuntimeDomain.applyRuntimeStatusPayload(payload, { hydrateOnly });
}

function currentWarmActivityPayload() {
  return walletRuntimeDomain.currentWarmActivityPayload();
}

async function flushWarmActivity() {
  return walletRuntimeDomain.flushWarmActivity();
}

function queueWarmActivity({ immediate = false } = {}) {
  walletRuntimeDomain.queueWarmActivity({ immediate });
}

function startRuntimeStatusRefreshLoop() {
  walletRuntimeDomain.startRuntimeStatusRefreshLoop();
}

function clearFollowJobsRefreshTimer() {
  walletRuntimeDomain.clearFollowJobsRefreshTimer();
}

function followStatusSnapshot() {
  return walletRuntimeDomain.followStatusSnapshot();
}

function buildFollowJobsSummaryText(snapshot = followStatusSnapshot()) {
  return walletRuntimeDomain.buildFollowJobsSummaryText(snapshot);
}

function syncFollowStatusChrome() {
  walletRuntimeDomain.syncFollowStatusChrome();
}

async function refreshFollowJobs({ silent = false } = {}) {
  return walletRuntimeDomain.refreshFollowJobs({ silent });
}

async function cancelFollowJob(traceId, { note = "" } = {}) {
  return walletRuntimeDomain.cancelFollowJob(traceId, { note });
}

async function cancelAllFollowJobs() {
  return walletRuntimeDomain.cancelAllFollowJobs();
}

function activeFollowJobForTraceId(traceId) {
  return walletRuntimeDomain.activeFollowJobForTraceId(traceId);
}

function applyWalletStatusPayload(payload) {
  walletRuntimeDomain.applyWalletStatusPayload(payload);
}

async function bootstrapApp() {
  return runtimeActions.bootstrapApp();
}

async function refreshRuntimeStatus() {
  return runtimeActions.refreshRuntimeStatus();
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
    const result = resolveLocalDevBuyQuote(shape);
    if (result && result.quote) {
      applyDevBuyQuotePayload(result.quote, shape.mode);
      renderDevBuyQuoteMessage(result.quote, shape.mode);
      return;
    }
    applyDevBuyPreviewPlaceholder(
      shape,
      result && result.placeholder ? result.placeholder : "Preview unavailable until local preview data is ready.",
    );
  } catch (error) {
    applyDevBuyPreviewPlaceholder(shape, error.message || "Preview unavailable.");
  }
}

function queueQuoteUpdate() {
  updateQuote().catch((error) => {
    if (quoteOutput) quoteOutput.textContent = error.message;
  });
}

async function uploadSelectedImage(file) {
  if (!imageMetadataDomain) return;
  return imageMetadataDomain.uploadSelectedImage(file);
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
      const response = await fetch("/images/solana-mark.png");
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
  return settingsDomain.validateSettingsModalBeforeSave();
}

function validateProviderFeeFields(scope) {
  return settingsDomain.validateProviderFeeFields(scope);
}

function validateAgentSplit(...args) {
  return splitEditorsDomain.validateAgentSplit(...args);
}

function validateFeeSplit(...args) {
  return splitEditorsDomain.validateFeeSplit(...args);
}

function validateForm() {
  const errors = [];
  const f = readForm();
  const launchpadCapabilities = getLaunchpadUiCapabilities(f.launchpad);
  if (!f.name.trim()) errors.push("Token name is required.");
  if (!f.symbol.trim()) errors.push("Ticker is required.");
  if (!hasAttachedImage()) errors.push("Token image is required.");
  if (!latestWalletStatus || !latestWalletStatus.connected) errors.push("No wallet connected.");
  if (f.automaticDevSellEnabled && !f.devBuyAmount) errors.push("Dev auto-sell requires a dev buy amount.");
  if (launchpadCapabilities.sniper) {
    validateSniperState().forEach((msg) => errors.push(msg));
  }
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
      if (isSocialRecipientType(entry.type)) {
        const provider = recipientTypeLabel(entry.type);
        return `${provider} @${entry.githubUsername} ${share}%`;
      }
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
  const sniperAutosellCount = f.automaticSniperSellEnabled
    ? (f.sniperWallets || []).filter((entry) => entry && entry.sellEnabled).length
    : 0;
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
    { label: "Sniper", value: `${sniperText}${f.automaticSniperSellEnabled ? ` | autosell wallets ${sniperAutosellCount}` : " | autosell off"}`, cls: "secondary" },
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
  return runtimeActions.run(action);
}

function buildSavedConfigFromForm() {
  return formDomain.buildSavedConfigFromForm();
}

async function saveSettings() {
  return runtimeActions.saveSettings();
}

function showSettingsModal() {
  return settingsDomain.showSettingsModal();
}

function hideSettingsModal(reason = "dismiss") {
  return settingsDomain.hideSettingsModal(reason);
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
  return reportsPresenters.normalizeReportsTerminalView(view);
}

function normalizeActiveLogsView(view) {
  return reportsPresenters.normalizeActiveLogsView(view);
}

function reportsTerminalMetaText(view = reportsTerminalState.view) {
  if (!reportsPresenters) return "Latest 25 transactions.";
  return reportsPresenters.reportsTerminalMetaText(view);
}

function syncReportsTerminalChrome() {
  if (!reportsPresenters) return;
  reportsPresenters.syncReportsTerminalChrome();
}

function syncReportsTerminalLayoutHeight() {
  if (!reportsPresenters) return;
  reportsPresenters.syncReportsTerminalLayoutHeight();
}

async function fetchLaunchMetadataSummary(metadataUriValue) {
  return reportsHistory.fetchLaunchMetadataSummary(metadataUriValue);
}

async function fetchReportBundleForLaunch(id) {
  return reportsHistory.fetchReportBundleForLaunch(id);
}

function getLaunchHistoryEntry(id) {
  return reportsHistory.getLaunchHistoryEntry(id);
}

function buildLaunchHistoryEntry(entry, bundle, metadata) {
  return reportsHistory.buildLaunchHistoryEntry(entry, bundle, metadata);
}

async function loadReportsTerminalLaunches() {
  return reportsHistory.loadReportsTerminalLaunches();
}

function describeReportEntry(entry) {
  return reportsPresenters.describeReportEntry
    ? reportsPresenters.describeReportEntry(entry)
    : "";
}

function captureFrozenBenchmarkSnapshot(reportId, payload) {
  reportsPresenters.captureFrozenBenchmarkSnapshot(reportId, payload);
}

function applyFrozenBenchmarkSnapshot(reportId, payload) {
  return reportsPresenters.applyFrozenBenchmarkSnapshot(reportId, payload);
}

function normalizeReportsTerminalTab(tab) {
  return reportsPresenters.normalizeReportsTerminalTab(tab);
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

function renderBenchmarksPopoutModal() {
  reportsPresenters.renderBenchmarksPopoutModal();
}

function renderReportsTerminalOutput() {
  reportsPresenters.renderReportsTerminalOutput();
}

function renderReportsTerminalList() {
  reportsPresenters.renderReportsTerminalList();
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
      sellEnabled: Boolean(entry.postBuySell && entry.postBuySell.enabled),
      sellPercent: entry.postBuySell && entry.postBuySell.percent != null
        ? String(entry.postBuySell.percent)
        : "",
      sellTriggerMode: entry.postBuySell && entry.postBuySell.marketCap && entry.postBuySell.marketCap.threshold
        ? "market-cap"
        : "block-offset",
      sellTargetBlockOffset: entry.postBuySell && entry.postBuySell.targetBlockOffset != null
        ? entry.postBuySell.targetBlockOffset
        : 0,
      sellMarketCapThreshold: entry.postBuySell && entry.postBuySell.marketCap && entry.postBuySell.marketCap.threshold
        ? String(entry.postBuySell.marketCap.threshold)
        : "",
      sellMarketCapTimeoutSeconds: entry.postBuySell && entry.postBuySell.marketCap
        && (entry.postBuySell.marketCap.scanTimeoutSeconds != null || entry.postBuySell.marketCap.scanTimeoutMinutes != null)
        ? (entry.postBuySell.marketCap.scanTimeoutSeconds != null
          ? entry.postBuySell.marketCap.scanTimeoutSeconds
          : entry.postBuySell.marketCap.scanTimeoutMinutes * 60)
        : 30,
      sellMarketCapTimeoutAction: entry.postBuySell
        && entry.postBuySell.marketCap
        && String(entry.postBuySell.marketCap.timeoutAction || "").trim().toLowerCase() === "sell"
        ? "sell"
        : "stop",
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
  syncLaunchpadModeOptions();
  syncBonkQuoteAssetUI();
  syncBagsIdentityUI();

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

  restoreLaunchHistoryImage(launch);

  const devBuy = launch.devBuy || { mode: "sol", amount: "" };
  setDevBuyHiddenState(devBuy.mode, devBuy.amount);
  syncingDevBuyInputs = true;
  if (devBuySolInput) devBuySolInput.value = devBuy.mode === "sol" ? devBuy.amount : "";
  if (devBuyPercentInput) {
    devBuyPercentInput.value = devBuy.mode === "tokens"
      ? tokenAmountToPercent(devBuy.amount, launch.launchpad || getLaunchpad())
      : "";
  }
  syncingDevBuyInputs = false;

  const sniperWallets = buildSniperWalletStateFromLaunch(launch);
  const hasSniperAutosell = Object.values(sniperWallets).some((entry) => entry && entry.sellEnabled);
  setNamedChecked("automaticSniperSellEnabled", hasSniperAutosell);
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

function setBootOverlayMessage(title, note, { animate = true } = {}) {
  if (!bootOverlay) return;
  if (title != null && title !== "") {
    const titleNode = bootOverlay.querySelector(".boot-overlay-title");
    if (titleNode) titleNode.textContent = title;
  }
  if (note != null) {
    const noteNode = bootOverlay.querySelector(".boot-overlay-note");
    const noteTextNode = noteNode ? noteNode.querySelector(".boot-overlay-note-text") : null;
    if (noteTextNode) {
      noteTextNode.textContent = note;
    } else if (noteNode) {
      noteNode.textContent = note;
    }
    if (noteNode) {
      noteNode.classList.toggle("is-animating", Boolean(note) && animate);
    }
  }
}

function completeInitialBoot() {
  if (window.__launchdeckBootFallback) {
    window.clearTimeout(window.__launchdeckBootFallback);
    window.__launchdeckBootFallback = null;
  }
  setBootOverlayMessage("LaunchDeck ready", "Opening workspace", { animate: false });
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

const localBinders = LocalBindersModule.create ? LocalBindersModule.create({
  elements: {
    form,
    launchpadInputs,
    bonkQuoteAssetToggle,
    bonkQuoteAssetInput,
    nameInput,
    descriptionToggle,
    descriptionInput,
    symbolInput,
    websiteInput,
    twitterInput,
    telegramInput,
    tickerCapsToggle,
    changeDevBuyPresetsButton,
    cancelDevBuyPresetsButton,
    saveDevBuyPresetsButton,
    devBuySolInput,
    devBuyPercentInput,
    providerSelect,
    creationMevModeSelect,
    buyProviderSelect,
    buyMevModeSelect,
    buySlippageInput,
    sellProviderSelect,
    sellMevModeSelect,
    sellSlippageInput,
    feeSplitPill,
    feeSplitEnabled,
    walletTriggerButton,
    walletRefreshButton,
    walletDropdownList,
    walletSelect,
    feeSplitAdd,
    feeSplitReset,
    feeSplitEven,
    feeSplitClearAll,
    feeSplitList,
    agentSplitAdd,
    agentSplitReset,
    agentSplitEven,
    agentSplitClearAll,
    agentSplitList,
    imageLayoutToggle,
    tokenSurfaceSection,
    imageInput,
    imageStatus,
    testFillButton,
    openVampButton,
    themeToggleButton,
    openSettingsButton,
    saveSettingsButton,
    buttons,
    modalClose,
    modalCancel,
    modalConfirm,
    benchmarksPopoutClose,
    benchmarksPopoutModal,
    settingsCancel,
    topPresetChipBar,
    settingsPresetChipBar,
    presetEditToggle,
    creationAutoFeeButton,
    creationAutoFeeInput,
    buyAutoFeeButton,
    buyAutoFeeInput,
    sellAutoFeeButton,
    sellAutoFeeInput,
    settingsInputs: [
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
    ],
    devBuyQuickButtons,
    devBuyCustomDeployButton,
    modeVanityButton,
    feeSplitClose,
    feeSplitSave,
    feeSplitDisable,
    feeSplitModal,
    agentSplitClose,
    agentSplitCancel,
    agentSplitSave,
    agentSplitModal,
    vanitySave,
    vanityPrivateKeyText,
    vanityModalError,
    vanityClear,
    vanityClose,
    vanityModal,
    vampImport,
    vampClose,
    vampCancel,
    vampContractInput,
    vampError,
    vampModal,
    deployModal,
    sniperModal,
  },
  constants: {
    defaultPresetId: DEFAULT_PRESET_ID,
    maxFeeSplitRecipients: MAX_FEE_SPLIT_RECIPIENTS,
  },
  fieldValidatorNames: Object.keys(fieldValidators),
  state: {
    getActiveFeeSplitDraftLaunchpad: () => splitEditorsDomain.getActiveFeeSplitDraftLaunchpad(),
    getFeeSplitClearAllRestoreSnapshot: () => splitEditorsDomain.getFeeSplitClearAllRestoreSnapshot(),
    setFeeSplitClearAllRestoreSnapshot: (value) => {
      splitEditorsDomain.setFeeSplitClearAllRestoreSnapshot(value);
    },
    getAgentSplitClearAllRestoreSnapshot: () => splitEditorsDomain.getAgentSplitClearAllRestoreSnapshot(),
    setAgentSplitClearAllRestoreSnapshot: (value) => {
      splitEditorsDomain.setAgentSplitClearAllRestoreSnapshot(value);
    },
    getTickerManuallyEdited: () => tickerManuallyEdited,
    setTickerManuallyEdited: (value) => {
      tickerManuallyEdited = value;
    },
    getSyncingTickerFromName: () => syncingTickerFromName,
    setSyncingTickerFromName: (value) => {
      syncingTickerFromName = value;
    },
    setTickerClearedForManualEntry: (value) => {
      tickerClearedForManualEntry = value;
    },
    isSyncingDevBuyInputs: () => syncingDevBuyInputs,
    getDevBuyPresetEditorOpen: () => isDevBuyPresetEditorOpen(),
    getLastDevBuyEditSource: () => lastDevBuyEditSource,
  },
  actions: {
    getMode,
    setStoredLaunchMode,
    updateModeVisibility,
    normalizeLaunchpad,
    setStoredFeeSplitDraft,
    serializeFeeSplitDraft,
    withSuspendedFeeSplitDraftPersistence,
    setStoredLaunchpad,
    setLaunchpad,
    restoreFeeSplitDraftForLaunchpad,
    applyLaunchpadTokenMetadata,
    getQuoteAsset,
    setStoredBonkQuoteAsset,
    syncBonkQuoteAssetUI,
    queueQuoteUpdate,
    syncTickerFromName,
    markMetadataUploadDirty,
    scheduleMetadataPreupload,
    toggleDescriptionDisclosure,
    updateDescriptionDisclosure,
    getAutoTickerValue,
    formatTickerValue,
    updateTokenFieldCounts,
    applyTickerCapsMode,
    isTickerCapsEnabled,
    setTickerCapsEnabled,
    setDevBuyPresetEditorOpen,
    populateDevBuyPresetEditor,
    getConfig,
    saveDevBuyPresetEditor,
    updateDevBuyFromSolInput,
    updateDevBuyFromPercentInput,
    isHelloMoonProvider,
    getProvider,
    setMevModeSelectValue,
    defaultMevModeForProvider,
    normalizeMevMode,
    syncActivePresetFromInputs,
    updateJitoVisibility,
    validateProviderFeeFields,
    getBuyProvider,
    ensureStandardRpcSlippageDefault,
    getSellProvider,
    showFeeSplitModal,
    hideSettingsModal,
    toggleWalletDropdown,
    refreshWalletStatus,
    copyWalletDropdownAddress,
    setWalletDropdownOpen,
    setStoredSelectedWalletKey,
    applySelectedWalletLocally,
    selectedWalletKey,
    clearFeeSplitRestoreState,
    getFeeSplitRows,
    createFeeSplitRow,
    updateFeeSplitRowValidationUi,
    syncFeeSplitTotals,
    syncFeeSplitModalPresentation,
    setFeeSplitModalError,
    setRecipientTargetLocked,
    scheduleFeeSplitLookup,
    updateFeeSplitRowType,
    clearFeeSplitRowState,
    usesImplicitCreatorShareMode,
    ensureFeeSplitDefaultRow,
    normalizeFeeSplitDraft,
    feeSplitClearAllDraft,
    applyFeeSplitDraft,
    updateFeeSplitClearAllButton,
    updateAgentSplitClearAllButton,
    clearAgentSplitRestoreState,
    getAgentSplitRows,
    createAgentSplitRow,
    syncAgentSplitTotals,
    setStoredAgentSplitDraft,
    serializeAgentSplitDraft,
    setAgentSplitModalError,
    normalizeAgentSplitDraft,
    agentSplitClearAllDraft,
    applyAgentSplitDraft,
    normalizeAgentSplitStructure,
    applyTestPreset,
    showVampModal,
    setThemeMode,
    showSettingsModal,
    saveSettings,
    validateForm,
    showValidationErrors,
    clearValidationErrors,
    showDeployModal,
    run,
    hideDeployModal,
    hideBenchmarksPopoutModal,
    setActivePreset,
    isPresetEditing,
    setPresetEditing,
    syncSettingsCapabilities,
    validateFieldByName,
    getNamedInput,
    clearDevBuyState,
    copyBagsResolvedWallet,
    renderSniperUI,
    hydrateModeActionState,
    hydrateDevAutoSellState,
    getDevBuyMode,
    triggerDeployWithDevBuy,
    attemptCloseFeeSplitModal,
    cancelFeeSplitModal,
    attemptCloseAgentSplitModal,
    cancelAgentSplitModal,
    resetAgentSplitToDefault,
    hideAgentSplitModal,
    showVanityModal,
    validateVanityPrivateKey,
    applyVanityValue,
    hideVanityModal,
    importVampToken,
    hideVampModal,
    scheduleVampAutoImport,
    setImageLayoutCompact,
    uploadSelectedImage,
    scheduleLiveSyncBroadcast: (options) => scheduleLiveSyncBroadcast(options),
  },
}) : null;

if (localBinders && typeof localBinders.bindEvents === "function") {
  localBinders.bindEvents();
}

form.querySelectorAll('input[name="launchpad"]').forEach((node) => {
  node.addEventListener("change", () => {
    if (!node.checked) return;
    const nextLaunchpad = normalizeLaunchpad(node.value);
    window.setTimeout(() => {
      hydrateModeActionState({ preferExistingFormFallback: false, launchpad: nextLaunchpad });
      hydrateDevAutoSellState({ preferExistingFormFallback: false, launchpad: nextLaunchpad });
    }, 0);
  });
});

document.addEventListener("click", (event) => {
  if (!walletDropdown || walletDropdown.hidden) return;
  const target = event.target;
  if (walletBox && walletBox.contains(target)) return;
  setWalletDropdownOpen(false);
});

const initialStoredLaunchpad = getStoredLaunchpad();
if (initialStoredLaunchpad) {
  setLaunchpad(initialStoredLaunchpad, {
    resetMode: true,
    persistMode: false,
    restoreScopedActions: true,
  });
}

updateModeVisibility();
setTickerCapsEnabled(getStoredTickerCapsEnabled());
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
    const restoredLaunchpad = getStoredLaunchpad();
    if (restoredLaunchpad && getLaunchpad() !== restoredLaunchpad) {
      setLaunchpad(restoredLaunchpad, {
        resetMode: true,
        persistMode: false,
        restoreScopedActions: true,
      });
    }
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
    setBootOverlayMessage(
      "LaunchDeck failed to load",
      error.message || "Refresh the page and check the backend runtime.",
      { animate: false },
    );
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

liveSyncSupport.bindGlobalListeners();
