(function initLaunchDeckWalletRuntimeDomain(global) {
  function createWalletRuntimeDomain(config) {
    const {
      elements = {},
      storageKeys = {},
      constants = {},
      renderCache = {},
      requestStates = {},
      requestUtils = {},
      renderUtils = {},
      state = {},
      helpers = {},
      actions = {},
    } = config || {};

    const {
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
    } = elements;

    const {
      selectedWallet: selectedWalletStorageKey = "",
      walletStatusLastRefreshAtMs: walletStatusLastRefreshStorageKey = "",
    } = storageKeys;

    const {
      runtimeStatusRefreshIntervalMs = 15000,
      followJobsRefreshIntervalMs = 5000,
      followJobsOfflineRetryMs = 15000,
      warmActivityDebounceMs = 1000,
    } = constants;

    const {
      selectedWalletKey = () => "",
      getConfig = () => ({}),
      getActivePreset = () => ({}),
      normalizeMevMode = (value) => String(value || ""),
      normalizeReportsTerminalView = (value) => String(value || ""),
      isTerminalFollowJobState = () => false,
      escapeHTML = (value) => String(value || ""),
      shortenAddress = (value) => String(value || ""),
      shortAddress = (value) => String(value || ""),
      walletIndexFromEnvKey = () => "?",
      shortenReportEndpoint = (value) => String(value || ""),
    } = helpers;

    const {
      renderSniperUI = () => {},
      updateLockedModeFields = () => {},
      renderBackendRegionSummary = () => {},
      applyPersistentDefaults = () => {},
      applyProviderAvailability = () => {},
      applyLaunchpadAvailability = () => {},
      renderQuickDevBuyButtons = () => {},
      populateDevBuyPresetEditor = () => {},
      updateQuote = async () => {},
      markBootstrapState = () => {},
      hasBootstrapConfig = () => false,
      setSettingsLoadingState = () => {},
      schedulePopoutAutosize = () => {},
      refreshActiveLogs = async () => {},
      renderReportsTerminalOutput = () => {},
      loadRuntimeStatus = async () => {},
    } = actions;

    function getLatestWalletStatus() {
      return typeof state.getLatestWalletStatus === "function" ? state.getLatestWalletStatus() : null;
    }

    function setLatestWalletStatus(value) {
      if (typeof state.setLatestWalletStatus === "function") state.setLatestWalletStatus(value);
    }

    function getLatestRuntimeStatus() {
      return typeof state.getLatestRuntimeStatus === "function" ? state.getLatestRuntimeStatus() : null;
    }

    function setLatestRuntimeStatus(value) {
      if (typeof state.setLatestRuntimeStatus === "function") state.setLatestRuntimeStatus(value);
    }

    function getStartupWarmState() {
      return typeof state.getStartupWarmState === "function"
        ? state.getStartupWarmState()
        : {
            started: false,
            ready: false,
            enabled: true,
            backendLoaded: false,
            backendPayload: null,
            backendError: "",
          };
    }

    function setStartupWarmState(value) {
      if (typeof state.setStartupWarmState === "function") state.setStartupWarmState(value);
    }

    function getAppBootstrapState() {
      return typeof state.getAppBootstrapState === "function" ? state.getAppBootstrapState() : {};
    }

    function getFollowJobsState() {
      return typeof state.getFollowJobsState === "function"
        ? state.getFollowJobsState()
        : {
            configured: false,
            reachable: false,
            jobs: [],
            health: null,
            error: "",
            loaded: false,
            refreshTimer: null,
          };
    }

    function setFollowJobsState(value) {
      if (typeof state.setFollowJobsState === "function") state.setFollowJobsState(value);
    }

    function getWarmActivityState() {
      return typeof state.getWarmActivityState === "function"
        ? state.getWarmActivityState()
        : {
            debounceTimer: null,
            inFlightPromise: null,
            lastSentAtMs: 0,
            pendingFlush: false,
          };
    }

    function setWarmActivityState(value) {
      if (typeof state.setWarmActivityState === "function") state.setWarmActivityState(value);
    }

    function getWalletStatusRefreshTimer() {
      return typeof state.getWalletStatusRefreshTimer === "function" ? state.getWalletStatusRefreshTimer() : null;
    }

    function setWalletStatusRefreshTimer(value) {
      if (typeof state.setWalletStatusRefreshTimer === "function") state.setWalletStatusRefreshTimer(value);
    }

    function getRuntimeStatusRefreshTimer() {
      return typeof state.getRuntimeStatusRefreshTimer === "function" ? state.getRuntimeStatusRefreshTimer() : null;
    }

    function setRuntimeStatusRefreshTimer(value) {
      if (typeof state.setRuntimeStatusRefreshTimer === "function") state.setRuntimeStatusRefreshTimer(value);
    }

    function getWalletStatusRefreshIntervalMs() {
      return typeof state.getWalletStatusRefreshIntervalMs === "function"
        ? state.getWalletStatusRefreshIntervalMs()
        : 30000;
    }

    function setWalletStatusRefreshIntervalMs(value) {
      if (typeof state.setWalletStatusRefreshIntervalMs === "function") {
        state.setWalletStatusRefreshIntervalMs(value);
      }
    }

    function getReportsTerminalState() {
      return typeof state.getReportsTerminalState === "function"
        ? state.getReportsTerminalState()
        : { view: "transactions" };
    }

    function selectedWalletRecord() {
      const latestWalletStatus = getLatestWalletStatus();
      const wallets = latestWalletStatus && Array.isArray(latestWalletStatus.wallets) ? latestWalletStatus.wallets : [];
      return wallets.find((wallet) => wallet.envKey === selectedWalletKey()) || null;
    }

    function getStoredSelectedWalletKey() {
      try {
        return global.localStorage.getItem(selectedWalletStorageKey) || "";
      } catch (_error) {
        return "";
      }
    }

    function setStoredSelectedWalletKey(walletKey) {
      try {
        const normalized = String(walletKey || "").trim();
        if (!normalized) {
          global.localStorage.removeItem(selectedWalletStorageKey);
          return;
        }
        global.localStorage.setItem(selectedWalletStorageKey, normalized);
      } catch (_error) {
        // Ignore storage access failures and keep the UI functional.
      }
    }

    function setStoredWalletStatusLastRefreshAtMs(value) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric) || numeric <= 0) return;
      try {
        global.localStorage.setItem(walletStatusLastRefreshStorageKey, String(Math.round(numeric)));
      } catch (_error) {
        // Ignore storage access failures and keep the UI functional.
      }
    }

    function walletDisplayName(wallet) {
      if (!wallet) return "No wallet";
      if (wallet.customName && String(wallet.customName).trim()) {
        return String(wallet.customName).trim();
      }
      const index = walletIndexFromEnvKey(wallet.envKey);
      return `#${index}`;
    }

    function walletLabel(wallet, balanceSol) {
      if (!wallet) return "No wallet";
      const displayName = walletDisplayName(wallet);
      if (!wallet.publicKey) return `${displayName}: invalid`;
      const bal = balanceSol != null ? ` | ${Number(balanceSol).toFixed(4)} SOL` : "";
      return `${displayName} - ${wallet.publicKey}${bal}`;
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

    function walletBalanceSol(wallet) {
      if (!wallet || wallet.balanceSol == null || Number.isNaN(Number(wallet.balanceSol))) return null;
      return Number(wallet.balanceSol);
    }

    function walletUsdValue(wallet) {
      if (!wallet || wallet.usd1Balance == null || Number.isNaN(Number(wallet.usd1Balance))) return null;
      return Number(wallet.usd1Balance);
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

    function renderWalletSummary() {
      if (!walletSummarySol || !walletSummaryUsd) return;
      const selectedWallet = selectedWalletRecord();
      walletSummarySol.textContent = formatWalletSol(walletBalanceSol(selectedWallet));
      walletSummaryUsd.textContent = formatWalletUsd(walletUsdValue(selectedWallet));
    }

    function renderWalletDropdownList(wallets = [], selectedKey = "") {
      if (!walletDropdownList) return;
      if (!wallets.length) {
        const appBootstrapState = getAppBootstrapState();
        const emptyMarkup = `<div class="wallet-empty-state">${appBootstrapState.walletsLoaded ? "No wallets found" : "Loading wallets..."}</div>`;
        if (renderUtils.setCachedHTML) {
          renderUtils.setCachedHTML(renderCache, "walletDropdown", walletDropdownList, emptyMarkup);
        } else {
          walletDropdownList.innerHTML = emptyMarkup;
        }
        return;
      }
      const markup = wallets.map((wallet) => {
        const solValue = walletBalanceSol(wallet);
        const usdValue = walletUsdValue(wallet);
        const walletAddress = String(wallet.publicKey || "").trim();
        return `
      <div
        class="wallet-option-button${wallet.envKey === selectedKey ? " is-selected" : ""}"
        data-wallet-key="${escapeHTML(wallet.envKey || "")}"
        role="button"
        tabindex="0"
        aria-pressed="${wallet.envKey === selectedKey ? "true" : "false"}"
      >
        <span class="wallet-option-main">
          <span class="wallet-option-name">${escapeHTML(walletDisplayName(wallet))}</span>
          <span class="wallet-option-meta">
            <span class="wallet-option-meta-address" title="${escapeHTML(walletAddress || "Unavailable")}">${escapeHTML(shortenAddress(walletAddress || "Unavailable"))}</span>
            ${walletAddress
              ? `<button type="button" class="wallet-option-copy" data-copy-value="${escapeHTML(walletAddress)}" aria-label="Copy wallet address" title="Copy wallet address">
                  <svg class="wallet-option-copy-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                    <path d="M16 1H4C2.9 1 2 1.9 2 3v12h2V3h12V1zm3 4H8C6.9 5 6 5.9 6 7v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"></path>
                  </svg>
                </button>`
              : ""}
          </span>
        </span>
        <span class="wallet-option-balances">
          <span class="wallet-option-sol">
            <img src="/images/solana-mark.png" alt="SOL" class="wallet-balance-icon">
            <span>${escapeHTML(formatWalletDropdownAmount(solValue))}</span>
          </span>
          <span class="wallet-option-usd">
            <img src="/images/usd1-mark.png" alt="USD1" class="wallet-balance-icon wallet-balance-icon-usd1">
            <span>${escapeHTML(formatWalletDropdownAmount(usdValue))}</span>
          </span>
        </span>
      </div>
    `;
      }).join("");
      if (renderUtils.setCachedHTML) {
        renderUtils.setCachedHTML(renderCache, "walletDropdown", walletDropdownList, markup);
      } else {
        walletDropdownList.innerHTML = markup;
      }
    }

    async function copyWalletDropdownAddress(button) {
      const value = String(button && button.dataset ? button.dataset.copyValue || "" : "").trim();
      if (!value) return;
      try {
        if (global.navigator && global.navigator.clipboard && global.navigator.clipboard.writeText) {
          await global.navigator.clipboard.writeText(value);
        } else if (global.document && typeof global.document.createElement === "function") {
          const probe = global.document.createElement("textarea");
          probe.value = value;
          probe.setAttribute("readonly", "");
          probe.style.position = "absolute";
          probe.style.left = "-9999px";
          global.document.body.appendChild(probe);
          probe.select();
          global.document.execCommand("copy");
          probe.remove();
        }
        if (button._copyFeedbackTimer) {
          global.clearTimeout(button._copyFeedbackTimer);
        }
        button.classList.remove("is-copied");
        void button.offsetWidth;
        button.classList.add("is-copied");
        button.title = "Copied";
        button.setAttribute("aria-label", "Copied wallet address");
        button._copyFeedbackTimer = global.setTimeout(() => {
          button.classList.remove("is-copied");
          button.title = "Copy wallet address";
          button.setAttribute("aria-label", "Copy wallet address");
          button._copyFeedbackTimer = null;
        }, 1200);
      } catch (_error) {}
    }

    function renderWalletOptions(wallets, selectedKey, balanceSol) {
      if (walletSelect) walletSelect.innerHTML = "";
      if (!wallets || wallets.length === 0) {
        if (walletSelect) {
          const option = global.document.createElement("option");
          option.value = "";
          option.textContent = "No wallets found";
          walletSelect.appendChild(option);
          walletSelect.disabled = true;
        }
        renderWalletDropdownList([], "");
        renderWalletSummary();
        return;
      }

      if (walletSelect) walletSelect.disabled = false;
      wallets.forEach((wallet) => {
        if (!walletSelect) return;
        const option = global.document.createElement("option");
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
      const latestWalletStatus = getLatestWalletStatus();
      if (!latestWalletStatus || !Array.isArray(latestWalletStatus.wallets)) return;
      const selectedWallet = latestWalletStatus.wallets.find((wallet) => wallet.envKey === nextKey) || null;
      setLatestWalletStatus({
        ...latestWalletStatus,
        selectedWalletKey: nextKey,
        connected: Boolean(selectedWallet && selectedWallet.publicKey),
        wallet: selectedWallet && selectedWallet.publicKey ? selectedWallet.publicKey : null,
        balanceLamports: selectedWallet && selectedWallet.balanceLamports != null ? selectedWallet.balanceLamports : null,
        balanceSol: selectedWallet && selectedWallet.balanceSol != null ? selectedWallet.balanceSol : null,
        usd1Balance: selectedWallet && selectedWallet.usd1Balance != null ? selectedWallet.usd1Balance : null,
      });
      const nextWalletStatus = getLatestWalletStatus();
      if (walletSelect) walletSelect.value = nextKey;
      renderWalletOptions(nextWalletStatus.wallets, nextKey, nextWalletStatus.balanceSol);
      renderSniperUI();
      if (!selectedWallet || !selectedWallet.publicKey) {
        if (walletBalance) walletBalance.textContent = "-";
        if (metaNode) metaNode.textContent = selectedWallet && selectedWallet.error ? selectedWallet.error : "Wallet unavailable";
        updateLockedModeFields();
        return;
      }
      if (walletBalance) {
        walletBalance.textContent = nextWalletStatus.balanceSol == null
          ? "--"
          : `${Number(nextWalletStatus.balanceSol).toFixed(4)} SOL`;
      }
      if (metaNode) metaNode.textContent = "";
      updateLockedModeFields();
    }

    async function refreshWalletStatus(preserveSelection = true, force = false) {
      try {
        const wallet = preserveSelection ? selectedWalletKey() : "";
        const query = new URLSearchParams();
        if (wallet) query.set("wallet", wallet);
        if (force) query.set("refresh", String(Date.now()));
        const url = query.size ? `/api/wallet-status?${query.toString()}` : "/api/wallet-status";
        const result = requestUtils.fetchJsonLatest
          ? await requestUtils.fetchJsonLatest("wallet-status", url, {
              cache: force ? "no-store" : "default",
            }, requestStates.walletStatus)
          : null;
        if (result && result.aborted) return;
        const response = result
          ? result.response
          : await fetch(url, { cache: force ? "no-store" : "default" });
        const payload = result ? result.payload : await response.json();
        if (result && !result.isLatest) return;
        if (!response.ok || !payload.ok) {
          throw new Error(payload.error || "Failed to load wallet status.");
        }
        applyWalletStatusPayload(payload);
        setStoredWalletStatusLastRefreshAtMs(Date.now());
      } catch (error) {
        if (walletBalance && !getLatestWalletStatus()) walletBalance.textContent = "-";
        if (metaNode) metaNode.textContent = error.message;
      } finally {
        scheduleWalletStatusRefresh();
      }
    }

    function applyWalletStatusPayload(payload) {
      const previousWalletStatus = getLatestWalletStatus();
      const normalizedWallets = normalizeVisibleWallets(payload.wallets || []);
      const selectedWalletKeyValue = resolveVisibleSelectedWalletKey(
        payload.selectedWalletKey || (previousWalletStatus && previousWalletStatus.selectedWalletKey) || "",
        normalizedWallets,
      );
      const selectedWallet = normalizedWallets.find((wallet) => wallet.envKey === selectedWalletKeyValue) || null;
      setLatestWalletStatus({
        ...(previousWalletStatus || {}),
        ...payload,
        selectedWalletKey: selectedWalletKeyValue,
        wallets: normalizedWallets,
        wallet: selectedWallet ? selectedWallet.publicKey : null,
        connected: Boolean(selectedWallet && selectedWallet.publicKey),
        balanceLamports: selectedWallet && selectedWallet.balanceLamports != null
          ? selectedWallet.balanceLamports
          : null,
        balanceSol: selectedWallet && selectedWallet.balanceSol != null
          ? selectedWallet.balanceSol
          : null,
        usd1Balance: selectedWallet && selectedWallet.usd1Balance != null
          ? selectedWallet.usd1Balance
          : null,
        config: payload.config || (previousWalletStatus && previousWalletStatus.config) || null,
        regionRouting: payload.regionRouting || (previousWalletStatus && previousWalletStatus.regionRouting) || null,
        providers: payload.providers || (previousWalletStatus && previousWalletStatus.providers) || {},
        launchpads: payload.launchpads || (previousWalletStatus && previousWalletStatus.launchpads) || {},
      });
      const latestWalletStatus = getLatestWalletStatus();
      const wallets = latestWalletStatus.wallets || [];
      renderWalletOptions(wallets, latestWalletStatus.selectedWalletKey || "", latestWalletStatus.balanceSol);
      renderSniperUI();
      markBootstrapState({ walletsLoaded: true });
      if (!latestWalletStatus.connected) {
        if (walletBalance) walletBalance.textContent = "-";
        if (metaNode) metaNode.textContent = "No wallet configured. Add SOLANA_PRIVATE_KEY to .env.";
        updateLockedModeFields();
        schedulePopoutAutosize();
        return;
      }

      if (walletBalance) {
        walletBalance.textContent = latestWalletStatus.balanceSol == null
          ? "--"
          : `${Number(latestWalletStatus.balanceSol).toFixed(4)} SOL`;
      }
      if (metaNode) metaNode.textContent = "";
      updateLockedModeFields();
      schedulePopoutAutosize();
    }

    function setWalletDropdownOpen(isOpen) {
      if (walletDropdown) walletDropdown.hidden = !isOpen;
      if (walletTriggerButton) walletTriggerButton.setAttribute("aria-expanded", String(isOpen));
    }

    function toggleWalletDropdown() {
      setWalletDropdownOpen(!walletDropdown || walletDropdown.hidden);
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
      const label = String(target && (target.label || target.provider) || "Target").trim() || "Target";
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

    function summarizeWarmRecoveries(targets, limit = 2) {
      const items = targets.slice(0, limit).map((target) => {
        const name = formatWarmTargetName(target);
        const error = truncateStatusText(target && target.lastRecoveredError || "", 100);
        return error ? `${name}: recovered from ${error}` : `${name}: recovered recently`;
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
          if (isWarmTelemetryTargetRecentlyRecovered(target, nowMs)) {
            const recoveredAt = formatWarmTimestamp(target && target.lastRecoveredAtMs);
            const recoveredError = truncateStatusText(target && target.lastRecoveredError || "", 120);
            return {
              tone: "blue",
              label,
              detail: recoveredError
                ? `Recovered recently${recoveredAt !== "--" ? ` • ${recoveredAt}` : ""} • ${recoveredError}`
                : `Recovered recently${recoveredAt !== "--" ? ` • ${recoveredAt}` : ""}`,
            };
          }
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
        case "active-follow-jobs":
          return "active because follow jobs are running";
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
      const latestRuntimeStatus = getLatestRuntimeStatus();
      const warm = latestRuntimeStatus && latestRuntimeStatus.warm && typeof latestRuntimeStatus.warm === "object"
        ? latestRuntimeStatus.warm
        : null;
      if (!warm || warm.idleSuspendEnabled === false) return false;
      if (warm.active === true) return false;
      return Boolean(warm.suspended) || isWarmAutoPausedReason(warm.reason);
    }

    function clearWalletStatusRefreshTimer() {
      const timer = getWalletStatusRefreshTimer();
      if (!timer) return;
      global.clearTimeout(timer);
      setWalletStatusRefreshTimer(null);
    }

    function syncWalletStatusRefreshLoop({ immediateResume = false } = {}) {
      if (walletRefreshPausedByIdleSuspend()) {
        clearWalletStatusRefreshTimer();
        return;
      }
      if (getWalletStatusRefreshTimer()) return;
      if (immediateResume) {
        refreshWalletStatus(true, true).catch(() => {});
        return;
      }
      scheduleWalletStatusRefresh();
    }

    const WARM_TELEMETRY_FRESH_MS = 180000;
    const WARM_TELEMETRY_RECENT_RECOVERY_MS = 120000;

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

    function isWarmTelemetryTargetRecentlyRecovered(target, refNowMs) {
      if (!isWarmTelemetryTargetHealthy(target, refNowMs)) {
        return false;
      }
      const recoveredError = String(target && target.lastRecoveredError || "").trim();
      if (!recoveredError) {
        return false;
      }
      const recoveredAt = Number(target && target.lastRecoveredAtMs || 0);
      return Number.isFinite(recoveredAt)
        && recoveredAt > 0
        && (refNowMs - recoveredAt) <= WARM_TELEMETRY_RECENT_RECOVERY_MS;
    }

    function startupWarmSnapshot() {
      const startupWarmState = getStartupWarmState();
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

    function runtimeFollowDaemonStatus() {
      const latestRuntimeStatus = getLatestRuntimeStatus();
      return latestRuntimeStatus && latestRuntimeStatus.followDaemon && typeof latestRuntimeStatus.followDaemon === "object"
        ? latestRuntimeStatus.followDaemon
        : null;
    }

    function clearFollowJobsRefreshTimer() {
      const followJobsState = getFollowJobsState();
      if (!followJobsState.refreshTimer) return;
      global.clearTimeout(followJobsState.refreshTimer);
      setFollowJobsState({
        ...followJobsState,
        refreshTimer: null,
      });
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
      const followJobsState = getFollowJobsState();
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
      const shouldRefresh = snapshot.offline
        || snapshot.counts.active > 0
        || Boolean(reportsTerminalSection && !reportsTerminalSection.hidden);
      if (!shouldRefresh) return;
      const delayMs = snapshot.offline ? followJobsOfflineRetryMs : followJobsRefreshIntervalMs;
      const followJobsState = getFollowJobsState();
      setFollowJobsState({
        ...followJobsState,
        refreshTimer: global.setTimeout(() => {
          refreshFollowJobs({ silent: true }).catch(() => {});
        }, delayMs),
      });
    }

    async function refreshFollowJobs({ silent = false } = {}) {
      clearFollowJobsRefreshTimer();
      const runtimeFollow = runtimeFollowDaemonStatus();
      if (runtimeFollow && runtimeFollow.configured === false) {
        setFollowJobsState({
          ...getFollowJobsState(),
          configured: false,
          reachable: false,
          jobs: [],
          health: null,
          error: "",
          loaded: true,
          refreshTimer: null,
        });
        syncFollowStatusChrome();
        return;
      }
      try {
        const result = requestUtils.fetchJsonLatest
          ? await requestUtils.fetchJsonLatest("follow-jobs", "/api/follow/jobs", {}, requestStates.followJobs)
          : null;
        if (result && result.aborted) return;
        const response = result ? result.response : await fetch("/api/follow/jobs");
        const payload = result ? result.payload : await response.json();
        if (result && !result.isLatest) return;
        if (!response.ok || !payload.ok) {
          throw new Error(payload.error || "Failed to load follow launch status.");
        }
        setFollowJobsState({
          ...getFollowJobsState(),
          configured: true,
          reachable: true,
          jobs: Array.isArray(payload.jobs) ? payload.jobs : [],
          health: payload.health && typeof payload.health === "object" ? payload.health : null,
          error: "",
          loaded: true,
          refreshTimer: null,
        });
      } catch (error) {
        const nextFollowJobsState = {
          ...getFollowJobsState(),
          configured: Boolean(runtimeFollow && runtimeFollow.configured),
          reachable: false,
          jobs: [],
          health: runtimeFollow && runtimeFollow.health && typeof runtimeFollow.health === "object" ? runtimeFollow.health : null,
          error: error && error.message ? error.message : "Failed to load follow launch status.",
          loaded: true,
          refreshTimer: null,
        };
        setFollowJobsState(nextFollowJobsState);
        if (!silent && reportsTerminalOutput && ["launches", "active-jobs"].includes(normalizeReportsTerminalView(getReportsTerminalState().view))) {
          getReportsTerminalState().activeText = nextFollowJobsState.error;
          renderReportsTerminalOutput();
        }
      }
      syncFollowStatusChrome();
      if (["launches", "active-jobs"].includes(normalizeReportsTerminalView(getReportsTerminalState().view)) && reportsTerminalOutput) {
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
      setFollowJobsState({
        ...getFollowJobsState(),
        jobs: Array.isArray(payload.jobs) ? payload.jobs : getFollowJobsState().jobs,
        health: payload.health && typeof payload.health === "object" ? payload.health : getFollowJobsState().health,
        reachable: true,
        configured: true,
        loaded: true,
        error: "",
        refreshTimer: null,
      });
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
      setFollowJobsState({
        ...getFollowJobsState(),
        jobs: Array.isArray(payload.jobs) ? payload.jobs : [],
        health: payload.health && typeof payload.health === "object" ? payload.health : getFollowJobsState().health,
        reachable: true,
        configured: true,
        loaded: true,
        error: "",
        refreshTimer: null,
      });
      syncFollowStatusChrome();
      scheduleFollowJobsRefresh();
    }

    function activeFollowJobForTraceId(traceId) {
      const normalized = String(traceId || "").trim();
      if (!normalized) return null;
      const followJobsState = getFollowJobsState();
      const job = followJobsState.jobs.find((entry) => String(entry && entry.traceId || "").trim() === normalized);
      if (!job || isTerminalFollowJobState(job.state)) return null;
      return job;
    }

    function currentWatchPathSnapshot() {
      const runtimeFollow = runtimeFollowDaemonStatus();
      const followJobsState = getFollowJobsState();
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
      const latestRuntimeStatus = getLatestRuntimeStatus();
      const startupWarmState = getStartupWarmState();
      const appBootstrapState = getAppBootstrapState();
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
      const recoveredActiveStateTargets = activeStateTargets.filter((target) => isWarmTelemetryTargetRecentlyRecovered(target, nowMs));
      const recoveredActiveEndpointTargets = activeEndpointTargets.filter((target) => isWarmTelemetryTargetRecentlyRecovered(target, nowMs));
      const recoveredActiveWatchTargets = activeWatchTargets.filter((target) => isWarmTelemetryTargetRecentlyRecovered(target, nowMs));
      const staleActiveStateTargets = activeStateTargets.filter((target) => !isWarmTelemetryTargetHealthy(target, nowMs) && !isWarmTelemetryTargetRateLimited(target) && !String(target && target.lastError || "").trim());
      const staleActiveEndpointTargets = activeEndpointTargets.filter((target) => !isWarmTelemetryTargetHealthy(target, nowMs) && !isWarmTelemetryTargetRateLimited(target) && !String(target && target.lastError || "").trim());
      const staleActiveWatchTargets = activeWatchTargets.filter((target) => !isWarmTelemetryTargetHealthy(target, nowMs) && !isWarmTelemetryTargetRateLimited(target) && !String(target && target.lastError || "").trim());
      const failingActiveStateTargets = activeStateTargets.filter((target) => String(target && target.lastError || "").trim());
      const failingActiveEndpointTargets = activeEndpointTargets.filter((target) => String(target && target.lastError || "").trim());
      const failingActiveWatchTargets = activeWatchTargets.filter((target) => String(target && target.lastError || "").trim());
      const failingStateTargets = stateTargets.filter((target) => String(target && target.lastError || "").trim());
      const failingEndpointTargets = endpointTargets.filter((target) => String(target && target.lastError || "").trim());
      const failingWatchTargets = watchTargets.filter((target) => String(target && target.lastError || "").trim());
      const warmProviders = warm && Array.isArray(warm.selectedProviders)
        ? warm.selectedProviders.map((value) => String(value || "").trim()).filter(Boolean)
        : [];
      const warmPassInFlight = Boolean(warm && warm.passInFlight);
      const endpointTargetProviders = uniqueWarmTargetProviders(activeEndpointTargets.length ? activeEndpointTargets : endpointTargets);
      const endpointLabelProviders = endpointTargetProviders.length ? endpointTargetProviders : warmProviders;
      const senderConnectionWarm = endpointLabelProviders.length === 1 && endpointLabelProviders[0] === "helius-sender";
      const endpointWarmLabel = senderConnectionWarm ? "Sender connection warm" : "Endpoint prewarm";
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
        } else if (recoveredActiveStateTargets.length > 0) {
          stateWarm = {
            tone: "blue",
            title: `State Warm: recently recovered. ${summarizeWarmRecoveries(recoveredActiveStateTargets)}`,
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
        } else if (warmPassInFlight && warm.active) {
          stateWarm = {
            tone: "blue",
            title: `State Warm: starting. ${describeWarmReason(warmReason)}.`,
          };
        } else if (warm.active && warmError) {
          stateWarm = {
            tone: "red",
            title: `State Warm: failing. ${warmError}`,
          };
        } else {
          const warmReasonText = describeWarmReason(warmReason);
          const autoPaused = isWarmAutoPausedReason(warmReason);
          stateWarm = {
            tone: autoPaused ? "blue" : "yellow",
            title: stateTargets.length > 0
              ? (autoPaused
                ? `State Warm: auto-paused. ${warmReasonText}. Last success ${lastWarmSuccess}.`
                : `State Warm: waiting. ${warmReasonText}. Last success ${lastWarmSuccess}.`)
              : startupWarmInProgress
                ? "State Warm: starting."
                : autoPaused
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
        } else if (recoveredActiveEndpointTargets.length > 0) {
          endpointPrewarm = {
            tone: "blue",
            title: `${endpointWarmLabel}: recently recovered. ${summarizeWarmRecoveries(recoveredActiveEndpointTargets)}`,
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
        } else if (warmPassInFlight && warm.active && warmProviders.length > 0) {
          endpointPrewarm = {
            tone: "blue",
            title: `${endpointWarmLabel}: starting. ${describeWarmReason(warmReason)}. Selected providers: ${formatWarmProviders(warmProviders)}.`,
          };
        } else if (warmProviders.length > 0) {
          const warmReasonText = describeWarmReason(warmReason);
          const autoPaused = isWarmAutoPausedReason(warmReason);
          endpointPrewarm = {
            tone: autoPaused ? "blue" : "yellow",
            title: endpointTargets.length > 0
              ? (autoPaused
                ? `${endpointWarmLabel}: auto-paused. ${warmReasonText}. Selected providers: ${formatWarmProviders(warmProviders)}.`
                : `${endpointWarmLabel}: waiting for telemetry. ${warmReasonText}. Selected providers: ${formatWarmProviders(warmProviders)}.`)
              : startupWarmInProgress
                ? `${endpointWarmLabel}: starting. Selected providers: ${formatWarmProviders(warmProviders)}.`
                : autoPaused
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
        } else if (recoveredActiveWatchTargets.length > 0) {
          watchPrewarm = {
            tone: "blue",
            title: `Watcher WS warm: recently recovered. ${summarizeWarmRecoveries(recoveredActiveWatchTargets)}`,
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
        } else if (warmPassInFlight && warm.active) {
          watchPrewarm = {
            tone: "blue",
            title: "Watcher WS warm: starting.",
          };
        } else {
          const warmReasonText = describeWarmReason(warmReason);
          const autoPaused = isWarmAutoPausedReason(warmReason);
          watchPrewarm = {
            tone: autoPaused ? "blue" : "yellow",
            title: watchTargets.length > 0
              ? (autoPaused
                ? `Watcher WS warm: auto-paused. ${warmReasonText}.`
                : `Watcher WS warm: waiting. ${warmReasonText}.`)
              : startupWarmInProgress
                ? "Watcher WS warm: starting."
                : autoPaused
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
      } else if (warmPassInFlight && !startupWarmInProgress && failingComponents === 0) {
        title = "Warm: starting.";
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
      const latestRuntimeStatus = getLatestRuntimeStatus();
      const rt = latestRuntimeStatus && latestRuntimeStatus.rpcTraffic && typeof latestRuntimeStatus.rpcTraffic === "object"
        ? latestRuntimeStatus.rpcTraffic
        : null;
      if (!latestRuntimeStatus || !rt) {
        return {
          text: "--/min",
          title: "Outbound RPC-credit requests in the last 60 seconds. Waiting for runtime status...",
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
          text: "--/min",
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
      if (renderUtils.setCachedHTML) {
        renderUtils.setCachedHTML(renderCache, "platformRuntimeIndicators", platformRuntimeIndicators, markup);
      } else {
        platformRuntimeIndicators.innerHTML = markup;
      }
    }

    function applyRuntimeStatusPayload(payload, { hydrateOnly = false } = {}) {
      setLatestRuntimeStatus(payload);
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
      const initialState = getWarmActivityState();
      if (initialState.debounceTimer) {
        global.clearTimeout(initialState.debounceTimer);
        setWarmActivityState({
          ...initialState,
          debounceTimer: null,
        });
      }
      const currentState = getWarmActivityState();
      if (currentState.inFlightPromise) {
        setWarmActivityState({
          ...currentState,
          pendingFlush: true,
        });
        return currentState.inFlightPromise;
      }
      const inFlightPromise = fetch("/api/warm/activity", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(currentWarmActivityPayload()),
      })
        .then((response) => response.json().catch(() => ({})).then((payload) => ({ response, payload })))
        .then(({ response, payload }) => {
          if (!response.ok || !payload.ok || !payload.warm) return;
          const latestRuntimeStatus = getLatestRuntimeStatus();
          setLatestRuntimeStatus({
            ...(latestRuntimeStatus || {}),
            warm: payload.warm,
            ...(payload.rpcTraffic && typeof payload.rpcTraffic === "object"
              ? { rpcTraffic: payload.rpcTraffic }
              : {}),
          });
          renderBackendRegionSummary();
          syncFollowStatusChrome();
          syncWalletStatusRefreshLoop({ immediateResume: true });
        })
        .catch(() => {})
        .finally(() => {
          const followUpState = getWarmActivityState();
          const shouldFlushAgain = Boolean(followUpState.pendingFlush);
          setWarmActivityState({
            ...followUpState,
            inFlightPromise: null,
            pendingFlush: false,
          });
          if (shouldFlushAgain) {
            flushWarmActivity().catch(() => {});
          }
        });
      setWarmActivityState({
        ...currentState,
        inFlightPromise,
        pendingFlush: false,
        lastSentAtMs: Date.now(),
        debounceTimer: null,
      });
      return inFlightPromise;
    }

    function queueWarmActivity({ immediate = false } = {}) {
      const warmActivityState = getWarmActivityState();
      const now = Date.now();
      if (immediate || now - warmActivityState.lastSentAtMs >= warmActivityDebounceMs) {
        flushWarmActivity().catch(() => {});
        return;
      }
      if (warmActivityState.debounceTimer) global.clearTimeout(warmActivityState.debounceTimer);
      setWarmActivityState({
        ...warmActivityState,
        debounceTimer: global.setTimeout(() => {
          flushWarmActivity().catch(() => {});
        }, warmActivityDebounceMs),
      });
    }

    function scheduleWalletStatusRefresh() {
      clearWalletStatusRefreshTimer();
      if (walletRefreshPausedByIdleSuspend()) return;
      const delayMs = getWalletStatusRefreshIntervalMs();
      setWalletStatusRefreshTimer(global.setTimeout(() => {
        setWalletStatusRefreshTimer(null);
        refreshWalletStatus(true, true).catch(() => {});
      }, delayMs));
    }

    function startRuntimeStatusRefreshLoop() {
      const runtimeStatusRefreshTimer = getRuntimeStatusRefreshTimer();
      if (runtimeStatusRefreshTimer) global.clearInterval(runtimeStatusRefreshTimer);
      loadRuntimeStatus().catch(() => {});
      if (reportsTerminalSection && !reportsTerminalSection.hidden && normalizeReportsTerminalView(getReportsTerminalState().view) === "active-logs") {
        refreshActiveLogs({ showLoading: false }).catch(() => {});
      }
      setRuntimeStatusRefreshTimer(global.setInterval(() => {
        loadRuntimeStatus().catch(() => {});
        if (reportsTerminalSection && !reportsTerminalSection.hidden && normalizeReportsTerminalView(getReportsTerminalState().view) === "active-logs") {
          refreshActiveLogs({ showLoading: false }).catch(() => {});
        }
      }, runtimeStatusRefreshIntervalMs));
    }

    function applyBootstrapFastPayload(payload) {
      const startupWarmState = getStartupWarmState();
      setStartupWarmState({
        ...startupWarmState,
        enabled: payload && payload.startupWarm
          ? payload.startupWarm.enabled !== false
          : true,
      });
      const configuredWalletStatusRefreshIntervalMs = Number(
        payload && payload.uiRefresh && payload.uiRefresh.walletStatusIntervalMs,
      );
      if (Number.isFinite(configuredWalletStatusRefreshIntervalMs) && configuredWalletStatusRefreshIntervalMs > 0) {
        setWalletStatusRefreshIntervalMs(Math.max(1000, Math.round(configuredWalletStatusRefreshIntervalMs)));
      }
      renderPlatformRuntimeIndicators();
      const previousWalletStatus = getLatestWalletStatus() || null;
      const previousWallets = previousWalletStatus && Array.isArray(previousWalletStatus.wallets)
        ? normalizeVisibleWallets(previousWalletStatus.wallets)
        : [];
      const nextWallets = Array.isArray(payload.wallets)
        ? normalizeVisibleWallets(payload.wallets)
        : previousWallets;
      const selectedWalletKeyValue = resolveVisibleSelectedWalletKey(
        payload.selectedWalletKey || (previousWalletStatus && previousWalletStatus.selectedWalletKey) || "",
        nextWallets,
      );
      const selectedWallet = nextWallets.find((wallet) => wallet.envKey === selectedWalletKeyValue) || null;
      setLatestWalletStatus({
        ...(previousWalletStatus || {}),
        selectedWalletKey: selectedWalletKeyValue,
        wallets: nextWallets,
        wallet: selectedWallet ? selectedWallet.publicKey : null,
        connected: Boolean(selectedWallet && selectedWallet.publicKey),
        balanceLamports: selectedWallet && selectedWallet.balanceLamports != null
          ? selectedWallet.balanceLamports
          : null,
        balanceSol: selectedWallet && selectedWallet.balanceSol != null
          ? selectedWallet.balanceSol
          : null,
        usd1Balance: selectedWallet && selectedWallet.usd1Balance != null
          ? selectedWallet.usd1Balance
          : null,
        config: payload.config || (previousWalletStatus && previousWalletStatus.config) || null,
        regionRouting: payload.regionRouting || (previousWalletStatus && previousWalletStatus.regionRouting) || null,
        providers: payload.providers || (previousWalletStatus && previousWalletStatus.providers) || {},
        launchpads: payload.launchpads || (previousWalletStatus && previousWalletStatus.launchpads) || {},
      });
      const latestWalletStatus = getLatestWalletStatus();
      renderWalletOptions(latestWalletStatus.wallets || [], latestWalletStatus.selectedWalletKey || "", latestWalletStatus.balanceSol);
      applyPersistentDefaults(payload.config);
      applyProviderAvailability(payload.providers || {});
      applyLaunchpadAvailability(payload.launchpads || {});
      const activeConfig = (getLatestWalletStatus() && getLatestWalletStatus().config) || payload.config || null;
      renderQuickDevBuyButtons(activeConfig);
      populateDevBuyPresetEditor(activeConfig);
      updateQuote().catch(() => {});
      renderSniperUI();
      renderBackendRegionSummary(payload.regionRouting);
      markBootstrapState({
        staticLoaded: true,
        configLoaded: Boolean(activeConfig),
      });
      setSettingsLoadingState(!hasBootstrapConfig());
      schedulePopoutAutosize();
    }

    return {
      activeFollowJobForTraceId,
      applyBootstrapFastPayload,
      applyRuntimeStatusPayload,
      applySelectedWalletLocally,
      applyWalletStatusPayload,
      buildFollowJobsSummaryText,
      cancelAllFollowJobs,
      cancelFollowJob,
      clearFollowJobsRefreshTimer,
      copyWalletDropdownAddress,
      currentWarmActivityPayload,
      flushWarmActivity,
      followStatusSnapshot,
      getStoredSelectedWalletKey,
      queueWarmActivity,
      refreshFollowJobs,
      refreshWalletStatus,
      renderPlatformRuntimeIndicators,
      setStoredSelectedWalletKey,
      setWalletDropdownOpen,
      startRuntimeStatusRefreshLoop,
      syncFollowStatusChrome,
      toggleWalletDropdown,
    };
  }

  global.LaunchDeckWalletRuntimeDomain = {
    create: createWalletRuntimeDomain,
  };
})(window);
