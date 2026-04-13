(function initSniperFeature(global) {
  function createSniperFeature(config) {
    const SNIPER_BUY_MAX_BLOCK_OFFSET = 37;
    const SNIPER_SELL_MAX_BLOCK_OFFSET = 23;
    const SNIPER_BUY_BLOCK_OFFSETS = Array.from(
      { length: SNIPER_BUY_MAX_BLOCK_OFFSET + 1 },
      (_, index) => index,
    );
    const {
      storageKey,
      readStoredDraft,
      writeStoredDraft,
      renderCache,
      balancePresets,
      executionReserveSol,
      elements,
      getLatestWalletStatus,
      getAppBootstrapState,
      getSelectedWalletKey,
      getNamedValue,
      walletDisplayName,
      walletIndexFromEnvKey,
      shortenAddress,
      escapeHTML,
      normalizeDecimalInput,
      isNamedChecked,
      getRouteCapabilities,
      getBuyProvider,
      getSellProvider,
      refreshWalletStatus,
      metaNode,
      onStateChange,
    } = config;

    const {
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
    } = elements;

    const sniperModalCard = sniperModal ? sniperModal.querySelector(".sniper-modal") : null;
    let sniperState = {
      enabled: false,
      wallets: {},
    };
    let sniperModalOverlayPointerDown = false;
    let eventsBound = false;

    function normalizeTriggerMode(value) {
      const mode = String(value || "").trim().toLowerCase();
      if (mode === "same-time" || mode === "on-submit" || mode === "block-offset") return mode;
      if (mode === "instant") return "on-submit";
      if (mode === "submit-delay") return "on-submit";
      return "on-submit";
    }

    function normalizeDelayMs(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(1500, Math.round(numeric)));
    }

    function normalizeBuyBlockOffset(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(SNIPER_BUY_MAX_BLOCK_OFFSET, Math.round(numeric)));
    }

    function normalizeSellBlockOffset(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric)) return 0;
      return Math.max(0, Math.min(SNIPER_SELL_MAX_BLOCK_OFFSET, Math.round(numeric)));
    }

    function normalizeSellPercent(value) {
      const numeric = Number(value || 0);
      if (!Number.isFinite(numeric) || numeric <= 0) return "";
      return String(Math.max(1, Math.min(100, Math.round(numeric))));
    }

    function normalizeSellTriggerMode(value, fallbackEntry = {}) {
      const mode = String(value || "").trim().toLowerCase();
      if (mode === "market-cap") return "market-cap";
      if (mode === "block-offset") return "block-offset";
      if (String(fallbackEntry.sellMarketCapThreshold || "").trim()) return "market-cap";
      return "block-offset";
    }

    function normalizeMarketCapTimeoutSeconds(value) {
      const raw = String(value ?? "").trim();
      if (!raw) return "";
      const numeric = Number(raw);
      if (!Number.isFinite(numeric)) return "";
      return Math.max(1, Math.min(86400, Math.round(numeric)));
    }

    function normalizeMarketCapTimeoutAction(value) {
      return String(value || "").trim().toLowerCase() === "sell" ? "sell" : "stop";
    }

    function normalizeWalletState(entry = {}) {
      const triggerMode = normalizeTriggerMode(entry.triggerMode);
      const sellTriggerMode = normalizeSellTriggerMode(entry.sellTriggerMode, entry);
      return {
        selected: Boolean(entry.selected),
        amountSol: normalizeDecimalInput(entry.amountSol || ""),
        triggerMode: entry && entry.triggerMode ? triggerMode : "block-offset",
        submitDelayMs: normalizeDelayMs(entry.submitDelayMs),
        targetBlockOffset: normalizeBuyBlockOffset(entry.targetBlockOffset),
        retryOnce: Boolean(entry.retryOnce),
        sellEnabled: Boolean(entry.sellEnabled),
        sellPercent: normalizeSellPercent(entry.sellPercent),
        sellTriggerMode,
        sellTargetBlockOffset: normalizeSellBlockOffset(entry.sellTargetBlockOffset),
        sellMarketCapThreshold: String(entry.sellMarketCapThreshold || "").trim(),
        sellMarketCapTimeoutSeconds: normalizeMarketCapTimeoutSeconds(entry.sellMarketCapTimeoutSeconds),
        sellMarketCapTimeoutAction: normalizeMarketCapTimeoutAction(entry.sellMarketCapTimeoutAction),
      };
    }

    function normalizeDraftState(value) {
      const wallets = value && typeof value.wallets === "object" && value.wallets
        ? Object.entries(value.wallets).reduce((accumulator, [envKey, entry]) => {
          if (!envKey) return accumulator;
          accumulator[envKey] = normalizeWalletState(entry || {});
          return accumulator;
        }, {})
        : {};
      const hasSelectedWallet = Object.values(wallets).some((entry) => entry && entry.selected);
      return {
        enabled: Boolean(value && value.enabled) && hasSelectedWallet,
        wallets,
      };
    }

    function getStoredDraft() {
      if (typeof readStoredDraft === "function") {
        try {
          const draft = readStoredDraft();
          return draft ? normalizeDraftState(draft) : null;
        } catch (_error) {
          return null;
        }
      }
      try {
        const raw = window.localStorage.getItem(storageKey);
        if (!raw) return null;
        return normalizeDraftState(JSON.parse(raw));
      } catch (_error) {
        return null;
      }
    }

    function persistDraft() {
      const hasWalletState = Object.values(sniperState.wallets || {}).some((entry) => {
        const normalized = normalizeWalletState(entry || {});
        return normalized.selected
          || Boolean(normalized.amountSol)
          || normalized.triggerMode !== "on-submit"
          || normalized.submitDelayMs > 0
          || normalized.targetBlockOffset > 0
          || normalized.retryOnce
          || normalized.sellEnabled
          || Boolean(normalized.sellPercent)
          || normalized.sellTriggerMode !== "block-offset"
          || normalized.sellTargetBlockOffset > 0
          || Boolean(normalized.sellMarketCapThreshold)
          || String(normalized.sellMarketCapTimeoutSeconds || "").trim() !== ""
          || normalized.sellMarketCapTimeoutAction !== "stop";
      });
      const normalizedDraft = normalizeDraftState(sniperState);
      if (typeof writeStoredDraft === "function") {
        writeStoredDraft(sniperState.enabled || hasWalletState ? normalizedDraft : null);
        return;
      }
      try {
        if (!sniperState.enabled && !hasWalletState) {
          window.localStorage.removeItem(storageKey);
          return;
        }
        window.localStorage.setItem(storageKey, JSON.stringify(normalizedDraft));
      } catch (_error) {
        // Ignore storage failures and keep sniper controls functional.
      }
    }

    function parseSolInputValue(value) {
      const numeric = Number(String(value || "").trim());
      if (!Number.isFinite(numeric) || numeric <= 0) return 0;
      return numeric;
    }

    function parseSolDecimalToLamports(value) {
      const trimmed = String(value || "").trim();
      if (!trimmed) return 0n;
      const normalized = trimmed.replace(/,/g, ".");
      const parts = normalized.split(".");
      if (parts.length > 2) return 0n;
      const whole = parts[0] || "0";
      const fractional = parts[1] || "";
      if (!/^\d+$/.test(whole) || (fractional && !/^\d+$/.test(fractional))) return 0n;
      const fractionalText = `${fractional.slice(0, 9)}${"0".repeat(9)}`.slice(0, 9);
      return (BigInt(whole) * 1000000000n) + BigInt(fractionalText);
    }

    function formatLamportsToSolDecimal(value) {
      const lamports = typeof value === "bigint" ? value : BigInt(value || 0);
      const whole = lamports / 1000000000n;
      const fractional = lamports % 1000000000n;
      if (fractional === 0n) return whole.toString();
      let fractionalText = fractional.toString().padStart(9, "0");
      fractionalText = fractionalText.replace(/0+$/, "");
      return `${whole.toString()}.${fractionalText}`;
    }

    function formatLamportsToSolDecimalMax4(value) {
      const sol = Number(formatLamportsToSolDecimal(value));
      if (!Number.isFinite(sol) || sol <= 0) return "0";
      return sol.toFixed(4).replace(/\.?0+$/, "");
    }

    function getExecutionReserveSol() {
      const buyCapabilities = getRouteCapabilities(getBuyProvider(), "buy");
      const sellCapabilities = getRouteCapabilities(getSellProvider(), "sell");
      const buyReserve = (buyCapabilities.priority ? parseSolInputValue(getNamedValue("buyPriorityFeeSol")) : 0)
        + (buyCapabilities.tip ? parseSolInputValue(getNamedValue("buyTipSol")) : 0);
      const sellReserve = (sellCapabilities.priority ? parseSolInputValue(getNamedValue("sellPriorityFeeSol")) : 0)
        + (sellCapabilities.tip ? parseSolInputValue(getNamedValue("sellTipSol")) : 0);
      return buyReserve + sellReserve + executionReserveSol;
    }

    function getWalletBalanceForSniper(wallet) {
      if (!wallet) return null;
      if (wallet.balanceSol != null && Number.isFinite(Number(wallet.balanceSol))) {
        return Number(wallet.balanceSol);
      }
      const latestWalletStatus = getLatestWalletStatus();
      if (latestWalletStatus && wallet.envKey === latestWalletStatus.selectedWalletKey) {
        return latestWalletStatus.balanceSol == null ? null : Number(latestWalletStatus.balanceSol);
      }
      return null;
    }

    function getSpendableBalanceSol(wallet) {
      const balance = getWalletBalanceForSniper(wallet);
      if (balance == null || !Number.isFinite(Number(balance))) return null;
      return Math.max(0, Number(balance) - getExecutionReserveSol());
    }

    function floorDecimal(value, decimals = 6) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric) || numeric <= 0) return 0;
      const factor = 10 ** decimals;
      return Math.floor(numeric * factor) / factor;
    }

    function getWalletWarning(entry = {}, balanceSol, spendableBalanceSol = null) {
      const amount = Number(entry.amountSol || 0);
      if (!Number.isFinite(amount) || amount <= 0 || balanceSol == null) return "";
      const effectiveMax = spendableBalanceSol == null ? Number(balanceSol) : Number(spendableBalanceSol);
      if (amount <= effectiveMax + 0.000001) return "";
      return "Amount exceeds spendable balance after fee reserve.";
    }

    function getTriggerSummary(entry = {}) {
      const triggerMode = normalizeTriggerMode(entry.triggerMode);
      if (triggerMode === "same-time") {
        return "Same Time";
      }
      if (triggerMode === "on-submit") {
        const delayMs = normalizeDelayMs(entry.submitDelayMs);
        return delayMs > 0 ? `Submit + ${delayMs}ms` : "On Submit";
      }
      if (triggerMode === "block-offset") {
        return `On Confirmed Slot + ${normalizeBuyBlockOffset(entry.targetBlockOffset)}`;
      }
      return "On Submit";
    }

    function getTriggerDescription(entry = {}) {
      const triggerMode = normalizeTriggerMode(entry.triggerMode);
      if (triggerMode === "same-time") {
        return "Compiled into the launch send batch. Might fail if landing before creation tx. Enable retry to retry if fail.";
      }
      if (triggerMode === "on-submit") {
        const delayMs = normalizeDelayMs(entry.submitDelayMs);
        return delayMs > 0
          ? `Sent ${delayMs}ms after launch submit is observed.`
          : "Sent right after launch submit is observed.";
      }
      if (triggerMode === "block-offset") {
        return `Send the buy transaction on confirmed slot + ${normalizeBuyBlockOffset(entry.targetBlockOffset)} from launch confirmation.`;
      }
      return "";
    }

    function getTriggerTooltip(mode, entry = {}) {
      return getTriggerDescription({
        ...entry,
        triggerMode: mode,
      });
    }

    function getSelectedEntries() {
      return Object.entries(sniperState.wallets || {})
        .filter(([, entry]) => entry && entry.selected)
        .map(([envKey, entry]) => {
          const normalized = normalizeWalletState(entry);
          const sellPercentNumeric = Number(normalized.sellPercent || 0);
          const sellEnabled = normalized.sellEnabled && sellPercentNumeric > 0;
          return {
            envKey,
            amountSol: normalized.amountSol,
            triggerMode: normalized.triggerMode,
            submitWithLaunch: normalized.triggerMode === "same-time",
            submitDelayMs: normalized.triggerMode === "on-submit" ? normalized.submitDelayMs : 0,
            targetBlockOffset: normalized.triggerMode === "block-offset" ? normalized.targetBlockOffset : null,
            retryOnce: normalized.triggerMode === "same-time" ? normalized.retryOnce : false,
            sellEnabled,
            sellPercent: sellEnabled ? sellPercentNumeric : null,
            sellTriggerMode: normalized.sellTriggerMode,
            sellTargetBlockOffset: sellEnabled && normalized.sellTriggerMode === "block-offset"
              ? normalized.sellTargetBlockOffset
              : null,
            sellMarketCapThreshold: sellEnabled && normalized.sellTriggerMode === "market-cap"
              ? normalized.sellMarketCapThreshold
              : "",
            sellMarketCapTimeoutSeconds: sellEnabled && normalized.sellTriggerMode === "market-cap"
              ? normalized.sellMarketCapTimeoutSeconds
              : null,
            sellMarketCapTimeoutAction: sellEnabled && normalized.sellTriggerMode === "market-cap"
              ? normalized.sellMarketCapTimeoutAction
              : "stop",
            sellMarketCapDirection: "gte",
          };
        });
    }

    function getSameTimeFeeGuardNotice() {
      if (!sniperState.enabled) return null;
      const selectedEntries = getSelectedEntries().filter((entry) => normalizeTriggerMode(entry.triggerMode) === "same-time");
      if (!selectedEntries.length) return null;
      const creationPriority = parseSolDecimalToLamports(getNamedValue("creationPriorityFeeSol"));
      const creationTip = parseSolDecimalToLamports(getNamedValue("creationTipSol"));
      const buyPriority = parseSolDecimalToLamports(getNamedValue("buyPriorityFeeSol"));
      const buyTip = parseSolDecimalToLamports(getNamedValue("buyTipSol"));
      const adjustedFields = [];
      let extraLamports = 0n;
      if (buyPriority > creationPriority) {
        adjustedFields.push("priority fee");
        extraLamports += (buyPriority + 1n) - creationPriority;
      }
      if (buyTip > creationTip) {
        adjustedFields.push("tip");
        extraLamports += (buyTip + 1n) - creationTip;
      }
      if (!adjustedFields.length) {
        return null;
      }
      const extraCostText = formatLamportsToSolDecimalMax4(extraLamports);
      return {
        kind: "warning",
        message: `Same Time safeguard active. Launch ${adjustedFields.join(" and ")} will be raised above same-time buy fees automatically at send time. Extra creator fee cost: ${extraCostText} SOL.`,
      };
    }

    function sortWallets(wallets = [], selectedKey = "") {
      return [...wallets].sort((left, right) => {
        const leftIsSelected = left && left.envKey === selectedKey;
        const rightIsSelected = right && right.envKey === selectedKey;
        if (leftIsSelected !== rightIsSelected) return leftIsSelected ? -1 : 1;
        const leftBalance = getWalletBalanceForSniper(left);
        const rightBalance = getWalletBalanceForSniper(right);
        if (leftBalance == null && rightBalance != null) return 1;
        if (leftBalance != null && rightBalance == null) return -1;
        if (leftBalance != null && rightBalance != null && leftBalance !== rightBalance) {
          return rightBalance - leftBalance;
        }
        return String(left && left.envKey || "").localeCompare(String(right && right.envKey || ""));
      });
    }

    function setModalError(message = "") {
      if (sniperModalError) sniperModalError.textContent = message;
    }

    function applyStateToForm() {
      const selectedEntries = getSelectedEntries().filter((entry) => Number(entry.amountSol) > 0);
      if (sniperEnabledInput) sniperEnabledInput.value = sniperState.enabled ? "true" : "false";
      if (sniperConfigJsonInput) sniperConfigJsonInput.value = JSON.stringify(selectedEntries);
      if (postLaunchStrategyInput) {
        postLaunchStrategyInput.value = sniperState.enabled && selectedEntries.length > 0 ? "snipe-own-launch" : "none";
      }
      if (snipeBuyAmountInput) {
        const total = selectedEntries.reduce((sum, entry) => sum + Number(entry.amountSol || 0), 0);
        snipeBuyAmountInput.value = total > 0 ? total.toFixed(6).replace(/\.?0+$/, "") : "";
      }
      persistDraft();
    }

    function renderButtonState() {
      const selectedEntries = getSelectedEntries().filter((entry) => Number(entry.amountSol) > 0);
      if (modeSniperButton) {
        modeSniperButton.classList.toggle("active", sniperState.enabled && selectedEntries.length > 0);
      }
      if (modeSniperProgress) {
        modeSniperProgress.textContent = `(${selectedEntries.length})`;
        modeSniperProgress.title = `${selectedEntries.length} wallet${selectedEntries.length === 1 ? "" : "s"} selected for snipe`;
        modeSniperProgress.setAttribute("aria-label", modeSniperProgress.title);
      }
    }

    function renderWalletList() {
      if (!sniperWalletList) return;
      const latestWalletStatus = getLatestWalletStatus();
      const appBootstrapState = getAppBootstrapState();
      const wallets = latestWalletStatus && Array.isArray(latestWalletStatus.wallets) ? latestWalletStatus.wallets : [];
      const selectedKey = latestWalletStatus && latestWalletStatus.selectedWalletKey ? latestWalletStatus.selectedWalletKey : "";
      if (selectedKey && sniperState.wallets[selectedKey]) {
        sniperState.wallets[selectedKey] = {
          ...normalizeWalletState(sniperState.wallets[selectedKey]),
          selected: false,
          amountSol: "",
        };
        applyStateToForm();
        renderButtonState();
      }
      const selectedEntries = getSelectedEntries();
      const selectedCount = selectedEntries.length;
      const totalAmount = selectedEntries
        .filter((entry) => Number(entry.amountSol || 0) > 0)
        .reduce((sum, entry) => sum + Number(entry.amountSol || 0), 0);
      if (sniperSelectionSummary) {
        sniperSelectionSummary.textContent = `${selectedCount} wallet${selectedCount === 1 ? "" : "s"} selected`;
      }
      if (sniperTotalSummary) {
        sniperTotalSummary.textContent = `${totalAmount > 0 ? totalAmount.toFixed(4).replace(/\.?0+$/, "") : "0"} SOL`;
      }
      if (sniperWalletsSection) sniperWalletsSection.hidden = !sniperState.enabled;
      if (sniperEnabledState) sniperEnabledState.textContent = sniperState.enabled ? "On" : "Off";
      if (sniperEnabledToggle) sniperEnabledToggle.checked = sniperState.enabled;

      if (!appBootstrapState.walletsLoaded) {
        const loadingMarkup = `<div class="sniper-wallet-empty muted">Loading wallets...</div>`;
        if (global.RenderUtils && global.RenderUtils.setCachedHTML) {
          global.RenderUtils.setCachedHTML(renderCache, "sniperWalletList", sniperWalletList, loadingMarkup);
        } else {
          sniperWalletList.innerHTML = loadingMarkup;
        }
        return;
      }

      if (wallets.length === 0) {
        const emptyMarkup = "<div class=\"sniper-wallet-empty muted\">No wallets found in `.env`.</div>";
        if (global.RenderUtils && global.RenderUtils.setCachedHTML) {
          global.RenderUtils.setCachedHTML(renderCache, "sniperWalletList", sniperWalletList, emptyMarkup);
        } else {
          sniperWalletList.innerHTML = emptyMarkup;
        }
        return;
      }

      const sortedWallets = sortWallets(wallets, selectedKey);
      const feeGuardNotice = getSameTimeFeeGuardNotice();
      const markup = sortedWallets.map((wallet) => {
        const disabled = wallet.envKey === selectedKey;
        const balanceSol = getWalletBalanceForSniper(wallet);
        const spendableBalanceSol = getSpendableBalanceSol(wallet);
        const state = normalizeWalletState(sniperState.wallets[wallet.envKey] || {});
        const amountWarning = state.selected && !disabled ? getWalletWarning(state, balanceSol, spendableBalanceSol) : "";
        return `
      <div class="sniper-wallet-row${disabled ? " is-disabled" : ""}${state.selected ? " is-selected" : ""}" data-sniper-wallet-row="${escapeHTML(wallet.envKey)}">
        <label class="sniper-wallet-main">
          <input
            type="checkbox"
            class="sniper-wallet-checkbox"
            data-sniper-wallet-checkbox="${escapeHTML(wallet.envKey)}"
            ${state.selected ? "checked" : ""}
            ${disabled ? "disabled" : ""}
          >
          <div class="sniper-wallet-info">
            <div class="sniper-wallet-name">${escapeHTML(walletDisplayName(wallet))}</div>
            <div class="sniper-wallet-meta">
              <span>${escapeHTML(shortenAddress(wallet.publicKey || "invalid", 5))}</span>
              ${state.selected && !disabled ? `<span class="sniper-wallet-pill">${escapeHTML(getTriggerSummary(state))}</span>` : ""}
              ${disabled ? "<span class=\"sniper-wallet-pill\">Deployer</span>" : ""}
            </div>
          </div>
          <div class="sniper-wallet-balance">
            <img src="/images/solana-mark.png" alt="SOL" class="sol-logo inline-sol-logo sniper-wallet-balance-icon">
            <span>${balanceSol == null ? "--" : Number(balanceSol).toFixed(3)}</span>
          </div>
        </label>
        <div class="sniper-wallet-config"${!state.selected || disabled ? " hidden" : ""}>
          <div class="sniper-wallet-config-top">
            <label class="sniper-wallet-amount">
              <span class="sniper-wallet-amount-input-wrap">
                <img src="/images/solana-mark.png" alt="SOL" class="sol-logo inline-sol-logo sniper-wallet-amount-icon">
                <input type="text" inputmode="decimal" value="${escapeHTML(state.amountSol || "")}" data-sniper-wallet-amount="${escapeHTML(wallet.envKey)}" placeholder="0">
              </span>
            </label>
            <div class="sniper-wallet-presets">
              ${balancePresets.map((preset) => `
                <button type="button" class="button subtle sniper-preset-button" data-sniper-preset="${escapeHTML(wallet.envKey)}" data-sniper-ratio="${preset.ratio}">
                  ${escapeHTML(preset.label)}
                </button>
              `).join("")}
            </div>
          </div>
          ${amountWarning ? `<div class="sniper-wallet-warning">${escapeHTML(amountWarning)}</div>` : ""}
          <div class="sniper-wallet-trigger">
            <div class="sniper-wallet-trigger-grid">
              <button type="button" class="sniper-trigger-chip${state.triggerMode === "same-time" ? " active" : ""}" data-sniper-trigger-mode="${escapeHTML(wallet.envKey)}" data-sniper-trigger-value="same-time" title="${escapeHTML(getTriggerTooltip("same-time", state))}">Same Time</button>
              <button type="button" class="sniper-trigger-chip${state.triggerMode === "on-submit" ? " active" : ""}" data-sniper-trigger-mode="${escapeHTML(wallet.envKey)}" data-sniper-trigger-value="on-submit" title="${escapeHTML(getTriggerTooltip("on-submit", state))}">On Submit + Delay</button>
              <button type="button" class="sniper-trigger-chip${state.triggerMode === "block-offset" ? " active" : ""}" data-sniper-trigger-mode="${escapeHTML(wallet.envKey)}" data-sniper-trigger-value="block-offset" title="${escapeHTML(getTriggerTooltip("block-offset", state))}">On Confirmed Slot</button>
            </div>
            ${state.triggerMode === "same-time" && feeGuardNotice ? `<div class="sniper-modal-notice${feeGuardNotice.kind === "warning" ? " is-warning" : ""}">${escapeHTML(feeGuardNotice.message)}</div>` : ""}
            <div class="sniper-wallet-trigger-detail"${state.triggerMode === "on-submit" ? "" : " hidden"}>
              <div class="auto-sell-slider-block sniper-delay-slider-block">
                <div class="auto-sell-slider-head">
                  <span>Delay</span>
                  <strong>${escapeHTML(`${state.submitDelayMs}ms`)}</strong>
                </div>
                <input class="auto-sell-slider" type="range" min="0" max="1500" step="25" value="${state.submitDelayMs}" data-sniper-wallet-delay="${escapeHTML(wallet.envKey)}">
              </div>
            </div>
            <div class="sniper-wallet-trigger-detail sniper-wallet-retry-row"${state.triggerMode === "same-time" ? "" : " hidden"}>
              <button
                type="button"
                class="button subtle sniper-retry-button${state.retryOnce ? " active" : ""}"
                data-sniper-wallet-retry="${escapeHTML(wallet.envKey)}"
                aria-pressed="${state.retryOnce ? "true" : "false"}"
              >${state.retryOnce ? "Retry On" : "Retry Off"}</button>
            </div>
            <div class="sniper-wallet-trigger-detail"${state.triggerMode === "block-offset" ? "" : " hidden"}>
              <div class="sniper-wallet-trigger-grid sniper-wallet-block-grid">
                ${SNIPER_BUY_BLOCK_OFFSETS.map((offset) => `
                  <button type="button" class="sniper-trigger-chip${state.targetBlockOffset === offset ? " active" : ""}" data-sniper-block-offset="${escapeHTML(wallet.envKey)}" data-sniper-block-value="${offset}">${offset}</button>
                `).join("")}
              </div>
            </div>
          </div>
        </div>
      </div>
    `;
      }).join("");
      if (global.RenderUtils && global.RenderUtils.setCachedHTML) {
        global.RenderUtils.setCachedHTML(renderCache, "sniperWalletList", sniperWalletList, markup);
      } else {
        sniperWalletList.innerHTML = markup;
      }
    }

    function renderUI() {
      applyStateToForm();
      renderButtonState();
      if (sniperSave) sniperSave.disabled = validateState().length > 0;
      if (sniperModalCard) {
        sniperModalCard.classList.toggle("is-expanded", sniperState.enabled);
      }
      renderWalletList();
      if (typeof onStateChange === "function") onStateChange();
    }

    function showModal() {
      setModalError("");
      const appBootstrapState = getAppBootstrapState();
      if (!appBootstrapState.walletsLoaded) {
        metaNode.textContent = "Wallet balances are still loading.";
      }
      renderUI();
      sniperModalOverlayPointerDown = false;
      if (sniperModal) sniperModal.hidden = false;
    }

    function hideModal() {
      sniperModalOverlayPointerDown = false;
      if (sniperModal) sniperModal.hidden = true;
    }

    function resetState() {
      sniperState = {
        enabled: false,
        wallets: {},
      };
      applyStateToForm();
      renderUI();
    }

    function validateState() {
      if (!sniperState.enabled) return [];
      const wallets = getSelectedEntries();
      if (wallets.length === 0) return ["Select at least one sniper wallet."];
      const errors = [];
      const sniperAutosellMasterEnabled = isNamedChecked("automaticSniperSellEnabled");
      wallets.forEach((entry) => {
        const amount = Number(entry.amountSol);
        if (!entry.amountSol || !Number.isFinite(amount) || amount <= 0) {
          errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} needs a positive buy amount.`);
        }
        const delayMs = Number(entry.submitDelayMs || 0);
        if (entry.targetBlockOffset != null) {
          const blockOffset = Number(entry.targetBlockOffset);
          if (!Number.isFinite(blockOffset) || blockOffset < 0 || blockOffset > SNIPER_BUY_MAX_BLOCK_OFFSET) {
            errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} needs block offset 0-${SNIPER_BUY_MAX_BLOCK_OFFSET}.`);
          }
        } else if (!Number.isFinite(delayMs) || delayMs < 0 || delayMs > 1500) {
          errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} needs delay 0-1500ms.`);
        }
        if (!sniperAutosellMasterEnabled || !entry.sellEnabled) return;
        const sellPercent = Number(entry.sellPercent || 0);
        if (!Number.isFinite(sellPercent) || sellPercent <= 0 || sellPercent > 100) {
          errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} autosell needs sell % 1-100.`);
          return;
        }
        if (entry.sellTriggerMode === "market-cap") {
          const threshold = String(entry.sellMarketCapThreshold || "").trim();
          if (!threshold) {
            errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} autosell market cap is required.`);
            return;
          }
          const timeoutRaw = String(entry.sellMarketCapTimeoutSeconds || "").trim();
          if (!timeoutRaw) {
            errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} autosell timeout is required.`);
            return;
          }
          const timeoutSeconds = Number(timeoutRaw);
          if (!Number.isFinite(timeoutSeconds) || timeoutSeconds < 1 || timeoutSeconds > 86400) {
            errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} autosell timeout must be 1-86400 sec.`);
            return;
          }
        } else {
          const sellBlockOffset = Number(entry.sellTargetBlockOffset);
          if (!Number.isFinite(sellBlockOffset) || sellBlockOffset < 0 || sellBlockOffset > SNIPER_SELL_MAX_BLOCK_OFFSET) {
            errors.push(`Sniper wallet #${walletIndexFromEnvKey(entry.envKey)} autosell after-buy slot offset must be 0-${SNIPER_SELL_MAX_BLOCK_OFFSET}.`);
          }
        }
      });
      return errors;
    }

    function bindEvents() {
      if (eventsBound) return;
      eventsBound = true;

      if (modeSniperButton) {
        modeSniperButton.addEventListener("click", () => {
          showModal();
        });
      }
      if (sniperEnabledToggle) {
        sniperEnabledToggle.addEventListener("change", () => {
          sniperState.enabled = sniperEnabledToggle.checked;
          setModalError("");
          renderUI();
        });
      }
      if (sniperWalletList) {
        sniperWalletList.addEventListener("change", (event) => {
          const checkbox = event.target.closest("[data-sniper-wallet-checkbox]");
          if (!checkbox) return;
          const envKey = checkbox.getAttribute("data-sniper-wallet-checkbox");
          if (!envKey) return;
          const currentState = normalizeWalletState(sniperState.wallets[envKey] || {});
          sniperState.wallets[envKey] = {
            ...currentState,
            selected: checkbox.checked,
          };
          setModalError("");
          renderUI();
        });
        sniperWalletList.addEventListener("input", (event) => {
          const amountInput = event.target.closest("[data-sniper-wallet-amount]");
          if (!amountInput) return;
          const envKey = amountInput.getAttribute("data-sniper-wallet-amount");
          if (!envKey) return;
          const normalized = normalizeDecimalInput(amountInput.value);
          amountInput.value = normalized;
          sniperState.wallets[envKey] = {
            ...normalizeWalletState(sniperState.wallets[envKey] || {}),
            selected: true,
            amountSol: normalized,
            triggerMode: normalizeTriggerMode((sniperState.wallets[envKey] && sniperState.wallets[envKey].triggerMode) || "block-offset"),
          };
          applyStateToForm();
          renderButtonState();
          setModalError("");
        });
        sniperWalletList.addEventListener("click", (event) => {
          const presetButton = event.target.closest("[data-sniper-preset]");
          if (!presetButton) return;
          const envKey = presetButton.getAttribute("data-sniper-preset");
          const ratio = Number(presetButton.getAttribute("data-sniper-ratio") || 0);
          const latestWalletStatus = getLatestWalletStatus();
          const wallet = latestWalletStatus && Array.isArray(latestWalletStatus.wallets)
            ? latestWalletStatus.wallets.find((entry) => entry.envKey === envKey)
            : null;
          if (!envKey || !wallet || !Number.isFinite(ratio)) return;
          const spendableBalance = getSpendableBalanceSol(wallet);
          if (spendableBalance == null) return;
          const amount = normalizeDecimalInput(String(floorDecimal(spendableBalance * ratio, 6)));
          sniperState.wallets[envKey] = {
            ...normalizeWalletState(sniperState.wallets[envKey] || {}),
            selected: true,
            amountSol: amount,
            triggerMode: normalizeTriggerMode((sniperState.wallets[envKey] && sniperState.wallets[envKey].triggerMode) || "block-offset"),
          };
          setModalError("");
          renderUI();
        });
        sniperWalletList.addEventListener("input", (event) => {
          const delayInput = event.target.closest("[data-sniper-wallet-delay]");
          if (!delayInput) return;
          const envKey = delayInput.getAttribute("data-sniper-wallet-delay");
          if (!envKey) return;
          const normalizedDelayMs = normalizeDelayMs(delayInput.value);
          sniperState.wallets[envKey] = {
            ...normalizeWalletState(sniperState.wallets[envKey] || {}),
            selected: true,
            triggerMode: "on-submit",
            submitDelayMs: normalizedDelayMs,
          };
          applyStateToForm();
          setModalError("");
          const valueLabel = delayInput
            .closest(".sniper-delay-slider-block")
            ?.querySelector(".auto-sell-slider-head strong");
          if (valueLabel) valueLabel.textContent = `${normalizedDelayMs}ms`;
        });
        sniperWalletList.addEventListener("change", (event) => {
          const delayInput = event.target.closest("[data-sniper-wallet-delay]");
          if (!delayInput) return;
          const envKey = delayInput.getAttribute("data-sniper-wallet-delay");
          if (!envKey) return;
          sniperState.wallets[envKey] = {
            ...normalizeWalletState(sniperState.wallets[envKey] || {}),
            selected: true,
            triggerMode: "on-submit",
            submitDelayMs: normalizeDelayMs(delayInput.value),
          };
          applyStateToForm();
          setModalError("");
          renderUI();
        });
        sniperWalletList.addEventListener("click", (event) => {
          const retryButton = event.target.closest("[data-sniper-wallet-retry]");
          if (retryButton) {
            event.preventDefault();
            const envKey = retryButton.getAttribute("data-sniper-wallet-retry");
            if (!envKey) return;
            const currentState = normalizeWalletState(sniperState.wallets[envKey] || {});
            sniperState.wallets[envKey] = {
              ...currentState,
              retryOnce: !currentState.retryOnce,
            };
            setModalError("");
            renderUI();
            return;
          }
          const sellToggleButton = event.target.closest("[data-sniper-sell-toggle]");
          if (sellToggleButton) {
            event.preventDefault();
            const envKey = sellToggleButton.getAttribute("data-sniper-sell-toggle");
            if (!envKey) return;
            const currentState = normalizeWalletState(sniperState.wallets[envKey] || {});
            sniperState.wallets[envKey] = {
              ...currentState,
              selected: true,
              sellEnabled: !currentState.sellEnabled,
              sellPercent: currentState.sellPercent || "100",
            };
            setModalError("");
            renderUI();
            return;
          }
          const triggerButton = event.target.closest("[data-sniper-trigger-mode]");
          if (triggerButton) {
            const envKey = triggerButton.getAttribute("data-sniper-trigger-mode");
            const mode = triggerButton.getAttribute("data-sniper-trigger-value");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              triggerMode: normalizeTriggerMode(mode),
            };
            setModalError("");
            renderUI();
            return;
          }
          const sellTriggerButton = event.target.closest("[data-sniper-sell-trigger-mode]");
          if (sellTriggerButton) {
            const envKey = sellTriggerButton.getAttribute("data-sniper-sell-trigger-mode");
            const mode = sellTriggerButton.getAttribute("data-sniper-sell-trigger-value");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              sellEnabled: true,
              sellTriggerMode: normalizeSellTriggerMode(mode),
            };
            setModalError("");
            renderUI();
            return;
          }
          const blockButton = event.target.closest("[data-sniper-block-offset]");
          if (blockButton) {
            const envKey = blockButton.getAttribute("data-sniper-block-offset");
            const value = blockButton.getAttribute("data-sniper-block-value");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              triggerMode: "block-offset",
              targetBlockOffset: normalizeBuyBlockOffset(value),
            };
            setModalError("");
            renderUI();
          }
          const sellBlockButton = event.target.closest("[data-sniper-sell-block-offset]");
          if (sellBlockButton) {
            const envKey = sellBlockButton.getAttribute("data-sniper-sell-block-offset");
            const value = sellBlockButton.getAttribute("data-sniper-sell-block-value");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              sellEnabled: true,
              sellTriggerMode: "block-offset",
              sellTargetBlockOffset: normalizeSellBlockOffset(value),
            };
            setModalError("");
            renderUI();
          }
        });
        sniperWalletList.addEventListener("input", (event) => {
          const sellPercentInput = event.target.closest("[data-sniper-sell-percent]");
          if (sellPercentInput) {
            const envKey = sellPercentInput.getAttribute("data-sniper-sell-percent");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              sellEnabled: true,
              sellPercent: normalizeSellPercent(sellPercentInput.value),
            };
            applyStateToForm();
            setModalError("");
            return;
          }
          const sellMarketThresholdInput = event.target.closest("[data-sniper-sell-market-cap-threshold]");
          if (sellMarketThresholdInput) {
            const envKey = sellMarketThresholdInput.getAttribute("data-sniper-sell-market-cap-threshold");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              sellEnabled: true,
              sellTriggerMode: "market-cap",
              sellMarketCapThreshold: String(sellMarketThresholdInput.value || "").trim(),
            };
            applyStateToForm();
            setModalError("");
            return;
          }
          const sellMarketTimeoutInput = event.target.closest("[data-sniper-sell-market-cap-timeout]");
          if (sellMarketTimeoutInput) {
            const envKey = sellMarketTimeoutInput.getAttribute("data-sniper-sell-market-cap-timeout");
            if (!envKey) return;
            sniperState.wallets[envKey] = {
              ...normalizeWalletState(sniperState.wallets[envKey] || {}),
              selected: true,
              sellEnabled: true,
              sellTriggerMode: "market-cap",
              sellMarketCapTimeoutSeconds: normalizeMarketCapTimeoutSeconds(sellMarketTimeoutInput.value),
            };
            applyStateToForm();
            setModalError("");
          }
        });
        sniperWalletList.addEventListener("change", (event) => {
          const sellMarketTimeoutAction = event.target.closest("[data-sniper-sell-market-cap-timeout-action]");
          if (!sellMarketTimeoutAction) return;
          const envKey = sellMarketTimeoutAction.getAttribute("data-sniper-sell-market-cap-timeout-action");
          if (!envKey) return;
          sniperState.wallets[envKey] = {
            ...normalizeWalletState(sniperState.wallets[envKey] || {}),
            selected: true,
            sellEnabled: true,
            sellTriggerMode: "market-cap",
            sellMarketCapTimeoutAction: normalizeMarketCapTimeoutAction(sellMarketTimeoutAction.value),
          };
          applyStateToForm();
          setModalError("");
          renderUI();
        });
      }
      if (sniperRefreshButton) {
        sniperRefreshButton.addEventListener("click", async () => {
          setModalError("");
          try {
            await refreshWalletStatus(true);
          } catch (error) {
            setModalError(error.message || "Failed to refresh balances.");
          }
        });
      }
      if (sniperResetButton) {
        sniperResetButton.addEventListener("click", () => {
          setModalError("");
          resetState();
        });
      }
      if (sniperSave) {
        sniperSave.addEventListener("click", () => {
          const errors = validateState();
          if (errors.length > 0) {
            setModalError(errors[0]);
            return;
          }
          setModalError("");
          hideModal();
        });
      }
      if (sniperClose) sniperClose.addEventListener("click", hideModal);
      if (sniperCancel) sniperCancel.addEventListener("click", hideModal);
      if (sniperModal) {
        sniperModal.addEventListener("pointerdown", (event) => {
          sniperModalOverlayPointerDown = event.target === sniperModal;
        });
        sniperModal.addEventListener("click", (event) => {
          if (event.target !== sniperModal || !sniperModalOverlayPointerDown) {
            sniperModalOverlayPointerDown = false;
            return;
          }
          sniperModalOverlayPointerDown = false;
          const selection = typeof window.getSelection === "function" ? window.getSelection() : null;
          if (selection && !selection.isCollapsed && String(selection).trim()) return;
          hideModal();
        });
      }
    }

    return {
      bindEvents,
      getState() {
        return normalizeDraftState(sniperState);
      },
      setState(value) {
        sniperState = normalizeDraftState(value);
      },
      normalizeDraftState,
      getStoredDraft,
      getTriggerSummary,
      renderUI,
      showModal,
      hideModal,
      validateState,
      isSaveDisabled() {
        return validateState().length > 0;
      },
      applyStateToForm,
      setModalError,
      resetState,
    };
  }

  global.SniperFeature = {
    create: createSniperFeature,
  };
})(window);
