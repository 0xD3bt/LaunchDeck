(function initLaunchDeckSplitEditorsDomain(global) {
  function create(config) {
    const {
      elements = {},
      constants = {},
      helpers = {},
    } = config || {};

    const {
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
    } = elements;

    const {
      legacyFeeSplitDraftStorageKey = "launchdeck.feeSplitDraft.v1",
      agentSplitDraftStorageKey = "launchdeck.agentSplitDraft.v1",
      initialFeeSplitDraftLaunchpad = "pump",
      maxFeeSplitRecipients = 10,
      splitColors = [],
      bagsFeeSplitVisibleCardCount = 5,
      bagsFeeSplitViewportBufferPx = 4,
    } = constants;

    const {
      feeRouting = {},
      getLaunchpad = () => "pump",
      getMode = () => "regular",
      normalizeLaunchpad = (value) => String(value || "").trim().toLowerCase(),
      normalizeLaunchMode = (value) => String(value || "").trim().toLowerCase(),
      normalizeDecimalInput = (value) => String(value || "").trim(),
      escapeHTML = (value) => String(value || ""),
      shortenAddress = (value) => String(value || ""),
      formatPercentNumber = (value) => String(value || "0"),
      parseGithubRecipientTarget = () => ({ githubUsername: "", githubUserId: "" }),
      looksLikeSolanaAddress = () => false,
      getDeployerFeeSplitAddress = () => "",
      syncSettingsCapabilities = () => {},
      scheduleLiveSyncBroadcast = () => {},
    } = helpers;

    let activeFeeSplitDraftLaunchpad = normalizeLaunchpad(initialFeeSplitDraftLaunchpad);
    let suspendFeeSplitDraftPersistence = false;
    let feeSplitModalSnapshot = null;
    let agentSplitModalSnapshot = null;
    let feeSplitClearAllRestoreSnapshot = null;
    let feeSplitRowSerial = 0;
    const bagsFeeSplitLookupCache = new Map();
    const bagsFeeSplitLookupTimers = new Map();
    const bagsFeeSplitLookupState = new Map();
    let agentSplitClearAllRestoreSnapshot = null;

    function normalizeRecipientType(type, options = {}) {
      return typeof feeRouting.normalizeRecipientType === "function"
        ? feeRouting.normalizeRecipientType(type, options)
        : String(type || "").trim().toLowerCase();
    }

    function isRecipientTypeSupportedForLaunchpad(type, launchpad = getLaunchpad(), options = {}) {
      return typeof feeRouting.isRecipientTypeSupportedForLaunchpad === "function"
        ? feeRouting.isRecipientTypeSupportedForLaunchpad(type, launchpad, options)
        : normalizeRecipientType(type, options) === "wallet";
    }

    function isSocialRecipientType(type) {
      return typeof feeRouting.isSocialRecipientType === "function"
        ? feeRouting.isSocialRecipientType(type)
        : false;
    }

    function recipientTypeLabel(type) {
      return typeof feeRouting.recipientTypeLabel === "function"
        ? feeRouting.recipientTypeLabel(type)
        : "Wallet";
    }

    function recipientTypeTabsMarkup() {
      return typeof feeRouting.recipientTypeTabsMarkup === "function"
        ? feeRouting.recipientTypeTabsMarkup()
        : "";
    }

    function syncRecipientTypeTabVisibility(row) {
      if (typeof feeRouting.syncRecipientTypeTabVisibility === "function") {
        feeRouting.syncRecipientTypeTabVisibility(row);
      }
    }

    function recipientTargetPlaceholder(type) {
      return typeof feeRouting.recipientTargetPlaceholder === "function"
        ? feeRouting.recipientTargetPlaceholder(type)
        : "Wallet address";
    }

    function recipientTypeIconMarkup(type) {
      return typeof feeRouting.recipientTypeIconMarkup === "function"
        ? feeRouting.recipientTypeIconMarkup(type)
        : "";
    }

    function isBagsFeeSplitLaunchpad(launchpad = getLaunchpad()) {
      return normalizeLaunchpad(launchpad) === "bagsapp";
    }

    function isPumpFeeSplitLaunchpad(launchpad = getLaunchpad()) {
      return normalizeLaunchpad(launchpad) === "pump";
    }

    function usesImplicitBagsCreatorShareMode(mode = getMode(), launchpad = getLaunchpad()) {
      return isBagsFeeSplitLaunchpad(launchpad) && String(mode || "").trim().startsWith("bags-");
    }

    function usesImplicitPumpCreatorShareMode(mode = getMode(), launchpad = getLaunchpad()) {
      return isPumpFeeSplitLaunchpad(launchpad) && normalizeLaunchMode(mode) === "regular";
    }

    function usesImplicitCreatorShareMode(mode = getMode(), launchpad = getLaunchpad()) {
      return usesImplicitBagsCreatorShareMode(mode, launchpad) || usesImplicitPumpCreatorShareMode(mode, launchpad);
    }

    function getFeeSplitRows() {
      return feeSplitList ? Array.from(feeSplitList.querySelectorAll(".fee-split-row")) : [];
    }

    function getAgentSplitRows() {
      return agentSplitList ? Array.from(agentSplitList.querySelectorAll(".fee-split-row")) : [];
    }

    function nextFeeSplitRowId() {
      feeSplitRowSerial += 1;
      return `fee-split-row-${feeSplitRowSerial}`;
    }

    function getFeeSplitRowState(row) {
      if (!row) return null;
      return bagsFeeSplitLookupState.get(String(row.dataset.rowId || "").trim()) || null;
    }

    function clearFeeSplitRowLookupTimer(rowId) {
      const activeTimer = bagsFeeSplitLookupTimers.get(rowId);
      if (activeTimer) {
        global.clearTimeout(activeTimer);
        bagsFeeSplitLookupTimers.delete(rowId);
      }
    }

    function clearFeeSplitRowState(row) {
      if (!row) return;
      const rowId = String(row.dataset.rowId || "").trim();
      if (!rowId) return;
      clearFeeSplitRowLookupTimer(rowId);
      bagsFeeSplitLookupState.delete(rowId);
      delete row.dataset.validationState;
    }

    function normalizeFeeSplitLookupState(value) {
      if (!value || typeof value !== "object") return null;
      const status = String(value.status || "").trim();
      const rawLookup = value.lookup && typeof value.lookup === "object" ? value.lookup : {};
      const normalized = {
        status: ["idle", "checking", "valid", "invalid"].includes(status) ? status : "idle",
        key: String(value.key || rawLookup.cacheKey || "").trim(),
        lookup: {
          provider: String(rawLookup.provider || "").trim(),
          username: String(rawLookup.username || "").trim(),
          githubUserId: String(rawLookup.githubUserId || "").trim(),
          lookupTarget: String(rawLookup.lookupTarget || "").trim(),
          cacheKey: String(rawLookup.cacheKey || "").trim(),
          wallet: String(rawLookup.wallet || "").trim(),
          resolvedUsername: String(rawLookup.resolvedUsername || "").trim(),
          error: String(rawLookup.error || "").trim(),
          found: Boolean(rawLookup.found),
          notFound: Boolean(rawLookup.notFound),
        },
        wallet: String(value.wallet || rawLookup.wallet || "").trim(),
        message: String(value.message || "").trim(),
      };
      return normalized.key || normalized.wallet || normalized.message ? normalized : null;
    }

    function serializeFeeSplitLookupState(row) {
      return normalizeFeeSplitLookupState(getFeeSplitRowState(row));
    }

    function restoreFeeSplitLookupState(row, value) {
      if (!row) return;
      const rowId = String(row.dataset.rowId || "").trim();
      if (!rowId) return;
      const normalized = normalizeFeeSplitLookupState(value);
      if (!normalized) return;
      bagsFeeSplitLookupState.set(rowId, normalized);
      if (normalized.key) {
        bagsFeeSplitLookupCache.set(normalized.key, normalized);
      }
    }

    function validateFeeSplitSocialTarget(type, value, { githubUserId = "" } = {}) {
      const normalizedType = normalizeRecipientType(type);
      const rawValue = String(value || "").trim();
      if (!isSocialRecipientType(normalizedType)) {
        return {
          valid: true,
          empty: !rawValue,
          username: "",
          githubUserId: "",
          lookupTarget: "",
          error: "",
        };
      }
      const providerLabel = recipientTypeLabel(normalizedType);
      if (normalizedType === "github") {
        const parsedGithubTarget = parseGithubRecipientTarget(rawValue);
        const resolvedGithubUserId = String(githubUserId || parsedGithubTarget.githubUserId || "").trim();
        if (resolvedGithubUserId && !parsedGithubTarget.githubUsername) {
          return {
            valid: true,
            empty: false,
            username: "",
            githubUserId: resolvedGithubUserId,
            lookupTarget: resolvedGithubUserId,
            error: "",
          };
        }
        const username = parsedGithubTarget.githubUsername;
        if (!username) {
          return {
            valid: false,
            empty: true,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "",
          };
        }
        if (/\s/.test(username)) {
          return { valid: false, empty: false, username: "", githubUserId: "", lookupTarget: "", error: "GitHub usernames cannot contain spaces." };
        }
        if (username.length > 39) {
          return { valid: false, empty: false, username: "", githubUserId: "", lookupTarget: "", error: "GitHub usernames can be at most 39 characters." };
        }
        if (!/^[A-Za-z0-9-]+$/.test(username)) {
          return { valid: false, empty: false, username: "", githubUserId: "", lookupTarget: "", error: "GitHub usernames can only use letters, numbers, and hyphens." };
        }
        if (username.startsWith("-") || username.endsWith("-")) {
          return { valid: false, empty: false, username: "", githubUserId: "", lookupTarget: "", error: "GitHub usernames cannot start or end with a hyphen." };
        }
        if (username.includes("--")) {
          return { valid: false, empty: false, username: "", githubUserId: "", lookupTarget: "", error: "GitHub usernames cannot contain consecutive hyphens." };
        }
        return {
          valid: true,
          empty: false,
          username,
          githubUserId: "",
          lookupTarget: username,
          error: "",
        };
      }
      const username = rawValue.replace(/^@+/, "");
      if (!username) {
        return {
          valid: false,
          empty: true,
          username: "",
          githubUserId: "",
          lookupTarget: "",
          error: "",
        };
      }
      if (/\s/.test(username)) {
        return {
          valid: false,
          empty: false,
          username: "",
          githubUserId: "",
          lookupTarget: "",
          error: `${providerLabel} usernames cannot contain spaces.`,
        };
      }
      if (normalizedType === "twitter") {
        if (username.length < 4 || username.length > 15) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "X usernames must be between 4 and 15 characters.",
          };
        }
        if (!/^[A-Za-z0-9_]+$/.test(username)) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "X usernames can only use letters, numbers, and underscores.",
          };
        }
      } else if (normalizedType === "kick") {
        if (username.length < 4 || username.length > 24) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "Kick usernames must be between 4 and 24 characters.",
          };
        }
        if (!/^[A-Za-z0-9_-]+$/.test(username)) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "Kick usernames can only use letters, numbers, hyphens, and underscores.",
          };
        }
      } else if (normalizedType === "tiktok") {
        if (username.length < 2 || username.length > 24) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "TikTok usernames must be between 2 and 24 characters.",
          };
        }
        if (!/^[A-Za-z0-9._]+$/.test(username)) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "TikTok usernames can only use letters, numbers, periods, and underscores.",
          };
        }
        if (/^[._]|[._]$/.test(username)) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "TikTok usernames cannot start or end with a period or underscore.",
          };
        }
        if (username.includes("..")) {
          return {
            valid: false,
            empty: false,
            username: "",
            githubUserId: "",
            lookupTarget: "",
            error: "TikTok usernames cannot contain consecutive periods.",
          };
        }
      }
      return {
        valid: true,
        empty: false,
        username,
        githubUserId: "",
        lookupTarget: username,
        error: "",
      };
    }

    function buildFeeSplitLookupDescriptor(row) {
      if (!row) return null;
      const type = normalizeRecipientType(row.dataset.type);
      if (!isSocialRecipientType(type)) return null;
      const value = String(row.querySelector(".recipient-target")?.value || "").trim();
      const githubUserIdFromDataset = String(row.dataset.githubUserId || "").trim();
      const validation = validateFeeSplitSocialTarget(type, value, { githubUserId: githubUserIdFromDataset });
      if (!validation.valid || validation.empty || !validation.lookupTarget) return null;
      return {
        provider: type,
        username: validation.username,
        githubUserId: validation.githubUserId,
        lookupTarget: validation.lookupTarget,
        cacheKey: `${type}:${validation.githubUserId}:${validation.username}`,
      };
    }

    function describeFeeSplitLookupFailure(descriptor, lookup = {}) {
      const providerLabel = recipientTypeLabel(descriptor.provider);
      const targetLabel = descriptor.provider === "github" && descriptor.githubUserId
        ? `GitHub id ${descriptor.githubUserId}`
        : `${providerLabel} @${descriptor.username || descriptor.lookupTarget}`;
      if (lookup && lookup.notFound) {
        return `${targetLabel} is not linked to a Bags launch wallet.`;
      }
      return String((lookup && lookup.error) || "").trim() || `${targetLabel} could not be validated with Bags.`;
    }

    function formatLegendRecipientLabel(type, value, fallback = "wallet", { compactSocialLabel = false } = {}) {
      const normalized = String(value || "").trim();
      if (!normalized) {
        return { full: fallback, short: fallback };
      }
      if (isSocialRecipientType(type)) {
        const handle = `@${normalized.replace(/^@+/, "")}`;
        const full = compactSocialLabel
          ? handle
          : (recipientTypeLabel(type) === "GitHub" ? handle : `${recipientTypeLabel(type)} ${handle}`);
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

    function formatFeeSplitTotalLabel(total) {
      return `Total: ${total.toFixed(2).replace(/\.00$/, "")}%`;
    }

    function setFeeSplitModalError(message = "") {
      if (feeSplitModalError) feeSplitModalError.textContent = message;
    }

    function setAgentSplitModalError(message = "") {
      if (agentSplitModalError) agentSplitModalError.textContent = message;
    }

    function syncBagsFeeSplitSummary() {
      if (!bagsFeeSplitSummary) return;
      const isBags = isBagsFeeSplitLaunchpad();
      const isImplicitPump = usesImplicitPumpCreatorShareMode();
      const showSummary = isBags || isImplicitPump;
      bagsFeeSplitSummary.hidden = !showSummary;
      if (!showSummary) return;
      const rows = getFeeSplitRows();
      const total = rows.reduce((sum, row) => sum + (Number(row.querySelector(".recipient-share")?.value || 0) || 0), 0);
      const creatorShare = Math.max(0, Number((100 - total).toFixed(2)));
      const shared = Math.max(0, total);
      if (feeSplitSummaryPrimaryLabel) {
        feeSplitSummaryPrimaryLabel.textContent = isBags ? "You Keep" : "Your Share";
      }
      if (feeSplitSummarySecondaryLabel) {
        feeSplitSummarySecondaryLabel.textContent = isBags ? "Others Get" : "Recipients Get";
      }
      if (bagsFeeSplitCreatorShare) {
        bagsFeeSplitCreatorShare.textContent = `${creatorShare.toFixed(2).replace(/\.00$/, "")}%`;
      }
      if (bagsFeeSplitSharedShare) {
        bagsFeeSplitSharedShare.textContent = `${shared.toFixed(2).replace(/\.00$/, "")}%`;
      }
      if (!isBags) return;
      const socialRows = rows.filter((row) => isSocialRecipientType(row.dataset.type));
      const currentState = socialRows.reduce((accumulator, row) => {
        const targetValidation = validateFeeSplitSocialTarget(
          row.dataset.type,
          String(row.querySelector(".recipient-target")?.value || "").trim(),
          { githubUserId: String(row.dataset.githubUserId || "").trim() },
        );
        const descriptor = buildFeeSplitLookupDescriptor(row);
        const state = getFeeSplitRowState(row);
        if (!targetValidation.empty && targetValidation.error) {
          accumulator.invalid += 1;
          return accumulator;
        }
        if (!descriptor) {
          accumulator.pending += 1;
          return accumulator;
        }
        if (!state || state.key !== descriptor.cacheKey) {
          accumulator.pending += 1;
          return accumulator;
        }
        if (state.status === "valid") accumulator.valid += 1;
        else if (state.status === "invalid") accumulator.invalid += 1;
        else accumulator.pending += 1;
        return accumulator;
      }, { valid: 0, invalid: 0, pending: 0 });
      if (bagsFeeSplitValidationCopy) {
        if (socialRows.length === 0) {
          bagsFeeSplitValidationCopy.textContent = "Raw usernames are checked against Bags before deploy.";
        } else if (currentState.invalid > 0) {
          bagsFeeSplitValidationCopy.textContent = `${currentState.invalid} recipient${currentState.invalid === 1 ? "" : "s"} need attention before deploy.`;
        } else if (currentState.pending > 0) {
          bagsFeeSplitValidationCopy.textContent = `${currentState.valid} validated, ${currentState.pending} still checking.`;
        } else {
          bagsFeeSplitValidationCopy.textContent = `${currentState.valid} social recipient${currentState.valid === 1 ? "" : "s"} validated with Bags.`;
        }
      }
    }

    function removeImplicitCreatorRows() {
      if (!usesImplicitCreatorShareMode() || !feeSplitList) return false;
      const defaultRows = getFeeSplitRows().filter((row) => row.dataset.defaultReceiver === "true");
      if (defaultRows.length === 0) return false;
      defaultRows.forEach((row) => {
        clearFeeSplitRowState(row);
        row.remove();
      });
      return true;
    }

    function syncCompactFeeSplitListViewport() {
      if (!feeSplitList) return;
      const usesCompactPresentation = usesImplicitCreatorShareMode();
      if (!usesCompactPresentation) {
        feeSplitList.classList.remove("bags-fee-split-list-scrollable");
        feeSplitList.style.maxHeight = "";
        feeSplitList.style.overflowY = "";
        return;
      }
      const rows = getFeeSplitRows();
      if (!rows.length) {
        feeSplitList.classList.remove("bags-fee-split-list-scrollable");
        feeSplitList.style.maxHeight = "";
        feeSplitList.style.overflowY = "";
        return;
      }
      if (!feeSplitModal || feeSplitModal.hidden || feeSplitList.getBoundingClientRect().width === 0) return;
      const listStyles = global.getComputedStyle(feeSplitList);
      const rowGap = Number.parseFloat(listStyles.rowGap || listStyles.gap || "0") || 0;
      const visibleRows = rows.slice(0, bagsFeeSplitVisibleCardCount);
      const maxHeight = visibleRows.reduce((sum, row) => sum + row.getBoundingClientRect().height, 0)
        + Math.max(visibleRows.length - 1, 0) * rowGap
        + bagsFeeSplitViewportBufferPx;
      feeSplitList.style.maxHeight = `${Math.ceil(maxHeight)}px`;
      const shouldScroll = rows.length > bagsFeeSplitVisibleCardCount;
      feeSplitList.classList.toggle("bags-fee-split-list-scrollable", shouldScroll);
      feeSplitList.style.overflowY = shouldScroll ? "auto" : "hidden";
    }

    function syncAgentSplitListViewport() {
      if (!agentSplitList) return;
      const rows = getAgentSplitRows();
      if (!rows.length) {
        agentSplitList.classList.remove("bags-fee-split-list-scrollable");
        agentSplitList.style.maxHeight = "";
        agentSplitList.style.overflowY = "";
        return;
      }
      if (!agentSplitModal || agentSplitModal.hidden || agentSplitList.getBoundingClientRect().width === 0) return;
      const listStyles = global.getComputedStyle(agentSplitList);
      const rowGap = Number.parseFloat(listStyles.rowGap || listStyles.gap || "0") || 0;
      const visibleRows = rows.slice(0, bagsFeeSplitVisibleCardCount);
      const maxHeight = visibleRows.reduce((sum, row) => sum + row.getBoundingClientRect().height, 0)
        + Math.max(visibleRows.length - 1, 0) * rowGap
        + bagsFeeSplitViewportBufferPx;
      agentSplitList.style.maxHeight = `${Math.ceil(maxHeight)}px`;
      const shouldScroll = rows.length > bagsFeeSplitVisibleCardCount;
      agentSplitList.classList.toggle("bags-fee-split-list-scrollable", shouldScroll);
      agentSplitList.style.overflowY = shouldScroll ? "auto" : "hidden";
    }

    async function copyBagsResolvedWallet(button) {
      const value = String(button?.dataset.copyValue || "").trim();
      if (!value) return;
      try {
        if (navigator.clipboard?.writeText) {
          await navigator.clipboard.writeText(value);
        } else {
          const probe = document.createElement("textarea");
          probe.value = value;
          probe.setAttribute("readonly", "");
          probe.style.position = "absolute";
          probe.style.left = "-9999px";
          document.body.appendChild(probe);
          probe.select();
          document.execCommand("copy");
          probe.remove();
        }
        if (button._copyFeedbackTimer) {
          global.clearTimeout(button._copyFeedbackTimer);
        }
        button.classList.remove("is-copied");
        void button.offsetWidth;
        button.classList.add("is-copied");
        button.title = "Copied";
        button.setAttribute("aria-label", "Copied resolved wallet");
        button._copyFeedbackTimer = global.setTimeout(() => {
          button.classList.remove("is-copied");
          button.title = "Copy resolved wallet";
          button.setAttribute("aria-label", "Copy resolved wallet");
          button._copyFeedbackTimer = null;
        }, 1200);
      } catch {}
    }

    function updateFeeSplitRowValidationUi(row) {
      if (!row) return;
      const isBags = isBagsFeeSplitLaunchpad();
      const isWallet = normalizeRecipientType(row.dataset.type) === "wallet";
      const targetValue = String(row.querySelector(".recipient-target")?.value || "").trim();
      const providerChip = row.querySelector(".bags-fee-row-chip");
      const statusChip = row.querySelector(".bags-fee-row-status");
      const meta = row.querySelector(".bags-fee-row-meta");
      const copyButton = row.querySelector(".bags-fee-row-copy");
      if (providerChip) {
        providerChip.hidden = false;
        providerChip.innerHTML = `${recipientTypeIconMarkup(row.dataset.type)}<span class="bags-fee-row-chip-label">${escapeHTML(recipientTypeLabel(row.dataset.type))}</span>`;
        providerChip.title = recipientTypeLabel(row.dataset.type);
      }
      if (meta) meta.hidden = !isBags || isWallet;
      if (!isBags) {
        delete row.dataset.validationState;
        if (statusChip) statusChip.textContent = "";
        if (copyButton) {
          copyButton.hidden = true;
          copyButton.dataset.copyValue = "";
          copyButton.classList.remove("is-copied");
        }
        syncCompactFeeSplitListViewport();
        return;
      }
      const descriptor = buildFeeSplitLookupDescriptor(row);
      const targetValidation = validateFeeSplitSocialTarget(row.dataset.type, targetValue, {
        githubUserId: String(row.dataset.githubUserId || "").trim(),
      });
      const state = getFeeSplitRowState(row);
      let status = "idle";
      let copyValue = "";
      let message = row.dataset.type === "github"
        ? "GitHub username or id"
        : `${recipientTypeLabel(row.dataset.type)} username`;
      if (isWallet) {
        row.dataset.validationState = "idle";
        if (statusChip) {
          statusChip.textContent = "";
          statusChip.title = "";
        }
        if (copyButton) {
          copyButton.hidden = true;
          copyButton.dataset.copyValue = "";
          copyButton.classList.remove("is-copied");
        }
        syncCompactFeeSplitListViewport();
        syncBagsFeeSplitSummary();
        return;
      }
      if (!targetValidation.empty && targetValidation.error) {
        status = "invalid";
        message = targetValidation.error;
      } else if (descriptor && state && state.key === descriptor.cacheKey) {
        status = state.status || "idle";
        if (status === "checking") {
          message = "Checking...";
        } else if (status === "valid") {
          if (providerChip) providerChip.hidden = true;
          message = state.wallet ? `Resolved to ${shortenAddress(state.wallet)}` : "Validated with Bags.";
          copyValue = state.wallet || "";
        } else if (status === "invalid") {
          message = state.message || describeFeeSplitLookupFailure(descriptor, state.lookup || {});
        }
      } else if (descriptor) {
        message = "Pending validation";
      }
      row.dataset.validationState = status;
      if (statusChip) {
        statusChip.textContent = message;
        statusChip.title = message;
      }
      if (copyButton) {
        copyButton.hidden = !copyValue;
        copyButton.dataset.copyValue = copyValue;
        if (!copyValue) {
          copyButton.classList.remove("is-copied");
          copyButton.title = "Copy resolved wallet";
          copyButton.setAttribute("aria-label", "Copy resolved wallet");
        }
      }
      syncCompactFeeSplitListViewport();
      syncBagsFeeSplitSummary();
    }

    async function runFeeSplitLookup(row, descriptor) {
      if (!row || !descriptor || !isBagsFeeSplitLaunchpad()) return;
      const rowId = String(row.dataset.rowId || "").trim();
      if (!rowId) return;
      const setState = (nextState) => {
        bagsFeeSplitLookupState.set(rowId, nextState);
        updateFeeSplitRowValidationUi(row);
      };
      if (bagsFeeSplitLookupCache.has(descriptor.cacheKey)) {
        setState(bagsFeeSplitLookupCache.get(descriptor.cacheKey));
        return;
      }
      setState({ status: "checking", key: descriptor.cacheKey, lookup: descriptor, wallet: "", message: "Checking Bags wallet..." });
      const url = new URL("/api/bags/fee-recipient-lookup", global.location.origin);
      url.searchParams.set("provider", descriptor.provider);
      if (descriptor.username) url.searchParams.set("username", descriptor.username);
      if (descriptor.githubUserId) url.searchParams.set("githubUserId", descriptor.githubUserId);
      try {
        const response = await fetch(url.toString(), { headers: { Accept: "application/json" } });
        const payload = await response.json().catch(() => ({}));
        if (!response.ok || !payload.ok) {
          throw new Error(payload.error || "Failed to validate Bags recipient.");
        }
        const lookup = payload.lookup || {};
        const nextState = lookup && lookup.found
          ? {
            status: "valid",
            key: descriptor.cacheKey,
            lookup,
            wallet: String(lookup.wallet || "").trim(),
            message: String(lookup.wallet || "").trim(),
          }
          : {
            status: "invalid",
            key: descriptor.cacheKey,
            lookup,
            wallet: "",
            message: describeFeeSplitLookupFailure(descriptor, lookup),
          };
        bagsFeeSplitLookupCache.set(descriptor.cacheKey, nextState);
        if (row.isConnected && row.dataset.rowId === rowId) {
          const currentDescriptor = buildFeeSplitLookupDescriptor(row);
          if (currentDescriptor && currentDescriptor.cacheKey === descriptor.cacheKey) {
            setState(nextState);
          }
        }
      } catch (error) {
        const nextState = {
          status: "invalid",
          key: descriptor.cacheKey,
          lookup: descriptor,
          wallet: "",
          message: error && error.message ? error.message : "Failed to validate Bags recipient.",
        };
        if (row.isConnected && row.dataset.rowId === rowId) {
          const currentDescriptor = buildFeeSplitLookupDescriptor(row);
          if (currentDescriptor && currentDescriptor.cacheKey === descriptor.cacheKey) {
            setState(nextState);
          }
        }
      }
    }

    function scheduleFeeSplitLookup(row, { immediate = false } = {}) {
      if (!row) return;
      const rowId = String(row.dataset.rowId || "").trim();
      if (!rowId) return;
      clearFeeSplitRowLookupTimer(rowId);
      if (!isBagsFeeSplitLaunchpad()) {
        clearFeeSplitRowState(row);
        updateFeeSplitRowValidationUi(row);
        return;
      }
      const targetValidation = validateFeeSplitSocialTarget(
        row.dataset.type,
        String(row.querySelector(".recipient-target")?.value || "").trim(),
        { githubUserId: String(row.dataset.githubUserId || "").trim() },
      );
      if (!targetValidation.valid || targetValidation.empty) {
        bagsFeeSplitLookupState.delete(rowId);
        updateFeeSplitRowValidationUi(row);
        return;
      }
      const descriptor = buildFeeSplitLookupDescriptor(row);
      if (!descriptor) {
        bagsFeeSplitLookupState.delete(rowId);
        updateFeeSplitRowValidationUi(row);
        return;
      }
      if (bagsFeeSplitLookupCache.has(descriptor.cacheKey)) {
        bagsFeeSplitLookupState.set(rowId, bagsFeeSplitLookupCache.get(descriptor.cacheKey));
        updateFeeSplitRowValidationUi(row);
        return;
      }
      bagsFeeSplitLookupState.set(rowId, {
        status: "checking",
        key: descriptor.cacheKey,
        lookup: descriptor,
        wallet: "",
        message: "Checking Bags wallet...",
      });
      updateFeeSplitRowValidationUi(row);
      const timer = global.setTimeout(() => {
        bagsFeeSplitLookupTimers.delete(rowId);
        runFeeSplitLookup(row, descriptor);
      }, immediate ? 0 : 350);
      bagsFeeSplitLookupTimers.set(rowId, timer);
    }

    function serializeFeeSplitDraft(launchpad = currentFeeSplitDraftLaunchpad()) {
      const implicitCreatorShare = usesImplicitCreatorShareMode();
      return {
        launchpad: currentFeeSplitDraftLaunchpad(launchpad),
        enabled: Boolean(feeSplitEnabled && feeSplitEnabled.checked),
        suppressDefaultRow: feeSplitList ? feeSplitList.dataset.suppressDefaultRow === "true" : false,
        rows: getFeeSplitRows()
          .map((row) => ({
            type: row.dataset.type || "wallet",
            value: row.querySelector(".recipient-target")?.value?.trim() || "",
            githubUserId: row.dataset.githubUserId || "",
            sharePercent: row.querySelector(".recipient-share")?.value?.trim() || "",
            defaultReceiver: row.dataset.defaultReceiver === "true",
            targetLocked: row.dataset.targetLocked === "true",
            lookupState: serializeFeeSplitLookupState(row),
          }))
          .filter((row) => !implicitCreatorShare || !row.defaultReceiver),
      };
    }

    function normalizeFeeSplitDraft(value) {
      const rows = Array.isArray(value && value.rows)
        ? value.rows.map((entry) => ({
          type: normalizeRecipientType(entry && entry.type),
          value: String((entry && entry.value) || "").trim(),
          githubUserId: String((entry && entry.githubUserId) || "").trim(),
          sharePercent: normalizeDecimalInput((entry && entry.sharePercent) || "", 2),
          defaultReceiver: Boolean(entry && entry.defaultReceiver),
          targetLocked: Boolean(entry && entry.targetLocked),
          lookupState: normalizeFeeSplitLookupState(entry && entry.lookupState),
        }))
        : [];
      return {
        launchpad: String(value && value.launchpad || "").trim().toLowerCase(),
        enabled: Boolean(value && value.enabled),
        suppressDefaultRow: Boolean(value && value.suppressDefaultRow),
        rows,
      };
    }

    function currentFeeSplitDraftLaunchpad(launchpad = "") {
      return normalizeLaunchpad(launchpad || getLaunchpad() || activeFeeSplitDraftLaunchpad);
    }

    function filterFeeSplitDraftRowsForLaunchpad(rows, launchpad = currentFeeSplitDraftLaunchpad()) {
      const normalizedLaunchpad = normalizeLaunchpad(launchpad);
      return (Array.isArray(rows) ? rows : []).filter((row) => {
        if (!row || typeof row !== "object") return false;
        if (row.defaultReceiver) return true;
        return isRecipientTypeSupportedForLaunchpad(row.type, normalizedLaunchpad);
      });
    }

    function stripImplicitCreatorRowsFromFeeSplitDraftRows(rows, launchpad = currentFeeSplitDraftLaunchpad()) {
      const normalizedLaunchpad = normalizeLaunchpad(launchpad);
      const normalizedRows = Array.isArray(rows) ? rows.filter(Boolean) : [];
      if (normalizedLaunchpad !== "bagsapp" && normalizedLaunchpad !== "pump") {
        return normalizedRows;
      }
      const deployerAddress = getDeployerFeeSplitAddress();
      return normalizedRows.filter((row) => {
        if (row.defaultReceiver) return false;
        if (
          normalizedLaunchpad === "pump"
          && normalizeRecipientType(row.type) === "wallet"
          && row.targetLocked
          && deployerAddress
          && String(row.value || "").trim() === deployerAddress
        ) {
          return false;
        }
        return true;
      });
    }

    function normalizeFeeSplitDraftForLaunchpad(value, launchpad = currentFeeSplitDraftLaunchpad()) {
      const normalizedLaunchpad = normalizeLaunchpad(launchpad);
      const draft = normalizeFeeSplitDraft(value);
      if (draft.launchpad && draft.launchpad !== normalizedLaunchpad) {
        return normalizeFeeSplitDraft({
          launchpad: normalizedLaunchpad,
          enabled: false,
          suppressDefaultRow: false,
          rows: [],
        });
      }
      if (normalizedLaunchpad !== "bagsapp" && normalizedLaunchpad !== "pump") {
        return normalizeFeeSplitDraft({
          launchpad: normalizedLaunchpad,
          enabled: false,
          suppressDefaultRow: false,
          rows: [],
        });
      }
      const rows = stripImplicitCreatorRowsFromFeeSplitDraftRows(
        filterFeeSplitDraftRowsForLaunchpad(draft.rows, normalizedLaunchpad),
        normalizedLaunchpad,
      );
      return normalizeFeeSplitDraft({
        launchpad: normalizedLaunchpad,
        enabled: normalizedLaunchpad === "pump" ? (draft.enabled && rows.length > 0) : draft.enabled,
        suppressDefaultRow: false,
        rows,
      });
    }

    function feeSplitDraftStorageKey(launchpad = currentFeeSplitDraftLaunchpad()) {
      return `launchdeck.feeSplitDraft.${normalizeLaunchpad(launchpad)}.v2`;
    }

    function feeSplitDraftSessionStorageKey(launchpad = currentFeeSplitDraftLaunchpad()) {
      return `launchdeck.feeSplitDraft.${normalizeLaunchpad(launchpad)}.session.v2`;
    }

    function getStoredFeeSplitDraft(launchpad = currentFeeSplitDraftLaunchpad()) {
      try {
        const sessionKey = feeSplitDraftSessionStorageKey(launchpad);
        const localKey = feeSplitDraftStorageKey(launchpad);
        const keys = [
          { storage: global.sessionStorage, key: sessionKey, scopedLaunchpad: normalizeLaunchpad(launchpad) },
          { storage: global.localStorage, key: localKey, scopedLaunchpad: normalizeLaunchpad(launchpad) },
        ];
        for (const key of keys) {
          const raw = key.storage.getItem(key.key);
          if (!raw) continue;
          const parsed = JSON.parse(raw);
          if (!parsed || typeof parsed !== "object") {
            continue;
          }
          const hydrated = String(parsed.launchpad || "").trim()
            ? parsed
            : {
              ...parsed,
              launchpad: key.scopedLaunchpad,
            };
          const normalized = normalizeFeeSplitDraftForLaunchpad(hydrated, launchpad);
          if (key.key !== sessionKey) {
            global.sessionStorage.setItem(sessionKey, JSON.stringify(normalized));
          }
          if (key.key !== localKey) {
            global.localStorage.setItem(localKey, JSON.stringify(normalized));
          }
          return normalized;
        }
        return null;
      } catch (_error) {
        return null;
      }
    }

    function setStoredFeeSplitDraft(value, { launchpad = currentFeeSplitDraftLaunchpad() } = {}) {
      if (suspendFeeSplitDraftPersistence) return;
      try {
        const normalized = normalizeFeeSplitDraftForLaunchpad(value, launchpad);
        const key = feeSplitDraftStorageKey(launchpad);
        const sessionKey = feeSplitDraftSessionStorageKey(launchpad);
        if (!normalized.enabled && normalized.rows.length === 0) {
          global.sessionStorage.removeItem(sessionKey);
          global.localStorage.removeItem(key);
          return;
        }
        global.sessionStorage.setItem(sessionKey, JSON.stringify(normalized));
        global.localStorage.setItem(key, JSON.stringify(normalized));
      } catch (_error) {
        // Ignore storage failures and keep fee split controls functional.
      }
    }

    function withSuspendedFeeSplitDraftPersistence(callback) {
      const previous = suspendFeeSplitDraftPersistence;
      suspendFeeSplitDraftPersistence = true;
      try {
        return callback();
      } finally {
        suspendFeeSplitDraftPersistence = previous;
      }
    }

    function restoreFeeSplitDraftForLaunchpad(launchpad) {
      activeFeeSplitDraftLaunchpad = normalizeLaunchpad(launchpad);
      applyFeeSplitDraft(getStoredFeeSplitDraft(activeFeeSplitDraftLaunchpad), { persist: false });
      updateFeeSplitVisibility();
    }

    function updateFeeSplitRowType(row, type) {
      if (!row) return;
      row.dataset.type = normalizeRecipientType(type);
      if (row.dataset.type !== "github") delete row.dataset.githubUserId;
      clearFeeSplitRowState(row);
      syncRecipientTypeTabVisibility(row);
      row.querySelectorAll(".recipient-type-tab").forEach((button) => {
        button.classList.toggle("active", button.dataset.type === row.dataset.type);
      });
      const target = row.querySelector(".recipient-target");
      if (target) target.placeholder = recipientTargetPlaceholder(row.dataset.type);
      updateFeeSplitRowValidationUi(row);
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
        if (isSocialRecipientType(row.dataset.type) && looksLikeSolanaAddress(target.value)) {
          target.setCustomValidity(
            row.dataset.type === "github"
              ? "GitHub recipients must use a GitHub username or numeric user id, not a Solana address."
              : `${recipientTypeLabel(row.dataset.type)} recipients must use a username, not a Solana address.`,
          );
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
      updateFeeSplitRowValidationUi(row);
    }

    function createFeeSplitRow(entry = {}) {
      const row = document.createElement("div");
      row.className = "fee-split-row";
      row.dataset.rowId = nextFeeSplitRowId();
      row.dataset.type = normalizeRecipientType(entry.type);
      if (row.dataset.type === "github") {
        const parsedGithubTarget = parseGithubRecipientTarget(entry.value || "");
        const githubUserId = String(entry.githubUserId || parsedGithubTarget.githubUserId || "").trim();
        if (githubUserId) row.dataset.githubUserId = githubUserId;
      }
      if (entry.defaultReceiver) row.dataset.defaultReceiver = "true";
      row.innerHTML = `
        <div class="fee-split-row-top">
          <div class="bags-fee-row-header">
            <div class="recipient-type-tabs">
              ${recipientTypeTabsMarkup()}
            </div>
            <div class="bags-fee-row-meta" hidden>
              <span class="bags-fee-row-chip">Wallet</span>
              <span class="bags-fee-row-status"></span>
              <button type="button" class="bags-fee-row-copy" hidden aria-label="Copy resolved wallet" title="Copy resolved wallet">
                <img class="bags-fee-row-copy-icon" src="/images/recipient-copy.png" alt="">
              </button>
            </div>
          </div>
          <button type="button" class="recipient-remove" aria-label="Remove recipient">
            <img class="recipient-remove-icon" src="/images/recipient-remove.png" alt="">
          </button>
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
      restoreFeeSplitLookupState(row, entry.lookupState);
      updateFeeSplitRowValidationUi(row);
      return row;
    }

    function ensureFeeSplitDefaultRow() {
      if (!feeSplitList) return;
      if (usesImplicitCreatorShareMode()) return;
      const hasNonDefaultRows = getFeeSplitRows().some((row) => row.dataset.defaultReceiver !== "true");
      if (feeSplitList.dataset.suppressDefaultRow === "true") {
        if (hasNonDefaultRows) return;
        delete feeSplitList.dataset.suppressDefaultRow;
      }
      const deployerAddress = getDeployerFeeSplitAddress();
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

    function formatFeeSplitRecipientProgress(count, max = 100) {
      const numeric = Number(count);
      const normalizedMax = Math.max(1, Math.round(Number(max) || 100));
      if (!Number.isFinite(numeric) || numeric <= 0) return `0/${normalizedMax}`;
      return `${Math.max(0, Math.round(numeric))}/${normalizedMax}`;
    }

    function countConfiguredRecipientRows(rows, { agentCustom = false } = {}) {
      const sourceRows = Array.isArray(rows) ? rows : [];
      const configuredRows = sourceRows.filter((row) => {
        if (!row) return false;
        if (!agentCustom && row.dataset.defaultReceiver === "true") return false;
        if (agentCustom && row.dataset.locked === "true") {
          return Number(row.querySelector(".recipient-share")?.value || 0) > 0;
        }
        return true;
      });
      return configuredRows.filter((row) => {
        const value = row.querySelector(".recipient-target")?.value.trim();
        const share = Number(row.querySelector(".recipient-share")?.value || 0);
        return row.dataset.locked === "true" || Boolean(value) || (Number.isFinite(share) && share > 0);
      }).length;
    }

    function syncFeeSplitPillSummary(rows = getFeeSplitRows()) {
      const mode = getMode();
      const isAgentCustom = mode === "agent-custom";
      const isBags = isBagsFeeSplitLaunchpad();
      const supportsVisibleFeeSplitState = mode === "regular" || isAgentCustom || isBags;
      const configuredCount = countConfiguredRecipientRows(isAgentCustom ? getAgentSplitRows() : rows, {
        agentCustom: isAgentCustom,
      });
      const feeSplitConfiguredCount = countConfiguredRecipientRows(rows, { agentCustom: false });
      const agentSplitConfiguredCount = countConfiguredRecipientRows(getAgentSplitRows(), { agentCustom: true });
      const pumpSubmittedRecipientCount = !isAgentCustom && !isBags && usesImplicitPumpCreatorShareMode("regular", getLaunchpad())
        ? collectSubmittedFeeSplitRecipients("regular").length
        : feeSplitConfiguredCount;
      const progressCount = supportsVisibleFeeSplitState
        ? (isAgentCustom ? agentSplitConfiguredCount : (isBags ? feeSplitConfiguredCount : pumpSubmittedRecipientCount))
        : 0;
      const pillLabel = isAgentCustom ? "Agent Split" : (isBags ? "Fee Share" : "Fee Split");
      if (feeSplitPillTitle) feeSplitPillTitle.textContent = pillLabel;
      if (feeSplitPillProgress) {
        feeSplitPillProgress.textContent = `(${progressCount})`;
        feeSplitPillProgress.title = `${progressCount} recipient${progressCount === 1 ? "" : "s"} configured`;
        feeSplitPillProgress.setAttribute("aria-label", feeSplitPillProgress.title);
      }
      if (feeSplitTitle) {
        feeSplitTitle.textContent = isBags
          ? `Fee Share ${formatFeeSplitRecipientProgress(feeSplitConfiguredCount)}`
          : `Fee Split ${formatFeeSplitRecipientProgress(pumpSubmittedRecipientCount, maxFeeSplitRecipients)}`;
      }
      if (agentSplitTitle) {
        agentSplitTitle.textContent = `Agent Split ${formatFeeSplitRecipientProgress(agentSplitConfiguredCount, maxFeeSplitRecipients)}`;
      }
    }

    function syncFeeSplitTotals() {
      const rows = getFeeSplitRows();
      const total = rows.reduce((sum, row) => {
        const value = Number(row.querySelector(".recipient-share")?.value || 0);
        return sum + (Number.isFinite(value) ? value : 0);
      }, 0);
      if (feeSplitTotal) {
        feeSplitTotal.textContent = formatFeeSplitTotalLabel(total);
        feeSplitTotal.classList.toggle(
          "invalid",
          usesImplicitCreatorShareMode() ? total > 100.001 : (Math.abs(total - 100) > 0.001 && total !== 0),
        );
      }
      syncFeeSplitPillSummary(rows);
      if (feeSplitReset) feeSplitReset.disabled = rows.length === 0;
      if (feeSplitEven) feeSplitEven.disabled = rows.length === 0;
      const effectiveRecipientCount = usesImplicitPumpCreatorShareMode("regular", getLaunchpad())
        ? collectSubmittedFeeSplitRecipients("regular").length
        : rows.length;
      if (feeSplitAdd) feeSplitAdd.disabled = rows.length >= maxFeeSplitRecipients || effectiveRecipientCount >= maxFeeSplitRecipients;

      if (!feeSplitBar || !feeSplitLegendList) return;
      if (rows.length === 0 || total === 0) {
        feeSplitBar.style.background = "#1e2630";
        feeSplitLegendList.innerHTML = "";
        return;
      }

      let running = 0;
      const gradientStops = [];
      const legendItems = [];
      rows.forEach((row, index) => {
        const share = Number(row.querySelector(".recipient-share")?.value || 0);
        const color = splitColors[index % splitColors.length];
        const targetValue = row.querySelector(".recipient-target")?.value.trim();
        const label = formatLegendRecipientLabel(
          row.dataset.type,
          targetValue,
          isSocialRecipientType(row.dataset.type) ? recipientTypeLabel(row.dataset.type) : "wallet",
          { compactSocialLabel: usesImplicitCreatorShareMode() },
        );
        if (share > 0) {
          const start = running;
          running += share;
          gradientStops.push(`${color} ${start}%`, `${color} ${running}%`);
          legendItems.push(
            `<span class="legend-chip" title="${escapeHTML(label.full)}"><span class="legend-dot" style="background:${color}"></span><span class="legend-chip-label">${escapeHTML(label.short)}</span><span class="legend-chip-share">${share.toFixed(2).replace(/\.00$/, "")}%</span></span>`,
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
      syncBagsFeeSplitSummary();
    }

    function syncFeeSplitModalPresentation() {
      const removedImplicitCreatorRows = removeImplicitCreatorRows();
      const isBags = isBagsFeeSplitLaunchpad();
      const isImplicitPump = usesImplicitPumpCreatorShareMode();
      const usesCompactPresentation = isBags || isImplicitPump;
      const feeSplitModalShell = feeSplitModal ? feeSplitModal.querySelector(".fee-split-modal") : null;
      if (feeSplitModalShell) {
        feeSplitModalShell.classList.toggle("compact-fee-split-modal", usesCompactPresentation);
        feeSplitModalShell.classList.toggle("bags-fee-split-modal", isBags);
      }
      syncFeeSplitPillSummary();
      if (feeSplitIntro) {
        feeSplitIntro.hidden = usesCompactPresentation;
        feeSplitIntro.textContent = usesCompactPresentation
          ? ""
          : "Wallet addresses or GitHub usernames. Total must equal 100%.";
      }
      if (feeSplitRecipientsCopy) {
        feeSplitRecipientsCopy.hidden = usesCompactPresentation;
        feeSplitRecipientsCopy.textContent = usesCompactPresentation
          ? ""
          : "Creator rewards will be routed using this split after launch.";
      }
      if (feeSplitAdd) feeSplitAdd.textContent = isBags ? "+ Add claimer" : "+ Add recipient";
      getFeeSplitRows().forEach((row) => {
        row.classList.toggle("bags-fee-split-row", isBags);
      });
      if (removedImplicitCreatorRows) {
        syncFeeSplitTotals();
        setStoredFeeSplitDraft(serializeFeeSplitDraft());
      }
      syncCompactFeeSplitListViewport();
      syncBagsFeeSplitSummary();
    }

    function applyFeeSplitDraft(value, { persist = false } = {}) {
      const draft = normalizeFeeSplitDraftForLaunchpad(value, currentFeeSplitDraftLaunchpad());
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
      if (draft.enabled && !usesImplicitCreatorShareMode()) ensureFeeSplitDefaultRow();
      getFeeSplitRows().forEach((row) => {
        updateFeeSplitRowValidationUi(row);
        scheduleFeeSplitLookup(row, { immediate: true });
      });
      syncFeeSplitModalPresentation();
      syncFeeSplitTotals();
      if (persist) setStoredFeeSplitDraft(draft);
    }

    function feeSplitClearAllDraft() {
      if (usesImplicitCreatorShareMode()) {
        return normalizeFeeSplitDraft({
          enabled: true,
          suppressDefaultRow: false,
          rows: [],
        });
      }
      const deployerAddress = getDeployerFeeSplitAddress();
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

    function serializeAgentSplitDraft() {
      return {
        rows: getAgentSplitRows().map((row) => ({
          locked: row.dataset.locked === "true",
          type: row.dataset.type || "wallet",
          value: row.querySelector(".recipient-target")?.value?.trim() || "",
          githubUserId: row.dataset.githubUserId || "",
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
          type: normalizeRecipientType(entry && entry.type, { allowAgent: true }),
          value: String((entry && entry.value) || "").trim(),
          githubUserId: String((entry && entry.githubUserId) || "").trim(),
          sharePercent: normalizeDecimalInput((entry && entry.sharePercent) || "", 2),
          defaultReceiver: Boolean(entry && entry.defaultReceiver),
          targetLocked: Boolean(entry && entry.targetLocked),
        })).filter((entry) => (
          entry.locked
          || entry.type === "agent"
          || isRecipientTypeSupportedForLaunchpad(entry.type, "pump", { allowAgent: true })
        ))
        : [];
      return { rows };
    }

    function buildAgentSplitDraftFromFeeSplitDraft(value) {
      const draft = normalizeFeeSplitDraftForLaunchpad(value, getLaunchpad());
      const shouldSeedImplicitPumpCreator = usesImplicitPumpCreatorShareMode("regular", getLaunchpad());
      if (!draft.enabled && draft.rows.length === 0) {
        if (shouldSeedImplicitPumpCreator) {
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
        return normalizeAgentSplitDraft({ rows: [] });
      }
      const defaultReceiverRow = draft.rows.find((row) => row.defaultReceiver);
      const carriedRows = draft.rows
        .filter((row) => !row.defaultReceiver)
        .map((row) => ({
          locked: false,
          type: normalizeRecipientType(row.type),
          value: row.value,
          githubUserId: row.githubUserId || "",
          sharePercent: row.sharePercent,
          defaultReceiver: false,
          targetLocked: Boolean(row.targetLocked),
        }));
      const implicitCreatorSharePercent = !defaultReceiverRow && usesImplicitCreatorShareMode("regular", getLaunchpad())
        ? formatPercentNumber(Math.max(0, 100 - carriedRows.reduce((sum, row) => sum + (Number(row.sharePercent) || 0), 0)))
        : "";
      if (!defaultReceiverRow && carriedRows.length === 0) {
        if (shouldSeedImplicitPumpCreator) {
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
        return normalizeAgentSplitDraft({ rows: [] });
      }
      return normalizeAgentSplitDraft({
        rows: [
          {
            locked: true,
            type: "wallet",
            value: "",
            sharePercent: defaultReceiverRow
              ? defaultReceiverRow.sharePercent
              : (carriedRows.length > 0 ? implicitCreatorSharePercent : ""),
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
      const deployerAddress = getDeployerFeeSplitAddress();
      const carriedRows = draft.rows
        .filter((row) => !row.locked && row.type !== "agent")
        .map((row) => ({
          type: normalizeRecipientType(row.type),
          value: row.value,
          githubUserId: row.type === "github" ? (row.githubUserId || "") : "",
          sharePercent: row.sharePercent,
          defaultReceiver: false,
          targetLocked: Boolean(row.targetLocked),
        }));
      if ((!agentRow || !agentRow.sharePercent) && carriedRows.length === 0) {
        return normalizeFeeSplitDraft({ enabled: false, rows: [] });
      }
      if (usesImplicitPumpCreatorShareMode("regular", getLaunchpad())) {
        return normalizeFeeSplitDraftForLaunchpad({
          enabled: carriedRows.length > 0,
          suppressDefaultRow: false,
          rows: carriedRows,
        }, getLaunchpad());
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

    function scopedAgentSplitDraftStorageKey(launchpad = getLaunchpad()) {
      return `launchdeck.agentSplitDraft.${normalizeLaunchpad(launchpad)}.agent-custom.v2`;
    }

    function getStoredAgentSplitDraft(launchpad = getLaunchpad()) {
      try {
        const scopedKey = scopedAgentSplitDraftStorageKey(launchpad);
        const keys = [scopedKey, agentSplitDraftStorageKey];
        for (const key of keys) {
          const raw = global.localStorage.getItem(key);
          if (!raw) continue;
          const normalized = normalizeAgentSplitDraft(JSON.parse(raw));
          if (key !== scopedKey) {
            global.localStorage.setItem(scopedKey, JSON.stringify(normalized));
          }
          return normalized;
        }
        return null;
      } catch (_error) {
        return null;
      }
    }

    function setStoredAgentSplitDraft(value, { launchpad = getLaunchpad() } = {}) {
      try {
        const normalized = normalizeAgentSplitDraft(value);
        const key = scopedAgentSplitDraftStorageKey(launchpad);
        if (normalized.rows.length === 0) {
          global.localStorage.removeItem(key);
          return;
        }
        global.localStorage.setItem(key, JSON.stringify(normalized));
      } catch (_error) {
        // Ignore storage failures and keep agent split controls functional.
      }
    }

    function createAgentSplitRow(entry = {}) {
      const isAgent = entry.locked === true;
      const row = document.createElement("div");
      row.className = "fee-split-row";
      row.dataset.type = isAgent ? "agent" : normalizeRecipientType(entry.type);
      if (isAgent) row.dataset.locked = "true";
      if (row.dataset.type === "github") {
        const parsedGithubTarget = parseGithubRecipientTarget(entry.value || "");
        const githubUserId = String(entry.githubUserId || parsedGithubTarget.githubUserId || "").trim();
        if (githubUserId) row.dataset.githubUserId = githubUserId;
      }
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
              ${recipientTypeTabsMarkup()}
            </div>
            <button type="button" class="recipient-remove" aria-label="Remove recipient">
              <img class="recipient-remove-icon" src="/images/recipient-remove.png" alt="">
            </button>
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

    function syncAgentSplitTotals() {
      const rows = getAgentSplitRows();
      const total = rows.reduce((sum, row) => {
        const value = Number(row.querySelector(".recipient-share")?.value || 0);
        return sum + (Number.isFinite(value) ? value : 0);
      }, 0);
      if (agentSplitTotal) {
        agentSplitTotal.textContent = formatFeeSplitTotalLabel(total);
        agentSplitTotal.classList.toggle("invalid", Math.abs(total - 100) > 0.001 && total !== 0);
      }
      if (agentSplitReset) agentSplitReset.disabled = rows.length === 0;
      if (agentSplitEven) agentSplitEven.disabled = rows.length === 0;
      if (agentSplitAdd) agentSplitAdd.disabled = rows.length >= maxFeeSplitRecipients;
      syncFeeSplitPillSummary();

      if (!agentSplitBar || !agentSplitLegendList) return;
      if (rows.length === 0 || total === 0) {
        agentSplitBar.style.background = "#1e2630";
        agentSplitLegendList.innerHTML = "";
        syncAgentSplitListViewport();
        return;
      }

      let running = 0;
      const gradientStops = [];
      const legendItems = [];
      rows.forEach((row, index) => {
        const share = Number(row.querySelector(".recipient-share")?.value || 0);
        const color = splitColors[index % splitColors.length];
        const targetValue = row.querySelector(".recipient-target")?.value.trim();
        const label = row.dataset.locked
          ? { full: "Agent Buyback", short: "Agent" }
          : formatLegendRecipientLabel(row.dataset.type, targetValue, "wallet");
        if (share > 0) {
          const start = running;
          running += share;
          gradientStops.push(`${color} ${start}%`, `${color} ${running}%`);
          legendItems.push(
            `<span class="legend-chip" title="${escapeHTML(label.full)}"><span class="legend-dot" style="background:${color}"></span><span class="legend-chip-label">${escapeHTML(label.short)}</span><span class="legend-chip-share">${share.toFixed(2).replace(/\.00$/, "")}%</span></span>`,
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
      syncAgentSplitListViewport();
    }

    function normalizeAgentSplitStructure({ afterAdd = false } = {}) {
      const rows = getAgentSplitRows();
      const agentRow = rows.find((row) => row.dataset.locked === "true");
      const otherRows = rows.filter((row) => row.dataset.locked !== "true");
      if (!agentRow) return;

      const agentShareInput = agentRow.querySelector(".recipient-share");
      const agentSliderInput = agentRow.querySelector(".recipient-slider");

      if (!agentShareInput || !agentSliderInput) return;
      if (otherRows.length === 0) {
        agentShareInput.value = "100";
        agentSliderInput.value = "100";
        return;
      }

      if (afterAdd && otherRows.length === 1) {
        const currentAgentShare = Number(agentShareInput.value || 0);
        const currentOtherShare = Number(otherRows[0].querySelector(".recipient-share")?.value || 0);
        if (Math.abs(currentAgentShare - 100) < 0.001 && Math.abs(currentOtherShare) < 0.001) {
          agentShareInput.value = "50";
          agentSliderInput.value = "50";
          otherRows[0].querySelector(".recipient-share").value = "50";
          otherRows[0].querySelector(".recipient-slider").value = "50";
        }
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

    function collectAgentSplitRecipients() {
      return getAgentSplitRows().map((row) => {
        if (row.dataset.locked) {
          const sharePercent = row.querySelector(".recipient-share")?.value.trim() || "";
          const numericShare = Number(sharePercent);
          return {
            type: "agent",
            shareBps: Number.isFinite(numericShare) ? Math.round(numericShare * 100) : NaN,
          };
        }
        const type = row.dataset.type || "wallet";
        const value = row.querySelector(".recipient-target")?.value.trim() || "";
        const githubUserId = String(row.dataset.githubUserId || "").trim();
        const parsedGithubTarget = parseGithubRecipientTarget(value);
        const sharePercent = row.querySelector(".recipient-share")?.value.trim() || "";
        if (!value && !sharePercent) return null;
        const numericShare = Number(sharePercent);
        return {
          type,
          address: type === "wallet" ? value : "",
          githubUsername: isSocialRecipientType(type)
            ? (type === "github" ? parsedGithubTarget.githubUsername : value.replace(/^@+/, ""))
            : "",
          githubUserId: type === "github" ? (githubUserId || parsedGithubTarget.githubUserId) : "",
          shareBps: Number.isFinite(numericShare) ? Math.round(numericShare * 100) : NaN,
        };
      }).filter(Boolean);
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

    function hasMeaningfulFeeSplitConfiguration() {
      return hasMeaningfulFeeSplitRecipients(collectFeeSplitRecipients());
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

    function collectFeeSplitRecipients() {
      return getFeeSplitRows()
        .map((row) => {
          const type = row.dataset.type || "wallet";
          const value = row.querySelector(".recipient-target")?.value.trim() || "";
          const githubUserId = String(row.dataset.githubUserId || "").trim();
          const parsedGithubTarget = parseGithubRecipientTarget(value);
          const sharePercent = row.querySelector(".recipient-share")?.value.trim() || "";
          if (!value && !sharePercent) return null;
          const numericShare = Number(sharePercent);
          const descriptor = isSocialRecipientType(type) ? buildFeeSplitLookupDescriptor(row) : null;
          const state = descriptor ? getFeeSplitRowState(row) : null;
          const resolvedWallet = descriptor
            && state
            && state.key === descriptor.cacheKey
            && state.status === "valid"
            && String(state.wallet || "").trim()
            ? String(state.wallet || "").trim()
            : "";
          const normalizedType = resolvedWallet ? "wallet" : type;
          return {
            type: normalizedType,
            address: normalizedType === "wallet" ? (resolvedWallet || value) : "",
            githubUsername: normalizedType !== "wallet" && isSocialRecipientType(type)
              ? (type === "github" ? parsedGithubTarget.githubUsername : value.replace(/^@+/, ""))
              : "",
            githubUserId: normalizedType !== "wallet" && type === "github" ? (githubUserId || parsedGithubTarget.githubUserId) : "",
            shareBps: Number.isFinite(numericShare) ? Math.round(numericShare * 100) : NaN,
          };
        })
        .filter(Boolean);
    }

    function collectSubmittedFeeSplitRecipients(mode = getMode()) {
      const recipients = collectFeeSplitRecipients();
      if (!usesImplicitCreatorShareMode(mode, getLaunchpad())) {
        return recipients;
      }
      const total = recipients.reduce((sum, entry) => sum + (Number(entry.shareBps) || 0), 0);
      const remainder = Math.max(0, 10_000 - total);
      if (remainder <= 0) {
        return recipients;
      }
      const deployerAddress = getDeployerFeeSplitAddress();
      if (!deployerAddress) {
        return recipients;
      }
      return recipients.concat({
        type: "wallet",
        address: deployerAddress,
        githubUsername: "",
        githubUserId: "",
        shareBps: remainder,
      });
    }

    function updateFeeSplitVisibility() {
      const mode = getMode();
      const isBagsMode = mode.startsWith("bags-");
      const active = (mode === "agent-custom" && hasMeaningfulAgentSplitConfiguration())
        || (mode === "regular" && feeSplitEnabled && feeSplitEnabled.checked && hasMeaningfulFeeSplitConfiguration())
        || (isBagsMode && hasMeaningfulFeeSplitConfiguration());
      if (feeSplitPill) {
        feeSplitPill.classList.toggle("active", active);
        feeSplitPill.disabled = mode !== "regular" && mode !== "agent-custom" && !isBagsMode;
      }
      syncFeeSplitPillSummary();
      if (mode === "regular" && feeSplitEnabled && feeSplitEnabled.checked) ensureFeeSplitDefaultRow();
      if (mode !== "regular" && !isBagsMode && feeSplitModal) feeSplitModal.hidden = true;
      syncFeeSplitModalPresentation();
      syncFeeSplitTotals();
      syncSettingsCapabilities();
    }

    function initAgentSplitIfEmpty() {
      if (agentSplitList && agentSplitList.children.length === 0) {
        const storedDraft = getStoredAgentSplitDraft();
        if (storedDraft) {
          applyAgentSplitDraft(storedDraft, { persist: false });
          return;
        }
        resetAgentSplitToDefault();
      }
    }

    function showFeeSplitModal() {
      const mode = getMode();
      if (mode === "regular" || mode.startsWith("bags-")) {
        feeSplitModalSnapshot = normalizeFeeSplitDraft(serializeFeeSplitDraft());
        clearFeeSplitRestoreState();
        if (feeSplitEnabled) feeSplitEnabled.checked = true;
        updateFeeSplitVisibility();
        if (!usesImplicitCreatorShareMode()) ensureFeeSplitDefaultRow();
        getFeeSplitRows().forEach((row) => {
          updateFeeSplitRowValidationUi(row);
          scheduleFeeSplitLookup(row, { immediate: true });
        });
        setFeeSplitModalError("");
        if (feeSplitModal) feeSplitModal.hidden = false;
        syncFeeSplitModalPresentation();
        return;
      }
      if (mode === "agent-custom") {
        showAgentSplitModal();
      }
    }

    function hideFeeSplitModal() {
      setFeeSplitModalError("");
      clearFeeSplitRestoreState();
      syncFeeSplitModalPresentation();
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
      scheduleLiveSyncBroadcast({ immediate: true });
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

    function seedAgentSplitFromFeeSplit() {
      if (!agentSplitList) return false;
      const regularRows = getFeeSplitRows();
      if (!regularRows.length) {
        if (!usesImplicitPumpCreatorShareMode("regular", getLaunchpad())) return false;
        agentSplitList.innerHTML = "";
        agentSplitList.appendChild(createAgentSplitRow({ locked: true, sharePercent: "100" }));
        syncAgentSplitTotals();
        setAgentSplitModalError("");
        return true;
      }
      const defaultReceiverRow = regularRows.find((row) => row.dataset.defaultReceiver === "true");
      if (!defaultReceiverRow && !usesImplicitCreatorShareMode("regular", getLaunchpad())) return false;

      const carriedRows = regularRows
        .filter((row) => row !== defaultReceiverRow)
        .map((row) => ({
          type: row.dataset.type || "wallet",
          value: row.querySelector(".recipient-target")?.value.trim() || "",
          sharePercent: row.querySelector(".recipient-share")?.value.trim() || "",
          targetLocked: row.dataset.targetLocked === "true",
        }))
        .filter((entry) => entry.value || entry.sharePercent);
      const agentSharePercent = defaultReceiverRow
        ? (defaultReceiverRow.querySelector(".recipient-share")?.value.trim() || "0")
        : formatPercentNumber(Math.max(0, 100 - carriedRows.reduce((sum, row) => sum + (Number(row.sharePercent) || 0), 0)));

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
          value: getDeployerFeeSplitAddress(),
          sharePercent: "50",
          defaultReceiver: true,
          targetLocked: true,
        }),
      );
      syncAgentSplitTotals();
      setAgentSplitModalError("");
    }

    function showAgentSplitModal() {
      if (getMode() !== "agent-custom") return;
      initAgentSplitIfEmpty();
      agentSplitModalSnapshot = normalizeAgentSplitDraft(serializeAgentSplitDraft());
      clearAgentSplitRestoreState();
      syncAgentSplitTotals();
      if (agentSplitModalError) agentSplitModalError.textContent = "";
      if (agentSplitModal) agentSplitModal.hidden = false;
      syncAgentSplitListViewport();
    }

    function hideAgentSplitModal() {
      clearAgentSplitRestoreState();
      if (agentSplitModal) agentSplitModal.hidden = true;
      if (agentSplitList) {
        agentSplitList.classList.remove("bags-fee-split-list-scrollable");
        agentSplitList.style.maxHeight = "";
        agentSplitList.style.overflowY = "";
      }
    }

    function cancelAgentSplitModal() {
      applyAgentSplitDraft(agentSplitModalSnapshot, { persist: false });
      syncAgentSplitTotals();
      agentSplitModalSnapshot = null;
      hideAgentSplitModal();
    }

    function attemptCloseAgentSplitModal() {
      const errors = validateAgentSplit();
      if (errors.length > 0) {
        setAgentSplitModalError(errors[0]);
        return false;
      }
      const nextDraft = normalizeAgentSplitDraft(serializeAgentSplitDraft());
      setStoredAgentSplitDraft(nextDraft);
      scheduleLiveSyncBroadcast({ immediate: true });
      setAgentSplitModalError("");
      agentSplitModalSnapshot = nextDraft;
      hideAgentSplitModal();
      return true;
    }

    function updateLockedModeFields() {
      const full = getDeployerFeeSplitAddress();
      const short = full ? shortenAddress(full) : "Connected wallet";
      if (agentUnlockedAuthority) {
        agentUnlockedAuthority.value = short;
        agentUnlockedAuthority.title = full;
      }

      const defaultReceiverRow = getAgentSplitRows().find((row) => row.dataset.defaultReceiver === "true");
      if (defaultReceiverRow) {
        const target = defaultReceiverRow.querySelector(".recipient-target");
        if (target && !target.value.trim() && full) {
          target.value = full;
          setRecipientTargetLocked(defaultReceiverRow, true);
        }
      }
    }

    function validateAgentSplit() {
      const errors = [];
      const recipients = collectAgentSplitRecipients();
      if (getMode() !== "agent-custom") return errors;

      if (recipients.length === 0) {
        errors.push("Agent fee split is required.");
        return errors;
      }
      if (recipients.length > maxFeeSplitRecipients) {
        errors.push(`Agent custom fee split supports at most ${maxFeeSplitRecipients} recipients.`);
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
        if (!isRecipientTypeSupportedForLaunchpad(entry.type, getLaunchpad(), { allowAgent: true })) {
          errors.push(
            `Agent split recipient ${index + 1} uses ${recipientTypeLabel(entry.type)}, which is only supported for Bags fee splits.`,
          );
          return;
        }
        if (entry.type === "wallet" && !entry.address) {
          errors.push(`Agent split recipient ${index + 1} is missing a wallet address.`);
        }
        if (isSocialRecipientType(entry.type) && looksLikeSolanaAddress(entry.githubUsername || entry.githubUserId)) {
          errors.push(
            `Agent split recipient ${index + 1} cannot use a Solana address while ${recipientTypeLabel(entry.type)} is selected.`,
          );
        }
        if (isSocialRecipientType(entry.type) && !entry.githubUsername && !entry.githubUserId) {
          errors.push(
            `Agent split recipient ${index + 1} is missing a ${recipientTypeLabel(entry.type)} ${entry.type === "github" ? "username or user id" : "username"}.`,
          );
        }
      });

      return errors;
    }

    function validateFeeSplit() {
      const errors = [];
      const mode = getMode();
      const isBagsMode = mode.startsWith("bags-");
      if (mode !== "regular" && !isBagsMode) return errors;
      if (!isBagsMode && (!feeSplitEnabled || !feeSplitEnabled.checked)) return errors;
      const rows = getFeeSplitRows();
      const recipientRows = rows.filter((row) => {
        const value = String(row.querySelector(".recipient-target")?.value || "").trim();
        const share = String(row.querySelector(".recipient-share")?.value || "").trim();
        return value || share;
      });
      const recipients = collectFeeSplitRecipients();
      const submittedRecipients = collectSubmittedFeeSplitRecipients(mode);
      if (submittedRecipients.length > maxFeeSplitRecipients) {
        errors.push(`Fee split supports at most ${maxFeeSplitRecipients} recipients.`);
      }
      recipients.forEach((entry, index) => {
        if (!Number.isFinite(entry.shareBps) || entry.shareBps <= 0) {
          errors.push(`Fee split recipient ${index + 1} has an invalid %.`);
          return;
        }
        if (!isRecipientTypeSupportedForLaunchpad(entry.type, getLaunchpad())) {
          errors.push(
            `Fee split recipient ${index + 1} uses ${recipientTypeLabel(entry.type)}, which is only supported for Bags launches.`,
          );
          return;
        }
        if (entry.type === "wallet" && !entry.address) {
          errors.push(`Fee split recipient ${index + 1} is missing a wallet address.`);
          return;
        }
        if (isSocialRecipientType(entry.type) && !entry.githubUsername && !entry.githubUserId) {
          errors.push(
            `Fee split recipient ${index + 1} is missing a ${recipientTypeLabel(entry.type)} ${entry.type === "github" ? "username or user id" : "username"}.`,
          );
          return;
        }
        if (isSocialRecipientType(entry.type) && looksLikeSolanaAddress(entry.githubUsername || entry.githubUserId)) {
          errors.push(
            `Fee split recipient ${index + 1} cannot use a Solana address while ${recipientTypeLabel(entry.type)} is selected.`,
          );
          return;
        }
        if (isSocialRecipientType(entry.type)) {
          const row = recipientRows[index];
          const targetValidation = validateFeeSplitSocialTarget(
            entry.type,
            row ? String(row.querySelector(".recipient-target")?.value || "").trim() : (entry.githubUsername || entry.githubUserId),
            { githubUserId: row ? String(row.dataset.githubUserId || "").trim() : String(entry.githubUserId || "").trim() },
          );
          if (!targetValidation.valid && targetValidation.error) {
            errors.push(`Fee split recipient ${index + 1}: ${targetValidation.error}`);
            return;
          }
        }
        if (isBagsMode && isSocialRecipientType(entry.type)) {
          const row = recipientRows[index];
          const descriptor = buildFeeSplitLookupDescriptor(row);
          const state = getFeeSplitRowState(row);
          if (!descriptor || !state || state.key !== descriptor.cacheKey) {
            errors.push(`Fee split recipient ${index + 1} still needs Bags username validation.`);
            return;
          }
          if (state.status === "checking") {
            errors.push(`Fee split recipient ${index + 1} is still validating with Bags.`);
            return;
          }
          if (state.status !== "valid") {
            errors.push(state.message || `Fee split recipient ${index + 1} could not be resolved by Bags.`);
          }
        }
      });
      const total = recipients.reduce((sum, entry) => sum + (Number(entry.shareBps) || 0), 0);
      const implicitCreatorShare = usesImplicitCreatorShareMode(mode, getLaunchpad());
      if (implicitCreatorShare && total > 10_000) {
        errors.push("Fee split cannot exceed 100%.");
      } else if (!implicitCreatorShare && recipients.length > 0 && total !== 10_000) {
        errors.push("Fee split must total 100%.");
      }
      return errors;
    }

    function getActiveFeeSplitDraftLaunchpad() {
      return activeFeeSplitDraftLaunchpad;
    }

    function setActiveFeeSplitDraftLaunchpad(value) {
      activeFeeSplitDraftLaunchpad = normalizeLaunchpad(value);
    }

    function getFeeSplitClearAllRestoreSnapshot() {
      return feeSplitClearAllRestoreSnapshot;
    }

    function setFeeSplitClearAllRestoreSnapshot(value) {
      feeSplitClearAllRestoreSnapshot = value;
      updateFeeSplitClearAllButton();
    }

    function getAgentSplitClearAllRestoreSnapshot() {
      return agentSplitClearAllRestoreSnapshot;
    }

    function setAgentSplitClearAllRestoreSnapshot(value) {
      agentSplitClearAllRestoreSnapshot = value;
      updateAgentSplitClearAllButton();
    }

    return {
      agentSplitClearAllDraft,
      applyAgentSplitDraft,
      applyFeeSplitDraft,
      attemptCloseAgentSplitModal,
      attemptCloseFeeSplitModal,
      buildAgentSplitDraftFromFeeSplitDraft,
      buildFeeSplitLookupDescriptor,
      buildFeeSplitDraftFromAgentSplitDraft,
      cancelAgentSplitModal,
      cancelFeeSplitModal,
      clearAgentSplitRestoreState,
      clearFeeSplitRestoreState,
      clearFeeSplitRowLookupTimer,
      clearFeeSplitRowState,
      collectAgentSplitRecipients,
      collectFeeSplitRecipients,
      collectSubmittedFeeSplitRecipients,
      copyBagsResolvedWallet,
      createAgentSplitRow,
      createFeeSplitRow,
      currentFeeSplitDraftLaunchpad,
      describeFeeSplitLookupFailure,
      ensureFeeSplitDefaultRow,
      feeSplitClearAllDraft,
      feeSplitDraftSessionStorageKey,
      feeSplitDraftStorageKey,
      finalizeFeeSplitDraftForMode,
      formatFeeSplitRecipientProgress,
      formatFeeSplitTotalLabel,
      formatLegendRecipientLabel,
      getActiveFeeSplitDraftLaunchpad,
      getAgentSplitClearAllRestoreSnapshot,
      getAgentSplitRows,
      getFeeSplitClearAllRestoreSnapshot,
      getFeeSplitRowState,
      getFeeSplitRows,
      getStoredAgentSplitDraft,
      getStoredFeeSplitDraft,
      hasMeaningfulAgentSplitConfiguration,
      hasMeaningfulAgentSplitRecipients,
      hasMeaningfulFeeSplitConfiguration,
      hasMeaningfulFeeSplitRecipients,
      hideAgentSplitModal,
      hideFeeSplitModal,
      initAgentSplitIfEmpty,
      isBagsFeeSplitLaunchpad,
      isPumpFeeSplitLaunchpad,
      normalizeAgentSplitDraft,
      normalizeAgentSplitStructure,
      normalizeFeeSplitDraft,
      normalizeFeeSplitDraftForLaunchpad,
      normalizeFeeSplitLookupState,
      nextFeeSplitRowId,
      resetAgentSplitToDefault,
      removeImplicitCreatorRows,
      restoreFeeSplitLookupState,
      restoreFeeSplitDraftForLaunchpad,
      runFeeSplitLookup,
      scheduleFeeSplitLookup,
      seedAgentSplitFromFeeSplit,
      serializeAgentSplitDraft,
      serializeFeeSplitDraft,
      serializeFeeSplitLookupState,
      setActiveFeeSplitDraftLaunchpad,
      setAgentSplitClearAllRestoreSnapshot,
      setAgentSplitModalError,
      setFeeSplitClearAllRestoreSnapshot,
      setFeeSplitModalError,
      setRecipientTargetLocked,
      setStoredAgentSplitDraft,
      setStoredFeeSplitDraft,
      showAgentSplitModal,
      showFeeSplitModal,
      syncAgentSplitDraftFromFeeSplitDraft,
      syncAgentSplitListViewport,
      syncAgentSplitTotals,
      syncBagsFeeSplitSummary,
      syncCompactFeeSplitListViewport,
      syncFeeSplitDraftFromAgentSplitDraft,
      syncFeeSplitModalPresentation,
      syncFeeSplitPillSummary,
      syncFeeSplitTotals,
      stripImplicitCreatorRowsFromFeeSplitDraftRows,
      updateFeeSplitRowType,
      updateFeeSplitRowValidationUi,
      updateFeeSplitVisibility,
      updateLockedModeFields,
      usesImplicitBagsCreatorShareMode,
      usesImplicitCreatorShareMode,
      usesImplicitPumpCreatorShareMode,
      validateFeeSplitSocialTarget,
      validateAgentSplit,
      validateFeeSplit,
      withSuspendedFeeSplitDraftPersistence,
    };
  }

  global.LaunchDeckSplitEditorsDomain = {
    create,
  };
})(window);
