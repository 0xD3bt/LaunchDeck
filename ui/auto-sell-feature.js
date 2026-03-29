(function initAutoSellFeature(global) {
  function createAutoSellFeature(config) {
    const {
      elements,
      getNamedValue,
      setNamedValue,
      isNamedChecked,
      formatSliderValue,
      syncSettingsCapabilities,
      syncActivePresetFromInputs,
      validateFieldByName,
      documentNode,
    } = config;

    const {
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
    } = elements;

    let eventsBound = false;

    function normalizeTriggerMode(value) {
      const mode = String(value || "").trim().toLowerCase();
      if (mode === "submit-delay" || mode === "block-offset" || mode === "confirmation") {
        return mode;
      }
      return "confirmation";
    }

    function getTriggerMode() {
      return normalizeTriggerMode(getNamedValue("automaticDevSellTriggerMode"));
    }

    function getDelayMs() {
      const numeric = Number(getNamedValue("automaticDevSellDelayMs") || "0");
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(1500, numeric));
    }

    function getBlockOffset() {
      const numeric = Number(getNamedValue("automaticDevSellBlockOffset") || "0");
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(5, Math.round(numeric)));
    }

    function getTriggerLabel(mode = getTriggerMode()) {
      if (mode === "submit-delay") return "On Submit + Delay";
      if (mode === "block-offset") return "Block Offset";
      return "Safe Confirmed";
    }

    function getTriggerDescription(mode = getTriggerMode()) {
      if (mode === "submit-delay") {
        return `Sell ${getDelayMs()}ms after submit is observed without waiting for confirmation.`;
      }
      if (mode === "block-offset") {
        return `Send the sell transaction when observed block ${getBlockOffset()} is reached, without waiting for confirmation.`;
      }
      return "Wait for the launch to confirm first, then sell immediately. Safest option.";
    }

    function getSummaryText(formValues) {
      const percent = `${formValues.automaticDevSellPercent || "0"}%`;
      const mode = normalizeTriggerMode(formValues.automaticDevSellTriggerMode);
      if (mode === "submit-delay") {
        return `${percent} at submit + ${Number(formValues.automaticDevSellDelayMs || 0)}ms`;
      }
      if (mode === "block-offset") {
        return `${percent} at block ${Number(formValues.automaticDevSellBlockOffset || 0)}`;
      }
      return `${percent} after confirmation`;
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
      const triggerMode = getTriggerMode();
      const delayMs = String(getDelayMs());
      const blockOffset = String(getBlockOffset());
      if (enabled && getNamedValue("automaticDevSellPercent") !== percent) {
        setNamedValue("automaticDevSellPercent", percent);
      }
      if (getNamedValue("automaticDevSellTriggerMode") !== triggerMode) {
        setNamedValue("automaticDevSellTriggerMode", triggerMode);
      }
      if (getNamedValue("automaticDevSellDelayMs") !== delayMs) {
        setNamedValue("automaticDevSellDelayMs", delayMs);
      }
      if (getNamedValue("automaticDevSellBlockOffset") !== blockOffset) {
        setNamedValue("automaticDevSellBlockOffset", blockOffset);
      }

      if (devAutoSellButton) devAutoSellButton.classList.toggle("active", enabled);
      if (autoSellToggleState) autoSellToggleState.textContent = enabled ? "ON" : "OFF";
      if (autoSellEnabledInput) autoSellEnabledInput.checked = enabled;
      if (autoSellSettings) autoSellSettings.hidden = !enabled;
      if (autoSellTriggerValue) autoSellTriggerValue.textContent = getTriggerLabel(triggerMode);
      if (autoSellTriggerDescription) autoSellTriggerDescription.textContent = getTriggerDescription(triggerMode);
      autoSellTriggerModeButtons.forEach((button) => {
        button.classList.toggle("active", button.getAttribute("data-auto-sell-trigger-mode") === triggerMode);
        button.disabled = !enabled;
      });
      if (autoSellDelaySlider) {
        autoSellDelaySlider.value = delayMs;
        autoSellDelaySlider.disabled = !enabled || triggerMode !== "submit-delay";
      }
      if (autoSellDelayControl) autoSellDelayControl.hidden = !enabled || triggerMode !== "submit-delay";
      if (autoSellBlockControl) autoSellBlockControl.hidden = !enabled || triggerMode !== "block-offset";
      autoSellBlockOffsetButtons.forEach((button) => {
        button.classList.toggle("active", button.getAttribute("data-auto-sell-block-offset") === blockOffset);
        button.disabled = !enabled || triggerMode !== "block-offset";
      });
      if (autoSellPercentSlider) {
        autoSellPercentSlider.value = percent;
        autoSellPercentSlider.disabled = !enabled;
      }
      if (autoSellDelayValue) autoSellDelayValue.textContent = formatSliderValue(delayMs, "ms", 0);
      if (autoSellBlockValue) autoSellBlockValue.textContent = `Block ${blockOffset}`;
      if (autoSellPercentValue) autoSellPercentValue.textContent = formatSliderValue(percent, "%", 0);
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
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellPercent");
          validateFieldByName("automaticDevSellDelayMs");
          validateFieldByName("automaticDevSellBlockOffset");
        });
      }
      autoSellTriggerModeButtons.forEach((button) => {
        button.addEventListener("click", () => {
          setNamedValue("automaticDevSellTriggerMode", button.getAttribute("data-auto-sell-trigger-mode") || "confirmation");
          syncUI();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellDelayMs");
          validateFieldByName("automaticDevSellBlockOffset");
        });
      });
      if (autoSellDelaySlider) {
        autoSellDelaySlider.addEventListener("input", () => {
          setNamedValue("automaticDevSellDelayMs", autoSellDelaySlider.value);
          syncUI();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellDelayMs");
        });
      }
      autoSellBlockOffsetButtons.forEach((button) => {
        button.addEventListener("click", () => {
          setNamedValue("automaticDevSellBlockOffset", button.getAttribute("data-auto-sell-block-offset") || "0");
          syncUI();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellBlockOffset");
        });
      });
      if (autoSellPercentSlider) {
        autoSellPercentSlider.addEventListener("input", () => {
          setNamedValue("automaticDevSellPercent", autoSellPercentSlider.value);
          syncUI();
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
      getDelayMs,
      getBlockOffset,
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
