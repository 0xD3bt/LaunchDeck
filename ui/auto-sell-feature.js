(function initAutoSellFeature(global) {
  function createAutoSellFeature(config) {
    const {
      elements,
      getNamedValue,
      setNamedValue,
      setNamedChecked,
      isNamedChecked,
      formatSliderValue,
      syncSettingsCapabilities,
      syncActivePresetFromInputs,
      validateFieldByName,
      documentNode,
      persistDraft,
    } = config;

    const {
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
    } = elements;

    let eventsBound = false;

    function normalizeTriggerMode(value) {
      const mode = String(value || "").trim().toLowerCase();
      if (mode === "submit-delay" || mode === "block-offset") {
        return mode;
      }
      if (mode === "confirmation") return "block-offset";
      return "block-offset";
    }

    function getTriggerMode() {
      return normalizeTriggerMode(getNamedValue("automaticDevSellTriggerMode"));
    }

    function normalizeTriggerFamily(value) {
      const family = String(value || "").trim().toLowerCase();
      return family === "market-cap" ? "market-cap" : "time";
    }

    function getTriggerFamily() {
      return normalizeTriggerFamily(getNamedValue("automaticDevSellTriggerFamily"));
    }

    function getDelayMs() {
      const numeric = Number(getNamedValue("automaticDevSellDelayMs") || "0");
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(1500, numeric));
    }

    function getBlockOffset() {
      const numeric = Number(getNamedValue("automaticDevSellBlockOffset") || "0");
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(23, Math.round(numeric)));
    }

    function isMarketCapEnabled() {
      return getTriggerFamily() === "market-cap";
    }

    function getMarketCapThreshold() {
      return String(getNamedValue("automaticDevSellMarketCapThreshold") || "").trim();
    }

    function parseMarketCapThreshold(value) {
      const normalized = String(value || "")
        .trim()
        .toLowerCase()
        .replace(/,/g, "");
      if (!normalized) return null;
      const match = normalized.match(/^(\d+(?:\.\d+)?)([kmbt])?$/i);
      if (!match) return null;
      const base = Number(match[1]);
      if (!Number.isFinite(base) || base <= 0) return null;
      const multipliers = {
        k: 1e3,
        m: 1e6,
        b: 1e9,
        t: 1e12,
      };
      const suffix = (match[2] || "").toLowerCase();
      const multiplier = multipliers[suffix] || 1;
      const expanded = Math.round(base * multiplier);
      if (!Number.isFinite(expanded) || expanded <= 0) return null;
      return expanded;
    }

    function getMarketCapTimeoutSeconds() {
      const explicitSeconds = getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds");
      if (String(explicitSeconds || "").trim()) {
        const numeric = Number(explicitSeconds);
        if (!Number.isFinite(numeric)) return 30;
        return Math.max(1, Math.min(86400, Math.round(numeric)));
      }
      const legacyMinutesRaw = String(getNamedValue("automaticDevSellMarketCapScanTimeoutMinutes") || "").trim();
      if (!legacyMinutesRaw) return 30;
      const legacyMinutes = Number(legacyMinutesRaw);
      if (!Number.isFinite(legacyMinutes)) return 30;
      return Math.max(1, Math.min(86400, Math.round(legacyMinutes * 60)));
    }

    function getMarketCapTimeoutAction() {
      const value = String(getNamedValue("automaticDevSellMarketCapTimeoutAction") || "").trim().toLowerCase();
      return value === "sell" ? "sell" : "stop";
    }

    function getTriggerLabel(mode = getTriggerMode()) {
      if (mode === "submit-delay") return "On Submit + Delay";
      return "On Confirmed Block";
    }

    function getTriggerDescription(mode = getTriggerMode()) {
      if (mode === "submit-delay") {
        return `Sell ${getDelayMs()}ms after submit is observed without waiting for confirmation.`;
      }
      if (mode === "block-offset") {
        return `Send the sell transaction on confirmed block + ${getBlockOffset()} from launch confirmation.`;
      }
      return `Send the sell transaction on confirmed block + ${getBlockOffset()} from launch confirmation.`;
    }

    function getBlockOffsetDescription(offset = getBlockOffset()) {
      return `Send the sell transaction on confirmed block + ${offset} from launch confirmation.`;
    }

    function getSummaryText(formValues) {
      const percent = `${formValues.automaticDevSellPercent || "0"}%`;
      const triggerFamily = normalizeTriggerFamily(
        formValues.automaticDevSellTriggerFamily
          || ((String(formValues.automaticDevSellMarketCapEnabled || "").trim() === "true"
            && String(formValues.automaticDevSellMarketCapThreshold || "").trim())
            ? "market-cap"
            : "time")
      );
      const mode = normalizeTriggerMode(formValues.automaticDevSellTriggerMode);
      const marketCapThreshold = String(formValues.automaticDevSellMarketCapThreshold || "").trim();
      const explicitTimeoutSeconds = String(formValues.automaticDevSellMarketCapScanTimeoutSeconds || "").trim();
      const legacyTimeoutMinutes = String(formValues.automaticDevSellMarketCapScanTimeoutMinutes || "").trim();
      const marketCapTimeout = explicitTimeoutSeconds
        ? Math.max(1, Math.min(86400, Number(explicitTimeoutSeconds || 30) || 30))
        : (legacyTimeoutMinutes
          ? Math.max(1, Math.min(86400, (Number(legacyTimeoutMinutes || 15) || 15) * 60))
          : 30);
      const marketCapTimeoutAction = String(formValues.automaticDevSellMarketCapTimeoutAction || "").trim().toLowerCase() === "sell"
        ? "sell"
        : "stop";
      if (triggerFamily === "market-cap") {
        const thresholdLabel = marketCapThreshold ? `$${marketCapThreshold}` : "USD market cap";
        return `${percent} on ${thresholdLabel} (${marketCapTimeout}s, ${marketCapTimeoutAction})`;
      }
      if (mode === "submit-delay") {
        return `${percent} at submit + ${Number(formValues.automaticDevSellDelayMs || 0)}ms`;
      }
      if (mode === "block-offset") {
        return `${percent} on confirmed + ${Number(formValues.automaticDevSellBlockOffset || 0)}`;
      }
      return `${percent} on confirmed + ${Number(formValues.automaticDevSellBlockOffset || 0)}`;
    }

    function togglePanel(forceOpen) {
      if (!devAutoSellPanel) return;
      const shouldOpen = typeof forceOpen === "boolean" ? forceOpen : devAutoSellPanel.hidden;
      devAutoSellPanel.hidden = !shouldOpen;
    }

    function syncUI() {
      const enabled = isNamedChecked("automaticDevSellEnabled");
      const rawPercent = Number(getNamedValue("automaticDevSellPercent") || "100");
      const percent = String(Math.max(1, Math.min(100, Number.isFinite(rawPercent) ? rawPercent : 100)));
      const triggerFamily = getTriggerFamily();
      const triggerMode = getTriggerMode();
      const delayMs = String(getDelayMs());
      const blockOffset = String(getBlockOffset());
      const marketCapEnabled = isMarketCapEnabled();
      const marketCapThreshold = getMarketCapThreshold();
      const marketCapTimeoutSeconds = String(getMarketCapTimeoutSeconds());
      const marketCapTimeoutAction = getMarketCapTimeoutAction();
      if (enabled && getNamedValue("automaticDevSellPercent") !== percent) {
        setNamedValue("automaticDevSellPercent", percent);
      }
      if (getNamedValue("automaticDevSellTriggerMode") !== triggerMode) {
        setNamedValue("automaticDevSellTriggerMode", triggerMode);
      }
      if (getNamedValue("automaticDevSellTriggerFamily") !== triggerFamily) {
        setNamedValue("automaticDevSellTriggerFamily", triggerFamily);
      }
      if (getNamedValue("automaticDevSellDelayMs") !== delayMs) {
        setNamedValue("automaticDevSellDelayMs", delayMs);
      }
      if (getNamedValue("automaticDevSellBlockOffset") !== blockOffset) {
        setNamedValue("automaticDevSellBlockOffset", blockOffset);
      }
      if (typeof setNamedChecked === "function") {
        setNamedChecked("automaticDevSellMarketCapEnabled", triggerFamily === "market-cap");
      }
      if (marketCapEnabled && getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds") !== marketCapTimeoutSeconds) {
        setNamedValue("automaticDevSellMarketCapScanTimeoutSeconds", marketCapTimeoutSeconds);
      }
      if (getNamedValue("automaticDevSellMarketCapTimeoutAction") !== marketCapTimeoutAction) {
        setNamedValue("automaticDevSellMarketCapTimeoutAction", marketCapTimeoutAction);
      }

      if (devAutoSellButton) devAutoSellButton.classList.toggle("active", enabled);
      if (autoSellToggleState) autoSellToggleState.textContent = enabled ? "ON" : "OFF";
      if (autoSellEnabledInput) autoSellEnabledInput.checked = enabled;
      if (autoSellSettings) autoSellSettings.hidden = !enabled;
      if (autoSellTriggerFamilyValue) autoSellTriggerFamilyValue.textContent = triggerFamily === "market-cap" ? "Mcap" : "Time";
      autoSellTriggerFamilyButtons.forEach((button) => {
        const buttonFamily = normalizeTriggerFamily(button.getAttribute("data-auto-sell-trigger-family"));
        button.classList.toggle("active", buttonFamily === triggerFamily);
        button.disabled = !enabled;
      });
      if (autoSellTimeSettings) autoSellTimeSettings.hidden = !enabled || triggerFamily !== "time";
      if (autoSellTriggerValue) {
        autoSellTriggerValue.textContent = triggerFamily === "market-cap" ? "Market Cap" : getTriggerLabel(triggerMode);
      }
      autoSellTriggerModeButtons.forEach((button) => {
        const buttonMode = button.getAttribute("data-auto-sell-trigger-mode") || "block-offset";
        button.classList.toggle("active", buttonMode === triggerMode);
        button.disabled = !enabled || triggerFamily !== "time";
        button.title = getTriggerDescription(buttonMode);
      });
      if (autoSellDelaySlider) {
        autoSellDelaySlider.value = delayMs;
        autoSellDelaySlider.disabled = !enabled || triggerFamily !== "time" || triggerMode !== "submit-delay";
      }
      if (autoSellDelayControl) autoSellDelayControl.hidden = !enabled || triggerFamily !== "time" || triggerMode !== "submit-delay";
      if (autoSellBlockControl) autoSellBlockControl.hidden = !enabled || triggerFamily !== "time" || triggerMode !== "block-offset";
      autoSellBlockOffsetButtons.forEach((button) => {
        const offsetValue = Number(button.getAttribute("data-auto-sell-block-offset") || "0");
        button.classList.toggle("active", String(offsetValue) === blockOffset);
        button.disabled = !enabled || triggerFamily !== "time" || triggerMode !== "block-offset";
        button.title = getBlockOffsetDescription(offsetValue);
      });
      if (autoSellPercentSlider) {
        autoSellPercentSlider.value = percent;
        autoSellPercentSlider.disabled = !enabled;
      }
      if (autoSellDelayValue) autoSellDelayValue.textContent = formatSliderValue(delayMs, "ms", 0);
      if (autoSellBlockValue) autoSellBlockValue.textContent = blockOffset;
      if (autoSellPercentValue) autoSellPercentValue.textContent = formatSliderValue(percent, "%", 0);
      if (autoSellMarketCapEnabledInput) autoSellMarketCapEnabledInput.checked = marketCapEnabled;
      if (autoSellMarketCapSettings) autoSellMarketCapSettings.hidden = !enabled || triggerFamily !== "market-cap";
      if (autoSellMarketCapThresholdInput) {
        autoSellMarketCapThresholdInput.value = marketCapThreshold;
        autoSellMarketCapThresholdInput.disabled = !enabled || triggerFamily !== "market-cap";
      }
      if (autoSellMarketCapThresholdValue) {
        autoSellMarketCapThresholdValue.textContent = triggerFamily === "market-cap" && marketCapThreshold
          ? marketCapThreshold
          : (triggerFamily === "market-cap" ? "Pending" : "Disabled");
      }
      if (autoSellMarketCapTimeoutInput) {
        autoSellMarketCapTimeoutInput.value = marketCapTimeoutSeconds;
        autoSellMarketCapTimeoutInput.disabled = !enabled || triggerFamily !== "market-cap";
      }
      if (autoSellMarketCapTimeoutActionInput) {
        autoSellMarketCapTimeoutActionInput.value = marketCapTimeoutAction;
        autoSellMarketCapTimeoutActionInput.disabled = !enabled || triggerFamily !== "market-cap";
      }
      syncSettingsCapabilities();
    }

    function bindEvents() {
      if (eventsBound) return;
      eventsBound = true;

      if (devAutoSellButton) {
        devAutoSellButton.addEventListener("click", (event) => {
          event.stopPropagation();
          togglePanel();
        });
      }
      if (autoSellEnabledInput) {
        autoSellEnabledInput.addEventListener("change", () => {
          if (autoSellEnabledInput.checked) {
            const currentPercent = Number(getNamedValue("automaticDevSellPercent") || "0");
            if (!Number.isFinite(currentPercent) || currentPercent <= 0) {
              setNamedValue("automaticDevSellPercent", "100");
            }
          }
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellPercent");
          validateFieldByName("automaticDevSellDelayMs");
          validateFieldByName("automaticDevSellBlockOffset");
          validateFieldByName("automaticDevSellMarketCapThreshold");
          validateFieldByName("automaticDevSellMarketCapScanTimeoutSeconds");
        });
      }
      autoSellTriggerFamilyButtons.forEach((button) => {
        button.addEventListener("click", () => {
          setNamedValue(
            "automaticDevSellTriggerFamily",
            normalizeTriggerFamily(button.getAttribute("data-auto-sell-trigger-family"))
          );
          setNamedChecked(
            "automaticDevSellMarketCapEnabled",
            normalizeTriggerFamily(button.getAttribute("data-auto-sell-trigger-family")) === "market-cap"
          );
          if (!getMarketCapTimeoutSeconds()) {
            setNamedValue("automaticDevSellMarketCapScanTimeoutSeconds", "30");
          }
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellDelayMs");
          validateFieldByName("automaticDevSellBlockOffset");
          validateFieldByName("automaticDevSellMarketCapThreshold");
          validateFieldByName("automaticDevSellMarketCapScanTimeoutSeconds");
        });
      });
      autoSellTriggerModeButtons.forEach((button) => {
        button.addEventListener("click", () => {
          setNamedValue("automaticDevSellTriggerMode", button.getAttribute("data-auto-sell-trigger-mode") || "block-offset");
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellDelayMs");
          validateFieldByName("automaticDevSellBlockOffset");
        });
      });
      if (autoSellDelaySlider) {
        autoSellDelaySlider.addEventListener("input", () => {
          setNamedValue("automaticDevSellDelayMs", autoSellDelaySlider.value);
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellDelayMs");
        });
      }
      autoSellBlockOffsetButtons.forEach((button) => {
        button.addEventListener("click", () => {
          setNamedValue("automaticDevSellBlockOffset", button.getAttribute("data-auto-sell-block-offset") || "0");
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellBlockOffset");
        });
      });
      if (autoSellMarketCapThresholdInput) {
        autoSellMarketCapThresholdInput.addEventListener("input", () => {
          setNamedValue("automaticDevSellMarketCapThreshold", autoSellMarketCapThresholdInput.value.trim());
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellMarketCapThreshold");
        });
      }
      if (autoSellMarketCapTimeoutInput) {
        autoSellMarketCapTimeoutInput.addEventListener("input", () => {
          setNamedValue("automaticDevSellMarketCapScanTimeoutSeconds", autoSellMarketCapTimeoutInput.value || "30");
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellMarketCapScanTimeoutSeconds");
        });
      }
      if (autoSellMarketCapTimeoutActionInput) {
        autoSellMarketCapTimeoutActionInput.addEventListener("change", () => {
          setNamedValue("automaticDevSellMarketCapTimeoutAction", autoSellMarketCapTimeoutActionInput.value || "stop");
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
        });
      }
      if (autoSellPercentSlider) {
        autoSellPercentSlider.addEventListener("input", () => {
          setNamedValue("automaticDevSellPercent", autoSellPercentSlider.value);
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellPercent");
        });
      }
      documentNode.addEventListener("click", (event) => {
        if (!devAutoSellPanel || devAutoSellPanel.hidden) return;
        if (devAutoSellPanel.contains(event.target) || (devAutoSellButton && devAutoSellButton.contains(event.target))) return;
        togglePanel(false);
      });
    }

    return {
      bindEvents,
      normalizeTriggerMode,
      getTriggerMode,
      normalizeTriggerFamily,
      getTriggerFamily,
      getDelayMs,
      getBlockOffset,
      isMarketCapEnabled,
      getMarketCapThreshold,
      parseMarketCapThreshold,
      getMarketCapDirection,
      getMarketCapTimeoutSeconds,
      getMarketCapTimeoutAction,
      getTriggerLabel,
      getTriggerDescription,
      getSummaryText,
      syncUI,
      togglePanel,
    };
  }

  global.AutoSellFeature = {
    create: createAutoSellFeature,
  };
})(window);
