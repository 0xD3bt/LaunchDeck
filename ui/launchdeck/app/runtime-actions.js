(function initLaunchDeckRuntimeActions(global) {
  function createRuntimeActions(config) {
    const {
      requestUtils,
      requestStates,
      markBootstrapState,
      setSettingsLoadingState,
      setBootOverlayMessage,
      getStoredSelectedWalletKey,
      applyBootstrapFastPayload,
      beginStartupWarmup,
      flushWarmActivity,
      refreshRuntimeStatus,
      refreshWalletStatus,
      ensureInteractiveBootstrapReady,
      setBusy,
      output,
      stopOutputFollowRefresh,
      ensureStartupWarmReady,
      ensureMetadataReadyForAction,
      readForm,
      buildOutputMetaTextFromReport,
      metaNode,
      metadataUri,
      getMetadataUploadState,
      metadataFingerprintFromForm,
      surfaceMetadataWarning,
      currentStatusLabel,
      setStatusLabel,
      getReportsTerminalState,
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
      buildSavedConfigFromForm,
      cloneConfig,
      capturePreviewInputsFromRunReport = () => {},
      setRegionRouting,
      getLatestWalletStatus,
      setConfig,
      renderQuickDevBuyButtons,
      populateDevBuyPresetEditor,
      renderBackendRegionSummary,
      queueWarmActivity,
      hideSettingsModal,
    } = config;

    function formatBootTaskList(labels) {
      const items = (Array.isArray(labels) ? labels : [])
        .map((label) => String(label || "").trim())
        .filter(Boolean);
      if (!items.length) return "";
      if (items.length === 1) return items[0];
      if (items.length === 2) return `${items[0]} and ${items[1]}`;
      return `${items.slice(0, -1).join(", ")}, and ${items[items.length - 1]}`;
    }

    function updateBootTaskProgress(summary, pendingLabels) {
      const remaining = formatBootTaskList(pendingLabels);
      setBootOverlayMessage(
        "Loading LaunchDeck",
        remaining ? `${summary} Still syncing ${remaining}` : "Finishing workspace and controls",
      );
    }

    async function bootstrapApp() {
      markBootstrapState({ started: true });
      setSettingsLoadingState(true);
      setBootOverlayMessage("Loading LaunchDeck", "Connecting to engine");
      const storedWalletKey = getStoredSelectedWalletKey();
      const bootstrapUrl = storedWalletKey
        ? `/api/bootstrap-fast?wallet=${encodeURIComponent(storedWalletKey)}`
        : "/api/bootstrap-fast";
      const result = requestUtils.fetchJsonLatest
        ? await requestUtils.fetchJsonLatest("bootstrap-fast", bootstrapUrl, {}, requestStates.bootstrap)
        : null;
      if (result && result.aborted) return;
      const response = result ? result.response : await fetch(bootstrapUrl);
      const payload = result ? result.payload : await response.json();
      if (result && !result.isLatest) return;
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Failed to load app bootstrap.");
      }
      applyBootstrapFastPayload(payload);
      setBootOverlayMessage(
        "Loading LaunchDeck",
        "Restoring saved workspace, launchpad defaults, and selected wallet",
      );
      const bootTasks = [
        {
          pendingLabel: "hot launch caches",
          successLabel: "Hot launch caches ready.",
          failureLabel: "Hot launch caches unavailable; continuing.",
          promise: beginStartupWarmup(),
        },
        {
          pendingLabel: "live routing state",
          successLabel: "Live routing state synced.",
          failureLabel: "Live routing state unavailable; continuing.",
          promise: flushWarmActivity().then(() => refreshRuntimeStatus()),
        },
        {
          pendingLabel: "runtime health",
          successLabel: "Runtime health ready.",
          failureLabel: "Runtime health unavailable; continuing.",
          promise: refreshRuntimeStatus(),
        },
        {
          pendingLabel: "wallet balances",
          successLabel: "Wallet balances ready.",
          failureLabel: "Wallet balances unavailable; continuing.",
          promise: refreshWalletStatus(true, true),
        },
      ];
      const pendingLabels = new Set(bootTasks.map((task) => task.pendingLabel));
      setBootOverlayMessage(
        "Loading LaunchDeck",
        `Refreshing ${formatBootTaskList(Array.from(pendingLabels))}`,
      );
      const trackedBootTasks = bootTasks.map((task) => Promise.resolve(task.promise)
        .then(() => {
          pendingLabels.delete(task.pendingLabel);
          updateBootTaskProgress(task.successLabel, Array.from(pendingLabels));
        })
        .catch(() => {
          pendingLabels.delete(task.pendingLabel);
          updateBootTaskProgress(task.failureLabel, Array.from(pendingLabels));
        }));
      await Promise.allSettled(trackedBootTasks);
    }

    async function loadRuntimeStatus() {
      try {
        const result = requestUtils.fetchJsonLatest
          ? await requestUtils.fetchJsonLatest("runtime-status", "/api/runtime-status", {}, requestStates.runtimeStatus)
          : null;
        if (result && result.aborted) return;
        const response = result ? result.response : await fetch("/api/runtime-status");
        const payload = result ? result.payload : await response.json();
        if (result && !result.isLatest) return;
        if (!response.ok || !payload.ok) {
          throw new Error(payload.error || "Failed to load runtime status.");
        }
        config.applyRuntimeStatusPayload(payload);
      } catch (_error) {
        // Keep runtime hydration best-effort so boot remains responsive.
      }
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
          const metadataUploadState = getMetadataUploadState();
          metadataUploadState.completedFingerprint = metadataFingerprintFromForm(readForm());
        }
        surfaceMetadataWarning(payload.metadataWarning);
        output.textContent = payload.text;
        capturePreviewInputsFromRunReport(payload.report || null);
        setBusy(false, currentStatusLabel());
        if (payload.sendLogPath) {
          const reportId = extractReportIdFromPath(payload.sendLogPath);
          const reportsTerminalState = getReportsTerminalState();
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
            if (config.reportsTerminalOutput && config.reportsTerminalSection && !config.reportsTerminalSection.hidden) {
              const nextReportsTerminalState = getReportsTerminalState();
              nextReportsTerminalState.activePayload = null;
              nextReportsTerminalState.activeText = error.message || "Failed to refresh reports.";
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
        const result = requestUtils.fetchJsonLatest
          ? await requestUtils.fetchJsonLatest("settings-save", "/api/settings", {
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
        setRegionRouting(payload.regionRouting || (getLatestWalletStatus() && getLatestWalletStatus().regionRouting));
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

    return {
      bootstrapApp,
      refreshRuntimeStatus: loadRuntimeStatus,
      run,
      saveSettings,
    };
  }

  global.LaunchDeckRuntimeActions = {
    createRuntimeActions,
  };
})(window);
