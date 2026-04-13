(function initLaunchDeckLiveSync(global) {
  const LIVE_SYNC_CHANNEL_NAME = "launchdeck-live-sync.v1";
  const LIVE_SYNC_STORAGE_KEY = "launchdeck.liveSyncEvent.v1";
  const LIVE_SYNC_SESSION_STORAGE_KEY = "launchdeck.liveSyncSnapshot.v1";
  const EARLY_BOOT_STORAGE_KEY = "launchdeck.earlyBootSnapshot.v1";
  const EARLY_BOOT_SESSION_STORAGE_KEY = "launchdeck.earlyBootSnapshot.session.v1";
  const LIVE_SYNC_MAX_AGE_MS = 5 * 60 * 1000;

  function isFreshLiveSyncPayload(payload) {
    if (!payload || typeof payload !== "object") return false;
    const timestampMs = Number(payload.timestampMs);
    return Number.isFinite(timestampMs) && (Date.now() - timestampMs) <= LIVE_SYNC_MAX_AGE_MS;
  }

  function readFreshStoredPayload(storage, key) {
    try {
      if (!storage || typeof storage.getItem !== "function") return null;
      const raw = storage.getItem(key);
      if (!raw) return null;
      const parsed = JSON.parse(raw);
      return isFreshLiveSyncPayload(parsed) ? parsed : null;
    } catch (_error) {
      return null;
    }
  }

  function readStoredLiveSyncPayload() {
    return readFreshStoredPayload(global.localStorage, LIVE_SYNC_STORAGE_KEY);
  }

  function readStoredEarlyBootSnapshot() {
    return readFreshStoredPayload(global.localStorage, EARLY_BOOT_STORAGE_KEY);
  }

  function readSessionEarlyBootSnapshot() {
    return readFreshStoredPayload(global.sessionStorage, EARLY_BOOT_SESSION_STORAGE_KEY);
  }

  function readSessionLiveSyncPayload() {
    return readFreshStoredPayload(global.sessionStorage, LIVE_SYNC_SESSION_STORAGE_KEY);
  }

  function readOpenerLiveSyncPayload() {
    try {
      if (!global.opener || global.opener === global) return null;
      const payload = global.opener.__launchdeckLiveSyncSnapshot;
      return isFreshLiveSyncPayload(payload) ? payload : null;
    } catch (_error) {
      return null;
    }
  }

  function readEarlyLiveSyncSnapshot() {
    return readOpenerLiveSyncPayload()
      || readSessionEarlyBootSnapshot()
      || readStoredEarlyBootSnapshot()
      || readSessionLiveSyncPayload()
      || readStoredLiveSyncPayload();
  }

  function createLiveSyncSupport(config) {
    const {
      liveSyncSourceId,
      getCurrentThemeMode,
      isOutputSectionCurrentlyVisible,
      isReportsTerminalCurrentlyVisible,
      getCurrentReportsTerminalListWidth,
      isImageLayoutCompactActive,
      getSelectedWalletValue,
      cloneConfig,
      getConfig,
      getLatestWalletStatus,
      getLatestRuntimeStatus,
      getStartupWarmState,
      setStartupWarmState,
      getReportsTerminalState,
      setReportsTerminalState,
      getFollowJobsState,
      setFollowJobsState,
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
      isPopoutMode = false,
    } = config;

    let liveSyncTimer = 0;
    let liveSyncReady = false;
    let isApplyingLiveSync = false;
    let globalListenersBound = false;
    const liveSyncChannel = typeof global.BroadcastChannel === "function"
      ? new global.BroadcastChannel(LIVE_SYNC_CHANNEL_NAME)
      : null;

    function buildLiveSyncPayload() {
      const startupWarmState = getStartupWarmState();
      const reportsTerminalState = getReportsTerminalState();
      const followJobsState = getFollowJobsState();
      return {
        sourceId: liveSyncSourceId,
        timestampMs: Date.now(),
        themeMode: getCurrentThemeMode(),
        outputVisible: isOutputSectionCurrentlyVisible(),
        reportsVisible: isReportsTerminalCurrentlyVisible(),
        reportsListWidth: getCurrentReportsTerminalListWidth(),
        imageLayoutCompact: isImageLayoutCompactActive(),
        selectedWalletKey: getSelectedWalletValue(),
        config: cloneConfig(getConfig()),
        walletStatusSnapshot: getLatestWalletStatus(),
        runtimeStatusSnapshot: getLatestRuntimeStatus(),
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
        feeSplitDraft: normalizeFeeSplitDraft(serializeFeeSplitDraft()),
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
        feeSplitDraft: payload.feeSplitDraft || null,
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
        feeSplitDraft: payload.feeSplitDraft || null,
        formControls: payload.formControls || {},
      };
    }

    function dispatchLiveSyncPayload(payload) {
      if (!payload || typeof payload !== "object") return;
      const earlyBootPayload = buildEarlyBootSnapshot(payload);
      const persistedPayload = buildPersistedLiveSyncPayload(payload);
      try {
        global.__launchdeckLiveSyncSnapshot = payload;
      } catch (_error) {
        // Ignore window assignment failures and continue with other sync paths.
      }
      try {
        global.__launchdeckEarlyLiveSyncSnapshot = earlyBootPayload;
      } catch (_error) {
        // Ignore window assignment failures and continue with storage fallbacks.
      }
      try {
        global.sessionStorage.setItem(EARLY_BOOT_SESSION_STORAGE_KEY, JSON.stringify(earlyBootPayload));
      } catch (_error) {
        // Ignore session storage failures and continue with other sync paths.
      }
      try {
        global.sessionStorage.setItem(LIVE_SYNC_SESSION_STORAGE_KEY, JSON.stringify(persistedPayload));
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
        global.localStorage.setItem(EARLY_BOOT_STORAGE_KEY, JSON.stringify(earlyBootPayload));
      } catch (_error) {
        // Ignore storage failures and keep live sync best-effort.
      }
      try {
        global.localStorage.setItem(LIVE_SYNC_STORAGE_KEY, JSON.stringify(persistedPayload));
      } catch (_error) {
        // Ignore storage failures and keep live sync best-effort.
      }
    }

    function scheduleLiveSyncBroadcast({ immediate = false } = {}) {
      if (!liveSyncReady || isApplyingLiveSync) return;
      if (liveSyncTimer) {
        global.clearTimeout(liveSyncTimer);
        liveSyncTimer = 0;
      }
      if (immediate) {
        dispatchLiveSyncPayload(buildLiveSyncPayload());
        return;
      }
      liveSyncTimer = global.setTimeout(() => {
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
          control.dispatchEvent(new global.Event("change", { bubbles: true }));
          return;
        }
        const nextValue = snapshot.value == null ? "" : String(snapshot.value);
        if (String(control.value) === nextValue) return;
        control.value = nextValue;
        const eventName = control.tagName === "SELECT" ? "change" : "input";
        control.dispatchEvent(new global.Event(eventName, { bubbles: true }));
      });
    }

    function applyIncomingLiveSyncPayload(payload, {
      allowBeforeReady = false,
      skipVisibilityState = false,
      skipDashboardViewState = false,
      skipThemeMode = false,
      skipFormControls = false,
      skipWalletStatusSnapshot = false,
      restorePersistedFormControlsOnly = false,
      restoreOutputFromSync = true,
    } = {}) {
      if (!allowBeforeReady && !liveSyncReady) return;
      if (!payload || typeof payload !== "object" || payload.sourceId === liveSyncSourceId) return;
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
        if (payload.config && typeof payload.config === "object" && !restorePersistedFormControlsOnly) {
          const nextConfig = cloneConfig(payload.config);
          setConfig(nextConfig);
          setPresetEditing(isPresetEditing(nextConfig));
          applyPresetToSettingsInputs(getActivePreset(nextConfig), { syncToMainForm: false });
          queueWarmActivity({ immediate: true });
        }
        if (payload.startupWarmSnapshot && typeof payload.startupWarmSnapshot === "object") {
          setStartupWarmState({
            ...getStartupWarmState(),
            started: Boolean(payload.startupWarmSnapshot.started),
            ready: Boolean(payload.startupWarmSnapshot.ready),
            enabled: payload.startupWarmSnapshot.enabled !== false,
            backendLoaded: Boolean(payload.startupWarmSnapshot.backendLoaded),
            backendPayload: payload.startupWarmSnapshot.backendPayload && typeof payload.startupWarmSnapshot.backendPayload === "object"
              ? payload.startupWarmSnapshot.backendPayload
              : null,
            backendError: String(payload.startupWarmSnapshot.backendError || ""),
            promise: null,
          });
          renderPlatformRuntimeIndicators();
        }
        if (!skipWalletStatusSnapshot && payload.walletStatusSnapshot && typeof payload.walletStatusSnapshot === "object") {
          applyWalletStatusPayload(payload.walletStatusSnapshot);
        }
        if (payload.runtimeStatusSnapshot && typeof payload.runtimeStatusSnapshot === "object") {
          applyRuntimeStatusPayload(payload.runtimeStatusSnapshot, { hydrateOnly: true });
        }
        if (payload.followJobsSnapshot && typeof payload.followJobsSnapshot === "object") {
          clearFollowJobsRefreshTimer();
          setFollowJobsState({
            ...getFollowJobsState(),
            ...payload.followJobsSnapshot,
            refreshTimer: null,
          });
          syncFollowStatusChrome();
          renderReportsTerminalList();
          renderReportsTerminalOutput();
        }
        if (payload.reportsTerminalSnapshot && typeof payload.reportsTerminalSnapshot === "object") {
          const nextReportsTerminalState = {
            ...getReportsTerminalState(),
            ...payload.reportsTerminalSnapshot,
          };
          if (skipDashboardViewState) {
            nextReportsTerminalState.view = getStoredReportsTerminalView();
            nextReportsTerminalState.activeLogsView = getStoredActiveLogsView();
          }
          setReportsTerminalState(nextReportsTerminalState);
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
          walletSelect.dispatchEvent(new global.Event("change", { bubbles: true }));
        }
        if (restorePersistedFormControlsOnly) {
          applyLiveSyncFormControls(filterRefreshPersistedFormControls(payload.formControls));
        } else if (!skipFormControls) {
          applyLiveSyncFormControls(payload.formControls);
        }
        if (!restorePersistedFormControlsOnly && payload.feeSplitDraft && typeof payload.feeSplitDraft === "object") {
          const normalizedDraft = normalizeFeeSplitDraft(payload.feeSplitDraft);
          applyFeeSplitDraft(normalizedDraft, { persist: false });
          setStoredFeeSplitDraft(normalizedDraft);
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
          skipWalletStatusSnapshot: true,
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
        || global.__launchdeckEarlyLiveSyncSnapshot
        || readSessionEarlyBootSnapshot()
        || readStoredEarlyBootSnapshot()
        || readSessionLiveSyncPayload()
        || readStoredLiveSyncPayload();
      if (!payload) return false;
      applyIncomingLiveSyncPayload(payload, {
        allowBeforeReady: true,
        skipThemeMode: true,
        skipFormControls: false,
        skipWalletStatusSnapshot: true,
        restorePersistedFormControlsOnly: !isPopoutMode,
        skipVisibilityState: true,
        skipDashboardViewState: true,
        restoreOutputFromSync: isPopoutMode,
      });
      return true;
    }

    function bindGlobalListeners() {
      if (globalListenersBound) return;
      globalListenersBound = true;
      if (liveSyncChannel) {
        liveSyncChannel.addEventListener("message", (event) => {
          applyIncomingLiveSyncPayload(event && event.data);
        });
      }
      global.addEventListener("storage", (event) => {
        if (event.key !== LIVE_SYNC_STORAGE_KEY || !event.newValue) return;
        try {
          applyIncomingLiveSyncPayload(JSON.parse(event.newValue));
        } catch (_error) {
          // Ignore malformed storage sync payloads.
        }
      });
    }

    return {
      applyIncomingLiveSyncPayload,
      applyLiveSyncFormControls,
      bindGlobalListeners,
      buildEarlyBootSnapshot,
      buildLiveSyncPayload,
      buildPersistedLiveSyncPayload,
      dispatchLiveSyncPayload,
      enableLiveSync,
      preloadLiveSyncSnapshot,
      readOpenerLiveSyncPayload,
      readSessionEarlyBootSnapshot,
      readSessionLiveSyncPayload,
      readStoredEarlyBootSnapshot,
      readStoredLiveSyncPayload,
      scheduleLiveSyncBroadcast,
    };
  }

  global.LaunchDeckLiveSync = {
    createLiveSyncSupport,
    readEarlyLiveSyncSnapshot,
  };
})(window);
