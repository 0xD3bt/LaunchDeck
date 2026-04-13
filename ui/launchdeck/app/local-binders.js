(function initLaunchDeckLocalBinders(global) {
  function createLocalBinders(config) {
    const {
      elements = {},
      constants = {},
      fieldValidatorNames = [],
      state = {},
      actions = {},
    } = config || {};

    const {
      form,
      launchpadInputs = [],
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
      buttons = [],
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
      settingsInputs = [],
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
    } = elements;

    const {
      defaultPresetId = "preset1",
      maxFeeSplitRecipients = 10,
    } = constants;

    const {
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
      scheduleLiveSyncBroadcast,
    } = actions;

    let eventsBound = false;

    function syncRouteMevModeForProvider(select, provider) {
      if (!select) return;
      const normalizedProvider = String(provider || "").trim().toLowerCase();
      const previousProvider = String(select.dataset.lastProvider || "").trim().toLowerCase();
      const currentMode = normalizeMevMode(
        select.value,
        defaultMevModeForProvider(previousProvider),
      );
      if (isHelloMoonProvider(previousProvider)) {
        select.dataset.lastHellomoonMode = currentMode;
      }
      if (isHelloMoonProvider(normalizedProvider)) {
        const nextMode = normalizeMevMode(
          select.dataset.lastHellomoonMode || currentMode,
          defaultMevModeForProvider(normalizedProvider),
        );
        setMevModeSelectValue(
          select,
          nextMode,
          defaultMevModeForProvider(normalizedProvider),
          normalizedProvider,
        );
        return;
      }
      setMevModeSelectValue(select, "off", "off", normalizedProvider);
    }

    function getTickerManuallyEdited() {
      return typeof state.getTickerManuallyEdited === "function" ? state.getTickerManuallyEdited() : false;
    }

    function setTickerManuallyEdited(value) {
      if (typeof state.setTickerManuallyEdited === "function") state.setTickerManuallyEdited(Boolean(value));
    }

    function getSyncingTickerFromName() {
      return typeof state.getSyncingTickerFromName === "function" ? state.getSyncingTickerFromName() : false;
    }

    function setSyncingTickerFromName(value) {
      if (typeof state.setSyncingTickerFromName === "function") state.setSyncingTickerFromName(Boolean(value));
    }

    function setTickerClearedForManualEntry(value) {
      if (typeof state.setTickerClearedForManualEntry === "function") {
        state.setTickerClearedForManualEntry(Boolean(value));
      }
    }

    function isSyncingDevBuyInputs() {
      return typeof state.isSyncingDevBuyInputs === "function" ? state.isSyncingDevBuyInputs() : false;
    }

    function isDevBuyPresetEditorOpen() {
      return typeof state.getDevBuyPresetEditorOpen === "function" ? state.getDevBuyPresetEditorOpen() : false;
    }

    function getLastDevBuyEditSource() {
      return typeof state.getLastDevBuyEditSource === "function" ? state.getLastDevBuyEditSource() : "sol";
    }

    function getActiveFeeSplitDraftLaunchpad() {
      return typeof state.getActiveFeeSplitDraftLaunchpad === "function"
        ? state.getActiveFeeSplitDraftLaunchpad()
        : "pump";
    }

    function getFeeSplitClearAllRestoreSnapshot() {
      return typeof state.getFeeSplitClearAllRestoreSnapshot === "function"
        ? state.getFeeSplitClearAllRestoreSnapshot()
        : null;
    }

    function setFeeSplitClearAllRestoreSnapshot(value) {
      if (typeof state.setFeeSplitClearAllRestoreSnapshot === "function") {
        state.setFeeSplitClearAllRestoreSnapshot(value);
      }
    }

    function getAgentSplitClearAllRestoreSnapshot() {
      return typeof state.getAgentSplitClearAllRestoreSnapshot === "function"
        ? state.getAgentSplitClearAllRestoreSnapshot()
        : null;
    }

    function setAgentSplitClearAllRestoreSnapshot(value) {
      if (typeof state.setAgentSplitClearAllRestoreSnapshot === "function") {
        state.setAgentSplitClearAllRestoreSnapshot(value);
      }
    }

    function bindEvents() {
      if (eventsBound) return;
      eventsBound = true;

      if (form) {
        form.querySelectorAll('input[name="mode"]').forEach((node) => {
          node.addEventListener("change", () => {
            setStoredLaunchMode(getMode());
            updateModeVisibility();
            queueQuoteUpdate();
            scheduleLiveSyncBroadcast({ immediate: true });
          });
        });
      }

      launchpadInputs.forEach((input) => {
        input.addEventListener("change", () => {
          if (!input.checked) return;
          const previousLaunchpad = getActiveFeeSplitDraftLaunchpad();
          const nextLaunchpad = normalizeLaunchpad(input.value);
          setStoredFeeSplitDraft(serializeFeeSplitDraft(previousLaunchpad), { launchpad: previousLaunchpad });
          setStoredLaunchpad(input.value);
          withSuspendedFeeSplitDraftPersistence(() => {
            setLaunchpad(nextLaunchpad, {
              resetMode: true,
              persistMode: true,
              restoreScopedActions: true,
            });
          });
          restoreFeeSplitDraftForLaunchpad(nextLaunchpad);
          global.setTimeout(() => {
            if (typeof hydrateModeActionState === "function") {
              hydrateModeActionState({ preferExistingFormFallback: false, launchpad: nextLaunchpad });
            }
            if (typeof hydrateDevAutoSellState === "function") {
              hydrateDevAutoSellState({ preferExistingFormFallback: false, launchpad: nextLaunchpad });
            }
          }, 0);
          applyLaunchpadTokenMetadata();
          queueQuoteUpdate();
          scheduleLiveSyncBroadcast({ immediate: true });
        });
      });

      if (bonkQuoteAssetToggle) {
        bonkQuoteAssetToggle.addEventListener("click", () => {
          const asset = getQuoteAsset() === "usd1" ? "sol" : "usd1";
          if (bonkQuoteAssetInput) bonkQuoteAssetInput.value = asset;
          setStoredBonkQuoteAsset(asset);
          syncBonkQuoteAssetUI();
          queueQuoteUpdate();
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
          if (!getTickerManuallyEdited() && autoTickerValue && symbolInput.value === autoTickerValue) {
            setSyncingTickerFromName(true);
            symbolInput.value = "";
            setSyncingTickerFromName(false);
            setTickerManuallyEdited(true);
            setTickerClearedForManualEntry(true);
            updateTokenFieldCounts();
          }
        });

        symbolInput.addEventListener("input", () => {
          if (getSyncingTickerFromName()) return;
          const formatted = formatTickerValue(symbolInput.value);
          setSyncingTickerFromName(true);
          if (symbolInput.value !== formatted) {
            symbolInput.value = formatted;
          }
          setSyncingTickerFromName(false);
          setTickerManuallyEdited(true);
          setTickerClearedForManualEntry(symbolInput.value.trim().length === 0);
          updateTokenFieldCounts();
          markMetadataUploadDirty();
          scheduleMetadataPreupload({ immediate: true });
        });

        symbolInput.addEventListener("blur", () => {
          if (!symbolInput.value.trim()) {
            setTickerManuallyEdited(false);
            setTickerClearedForManualEntry(false);
            syncTickerFromName();
            return;
          }
          setTickerClearedForManualEntry(false);
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
          setTickerCapsEnabled(!isTickerCapsEnabled(), { persist: true });
          applyTickerCapsMode();
          if (!getTickerManuallyEdited()) {
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
          if (isSyncingDevBuyInputs()) return;
          await updateDevBuyFromSolInput(devBuySolInput.value);
        });
      }

      if (devBuyPercentInput) {
        devBuyPercentInput.addEventListener("input", async () => {
          if (isSyncingDevBuyInputs()) return;
          await updateDevBuyFromPercentInput(devBuyPercentInput.value);
        });
      }

      if (providerSelect) {
        providerSelect.addEventListener("change", () => {
          syncRouteMevModeForProvider(creationMevModeSelect, getProvider());
          syncActivePresetFromInputs();
          updateJitoVisibility();
          validateProviderFeeFields("creation");
        });
      }

      if (buyProviderSelect) {
        buyProviderSelect.addEventListener("change", () => {
          syncRouteMevModeForProvider(buyMevModeSelect, getBuyProvider());
          ensureStandardRpcSlippageDefault(buySlippageInput, getBuyProvider());
          syncActivePresetFromInputs();
          validateProviderFeeFields("buy");
        });
      }

      if (sellProviderSelect) {
        sellProviderSelect.addEventListener("change", () => {
          syncRouteMevModeForProvider(sellMevModeSelect, getSellProvider());
          ensureStandardRpcSlippageDefault(sellSlippageInput, getSellProvider());
          syncActivePresetFromInputs();
          validateProviderFeeFields("sell");
        });
      }

      if (feeSplitPill) {
        feeSplitPill.addEventListener("click", () => {
          const mode = getMode();
          if (mode !== "regular" && mode !== "agent-custom" && !mode.startsWith("bags-")) return;
          if (!mode.startsWith("bags-") && feeSplitEnabled && !feeSplitEnabled.checked) {
            feeSplitEnabled.checked = true;
            showFeeSplitModal();
            return;
          }
          showFeeSplitModal();
        });
      }

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
          walletRefreshButton.classList.remove("is-refreshing");
          void walletRefreshButton.offsetWidth;
          walletRefreshButton.classList.add("is-refreshing");
          const refreshAnimationStart = performance.now();
          try {
            await refreshWalletStatus(true, true);
            const remainingAnimationMs = Math.max(0, 600 - (performance.now() - refreshAnimationStart));
            if (remainingAnimationMs > 0) {
              await new Promise((resolve) => global.setTimeout(resolve, remainingAnimationMs));
            }
          } finally {
            walletRefreshButton.classList.remove("is-refreshing");
            walletRefreshButton.disabled = false;
          }
        });
      }

      if (walletDropdownList) {
        walletDropdownList.addEventListener("click", async (event) => {
          const copyButton = event.target.closest(".wallet-option-copy");
          if (copyButton) {
            event.preventDefault();
            event.stopPropagation();
            await copyWalletDropdownAddress(copyButton);
            return;
          }
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

        walletDropdownList.addEventListener("keydown", (event) => {
          const button = event.target.closest(".wallet-option-button");
          if (!button || !walletSelect) return;
          if (event.key !== "Enter" && event.key !== " ") return;
          event.preventDefault();
          const nextKey = String(button.dataset.walletKey || "").trim();
          if (!nextKey) return;
          walletSelect.value = nextKey;
          setStoredSelectedWalletKey(nextKey);
          applySelectedWalletLocally(nextKey);
          setWalletDropdownOpen(false);
          refreshWalletStatus(true);
        });
      }

      if (walletSelect) {
        walletSelect.addEventListener("change", () => {
          const nextKey = selectedWalletKey();
          setStoredSelectedWalletKey(nextKey);
          applySelectedWalletLocally(nextKey);
          refreshWalletStatus(true);
        });
      }

      if (feeSplitAdd) {
        feeSplitAdd.addEventListener("click", () => {
          clearFeeSplitRestoreState();
          if (getFeeSplitRows().length >= maxFeeSplitRecipients) return;
          const row = createFeeSplitRow({ type: "wallet", sharePercent: "" });
          feeSplitList.appendChild(row);
          updateFeeSplitRowValidationUi(row);
          syncFeeSplitTotals();
          syncFeeSplitModalPresentation();
          setStoredFeeSplitDraft(serializeFeeSplitDraft());
          setFeeSplitModalError("");
        });
      }

      if (feeSplitReset) {
        feeSplitReset.addEventListener("click", () => {
          clearFeeSplitRestoreState();
          getFeeSplitRows().forEach((row) => {
            row.querySelector(".recipient-share").value = "0";
            row.querySelector(".recipient-slider").value = "0";
          });
          syncFeeSplitTotals();
          syncFeeSplitModalPresentation();
          setStoredFeeSplitDraft(serializeFeeSplitDraft());
          setFeeSplitModalError("");
        });
      }

      if (feeSplitEven) {
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
          syncFeeSplitModalPresentation();
          setStoredFeeSplitDraft(serializeFeeSplitDraft());
          setFeeSplitModalError("");
        });
      }

      if (feeSplitClearAll) {
        feeSplitClearAll.addEventListener("click", () => {
          if (getFeeSplitClearAllRestoreSnapshot()) {
            applyFeeSplitDraft(getFeeSplitClearAllRestoreSnapshot(), { persist: false });
            syncFeeSplitTotals();
            syncFeeSplitModalPresentation();
            setStoredFeeSplitDraft(serializeFeeSplitDraft());
            clearFeeSplitRestoreState();
            setFeeSplitModalError("");
            return;
          }
          setFeeSplitClearAllRestoreSnapshot(normalizeFeeSplitDraft(serializeFeeSplitDraft()));
          applyFeeSplitDraft(feeSplitClearAllDraft(), { persist: false });
          syncFeeSplitTotals();
          syncFeeSplitModalPresentation();
          setStoredFeeSplitDraft(serializeFeeSplitDraft());
          updateFeeSplitClearAllButton();
          setFeeSplitModalError("");
        });
      }

      if (feeSplitList) {
        feeSplitList.addEventListener("click", (event) => {
          const copyButton = event.target.closest(".bags-fee-row-copy");
          if (copyButton) {
            void copyBagsResolvedWallet(copyButton);
            return;
          }
          const lockToggle = event.target.closest(".recipient-lock-toggle");
          if (lockToggle) {
            clearFeeSplitRestoreState();
            const row = lockToggle.closest(".fee-split-row");
            setRecipientTargetLocked(row, row.dataset.targetLocked !== "true");
            scheduleFeeSplitLookup(row, { immediate: true });
            syncFeeSplitTotals();
            setStoredFeeSplitDraft(serializeFeeSplitDraft());
            setFeeSplitModalError("");
            return;
          }
          const tab = event.target.closest(".recipient-type-tab");
          if (tab) {
            clearFeeSplitRestoreState();
            const row = tab.closest(".fee-split-row");
            updateFeeSplitRowType(row, tab.dataset.type);
            scheduleFeeSplitLookup(row, { immediate: true });
            setStoredFeeSplitDraft(serializeFeeSplitDraft());
            setFeeSplitModalError("");
            return;
          }
          const removeButton = event.target.closest(".recipient-remove");
          if (removeButton) {
            clearFeeSplitRestoreState();
            const row = removeButton.closest(".fee-split-row");
            clearFeeSplitRowState(row);
            row.remove();
            if (!usesImplicitCreatorShareMode()) ensureFeeSplitDefaultRow();
            syncFeeSplitTotals();
            syncFeeSplitModalPresentation();
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
          if (event.target.classList.contains("recipient-target")) {
            clearFeeSplitRowState(row);
            scheduleFeeSplitLookup(row);
          }
          if (event.target.classList.contains("recipient-slider")) {
            row.querySelector(".recipient-share").value = event.target.value;
          }
          if (event.target.classList.contains("recipient-share")) {
            row.querySelector(".recipient-slider").value = event.target.value || "0";
          }
          updateFeeSplitRowValidationUi(row);
          syncFeeSplitTotals();
          syncFeeSplitModalPresentation();
          setStoredFeeSplitDraft(serializeFeeSplitDraft());
          setFeeSplitModalError("");
        });
      }

      if (agentSplitAdd) {
        agentSplitAdd.addEventListener("click", () => {
          clearAgentSplitRestoreState();
          if (getAgentSplitRows().length >= maxFeeSplitRecipients) {
            setAgentSplitModalError(`Agent custom fee split supports at most ${maxFeeSplitRecipients} recipients.`);
            return;
          }
          agentSplitList.appendChild(createAgentSplitRow({ type: "wallet", sharePercent: "" }));
          normalizeAgentSplitStructure({ afterAdd: true });
          syncAgentSplitTotals();
          setStoredAgentSplitDraft(serializeAgentSplitDraft());
          setAgentSplitModalError("");
        });
      }

      if (agentSplitReset) {
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
      }

      if (agentSplitEven) {
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
      }

      if (agentSplitClearAll) {
        agentSplitClearAll.addEventListener("click", () => {
          if (getAgentSplitClearAllRestoreSnapshot()) {
            applyAgentSplitDraft(getAgentSplitClearAllRestoreSnapshot(), { persist: false });
            syncAgentSplitTotals();
            setStoredAgentSplitDraft(serializeAgentSplitDraft());
            clearAgentSplitRestoreState();
            setAgentSplitModalError("");
            return;
          }
          setAgentSplitClearAllRestoreSnapshot(normalizeAgentSplitDraft(serializeAgentSplitDraft()));
          applyAgentSplitDraft(agentSplitClearAllDraft(), { persist: false });
          syncAgentSplitTotals();
          setStoredAgentSplitDraft(serializeAgentSplitDraft());
          updateAgentSplitClearAllButton();
          setAgentSplitModalError("");
        });
      }

      if (agentSplitList) {
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
            const row = tab.closest(".fee-split-row");
            updateFeeSplitRowType(row, tab.dataset.type);
            if (row && row.dataset.type !== "github") delete row.dataset.githubUserId;
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
            if (row.dataset.type === "github") delete row.dataset.githubUserId;
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
      }

      fieldValidatorNames.forEach((name) => {
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

      if (imageInput) {
        imageInput.addEventListener("change", async () => {
          const [file] = imageInput.files || [];
          if (!file) return;
          if (imageStatus) imageStatus.textContent = "Uploading image to library...";
          try {
            await uploadSelectedImage(file);
          } catch (error) {
            if (imageStatus) imageStatus.textContent = error.message;
          } finally {
            imageInput.value = "";
          }
        });
      }

      if (testFillButton) {
        testFillButton.addEventListener("click", async () => {
          await applyTestPreset();
        });
      }

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

      if (saveSettingsButton) {
        saveSettingsButton.addEventListener("click", async () => {
          await saveSettings();
        });
      }

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

      if (modalClose) modalClose.addEventListener("click", hideDeployModal);
      if (modalCancel) modalCancel.addEventListener("click", hideDeployModal);
      if (modalConfirm) {
        modalConfirm.addEventListener("click", () => {
          hideDeployModal();
          run("deploy");
        });
      }

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
          setActivePreset(chip.getAttribute("data-preset-id") || defaultPresetId);
        });
      }

      if (settingsPresetChipBar) {
        settingsPresetChipBar.addEventListener("click", (event) => {
          const chip = event.target.closest("[data-preset-id]");
          if (!chip) return;
          setActivePreset(chip.getAttribute("data-preset-id") || defaultPresetId);
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
          input.dispatchEvent(new global.Event("change", { bubbles: true }));
        });
      });

      settingsInputs.forEach((input) => {
        if (!input) return;
        const eventName = input.tagName === "SELECT" || input.type === "checkbox" ? "change" : "input";
        input.addEventListener(eventName, () => {
          if (input === creationMevModeSelect) {
            creationMevModeSelect.dataset.lastProvider = String(getProvider() || "").trim().toLowerCase();
            if (isHelloMoonProvider(getProvider())) {
              creationMevModeSelect.dataset.lastHellomoonMode = normalizeMevMode(
                creationMevModeSelect.value,
                defaultMevModeForProvider(getProvider()),
              );
            }
          }
          if (input === buyMevModeSelect) {
            buyMevModeSelect.dataset.lastProvider = String(getBuyProvider() || "").trim().toLowerCase();
            if (isHelloMoonProvider(getBuyProvider())) {
              buyMevModeSelect.dataset.lastHellomoonMode = normalizeMevMode(
                buyMevModeSelect.value,
                defaultMevModeForProvider(getBuyProvider()),
              );
            }
          }
          if (input === sellMevModeSelect) {
            sellMevModeSelect.dataset.lastProvider = String(getSellProvider() || "").trim().toLowerCase();
            if (isHelloMoonProvider(getSellProvider())) {
              sellMevModeSelect.dataset.lastHellomoonMode = normalizeMevMode(
                sellMevModeSelect.value,
                defaultMevModeForProvider(getSellProvider()),
              );
            }
          }
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
          if (isDevBuyPresetEditorOpen()) return;
          const button = event.target.closest("[data-quick-buy-amount]");
          if (!button) return;
          const presetId = button.getAttribute("data-quick-buy-preset-id") || defaultPresetId;
          const amount = button.getAttribute("data-quick-buy-amount") || "";
          if (!amount) return;
          setActivePreset(presetId);
          await triggerDeployWithDevBuy("sol", amount, "sol");
        });
      }

      if (devBuyCustomDeployButton) {
        devBuyCustomDeployButton.addEventListener("click", async () => {
          const mode = getDevBuyMode();
          const amountInput = getNamedInput("devBuyAmount");
          const amount = amountInput ? String(amountInput.value || "").trim() : "";
          if (!amount) {
            clearDevBuyState();
            const errors = validateForm();
            if (showValidationErrors(errors)) return;
            clearValidationErrors();
            showDeployModal();
            return;
          }
          await triggerDeployWithDevBuy(mode, amount, getLastDevBuyEditSource());
        });
      }

      if (modeVanityButton) {
        modeVanityButton.addEventListener("click", () => {
          showVanityModal();
        });
      }

      if (feeSplitClose) feeSplitClose.addEventListener("click", attemptCloseFeeSplitModal);
      if (feeSplitSave) feeSplitSave.addEventListener("click", attemptCloseFeeSplitModal);
      if (feeSplitDisable) feeSplitDisable.addEventListener("click", cancelFeeSplitModal);
      if (feeSplitModal) {
        feeSplitModal.addEventListener("click", (event) => {
          if (event.target === feeSplitModal) attemptCloseFeeSplitModal();
        });
      }

      if (agentSplitClose) agentSplitClose.addEventListener("click", attemptCloseAgentSplitModal);
      if (agentSplitCancel) {
        agentSplitCancel.addEventListener("click", () => {
          cancelAgentSplitModal();
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

      if (deployModal) {
        deployModal.addEventListener("click", (event) => {
          if (event.target === deployModal) hideDeployModal();
        });
      }
    }

    return {
      bindEvents,
    };
  }

  global.LaunchDeckLocalBinders = {
    create: createLocalBinders,
  };
})(window);
