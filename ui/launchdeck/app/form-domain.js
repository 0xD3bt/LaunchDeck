(function initLaunchDeckFormDomain(global) {
  function createFormDomain(config) {
    const {
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
      getImportedCreatorFeeState,
      getLaunchpadUiCapabilities,
      getAutoSellTriggerFamily,
      getAutoSellTriggerMode,
      getAutoSellDelayMs,
      getAutoSellBlockOffset,
      getUploadedImage,
      getMetadataUploadState,
      cloneConfig,
      getConfig,
      createFallbackConfig,
      defaultPresetId,
      normalizeSniperDraftState,
      getSniperState,
      normalizeFeeSplitDraft,
      serializeFeeSplitDraft,
      normalizeAgentSplitDraft,
      serializeAgentSplitDraft,
      normalizeAutoSellTriggerFamily,
      normalizeAutoSellTriggerMode,
    } = config;

    function readForm() {
      const data = new FormData(form);
      const values = Object.fromEntries(data.entries());
      const mode = values.mode || "regular";
      const launchpad = getLaunchpad();
      const launchpadCapabilities = getLaunchpadUiCapabilities(launchpad);
      const sniperSupported = Boolean(launchpadCapabilities && launchpadCapabilities.sniper);
      const autoSellSupported = Boolean(launchpadCapabilities && launchpadCapabilities.autoSell);
      const currentConfig = getConfig() || createFallbackConfig();
      const presetItems = currentConfig
        && currentConfig.presets
        && Array.isArray(currentConfig.presets.items)
        ? currentConfig.presets.items
        : [];
      const configuredActivePresetId = currentConfig
        && currentConfig.defaults
        && currentConfig.defaults.activePresetId
        ? String(currentConfig.defaults.activePresetId).trim()
        : "";
      const activePresetId = getActivePresetId() || configuredActivePresetId || defaultPresetId;
      const activePreset = presetItems.find((entry) => entry && entry.id === activePresetId)
        || presetItems[0]
        || {};
      const creationSettings = activePreset && typeof activePreset.creationSettings === "object"
        ? activePreset.creationSettings
        : {};
      const buySettings = activePreset && typeof activePreset.buySettings === "object"
        ? activePreset.buySettings
        : {};
      const sellSettings = activePreset && typeof activePreset.sellSettings === "object"
        ? activePreset.sellSettings
        : {};
      const creationCapabilities = getRouteCapabilities(getProvider(), "creation");
      const buyCapabilities = getRouteCapabilities(getBuyProvider(), "buy");
      const sellCapabilities = getRouteCapabilities(getSellProvider(), "sell");
      const devBuyAmount = String(values.devBuyAmount || "").trim();
      const autoSellRequested = autoSellSupported && isNamedChecked("automaticDevSellEnabled");
      const automaticDevSellEnabled = autoSellRequested && Boolean(devBuyAmount);
      const automaticSniperSellEnabled = autoSellSupported && isNamedChecked("automaticSniperSellEnabled");
      const rawAgentSplitRecipients = mode === "agent-custom" ? collectAgentSplitRecipients() : [];
      const agentSplitRecipients = mode === "agent-custom" && hasMeaningfulAgentSplitRecipients(rawAgentSplitRecipients)
        ? rawAgentSplitRecipients
        : [];
      const agentBuyback = rawAgentSplitRecipients.find((entry) => entry.type === "agent");
      const meaningfulFeeSplitEnabled = mode === "regular"
        ? Boolean(feeSplitEnabled && feeSplitEnabled.checked && hasMeaningfulFeeSplitConfiguration())
        : mode.startsWith("bags-");
      let sniperWallets = [];
      if (sniperSupported) {
        try {
          const parsed = JSON.parse(getNamedValue("sniperConfigJson") || "[]");
          if (Array.isArray(parsed)) {
            sniperWallets = parsed.filter((entry) => entry && entry.envKey);
          }
        } catch (_error) {
          sniperWallets = [];
        }
      }

      const creationMaxFeeSol = normalizeAutoFeeCapValue(getNamedValue("creationMaxFeeSol"));
      const buyMaxFeeSol = normalizeAutoFeeCapValue(getNamedValue("buyMaxFeeSol"));
      const sellMaxFeeSol = normalizeAutoFeeCapValue(getNamedValue("sellMaxFeeSol"));
      const importedCreatorFeeState = getImportedCreatorFeeState();
      const uploadedImage = getUploadedImage();

      return {
        selectedWalletKey: selectedWalletKey(),
        launchpad,
        quoteAsset: getQuoteAsset(),
        provider: getProvider(),
        buyProvider: getBuyProvider(),
        sellProvider: getSellProvider(),
        creationEndpointProfile: String(creationSettings.endpointProfile || "").trim(),
        buyEndpointProfile: String(buySettings.endpointProfile || "").trim(),
        sellEndpointProfile: String(sellSettings.endpointProfile || "").trim(),
        creationMevMode: normalizeMevMode(getNamedValue("creationMevMode"), "off"),
        buyMevMode: normalizeMevMode(getNamedValue("buyMevMode"), "off"),
        sellMevMode: normalizeMevMode(getNamedValue("sellMevMode"), "off"),
        activePresetId,
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
          ? (meaningfulFeeSplitEnabled ? collectSubmittedFeeSplitRecipients(mode) : [])
          : (mode.startsWith("bags-") ? collectSubmittedFeeSplitRecipients(mode) : []),
        creatorFeeMode: importedCreatorFeeState.mode || "",
        creatorFeeAddress: importedCreatorFeeState.address || "",
        creatorFeeGithubUsername: importedCreatorFeeState.githubUsername || "",
        creatorFeeGithubUserId: importedCreatorFeeState.githubUserId || "",
        postLaunchStrategy: sniperSupported ? (getNamedValue("postLaunchStrategy") || "none") : "none",
        snipeBuyAmountSol: sniperSupported ? (getNamedValue("snipeBuyAmountSol") || "") : "",
        sniperEnabled: sniperSupported && getNamedValue("sniperEnabled") === "true",
        sniperWallets,
        sniperConfigJson: sniperSupported ? (getNamedValue("sniperConfigJson") || "[]") : "[]",
        automaticSniperSellEnabled,
        automaticDevSellEnabled,
        automaticDevSellPercent: autoSellSupported ? (getNamedValue("automaticDevSellPercent") || "0") : "0",
        automaticDevSellTriggerFamily: autoSellSupported ? getAutoSellTriggerFamily() : "time",
        automaticDevSellTriggerMode: autoSellSupported ? getAutoSellTriggerMode() : "block-offset",
        automaticDevSellDelayMs: autoSellSupported ? String(getAutoSellDelayMs()) : "0",
        automaticDevSellBlockOffset: autoSellSupported ? String(getAutoSellBlockOffset()) : "0",
        automaticDevSellMarketCapEnabled: autoSellSupported && getAutoSellTriggerFamily() === "market-cap",
        automaticDevSellMarketCapThreshold: autoSellSupported ? (getNamedValue("automaticDevSellMarketCapThreshold") || "") : "",
        automaticDevSellMarketCapScanTimeoutSeconds: autoSellSupported
          ? (getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds")
            || getNamedValue("automaticDevSellMarketCapScanTimeoutMinutes")
            || "30")
          : "30",
        automaticDevSellMarketCapTimeoutAction: autoSellSupported
          ? (getNamedValue("automaticDevSellMarketCapTimeoutAction") || "stop")
          : "stop",
        vanityPrivateKey: getNamedValue("vanityPrivateKey") || "",
        imageFileName: uploadedImage ? uploadedImage.fileName : "",
        metadataUri: metadataUri.value || "",
      };
    }

    function metadataFingerprintFromForm(formValues = readForm()) {
      const uploadedImage = getUploadedImage();
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

    function launchpadHandlesOwnMetadata(formValues = readForm()) {
      return String(formValues && formValues.launchpad || getLaunchpad() || "").trim().toLowerCase() === "bagsapp";
    }

    function canPreuploadMetadata(formValues = readForm()) {
      if (launchpadHandlesOwnMetadata(formValues)) return false;
      return Boolean(
        formValues.imageFileName
        && String(formValues.name || "").trim()
        && String(formValues.symbol || "").trim()
      );
    }

    function hasFreshPreuploadedMetadata(formValues = readForm()) {
      if (!canPreuploadMetadata(formValues) || !metadataUri.value) return false;
      const metadataUploadState = getMetadataUploadState();
      return metadataUploadState.completedFingerprint === metadataFingerprintFromForm(formValues);
    }

    function buildSavedConfigFromForm() {
      const current = cloneConfig(getConfig());
      const base = current || createFallbackConfig();
      const f = readForm();
      const launchpadCapabilities = getLaunchpadUiCapabilities(f.launchpad || "pump");
      const currentMisc = base.defaults && base.defaults.misc ? base.defaults.misc : {};
      const existingSniperDraft = currentMisc.sniperDraft
        || (
          currentMisc.sniperDraftsByLaunchpad
          && typeof currentMisc.sniperDraftsByLaunchpad === "object"
          ? Object.values(currentMisc.sniperDraftsByLaunchpad).find((entry) => entry && typeof entry === "object")
          : null
        );
      const sniperDraft = launchpadCapabilities && launchpadCapabilities.sniper
        ? normalizeSniperDraftState(getSniperState())
        : normalizeSniperDraftState(existingSniperDraft || { enabled: false, wallets: {} });
      const feeSplitDraft = normalizeFeeSplitDraft(serializeFeeSplitDraft());
      const agentSplitDraft = normalizeAgentSplitDraft(serializeAgentSplitDraft());
      const feeSplitLaunchpad = f.launchpad || "pump";
      const shouldPersistFeeSplitDraft = feeSplitLaunchpad === "bagsapp"
        || (feeSplitLaunchpad === "pump" && (f.mode || "regular") === "regular");
      const shouldPersistAgentSplitDraft = feeSplitLaunchpad === "pump"
        && (f.mode || "regular") === "agent-custom";
      const nextFeeSplitDraftsByLaunchpad = {
        ...(currentMisc.feeSplitDraftsByLaunchpad && typeof currentMisc.feeSplitDraftsByLaunchpad === "object"
          ? currentMisc.feeSplitDraftsByLaunchpad
          : {}),
      };
      if (shouldPersistFeeSplitDraft) {
        nextFeeSplitDraftsByLaunchpad[feeSplitLaunchpad] = feeSplitDraft;
      }
      const nextAgentSplitDraftsByLaunchpad = {
        ...(currentMisc.agentSplitDraftsByLaunchpad && typeof currentMisc.agentSplitDraftsByLaunchpad === "object"
          ? currentMisc.agentSplitDraftsByLaunchpad
          : {}),
      };
      if (shouldPersistAgentSplitDraft) {
        nextAgentSplitDraftsByLaunchpad[feeSplitLaunchpad] = agentSplitDraft;
      }
      const existingAutoSellDraft = (base.defaults && base.defaults.automaticDevSell)
        || (
          currentMisc.autoSellDraftsByLaunchpad
          && typeof currentMisc.autoSellDraftsByLaunchpad === "object"
          ? Object.values(currentMisc.autoSellDraftsByLaunchpad).find((entry) => entry && typeof entry === "object")
          : null
        )
        || null;
      const autoSellDraft = launchpadCapabilities && launchpadCapabilities.autoSell
        ? {
          enabled: Boolean(f.automaticDevSellEnabled),
          sniperEnabled: Boolean(f.automaticSniperSellEnabled),
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
        }
        : {
          enabled: Boolean(existingAutoSellDraft && existingAutoSellDraft.enabled),
          sniperEnabled: Boolean(existingAutoSellDraft && existingAutoSellDraft.sniperEnabled),
          percent: Number((existingAutoSellDraft && existingAutoSellDraft.percent) || 100),
          triggerFamily: normalizeAutoSellTriggerFamily(existingAutoSellDraft && existingAutoSellDraft.triggerFamily),
          triggerMode: normalizeAutoSellTriggerMode(existingAutoSellDraft && existingAutoSellDraft.triggerMode),
          delayMs: Number((existingAutoSellDraft && existingAutoSellDraft.delayMs) || 0),
          targetBlockOffset: Number((existingAutoSellDraft && existingAutoSellDraft.targetBlockOffset) || 0),
          marketCapEnabled: normalizeAutoSellTriggerFamily(existingAutoSellDraft && existingAutoSellDraft.triggerFamily) === "market-cap",
          marketCapThreshold: existingAutoSellDraft && existingAutoSellDraft.marketCapThreshold
            ? String(existingAutoSellDraft.marketCapThreshold)
            : "",
          marketCapScanTimeoutSeconds: Number(
            (existingAutoSellDraft && existingAutoSellDraft.marketCapScanTimeoutSeconds) || 30
          ) || 30,
          marketCapTimeoutAction: existingAutoSellDraft && existingAutoSellDraft.marketCapTimeoutAction
            ? String(existingAutoSellDraft.marketCapTimeoutAction)
            : "stop",
        };

      base.defaults = {
        ...(base.defaults || {}),
        launchpad: f.launchpad || "pump",
        mode: f.mode || "regular",
        activePresetId: f.activePresetId || defaultPresetId,
        presetEditing: false,
        misc: {
          ...currentMisc,
          sniperDraft,
          feeSplitDraft: shouldPersistFeeSplitDraft ? feeSplitDraft : currentMisc.feeSplitDraft,
          agentSplitDraft: shouldPersistAgentSplitDraft ? agentSplitDraft : currentMisc.agentSplitDraft,
          sniperDraftsByLaunchpad: {},
          feeSplitDraftsByLaunchpad: nextFeeSplitDraftsByLaunchpad,
          agentSplitDraftsByLaunchpad: nextAgentSplitDraftsByLaunchpad,
          autoSellDraftsByLaunchpad: {},
        },
        automaticDevSell: autoSellDraft,
      };

      return base;
    }

    return {
      buildSavedConfigFromForm,
      canPreuploadMetadata,
      hasFreshPreuploadedMetadata,
      launchpadHandlesOwnMetadata,
      metadataFingerprintFromForm,
      readForm,
    };
  }

  global.LaunchDeckFormDomain = {
    createFormDomain,
  };
})(window);
