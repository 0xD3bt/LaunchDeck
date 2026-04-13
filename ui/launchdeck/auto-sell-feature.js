(function initAutoSellFeature(global) {
  function createAutoSellFeature(config) {
    const SNIPER_AUTOSELL_MAX_BLOCK_OFFSET = 23;
    const SNIPER_AUTOSELL_BLOCK_OFFSETS = Array.from(
      { length: SNIPER_AUTOSELL_MAX_BLOCK_OFFSET + 1 },
      (_, index) => index,
    );
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
      getSniperAutosellRows,
      updateSniperAutosellWallet,
    } = config;

    const {
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
    } = elements;

    const autoSellCloseButtons = devAutoSellPanel
      ? Array.from(devAutoSellPanel.querySelectorAll("[data-auto-sell-close]"))
      : [];
    let eventsBound = false;
    let autoSellWarningPill = null;
    let autoSellOverlayPointerDown = false;

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

    function getMarketCapDirection() {
      return "gte";
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

    function getMarketCapTimeoutInputValue() {
      const explicitSeconds = String(getNamedValue("automaticDevSellMarketCapScanTimeoutSeconds") || "").trim();
      if (explicitSeconds) return explicitSeconds;
      const legacyMinutesRaw = String(getNamedValue("automaticDevSellMarketCapScanTimeoutMinutes") || "").trim();
      if (!legacyMinutesRaw) return "";
      const legacyMinutes = Number(legacyMinutesRaw);
      if (!Number.isFinite(legacyMinutes)) return "";
      return String(Math.max(1, Math.min(86400, Math.round(legacyMinutes * 60))));
    }

    function getMarketCapTimeoutAction() {
      const value = String(getNamedValue("automaticDevSellMarketCapTimeoutAction") || "").trim().toLowerCase();
      return value === "sell" ? "sell" : "stop";
    }

    function getTriggerLabel(mode = getTriggerMode()) {
      if (mode === "submit-delay") return "Submit+Delay";
      return "Slot Offset";
    }

    function getTriggerDescription(mode = getTriggerMode()) {
      if (mode === "submit-delay") {
        return `Sell ${getDelayMs()}ms after submit is observed without waiting for confirmation.`;
      }
      if (mode === "block-offset") {
        return `Send the sell transaction on confirmed slot + ${getBlockOffset()} from launch confirmation.`;
      }
      return `Send the sell transaction on confirmed slot + ${getBlockOffset()} from launch confirmation.`;
    }

    function getBlockOffsetDescription(offset = getBlockOffset()) {
      return `Send the sell transaction on confirmed slot + ${offset} from launch confirmation.`;
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
        return `${percent} on >= ${thresholdLabel} (${marketCapTimeout}s, ${marketCapTimeoutAction})`;
      }
      if (mode === "submit-delay") {
        return `${percent} at submit + ${Number(formValues.automaticDevSellDelayMs || 0)}ms`;
      }
      if (mode === "block-offset") {
        return `${percent} on confirmed + ${Number(formValues.automaticDevSellBlockOffset || 0)}`;
      }
      return `${percent} on confirmed + ${Number(formValues.automaticDevSellBlockOffset || 0)}`;
    }

    function shouldShowClosedPanelWarning() {
      const missingFields = getClosedPanelWarningMissingFields();
      return Boolean(
        devAutoSellPanel
        && devAutoSellPanel.hidden
        && isNamedChecked("automaticDevSellEnabled")
        && getTriggerFamily() === "market-cap"
        && missingFields.length
      );
    }

    function getClosedPanelWarningMissingFields() {
      if (!isNamedChecked("automaticDevSellEnabled") || getTriggerFamily() !== "market-cap") return [];
      const missing = [];
      if (!getMarketCapThreshold()) missing.push("USD market cap");
      if (!getMarketCapTimeoutInputValue()) missing.push("timeout");
      return missing;
    }

    function getClosedPanelWarningText() {
      const missing = getClosedPanelWarningMissingFields();
      if (!missing.length) return "";
      if (missing.length === 1) {
        return `Market cap auto-sell is selected, but ${missing[0]} is not set yet.`;
      }
      return `Market cap auto-sell is selected, but ${missing[0]} and ${missing[1]} are not set yet.`;
    }

    function ensureWarningPill() {
      if (autoSellWarningPill || !devAutoSellButton) return autoSellWarningPill;
      const actionRow = devAutoSellButton.closest(".mode-action-row");
      if (!actionRow) return null;
      autoSellWarningPill = document.createElement("div");
      autoSellWarningPill.id = "dev-auto-sell-warning";
      autoSellWarningPill.className = "modal-warning settings-provider-warning auto-sell-inline-warning";
      autoSellWarningPill.hidden = true;
      actionRow.insertAdjacentElement("afterend", autoSellWarningPill);
      return autoSellWarningPill;
    }

    function renderClosedPanelWarning() {
      const warningPill = ensureWarningPill();
      if (!warningPill) return;
      if (!shouldShowClosedPanelWarning()) {
        warningPill.hidden = true;
        warningPill.textContent = "";
        if (devAutoSellButton) {
          devAutoSellButton.classList.remove("has-warning");
          devAutoSellButton.removeAttribute("title");
        }
        return;
      }
      const warningText = getClosedPanelWarningText();
      warningPill.hidden = false;
      warningPill.textContent = warningText;
      if (devAutoSellButton) {
        devAutoSellButton.classList.add("has-warning");
        devAutoSellButton.title = warningText;
      }
    }

    function togglePanel(forceOpen) {
      if (!devAutoSellPanel) return;
      const shouldOpen = typeof forceOpen === "boolean" ? forceOpen : devAutoSellPanel.hidden;
      devAutoSellPanel.hidden = !shouldOpen;
      if (shouldOpen) {
        validateFieldByName("automaticDevSellMarketCapThreshold");
        validateFieldByName("automaticDevSellMarketCapScanTimeoutSeconds");
      }
      renderClosedPanelWarning();
    }

    function escapeHTML(value) {
      return String(value || "")
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
    }

    function normalizeSellPercent(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric) || numeric <= 0) return "";
      return String(Math.max(1, Math.min(100, Math.round(numeric))));
    }

    function getSniperSellPercentError(value) {
      const raw = String(value ?? "").trim();
      if (!raw) return "Sell % is required.";
      const numeric = Number(raw);
      if (!Number.isFinite(numeric) || numeric <= 0 || numeric > 100) {
        return "Sell % must be 1-100.";
      }
      return "";
    }

    function getSniperMarketCapThresholdError(value) {
      const raw = String(value ?? "").trim();
      if (!raw) return "USD market cap is required.";
      const normalized = parseMarketCapThreshold(raw);
      if (!Number.isFinite(normalized) || normalized <= 0) {
        return "Use a positive USD amount like 100000 or 100k.";
      }
      return "";
    }

    function getSniperMarketCapTimeoutError(value) {
      const raw = String(value ?? "").trim();
      if (!raw) return "Timeout is required.";
      const numeric = Number(raw);
      if (!Number.isFinite(numeric) || numeric < 1 || numeric > 86400) {
        return "Must be between 1 and 86400.";
      }
      return "";
    }

    function syncSniperPercentFieldState(input) {
      if (!input) return;
      const message = getSniperSellPercentError(input.value);
      input.classList.toggle("input-error", !!message);
      const row = input.closest(".auto-sell-sniper-wallet-row");
      const errorEl = row ? row.querySelector(".auto-sell-sniper-percent-error") : null;
      if (errorEl) errorEl.textContent = message;
      const percentPill = row ? row.querySelector(".auto-sell-sniper-wallet-percent-pill") : null;
      if (percentPill) {
        percentPill.textContent = message ? "Sell % required" : `${String(input.value || "").trim()}% sell`;
      }
    }

    function syncSniperMarketCapFieldState(input) {
      if (!input) return;
      const isThresholdField = input.hasAttribute("data-auto-sell-sniper-market-threshold");
      const message = isThresholdField
        ? getSniperMarketCapThresholdError(input.value)
        : getSniperMarketCapTimeoutError(input.value);
      input.classList.toggle("input-error", !!message);
      const field = input.closest(".auto-sell-inline-field");
      const errorEl = field ? field.querySelector(".field-error") : null;
      if (errorEl) errorEl.textContent = message;
    }

    function normalizeSellBlockOffset(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(SNIPER_AUTOSELL_MAX_BLOCK_OFFSET, Math.round(numeric)));
    }

    function normalizeSellTimeoutSeconds(value) {
      const numeric = Number(value || 30);
      if (!Number.isFinite(numeric)) return 30;
      return Math.max(1, Math.min(86400, Math.round(numeric)));
    }

    function normalizeSellTimeoutAction(value) {
      return String(value || "").trim().toLowerCase() === "sell" ? "sell" : "stop";
    }

    function normalizeDevPercent(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric) || numeric <= 0) return "100";
      return String(Math.max(1, Math.min(100, Math.round(numeric))));
    }

    function renderSniperAutosellRows(rowsOverride) {
      if (!autoSellSniperWalletList || typeof getSniperAutosellRows !== "function") return;
      const sniperAutoSellEnabled = isNamedChecked("automaticSniperSellEnabled");
      const rows = Array.isArray(rowsOverride) ? rowsOverride : getSniperAutosellRows();
      if (!rows.length) {
        autoSellSniperWalletList.innerHTML = "<div class=\"auto-sell-sniper-empty muted small\">Select sniper buy wallets in Snipe modal to configure autosells.</div>";
        return;
      }
      if (!sniperAutoSellEnabled) {
        autoSellSniperWalletList.innerHTML = `
          <div class="auto-sell-sniper-empty muted small">
            Sniper autosell is Off. ${rows.length} wallet${rows.length === 1 ? "" : "s"} configured and ready when enabled.
          </div>
        `;
        return;
      }
      const markup = rows.map((row) => {
        const sellEnabled = Boolean(row.sellEnabled);
        const sellMode = String(row.sellTriggerMode || "").trim().toLowerCase() === "market-cap" ? "market-cap" : "block-offset";
        const rawSellPercent = String(row.sellPercent ?? "").trim();
        const sellPercent = normalizeSellPercent(rawSellPercent);
        const sellPercentError = sellEnabled ? getSniperSellPercentError(rawSellPercent) : "";
        const sellBlockOffset = normalizeSellBlockOffset(row.sellTargetBlockOffset);
        const sellThreshold = String(row.sellMarketCapThreshold || "").trim();
        const sellThresholdError = sellEnabled && sellMode === "market-cap"
          ? getSniperMarketCapThresholdError(sellThreshold)
          : "";
        const timeoutSecondsRaw = String(row.sellMarketCapTimeoutSeconds || "").trim();
        const timeoutSeconds = timeoutSecondsRaw ? timeoutSecondsRaw : "";
        const timeoutSecondsError = sellEnabled && sellMode === "market-cap"
          ? getSniperMarketCapTimeoutError(timeoutSeconds)
          : "";
        const timeoutAction = normalizeSellTimeoutAction(row.sellMarketCapTimeoutAction);
        const buyAmountPillMarkup = row.buyAmountValue
          ? (String(row.buyAmountAssetLabel || "").toUpperCase() === "SOL"
            ? `<span class="auto-sell-sniper-wallet-pill auto-sell-sniper-wallet-buy-pill">${escapeHTML(row.buyAmountValue)} <img src="/images/solana-mark.png" alt="SOL" class="sol-logo inline-sol-logo auto-sell-sniper-wallet-buy-icon"></span>`
            : `<span class="auto-sell-sniper-wallet-pill auto-sell-sniper-wallet-buy-pill">${escapeHTML(`${row.buyAmountValue} ${row.buyAmountAssetLabel || ""}`.trim())}</span>`)
          : "";
        const percentPillMarkup = sellEnabled
          ? `<span class="auto-sell-sniper-wallet-pill auto-sell-sniper-wallet-percent-pill">${escapeHTML(sellPercentError ? "Sell % required" : `${sellPercent}% sell`)}</span>`
          : "";
        return `
          <div class="auto-sell-sniper-wallet-row${sellEnabled ? " is-active" : ""}">
            <div class="auto-sell-sniper-wallet-main">
              <div class="auto-sell-sniper-wallet-info">
                <div class="auto-sell-sniper-wallet-title-row">
                  <strong>${escapeHTML(row.walletLabel || row.envKey)}</strong>
                  ${buyAmountPillMarkup}
                  ${percentPillMarkup}
                </div>
              </div>
              <div class="auto-sell-sniper-wallet-head-actions">
                <button type="button" class="button subtle auto-sell-sniper-wallet-toggle${sellEnabled ? " active" : ""}" data-auto-sell-sniper-toggle="${escapeHTML(row.envKey)}" title="Enable or disable sniper autosell for this wallet." ${!sniperAutoSellEnabled ? "disabled" : ""}>${sellEnabled ? "Auto sell on" : "Auto sell off"}</button>
                <label class="auto-sell-sniper-percent-field">
                  <span class="auto-sell-percent-input-wrap">
                    <input class="auto-sell-percent-inline-input${sellPercentError ? " input-error" : ""}" type="number" min="1" max="100" step="1" value="${escapeHTML(rawSellPercent)}" title="Percent of the wallet position to sell." data-auto-sell-sniper-percent="${escapeHTML(row.envKey)}" ${!sniperAutoSellEnabled || !sellEnabled ? "disabled" : ""}>
                    <span class="auto-sell-percent-suffix">%</span>
                  </span>
                </label>
              </div>
              <div class="field-error auto-sell-sniper-percent-error">${escapeHTML(sellPercentError)}</div>
            </div>
            <div class="auto-sell-sniper-wallet-config"${sniperAutoSellEnabled && sellEnabled ? "" : " hidden"}>
              <div class="auto-sell-trigger-toolbar auto-sell-sniper-trigger-toolbar">
                <div class="auto-sell-segmented-control" title="Choose the sniper autosell trigger family.">
                  <button type="button" class="preset-chip compact wallet-chip-button auto-sell-segmented-button${sellMode === "block-offset" ? " active" : ""}" data-auto-sell-sniper-mode="${escapeHTML(row.envKey)}" data-auto-sell-sniper-mode-value="block-offset" title="Sell after the matching buy confirms, plus the selected extra confirmed slots.">
                    <span class="mode-title">After Buy</span>
                  </button>
                  <button type="button" class="preset-chip compact wallet-chip-button auto-sell-segmented-button${sellMode === "market-cap" ? " active" : ""}" data-auto-sell-sniper-mode="${escapeHTML(row.envKey)}" data-auto-sell-sniper-mode-value="market-cap" title="Start watching market cap after the matching buy confirms, then sell when the target is reached.">
                    <span class="mode-title">Mcap</span>
                  </button>
                </div>
                <div class="auto-sell-sniper-block-inline" title="Extra confirmed slots to wait after the matching buy confirms."${sellMode === "block-offset" ? "" : " hidden"}>
                  <div class="auto-sell-trigger-grid auto-sell-block-grid">
                    ${SNIPER_AUTOSELL_BLOCK_OFFSETS.map((offset) => `
                      <button type="button" class="auto-sell-trigger-chip${sellBlockOffset === offset ? " active" : ""}" data-auto-sell-sniper-block-offset="${escapeHTML(row.envKey)}" data-auto-sell-sniper-block-value="${offset}">${offset}</button>
                    `).join("")}
                  </div>
                </div>
                <div class="auto-sell-inline-settings auto-sell-sniper-market-inline"${sellMode === "market-cap" ? "" : " hidden"}>
                  <label class="auto-sell-inline-field auto-sell-inline-field-threshold">
                    <span>USD Mcap</span>
                    <input type="text" class="auto-sell-market-cap-input${sellThresholdError ? " input-error" : ""}" value="${escapeHTML(sellThreshold)}" placeholder="e.g 100k" title="USD market cap target that triggers the sell." data-auto-sell-sniper-market-threshold="${escapeHTML(row.envKey)}">
                    <div class="field-error">${escapeHTML(sellThresholdError)}</div>
                  </label>
                  <label class="auto-sell-inline-field auto-sell-inline-field-timeout">
                    <span>Timeout</span>
                    <div class="auto-sell-market-cap-timeout-row">
                      <input type="number" class="auto-sell-market-cap-input${timeoutSecondsError ? " input-error" : ""}" min="1" max="86400" step="1" value="${escapeHTML(timeoutSeconds)}" placeholder="e.g 30" title="Stop watching market cap after this many seconds." data-auto-sell-sniper-market-timeout="${escapeHTML(row.envKey)}">
                      <span class="auto-sell-market-cap-suffix">s</span>
                    </div>
                    <div class="field-error">${escapeHTML(timeoutSecondsError)}</div>
                  </label>
                  <label class="auto-sell-inline-field auto-sell-inline-field-action">
                    <span>Action</span>
                    <select class="auto-sell-market-cap-select" title="Action to take if market cap target is not reached before timeout." data-auto-sell-sniper-market-timeout-action="${escapeHTML(row.envKey)}">
                      <option value="stop"${timeoutAction === "stop" ? " selected" : ""}>Stop</option>
                      <option value="sell"${timeoutAction === "sell" ? " selected" : ""}>Sell</option>
                    </select>
                  </label>
                </div>
              </div>
            </div>
          </div>
        `;
      }).join("");
      autoSellSniperWalletList.innerHTML = markup;
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
      const marketCapTimeoutInputValue = getMarketCapTimeoutInputValue();
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
      if (getNamedValue("automaticDevSellMarketCapTimeoutAction") !== marketCapTimeoutAction) {
        setNamedValue("automaticDevSellMarketCapTimeoutAction", marketCapTimeoutAction);
      }

      const sniperAutoSellEnabled = isNamedChecked("automaticSniperSellEnabled");
      const sniperRows = typeof getSniperAutosellRows === "function" ? getSniperAutosellRows() : [];
      const sniperAutoSellCount = sniperAutoSellEnabled
        ? sniperRows.filter((row) => row && row.sellEnabled).length
        : 0;
      const autoSellButtonActive = enabled || sniperAutoSellCount > 0;
      if (devAutoSellButton) devAutoSellButton.classList.toggle("active", autoSellButtonActive);
      if (autoSellButtonProgress) {
        let badgeText = "(0)";
        let badgeTitle = "Auto-sell is off";
        if (enabled && sniperAutoSellCount > 0) {
          badgeText = `(D+${sniperAutoSellCount})`;
          badgeTitle = `Dev auto-sell is on and ${sniperAutoSellCount} sniper wallet${sniperAutoSellCount === 1 ? "" : "s"} have auto-sell enabled`;
        } else if (enabled) {
          badgeText = "(D)";
          badgeTitle = "Dev auto-sell is on";
        } else if (sniperAutoSellCount > 0) {
          badgeText = `(${sniperAutoSellCount})`;
          badgeTitle = `${sniperAutoSellCount} sniper wallet${sniperAutoSellCount === 1 ? "" : "s"} have auto-sell enabled`;
        }
        autoSellButtonProgress.textContent = badgeText;
        autoSellButtonProgress.title = badgeTitle;
        autoSellButtonProgress.setAttribute("aria-label", badgeTitle);
      }
      if (autoSellToggleState) autoSellToggleState.textContent = enabled ? "On" : "Off";
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
        autoSellTriggerValue.textContent = triggerFamily === "market-cap" ? "Mcap" : getTriggerLabel(triggerMode);
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
      if (autoSellDelayInput) {
        autoSellDelayInput.value = delayMs;
        autoSellDelayInput.disabled = !enabled || triggerFamily !== "time" || triggerMode !== "submit-delay";
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
      if (autoSellPercentInput) {
        autoSellPercentInput.value = percent;
        autoSellPercentInput.disabled = !enabled;
      }
      if (autoSellDelayValue) autoSellDelayValue.textContent = formatSliderValue(delayMs, "ms", 0);
      if (autoSellBlockValue) autoSellBlockValue.textContent = `+${blockOffset}`;
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
          : (triggerFamily === "market-cap" ? "Required" : "Disabled");
      }
      if (autoSellMarketCapTimeoutInput) {
        autoSellMarketCapTimeoutInput.value = marketCapTimeoutInputValue;
        autoSellMarketCapTimeoutInput.disabled = !enabled || triggerFamily !== "market-cap";
      }
      if (autoSellMarketCapTimeoutActionInput) {
        autoSellMarketCapTimeoutActionInput.value = marketCapTimeoutAction;
        autoSellMarketCapTimeoutActionInput.disabled = !enabled || triggerFamily !== "market-cap";
      }
      if (autoSellSniperEnabledInput) autoSellSniperEnabledInput.checked = sniperAutoSellEnabled;
      if (autoSellSniperToggleState) autoSellSniperToggleState.textContent = sniperAutoSellEnabled ? "On" : "Off";
      renderSniperAutosellRows(sniperRows);
      renderClosedPanelWarning();
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
      if (autoSellSniperEnabledInput) {
        autoSellSniperEnabledInput.addEventListener("change", () => {
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
        });
      }
      if (autoSellSniperWalletList) {
        autoSellSniperWalletList.addEventListener("click", (event) => {
          if (typeof updateSniperAutosellWallet !== "function") return;
          const toggleButton = event.target.closest("[data-auto-sell-sniper-toggle]");
          if (toggleButton) {
            const envKey = toggleButton.getAttribute("data-auto-sell-sniper-toggle");
            if (!envKey) return;
            const rows = typeof getSniperAutosellRows === "function" ? getSniperAutosellRows() : [];
            const current = rows.find((entry) => entry.envKey === envKey);
            updateSniperAutosellWallet(envKey, {
              sellEnabled: !Boolean(current && current.sellEnabled),
              sellPercent: normalizeSellPercent(current && current.sellPercent) || "100",
            });
            syncUI();
            if (typeof persistDraft === "function") persistDraft();
            syncActivePresetFromInputs();
            return;
          }
          const modeButton = event.target.closest("[data-auto-sell-sniper-mode]");
          if (modeButton) {
            const envKey = modeButton.getAttribute("data-auto-sell-sniper-mode");
            const mode = modeButton.getAttribute("data-auto-sell-sniper-mode-value");
            if (!envKey) return;
            updateSniperAutosellWallet(envKey, {
              sellEnabled: true,
              sellTriggerMode: String(mode || "").trim().toLowerCase() === "market-cap" ? "market-cap" : "block-offset",
            });
            syncUI();
            if (typeof persistDraft === "function") persistDraft();
            syncActivePresetFromInputs();
            return;
          }
          const blockButton = event.target.closest("[data-auto-sell-sniper-block-offset]");
          if (blockButton) {
            const envKey = blockButton.getAttribute("data-auto-sell-sniper-block-offset");
            const value = blockButton.getAttribute("data-auto-sell-sniper-block-value");
            if (!envKey) return;
            updateSniperAutosellWallet(envKey, {
              sellEnabled: true,
              sellTriggerMode: "block-offset",
              sellTargetBlockOffset: normalizeSellBlockOffset(value),
            });
            syncUI();
            if (typeof persistDraft === "function") persistDraft();
            syncActivePresetFromInputs();
          }
        });
        autoSellSniperWalletList.addEventListener("input", (event) => {
          if (typeof updateSniperAutosellWallet !== "function") return;
          const percentInput = event.target.closest("[data-auto-sell-sniper-percent]");
          if (percentInput) {
            syncSniperPercentFieldState(percentInput);
            return;
          }
          const thresholdInput = event.target.closest("[data-auto-sell-sniper-market-threshold]");
          if (thresholdInput) {
            syncSniperMarketCapFieldState(thresholdInput);
            return;
          }
          const timeoutInput = event.target.closest("[data-auto-sell-sniper-market-timeout]");
          if (timeoutInput) {
            syncSniperMarketCapFieldState(timeoutInput);
          }
        });
        autoSellSniperWalletList.addEventListener("change", (event) => {
          if (typeof updateSniperAutosellWallet !== "function") return;
          const thresholdInput = event.target.closest("[data-auto-sell-sniper-market-threshold]");
          if (thresholdInput) {
            const envKey = thresholdInput.getAttribute("data-auto-sell-sniper-market-threshold");
            if (!envKey) return;
            updateSniperAutosellWallet(envKey, {
              sellEnabled: true,
              sellTriggerMode: "market-cap",
              sellMarketCapThreshold: String(thresholdInput.value || "").trim(),
            });
            syncUI();
            if (typeof persistDraft === "function") persistDraft();
            syncActivePresetFromInputs();
            return;
          }
          const timeoutInput = event.target.closest("[data-auto-sell-sniper-market-timeout]");
          if (timeoutInput) {
            const envKey = timeoutInput.getAttribute("data-auto-sell-sniper-market-timeout");
            if (!envKey) return;
            updateSniperAutosellWallet(envKey, {
              sellEnabled: true,
              sellTriggerMode: "market-cap",
              sellMarketCapTimeoutSeconds: String(timeoutInput.value || "").trim(),
            });
            syncUI();
            if (typeof persistDraft === "function") persistDraft();
            syncActivePresetFromInputs();
            return;
          }
          const timeoutActionSelect = event.target.closest("[data-auto-sell-sniper-market-timeout-action]");
          if (!timeoutActionSelect) return;
          const envKey = timeoutActionSelect.getAttribute("data-auto-sell-sniper-market-timeout-action");
          if (!envKey) return;
          updateSniperAutosellWallet(envKey, {
            sellEnabled: true,
            sellTriggerMode: "market-cap",
            sellMarketCapTimeoutAction: normalizeSellTimeoutAction(timeoutActionSelect.value),
          });
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
        });
      }
      autoSellTriggerFamilyButtons.forEach((button) => {
        button.addEventListener("click", () => {
          const nextTriggerFamily = normalizeTriggerFamily(button.getAttribute("data-auto-sell-trigger-family"));
          setNamedValue(
            "automaticDevSellTriggerFamily",
            nextTriggerFamily
          );
          setNamedChecked(
            "automaticDevSellMarketCapEnabled",
            nextTriggerFamily === "market-cap"
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
          if (nextTriggerFamily === "market-cap" && !String(getMarketCapThreshold() || "").trim() && autoSellMarketCapThresholdInput) {
            autoSellMarketCapThresholdInput.focus();
            if (typeof autoSellMarketCapThresholdInput.select === "function") {
              autoSellMarketCapThresholdInput.select();
            }
          }
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
      if (autoSellDelayInput) {
        autoSellDelayInput.addEventListener("input", () => {
          const nextValue = String(autoSellDelayInput.value || "").trim();
          const parsed = Number(nextValue);
          const safeValue = Number.isFinite(parsed)
            ? String(Math.max(0, Math.min(1500, Math.round(parsed))))
            : "0";
          setNamedValue("automaticDevSellDelayMs", safeValue);
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
          setNamedValue("automaticDevSellMarketCapScanTimeoutSeconds", String(autoSellMarketCapTimeoutInput.value || "").trim());
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
      if (autoSellSniperWalletList) {
        autoSellSniperWalletList.addEventListener("change", (event) => {
          if (typeof updateSniperAutosellWallet !== "function") return;
          const percentInput = event.target.closest("[data-auto-sell-sniper-percent]");
          if (percentInput) {
            const envKey = percentInput.getAttribute("data-auto-sell-sniper-percent");
            if (!envKey) return;
            updateSniperAutosellWallet(envKey, {
              sellEnabled: true,
              sellPercent: normalizeSellPercent(percentInput.value),
            });
            syncUI();
            if (typeof persistDraft === "function") persistDraft();
            syncActivePresetFromInputs();
            return;
          }
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
      if (autoSellPercentInput) {
        autoSellPercentInput.addEventListener("input", () => {
          setNamedValue("automaticDevSellPercent", normalizeDevPercent(autoSellPercentInput.value));
          syncUI();
          if (typeof persistDraft === "function") persistDraft();
          syncActivePresetFromInputs();
          validateFieldByName("automaticDevSellPercent");
        });
      }
      autoSellCloseButtons.forEach((button) => button.addEventListener("click", () => togglePanel(false)));
      if (devAutoSellPanel) {
        devAutoSellPanel.addEventListener("pointerdown", (event) => {
          autoSellOverlayPointerDown = event.target === devAutoSellPanel;
        });
        devAutoSellPanel.addEventListener("click", (event) => {
          if (event.target !== devAutoSellPanel || !autoSellOverlayPointerDown) {
            autoSellOverlayPointerDown = false;
            return;
          }
          autoSellOverlayPointerDown = false;
          const selection = typeof window.getSelection === "function" ? window.getSelection() : null;
          if (selection && !selection.isCollapsed && String(selection).trim()) return;
          togglePanel(false);
        });
      }
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
