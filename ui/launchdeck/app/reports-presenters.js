(function initLaunchDeckReportsPresenters(global) {
  function createReportsPresenters(config) {
    const {
      elements,
      renderCache,
      setCachedHTML,
      getState,
      getFollowJobsState,
      getFollowStatusSnapshot,
      syncFollowStatusChrome,
      activeFollowJobForTraceId,
      isTerminalFollowJobState,
      escapeHTML,
      shortenAddress,
      shortAddress,
      walletIndexFromEnvKey,
      formatWalletHistoryLabel,
      getQuoteAssetLabel,
      getDevBuyAssetLabel,
      getSniperTriggerSummary,
      providerLabels,
    } = config;

    const {
      launchSurfaceCard,
      outputSection,
      reportsTerminalSection,
      reportsTerminalList,
      reportsTerminalOutput,
      reportsTerminalMeta,
      reportsTransactionsButton,
      reportsLaunchesButton,
      reportsActiveJobsButton,
      reportsActiveLogsButton,
      benchmarksPopoutModal,
      benchmarksPopoutTitle,
      benchmarksPopoutBody,
    } = elements;

    function reportsState() {
      return getState();
    }

    function followJobsState() {
      return getFollowJobsState();
    }

    function writeCachedHTML(cacheKey, node, markup) {
      if (!node) return;
      if (typeof setCachedHTML === "function") {
        setCachedHTML(renderCache, cacheKey, node, markup);
        return;
      }
      node.innerHTML = markup;
    }

    function normalizeReportsTerminalView(view) {
      const normalized = String(view || "").trim().toLowerCase();
      if (normalized === "launches") return "launches";
      if (normalized === "active-jobs") return "active-jobs";
      if (normalized === "active-logs") return "active-logs";
      return "transactions";
    }

    function normalizeActiveLogsView(view) {
      return String(view || "").trim().toLowerCase() === "errors" ? "errors" : "live";
    }

    function reportsTerminalMetaText(view) {
      const state = reportsState();
      const normalized = normalizeReportsTerminalView(view == null ? state.view : view);
      if (normalized === "launches") return "Latest 25 launches.";
      if (normalized === "active-jobs") {
        const snapshot = getFollowStatusSnapshot();
        if (snapshot.offline) return "Follow daemon offline.";
        if (!snapshot.configured) return "Follow daemon disabled.";
        if (snapshot.counts.active > 0) {
          return `${snapshot.counts.active} live follow job${snapshot.counts.active === 1 ? "" : "s"}.`;
        }
        return "Live follow-daemon jobs.";
      }
      if (normalized === "active-logs") {
        return normalizeActiveLogsView(state.activeLogsView) === "errors"
          ? "Persisted historic backend errors."
          : "Latest 100 in-memory backend activity logs.";
      }
      return "Latest 25 transactions.";
    }

    function syncReportsTerminalChrome() {
      const state = reportsState();
      const view = normalizeReportsTerminalView(state.view);
      state.view = view;
      if (reportsTerminalSection) {
        reportsTerminalSection.classList.toggle("is-launches-view", view === "launches");
        reportsTerminalSection.classList.toggle("is-active-jobs-view", view === "active-jobs");
        reportsTerminalSection.classList.toggle("is-active-logs-view", view === "active-logs");
      }
      if (reportsTransactionsButton) reportsTransactionsButton.classList.toggle("active", view === "transactions");
      if (reportsLaunchesButton) reportsLaunchesButton.classList.toggle("active", view === "launches");
      if (reportsActiveJobsButton) reportsActiveJobsButton.classList.toggle("active", view === "active-jobs");
      if (reportsActiveLogsButton) reportsActiveLogsButton.classList.toggle("active", view === "active-logs");
      if (reportsTerminalMeta) reportsTerminalMeta.textContent = reportsTerminalMetaText(view);
      syncReportsTerminalLayoutHeight();
      if (typeof syncFollowStatusChrome === "function") syncFollowStatusChrome();
    }

    function syncReportsTerminalLayoutHeight() {
      if (!reportsTerminalSection || !launchSurfaceCard) return;
      const launchSurfaceHeight = Math.round(launchSurfaceCard.getBoundingClientRect().height || 0);
      const outputVisible = Boolean(outputSection && !outputSection.hidden);
      const outputHeight = outputVisible
        ? Math.round(outputSection.getBoundingClientRect().height || 0)
        : 0;
      const measuredHeight = Math.max(0, launchSurfaceHeight - outputHeight);
      if (measuredHeight <= 0) {
        reportsTerminalSection.style.removeProperty("--reports-terminal-match-height");
        return;
      }
      reportsTerminalSection.style.setProperty("--reports-terminal-match-height", `${measuredHeight}px`);
    }

    function describeReportEntry(entry) {
      const parts = [];
      if (entry.displayTime) parts.push(entry.displayTime);
      if (entry.provider) parts.push(entry.provider);
      if (entry.signatureCount) parts.push(`${entry.signatureCount} sig${entry.signatureCount === 1 ? "" : "s"}`);
      if (entry.followEnabled) {
        const followBits = [];
        if (entry.followState) followBits.push(`follow ${entry.followState}`);
        if (entry.followActionCount) followBits.push(`${entry.followConfirmedCount || 0}/${entry.followActionCount} done`);
        if (entry.followProblemCount) followBits.push(`${entry.followProblemCount} issue${entry.followProblemCount === 1 ? "" : "s"}`);
        if (followBits.length) parts.push(followBits.join(" | "));
      }
      return parts.join(" | ");
    }

    function cloneReportValue(value) {
      if (value == null) return value;
      try {
        return JSON.parse(JSON.stringify(value));
      } catch (_error) {
        return value;
      }
    }

    function hasReportObjectFields(value) {
      return Boolean(value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).length);
    }

    function reportBenchmarkModeFromPayload(report) {
      if (!report || typeof report !== "object") return "";
      const benchmarkMode = report.benchmark
        && typeof report.benchmark === "object"
        && typeof report.benchmark.mode === "string"
        ? report.benchmark.mode
        : "";
      if (benchmarkMode) return String(benchmarkMode).trim().toLowerCase();
      const timingsMode = report.execution
        && typeof report.execution === "object"
        && report.execution.timings
        && typeof report.execution.timings === "object"
        && typeof report.execution.timings.benchmarkMode === "string"
        ? report.execution.timings.benchmarkMode
        : "";
      return String(timingsMode || "").trim().toLowerCase();
    }

    function captureFrozenBenchmarkSnapshot(reportId, payload) {
      const state = reportsState();
      const normalizedId = String(reportId || "").trim();
      if (!normalizedId || !payload || typeof payload !== "object") return;
      const report = payload.report && typeof payload.report === "object" ? payload.report : null;
      if (!report) return;
      const mode = reportBenchmarkModeFromPayload(report);
      const benchmark = report.benchmark && typeof report.benchmark === "object" ? report.benchmark : null;
      const executionTimings = report.execution && report.execution.timings && typeof report.execution.timings === "object"
        ? report.execution.timings
        : null;
      if (mode === "off") {
        state.activeBenchmarkReportId = normalizedId;
        state.activeBenchmarkSnapshot = {
          benchmark: cloneReportValue(benchmark),
          executionTimings: null,
        };
        return;
      }
      const hasBenchmark = hasReportObjectFields(benchmark);
      const hasExecutionTimings = hasReportObjectFields(executionTimings);
      if (!hasBenchmark && !hasExecutionTimings) return;
      const previous = state.activeBenchmarkReportId === normalizedId
        ? state.activeBenchmarkSnapshot
        : null;
      state.activeBenchmarkReportId = normalizedId;
      state.activeBenchmarkSnapshot = {
        benchmark: hasBenchmark ? cloneReportValue(benchmark) : cloneReportValue(previous && previous.benchmark),
        executionTimings: hasExecutionTimings
          ? cloneReportValue(executionTimings)
          : cloneReportValue(previous && previous.executionTimings),
      };
    }

    function applyFrozenBenchmarkSnapshot(reportId, payload) {
      const state = reportsState();
      const normalizedId = String(reportId || "").trim();
      if (!normalizedId || !payload || typeof payload !== "object") return payload;
      if (state.activeBenchmarkReportId !== normalizedId) return payload;
      const snapshot = state.activeBenchmarkSnapshot;
      if (!snapshot) return payload;
      const report = payload.report && typeof payload.report === "object" ? payload.report : null;
      if (!report) return payload;
      const mode = reportBenchmarkModeFromPayload(report);
      if (mode === "off") return payload;
      const nextPayload = cloneReportValue(payload);
      const nextReport = nextPayload.report && typeof nextPayload.report === "object" ? nextPayload.report : null;
      if (!nextReport) return payload;
      if (!hasReportObjectFields(nextReport.benchmark) && hasReportObjectFields(snapshot.benchmark)) {
        nextReport.benchmark = cloneReportValue(snapshot.benchmark);
      }
      if (nextReport.execution && typeof nextReport.execution === "object") {
        if (
          !hasReportObjectFields(nextReport.execution.timings)
          && hasReportObjectFields(snapshot.executionTimings)
        ) {
          nextReport.execution.timings = cloneReportValue(snapshot.executionTimings);
        }
      }
      return nextPayload;
    }

    function normalizeReportsTerminalTab(tab) {
      const normalized = String(tab || "").trim().toLowerCase();
      return ["overview", "actions", "benchmarks", "raw"].includes(normalized) ? normalized : "overview";
    }

    function currentReportsTerminalEntry() {
      const state = reportsState();
      return state.entries.find((entry) => entry.id === state.activeId) || null;
    }

    function currentReportsTerminalPayload() {
      const state = reportsState();
      return state.activePayload && typeof state.activePayload === "object"
        ? state.activePayload
        : null;
    }

    function currentReportsTerminalReport() {
      const payload = currentReportsTerminalPayload();
      return payload && payload.report && typeof payload.report === "object" ? payload.report : null;
    }

    function currentReportsTerminalFollowJob() {
      const report = currentReportsTerminalReport();
      return report && report.followDaemon && report.followDaemon.job && typeof report.followDaemon.job === "object"
        ? report.followDaemon.job
        : null;
    }

    function currentReportsTerminalFollowActions() {
      const job = currentReportsTerminalFollowJob();
      return job && Array.isArray(job.actions) ? job.actions : [];
    }

    function currentReportsTerminalBenchmark() {
      const report = currentReportsTerminalReport();
      return report && report.benchmark && typeof report.benchmark === "object" ? report.benchmark : null;
    }

    function currentReportsTerminalExecution() {
      const report = currentReportsTerminalReport();
      return report && report.execution && typeof report.execution === "object" ? report.execution : null;
    }

    function formatReportMetric(value, suffix = "", fallback = "--", digits = 0) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric)) return fallback;
      return `${numeric.toFixed(digits)}${suffix}`;
    }

    function parseReportMetricNumber(value) {
      const numeric = Number(value);
      return Number.isFinite(numeric) ? numeric : null;
    }

    function readReportSlotValue(source, ...keys) {
      if (!source || typeof source !== "object") return null;
      for (let index = 0; index < keys.length; index += 1) {
        const numeric = parseReportMetricNumber(source[keys[index]]);
        if (numeric != null) return numeric;
      }
      return null;
    }

    function formatReportSlotValue(value, prefix = "") {
      return value != null ? `${prefix}${String(value)}` : "--";
    }

    function normalizeReportStatusValue(status) {
      return String(status || "").trim().toLowerCase();
    }

    function formatConfirmationSourceLabel(source) {
      const normalized = String(source || "").trim().toLowerCase();
      if (!normalized) return "--";
      if (normalized === "helius-transaction-subscribe") return "Helius transactionSubscribe";
      if (normalized === "websocket") return "Standard websocket";
      if (normalized === "rpc-polling") return "RPC polling";
      if (normalized === "rpc-polling-batch") return "RPC polling batch";
      if (normalized === "jito-bundle-status") return "Jito bundle status";
      return String(source || "").trim();
    }

    function isLandedReportStatus(status) {
      const normalized = normalizeReportStatusValue(status);
      return ["confirmed", "finalized", "success", "succeeded", "landed"].includes(normalized);
    }

    function formatLandedValue(status) {
      const normalized = normalizeReportStatusValue(status);
      if (!normalized) return "--";
      return isLandedReportStatus(status) ? "Yes" : "No";
    }

    function computeObservedSlotsToConfirm(item) {
      const direct = readReportSlotValue(item, "slotsToConfirm", "blocksToConfirm");
      if (direct != null) return direct;
      const sendSlot = readReportSlotValue(item, "sendSlot", "sendBlockHeight", "sendObservedSlot", "sendObservedBlockHeight");
      const observedConfirmSlot = readReportSlotValue(item, "confirmedObservedSlot", "confirmedBlockHeight", "confirmedObservedBlockHeight");
      if (sendSlot == null || observedConfirmSlot == null) return null;
      return Math.max(0, observedConfirmSlot - sendSlot);
    }

    function summarizeLaunchStatuses(items = []) {
      const normalized = Array.isArray(items) ? items.filter(Boolean) : [];
      if (!normalized.length) return null;
      const labels = normalized
        .map((item) => String(item && item.confirmationStatus || "").trim())
        .filter(Boolean);
      if (!labels.length) return null;
      const uniqueLabels = Array.from(new Set(labels));
      const landedCount = normalized.filter((item) => isLandedReportStatus(item && item.confirmationStatus)).length;
      return {
        value: uniqueLabels.length === 1 ? uniqueLabels[0] : `${landedCount}/${normalized.length} landed`,
        detail: `${landedCount}/${normalized.length} landed`,
      };
    }

    function summarizeLaunchConfirmationSources(items = []) {
      const sources = Array.isArray(items)
        ? items
          .map((item) => formatConfirmationSourceLabel(item && item.confirmationSource))
          .filter((value) => value && value !== "--")
        : [];
      if (!sources.length) return null;
      const uniqueSources = Array.from(new Set(sources));
      return {
        value: uniqueSources[0] || "--",
        detail: uniqueSources.length > 1 ? uniqueSources.join(" | ") : "",
      };
    }

    function summarizeLaunchObservedSlots(items = []) {
      const normalized = Array.isArray(items) ? items.filter(Boolean) : [];
      if (!normalized.length) return null;
      const parts = normalized
        .map((item) => {
          const slots = computeObservedSlotsToConfirm(item);
          if (slots == null) return "";
          const label = formatLaunchTransactionLabel(item && item.label || "tx");
          return `${label} ${slots}`;
        })
        .filter(Boolean);
      if (!parts.length) return null;
      return {
        value: parts.length === 1 ? parts[0].split(" ").slice(-1)[0] : `${parts.length} tx`,
        detail: parts.join(" | "),
      };
    }

    function resolveBenchmarkSentItems(benchmark = {}, execution = {}) {
      const benchmarkSent = Array.isArray(benchmark && benchmark.sent) ? benchmark.sent : [];
      const executionSent = Array.isArray(execution && execution.sent) ? execution.sent : [];
      if (!benchmarkSent.length) return executionSent;
      const usedExecutionIndexes = new Set();
      const merged = benchmarkSent.map((item, benchmarkIndex) => {
        const itemLabel = String(item && item.label || "").trim().toLowerCase();
        const itemSignature = String(item && item.signature || "").trim();
        let matchedIndex = executionSent.findIndex((candidate, candidateIndex) => {
          if (usedExecutionIndexes.has(candidateIndex) || !candidate) return false;
          const candidateSignature = String(candidate.signature || "").trim();
          if (itemSignature && candidateSignature && candidateSignature === itemSignature) return true;
          return itemLabel && String(candidate.label || "").trim().toLowerCase() === itemLabel;
        });
        if (matchedIndex < 0 && executionSent[benchmarkIndex] && !usedExecutionIndexes.has(benchmarkIndex)) {
          matchedIndex = benchmarkIndex;
        }
        const matched = matchedIndex >= 0 ? executionSent[matchedIndex] : null;
        if (matchedIndex >= 0) usedExecutionIndexes.add(matchedIndex);
        return {
          ...(matched || {}),
          ...(item || {}),
          attemptedEndpoints: Array.isArray(item && item.attemptedEndpoints) && item.attemptedEndpoints.length
            ? item.attemptedEndpoints
            : (matched && matched.attemptedEndpoints) || [],
          attemptedBundleIds: Array.isArray(item && item.attemptedBundleIds) && item.attemptedBundleIds.length
            ? item.attemptedBundleIds
            : (matched && matched.attemptedBundleIds) || [],
        };
      });
      executionSent.forEach((item, index) => {
        if (!usedExecutionIndexes.has(index)) merged.push(item);
      });
      return merged;
    }

    function buildTimingMetricItem(label, value, detail = "", { hideZero = false, tone = "" } = {}) {
      const numeric = parseReportMetricNumber(value);
      if (numeric == null || (hideZero && numeric === 0)) return null;
      return {
        label,
        value: formatReportMetric(numeric, "ms"),
        detail,
        tone,
      };
    }

    function deriveRemainingTiming(totalValue, childValues = []) {
      const total = parseReportMetricNumber(totalValue);
      if (total == null) return null;
      let hasChild = false;
      const consumed = childValues.reduce((sum, value) => {
        const numeric = parseReportMetricNumber(value);
        if (numeric == null) return sum;
        hasChild = true;
        return sum + numeric;
      }, 0);
      if (!hasChild) return null;
      return Math.max(0, total - consumed);
    }

    function buildLegacyBenchmarkTimingSections(timings = {}) {
      const compileTotal = parseReportMetricNumber(timings.compileTransactionsMs);
      const compileAltLoad = parseReportMetricNumber(timings.compileAltLoadMs);
      const compileBlockhash = parseReportMetricNumber(timings.compileBlockhashFetchMs);
      const compileGlobal = parseReportMetricNumber(timings.compileGlobalFetchMs);
      const compileFollowUp = parseReportMetricNumber(timings.compileFollowUpPrepMs);
      const compileSerialize = parseReportMetricNumber(timings.compileTxSerializeMs);
      const bagsPrepare = parseReportMetricNumber(timings.bagsPrepareLaunchMs);
      const bagsMetadataUpload = parseReportMetricNumber(timings.bagsMetadataUploadMs);
      const bagsFeeRecipientResolve = parseReportMetricNumber(timings.bagsFeeRecipientResolveMs);
      const compileOther = deriveRemainingTiming(compileTotal, [
        compileAltLoad,
        compileBlockhash,
        compileGlobal,
        compileFollowUp,
        compileSerialize,
        bagsPrepare,
        bagsMetadataUpload,
        bagsFeeRecipientResolve,
      ]);

      const sendTotal = parseReportMetricNumber(timings.sendMs);
      const submitTotal = parseReportMetricNumber(timings.sendSubmitMs);
      const confirmTotal = parseReportMetricNumber(timings.sendConfirmMs);
      const bagsSetupSubmit = parseReportMetricNumber(timings.bagsSetupSubmitMs);
      const bagsSetupGate = parseReportMetricNumber(timings.bagsSetupGateMs != null ? timings.bagsSetupGateMs : timings.bagsSetupConfirmMs);
      const sendOther = deriveRemainingTiming(sendTotal, [submitTotal, confirmTotal]);

      return {
        topLevel: [
          buildTimingMetricItem("End-to-end", timings.totalElapsedMs, "client + backend"),
          buildTimingMetricItem("Client overhead", timings.clientPreRequestMs, "before engine work starts"),
          buildTimingMetricItem("Backend total", timings.backendTotalElapsedMs, "all engine work"),
          buildTimingMetricItem("Compile total", timings.compileTransactionsMs, "inclusive stage total"),
          buildTimingMetricItem("Send total", timings.sendMs, "inclusive of submit + confirm"),
          buildTimingMetricItem("Persist report", timings.persistReportMs, "final report write"),
        ],
        prep: [
          buildTimingMetricItem("Form -> Raw", timings.formToRawConfigMs, "UI payload to engine config"),
          buildTimingMetricItem("Normalize", timings.normalizeConfigMs, "config validation + normalization"),
          buildTimingMetricItem("Wallet load", timings.walletLoadMs, "wallet/env hydration"),
          buildTimingMetricItem("Report build", timings.reportBuildMs, "initial report assembly"),
        ],
        compile: [
          buildTimingMetricItem("Compile total", timings.compileTransactionsMs, "inclusive stage total"),
          buildTimingMetricItem("ALT load", timings.compileAltLoadMs, "lookup table fetch"),
          buildTimingMetricItem("Blockhash", timings.compileBlockhashFetchMs, "latest blockhash fetch"),
          buildTimingMetricItem("Global fetch", timings.compileGlobalFetchMs, "shared launch context"),
          buildTimingMetricItem("Follow-up prep", timings.compileFollowUpPrepMs, "follow action planning"),
          buildTimingMetricItem("Serialize tx", timings.compileTxSerializeMs, "tx serialization only"),
          buildTimingMetricItem("Bags prepare", timings.bagsPrepareLaunchMs, "helper prepare-launch total", { hideZero: true }),
          buildTimingMetricItem("Bags metadata upload", timings.bagsMetadataUploadMs, "helper metadata upload only", { hideZero: true }),
          buildTimingMetricItem("Bags recipient resolve", timings.bagsFeeRecipientResolveMs, "helper fee-recipient resolution", { hideZero: true }),
          buildTimingMetricItem("Compile other", compileOther, "remaining compile work", { hideZero: true }),
        ],
        send: [
          buildTimingMetricItem("Send total", timings.sendMs, "inclusive stage total"),
          buildTimingMetricItem("Submit total", timings.sendSubmitMs, "all transaction submissions"),
          buildTimingMetricItem("Confirm total", timings.sendConfirmMs, "all confirmation waits"),
          buildTimingMetricItem("Setup submit", timings.bagsSetupSubmitMs, "setup tx submit", { hideZero: true }),
          buildTimingMetricItem("Launch build", timings.bagsLaunchBuildMs, "build final launch tx", { hideZero: true }),
          buildTimingMetricItem("Setup gate", bagsSetupGate, "wait before final launch build", { hideZero: true }),
          buildTimingMetricItem("Transport submit", timings.sendTransportSubmitMs, "launch tx provider submit", { hideZero: true }),
          buildTimingMetricItem("Transport confirm", timings.sendTransportConfirmMs, "launch tx confirmation only", { hideZero: true }),
          buildTimingMetricItem("Send other", sendOther, "remaining transport overhead", { hideZero: true }),
        ],
      };
    }

    function benchmarkModeLabel(mode) {
      const normalized = String(mode || "").trim().toLowerCase();
      if (!normalized) return "";
      if (normalized === "off") return "Off";
      if (normalized === "light" || normalized === "basic") return "Light";
      if (normalized === "full") return "Full";
      return String(mode || "").trim();
    }

    function benchmarkMetricCardFromGroupItem(item) {
      if (!item || typeof item !== "object") return null;
      const numeric = parseReportMetricNumber(item.valueMs != null ? item.valueMs : item.value);
      if (numeric == null) return null;
      const detailParts = [];
      if (item.detail) detailParts.push(String(item.detail));
      if (item.inclusive) detailParts.push("Inclusive total");
      if (item.remainder) detailParts.push("Remainder");
      return {
        label: String(item.label || item.key || "--"),
        value: formatReportMetric(numeric, "ms"),
        detail: detailParts.join(" | "),
      };
    }

    function benchmarkTimingGroupsFromPayload(benchmark = {}, execution = {}) {
      const groups = Array.isArray(benchmark.timingGroups) ? benchmark.timingGroups : [];
      if (groups.length) {
        return groups.map((group) => ({
          key: String(group.key || ""),
          label: String(group.label || group.key || "Timings"),
          items: Array.isArray(group.items) ? group.items.map(benchmarkMetricCardFromGroupItem).filter(Boolean) : [],
        }));
      }
      const timings = benchmark.timings || execution.timings || {};
      const legacy = buildLegacyBenchmarkTimingSections(timings);
      return [
        { key: "topLevel", label: "Top-Level Timings", items: legacy.topLevel },
        { key: "prep", label: "Preparation", items: legacy.prep },
        { key: "compile", label: "Compile Breakdown", items: legacy.compile },
        { key: "send", label: "Send Breakdown", items: legacy.send },
      ];
    }

    function benchmarkTimingGroupByKey(groups, key) {
      return Array.isArray(groups) ? groups.find((group) => group && group.key === key) : null;
    }

    function currentReportsTerminalAutoFee() {
      const execution = currentReportsTerminalExecution();
      return execution && execution.autoFee && typeof execution.autoFee === "object"
        ? execution.autoFee
        : null;
    }

    function formatSolForReport(value) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric)) return "--";
      if (numeric === 0) return "0";
      const fixed = numeric.toFixed(9).replace(/\.?0+$/, "");
      return fixed === "-0" ? "0" : fixed;
    }

    function formatPriorityPriceForReport(value) {
      const numeric = parseReportMetricNumber(value);
      if (numeric == null) return "--";
      const solEquivalent = formatSolForReport(numeric / 1_000_000_000);
      return `${numeric.toLocaleString()} micro-lamports/CU (~${solEquivalent} SOL @ 1M CU)`;
    }

    function formatTipLamportsForReport(value) {
      const numeric = parseReportMetricNumber(value);
      if (numeric == null) return "--";
      const solEquivalent = formatSolForReport(numeric / 1_000_000_000);
      return `${numeric.toLocaleString()} lamports (${solEquivalent} SOL)`;
    }

    function autoFeeActionSignature(action) {
      if (!action || typeof action !== "object" || !action.enabled) return "";
      return JSON.stringify([
        action.provider || "",
        action.prioritySource || "",
        parseReportMetricNumber(action.priorityEstimateLamports),
        parseReportMetricNumber(action.resolvedPriorityLamports),
        action.tipSource || "",
        parseReportMetricNumber(action.tipEstimateLamports),
        parseReportMetricNumber(action.resolvedTipLamports),
        parseReportMetricNumber(action.capLamports),
      ]);
    }

    function groupAutoFeeActions(autoFee) {
      const entries = [
        { label: "Creation", action: autoFee && autoFee.creation },
        { label: "Buy", action: autoFee && autoFee.buy },
        { label: "Sell", action: autoFee && autoFee.sell },
      ].filter((entry) => entry.action && typeof entry.action === "object" && entry.action.enabled);
      const grouped = [];
      const indexesBySignature = new Map();
      entries.forEach((entry) => {
        const signature = autoFeeActionSignature(entry.action);
        if (!signature) return;
        if (indexesBySignature.has(signature)) {
          grouped[indexesBySignature.get(signature)].labels.push(entry.label);
          return;
        }
        indexesBySignature.set(signature, grouped.length);
        grouped.push({ labels: [entry.label], action: entry.action });
      });
      return grouped;
    }

    function buildAutoFeeActionCards(label, action) {
      if (!action || typeof action !== "object" || !action.enabled) return [];
      return [
        { label: `${label} Provider`, value: action.provider || "--" },
        { label: `${label} Priority Source`, value: action.prioritySource || "--", detail: action.priorityEstimateLamports != null ? `${formatPriorityPriceForReport(action.priorityEstimateLamports)} est` : "" },
        { label: `${label} Priority Used`, value: action.resolvedPriorityLamports != null ? formatPriorityPriceForReport(action.resolvedPriorityLamports) : "--" },
        { label: `${label} Tip Source`, value: action.tipSource || "--", detail: action.tipEstimateLamports != null ? `${formatTipLamportsForReport(action.tipEstimateLamports)} est` : "" },
        { label: `${label} Tip Used`, value: action.resolvedTipLamports != null ? formatTipLamportsForReport(action.resolvedTipLamports) : "--" },
        { label: `${label} Max Auto Fee`, value: action.capLamports != null ? formatTipLamportsForReport(action.capLamports) : "--" },
      ];
    }

    function buildAutoFeeBenchmarkSection(autoFee, benchmarkMode) {
      if (!autoFee || benchmarkMode !== "Full") return "";
      const jitoTipPercentile = String(autoFee.jitoTipPercentile || "p99").trim() || "p99";
      const snapshot = autoFee.snapshot && typeof autoFee.snapshot === "object" ? autoFee.snapshot : {};
      const snapshotCards = [
        { label: "Warm Launch Template Estimate", value: snapshot.helius_launch_priority_lamports != null ? formatPriorityPriceForReport(snapshot.helius_launch_priority_lamports) : "--" },
        { label: `Warm Jito ${jitoTipPercentile} Tip`, value: snapshot.jito_tip_p99_lamports != null ? formatTipLamportsForReport(snapshot.jito_tip_p99_lamports) : "--" },
      ].filter((card) => card.value !== "--");
      const actionCards = groupAutoFeeActions(autoFee)
        .flatMap(({ labels, action }) => buildAutoFeeActionCards(labels.join(" / "), action));
      return `
    <section class="reports-panel-section">
      <div class="reports-panel-title">Auto-Fee Sources</div>
      <div class="reports-panel-note">Full benchmark mode only. Shows the final per-action auto-fee values that were actually used.</div>
      ${renderReportMetricGrid(actionCards)}
      ${snapshotCards.length ? `
        <details class="reports-active-log-details">
          <summary>Auto-Fee Debug Snapshot</summary>
          ${renderReportMetricGrid(snapshotCards)}
        </details>
      ` : ""}
    </section>
  `;
    }

    function sumMetricNumbers(values = []) {
      let total = 0;
      let hasAny = false;
      values.forEach((value) => {
        const numeric = parseReportMetricNumber(value);
        if (numeric == null) return;
        total += numeric;
        hasAny = true;
      });
      return hasAny ? total : null;
    }

    function deriveBenchmarkRollup(timings = {}) {
      const totalElapsed = parseReportMetricNumber(timings.totalElapsedMs);
      const clientPreRequest = parseReportMetricNumber(timings.clientPreRequestMs);
      const prepareRequestPayload = parseReportMetricNumber(timings.prepareRequestPayloadMs);
      const backendTotal = parseReportMetricNumber(timings.backendTotalElapsedMs);
      const backendPrep = sumMetricNumbers([
        timings.formToRawConfigMs,
        timings.normalizeConfigMs,
        timings.walletLoadMs,
        timings.reportBuildMs,
      ]);
      const backendOrchestration = sumMetricNumbers([
        timings.transportPlanBuildMs,
        timings.autoFeeResolveMs,
        timings.sameTimeFeeGuardMs,
        timings.followDaemonReadyMs,
        timings.followDaemonReserveMs,
        timings.followDaemonArmMs,
        timings.followDaemonStatusRefreshMs,
      ]);
      const compileTotal = parseReportMetricNumber(timings.compileTransactionsMs);
      const simulateTotal = parseReportMetricNumber(timings.simulateMs);
      const sendTotal = parseReportMetricNumber(timings.sendMs);
      const reportingOverhead = sumMetricNumbers([
        timings.reportingOverheadMs,
      ]) ?? sumMetricNumbers([
        timings.persistInitialSnapshotMs,
        timings.persistFinalReportUpdateMs,
        timings.followSnapshotFlushMs,
        timings.reportRenderMs,
        timings.reportListRefreshMs,
      ]);
      const clientRemainder = deriveRemainingTiming(clientPreRequest, [prepareRequestPayload]);
      const backendMeasured = sumMetricNumbers([
        backendPrep,
        backendOrchestration,
        compileTotal,
        simulateTotal,
        sendTotal,
        reportingOverhead,
      ]);
      const backendRemainder = deriveRemainingTiming(backendTotal, [backendMeasured]);
      const executionDerived = backendTotal != null
        ? Math.max(0, backendTotal - (reportingOverhead || 0))
        : parseReportMetricNumber(timings.executionTotalMs);
      const endToEndRemainder = deriveRemainingTiming(totalElapsed, [clientPreRequest, backendTotal]);
      return {
        totalElapsed,
        clientPreRequest,
        prepareRequestPayload,
        clientRemainder,
        backendTotal,
        backendPrep,
        backendOrchestration,
        compileTotal,
        simulateTotal,
        sendTotal,
        reportingOverhead,
        backendRemainder,
        executionDerived,
        endToEndRemainder,
      };
    }

    function buildBenchmarkHeadlineCards(timings = {}) {
      const rollup = deriveBenchmarkRollup(timings);
      const submitTotal = parseReportMetricNumber(timings.sendSubmitMs);
      const confirmWait = parseReportMetricNumber(timings.sendConfirmMs);
      const bagsSetupGate = parseReportMetricNumber(timings.bagsSetupGateMs != null ? timings.bagsSetupGateMs : timings.bagsSetupConfirmMs);
      const submittedTotal = sumMetricNumbers([
        rollup.clientPreRequest,
        rollup.backendPrep,
        rollup.backendOrchestration,
        rollup.compileTotal,
        rollup.simulateTotal,
        submitTotal,
        bagsSetupGate,
      ]);
      return [
        buildTimingMetricItem("Submitted", submittedTotal, "client + backend through provider acceptance", { tone: "primary" }),
        buildTimingMetricItem("Confirmed", rollup.totalElapsed, "full path including confirmation"),
        buildTimingMetricItem("Confirm wait", confirmWait, "provider/RPC confirmation latency", { tone: "muted" }),
      ].filter(Boolean);
    }

    function buildBenchmarkReconciliationSections(timings = {}, benchmarkMode = "") {
      const rollup = deriveBenchmarkRollup(timings);
      const modeLabel = benchmarkModeLabel(benchmarkMode || timings.benchmarkMode);
      return {
        topLevel: [
          buildTimingMetricItem("End-to-end", rollup.totalElapsed, "top-level wall time for this report"),
          buildTimingMetricItem("Client overhead", rollup.clientPreRequest, "before backend work starts"),
          buildTimingMetricItem("Backend total", rollup.backendTotal, "all backend-observed work"),
          buildTimingMetricItem("End-to-end remainder", rollup.endToEndRemainder, "time not explained by client + backend totals", { hideZero: true }),
        ],
        client: [
          buildTimingMetricItem("Client overhead", rollup.clientPreRequest, "inclusive client-side total"),
          buildTimingMetricItem("Prepare request payload", rollup.prepareRequestPayload, "form serialization before the POST"),
          buildTimingMetricItem("Client remainder", rollup.clientRemainder, "client time not broken into smaller steps yet", { hideZero: true }),
        ],
        backend: [
          buildTimingMetricItem("Backend total", rollup.backendTotal, "inclusive backend total"),
          buildTimingMetricItem("Execution total", rollup.executionDerived, "backend total minus reporting overhead"),
          buildTimingMetricItem("Prep subtotal", rollup.backendPrep, "normalize + wallet + routing + fee/follow setup + initial report"),
          buildTimingMetricItem("Orchestration subtotal", rollup.backendOrchestration, "transport planning, auto-fee work, and follow daemon control calls"),
          buildTimingMetricItem("Compile total", rollup.compileTotal, "inclusive compile stage"),
          buildTimingMetricItem("Simulate total", rollup.simulateTotal, "inclusive simulate stage"),
          buildTimingMetricItem("Send total", rollup.sendTotal, "inclusive send stage"),
          buildTimingMetricItem("Reporting overhead", rollup.reportingOverhead, "persist/render/report-sync work kept out of core execution"),
          buildTimingMetricItem("Backend remainder", rollup.backendRemainder, modeLabel === "Light"
            ? "backend time not broken into smaller steps in light mode"
            : "backend time not yet broken into smaller steps", { hideZero: true }),
        ],
      };
    }

    function formatReportTimestamp(value) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric) || numeric <= 0) return "--";
      try {
        return new Date(numeric).toLocaleTimeString();
      } catch (_error) {
        return String(value);
      }
    }

    function formatReportLatencyDelta(startMs, endMs) {
      const start = Number(startMs);
      const end = Number(endMs);
      if (!Number.isFinite(start) || !Number.isFinite(end) || end < start) return "--";
      return `${Math.round(end - start)}ms`;
    }

    function reportStateClass(state) {
      const normalized = String(state || "").trim().toLowerCase();
      if (["confirmed", "completed", "success", "healthy", "stopped"].includes(normalized)) return "is-good";
      if (["running", "eligible", "armed", "queued", "sent"].includes(normalized)) return "is-warn";
      if (["failed", "cancelled", "expired", "completed-with-failures"].includes(normalized)) return "is-bad";
      return "";
    }

    function inferReportErrorCategory(message) {
      const normalized = String(message || "").toLowerCase();
      if (!normalized) return "";
      if (normalized.includes("os error 1224") || normalized.includes("user-mapped section")) return "local write race";
      if (normalized.includes("insufficient funds")) return "insufficient funds";
      if (normalized.includes("was not found") || normalized.includes("account")) return "account visibility";
      if (normalized.includes("custom") || normalized.includes("instructionerror")) return "on-chain failure";
      if (normalized.includes("timeout") || normalized.includes("too many requests") || normalized.includes("unavailable")) return "transport/rpc";
      return "action failure";
    }

    function formatMarketCapThresholdForDisplay(value) {
      const trimmed = String(value || "").trim();
      if (!/^\d+$/.test(trimmed)) return trimmed;
      try {
        const micros = BigInt(trimmed);
        if (micros < 1000000n) return trimmed;
        const wholeUsd = micros / 1000000n;
        const fractionalMicros = micros % 1000000n;
        const formatWithSuffix = (suffixValue, suffixLabel) => {
          const whole = wholeUsd / suffixValue;
          const remainder = wholeUsd % suffixValue;
          if (whole >= 100n || remainder === 0n) return `${whole.toString()}${suffixLabel}`;
          const decimal = (remainder * 10n) / suffixValue;
          if (decimal === 0n) return `${whole.toString()}${suffixLabel}`;
          return `${whole.toString()}.${decimal.toString()}${suffixLabel}`;
        };
        if (fractionalMicros === 0n) {
          if (wholeUsd >= 1000000000000n) return formatWithSuffix(1000000000000n, "t");
          if (wholeUsd >= 1000000000n) return formatWithSuffix(1000000000n, "b");
          if (wholeUsd >= 1000000n) return formatWithSuffix(1000000n, "m");
          if (wholeUsd >= 1000n) return formatWithSuffix(1000n, "k");
          return wholeUsd.toString();
        }
        const fractionalText = fractionalMicros.toString().padStart(6, "0").replace(/0+$/, "");
        return fractionalText ? `${wholeUsd.toString()}.${fractionalText}` : wholeUsd.toString();
      } catch (_error) {
        return trimmed;
      }
    }

    function describeFollowActionTrigger(action) {
      if (!action || typeof action !== "object") return "Immediate";
      const kind = String(action.kind || "").trim().toLowerCase();
      if (action.marketCap && String(action.marketCap.threshold || "").trim()) {
        const timeoutSeconds = action.marketCap.scanTimeoutSeconds != null
          ? Number(action.marketCap.scanTimeoutSeconds)
          : (action.marketCap.scanTimeoutMinutes != null ? Number(action.marketCap.scanTimeoutMinutes) * 60 : null);
        const timeoutAction = String(action.marketCap.timeoutAction || "").trim();
        const label = `Market Cap $${formatMarketCapThresholdForDisplay(action.marketCap.threshold)}${Number.isFinite(timeoutSeconds) && timeoutSeconds > 0
          ? ` (${timeoutSeconds}s${timeoutAction ? `, ${timeoutAction}` : ""})`
          : ""}`;
        return kind === "sniper-sell" ? `${label} after buy confirmed` : label;
      }
      if (kind === "sniper-sell" && action.targetBlockOffset != null) {
        return `After Buy Confirmed + ${action.targetBlockOffset} Slots`;
      }
      if (kind === "sniper-sell" && action.requireConfirmation) {
        return "After Buy Confirmed";
      }
      if (action.requireConfirmation) return "After confirmation";
      if (action.targetBlockOffset != null) return `On Confirmed Slot + ${action.targetBlockOffset}`;
      if (Number(action.submitDelayMs || 0) > 0) return `Submit + ${action.submitDelayMs}ms`;
      if (action.submitDelayMs != null) return "On Submit";
      if (Number(action.delayMs || 0) > 0) return `Delay ${action.delayMs}ms`;
      return "Immediate";
    }

    function describeFollowActionSize(action) {
      if (!action || typeof action !== "object") return "--";
      const quoteLabel = getQuoteAssetLabel(
        action.quoteAsset
          || (action.followJob && action.followJob.quoteAsset)
          || action.parentQuoteAsset
          || "sol",
      );
      if (action.buyAmountSol) return `${action.buyAmountSol} ${quoteLabel}`;
      if (action.sellPercent != null) return `${action.sellPercent}%`;
      return "--";
    }

    function describeFollowActionWallet(action) {
      if (!action || !action.walletEnvKey) return "--";
      return `Wallet #${walletIndexFromEnvKey(action.walletEnvKey)}`;
    }

    function formatProviderLabel(provider) {
      const normalized = String(provider || "").trim();
      if (!normalized) return "--";
      return providerLabels[normalized] || normalized;
    }

    function formatLaunchTransactionLabel(label) {
      const normalized = String(label || "").trim();
      if (!normalized) return "transaction";
      if (normalized === "follow-up") return "fee-sharing setup";
      if (normalized === "agent-setup") return "agent fee setup";
      return normalized;
    }

    function formatTransportLabel(transportType) {
      const normalized = String(transportType || "").trim().toLowerCase();
      if (!normalized) return "--";
      if (normalized === "helius-sender") return "Helius Sender";
      if (normalized === "hellomoon-quic") return "Hello Moon QUIC";
      if (normalized === "hellomoon-bundle") return "Hello Moon Bundle";
      if (normalized.startsWith("standard-rpc")) return "Standard RPC";
      if (normalized === "jito-bundle") return "Jito Bundle";
      return String(transportType || "").trim();
    }

    function buildBagsLaunchPhaseSummary(report, execution = {}) {
      const launchpad = String(report && report.launchpad || "").trim().toLowerCase();
      if (launchpad !== "bagsapp") return null;
      const sent = Array.isArray(execution.sent) ? execution.sent : [];
      const launchItems = sent.filter((item) => String(item && item.label || "").trim().toLowerCase() === "launch");
      const setupItems = sent.filter((item) => !launchItems.includes(item));
      const uniqueSetupTransports = Array.from(new Set(
        setupItems.map((item) => formatTransportLabel(item && item.transportType)).filter((value) => value && value !== "--"),
      ));
      const uniqueLaunchTransports = Array.from(new Set(
        launchItems.map((item) => formatTransportLabel(item && item.transportType)).filter((value) => value && value !== "--"),
      ));
      return {
        cards: [
          {
            label: "Launch Phases",
            value: setupItems.length ? "Setup + launch" : "Launch only",
            detail: setupItems.length
              ? `${setupItems.length} setup tx before final token creation`
              : "Single tracked launch phase",
          },
          {
            label: "Setup Phase",
            value: setupItems.length ? `${setupItems.length} tx` : "--",
            detail: uniqueSetupTransports.length ? uniqueSetupTransports.join(" | ") : "No setup transactions recorded",
          },
          {
            label: "Final Launch",
            value: launchItems.length ? `${launchItems.length} tx` : "--",
            detail: uniqueLaunchTransports.length ? uniqueLaunchTransports.join(" | ") : "Final launch transport unavailable",
          },
        ],
        note: "Bags launches are slower because they first submit and confirm setup/config transactions before the final token creation transaction. Those extra setup transactions are intentionally included in the report and benchmark path.",
      };
    }

    function followActionRouteDetails(action, followJob) {
      const kind = String(action && action.kind || "").trim().toLowerCase();
      const execution = followJob && followJob.execution && typeof followJob.execution === "object"
        ? followJob.execution
        : {};
      const isBuy = kind === "sniper-buy";
      const isSell = kind === "dev-auto-sell" || kind === "sniper-sell";
      return {
        provider: String(
          (action && action.provider)
          || (isBuy ? execution.buyProvider : "")
          || (isSell ? execution.sellProvider : "")
          || execution.provider
          || "",
        ).trim(),
        endpointProfile: String(
          (action && action.endpointProfile)
          || (isBuy ? execution.buyEndpointProfile : "")
          || (isSell ? execution.sellEndpointProfile : "")
          || execution.endpointProfile
          || "",
        ).trim(),
        transportType: String(action && action.transportType || "").trim(),
      };
    }

    function describeFollowActionRoute(action, followJob) {
      const route = followActionRouteDetails(action, followJob);
      const parts = [];
      if (route.provider) parts.push(formatProviderLabel(route.provider));
      if (route.transportType && route.transportType !== route.provider) parts.push(route.transportType);
      return parts.join(" | ");
    }

    function shortenReportEndpoint(endpoint) {
      const raw = String(endpoint || "").trim();
      if (!raw) return "--";
      try {
        const url = new URL(raw);
        const host = url.hostname.toLowerCase();
        if (host.includes("helius-rpc.com")) {
          if (host.startsWith("mainnet.")) return "Helius WS";
          if (host.includes("-sender.")) {
            const region = host.split("-sender.")[0];
            return `${region.toUpperCase()} sender`;
          }
          return "Helius";
        }
        if (host.includes("jito")) return host.replace(/^https?:\/\//, "");
        const label = host.replace(/^www\./, "");
        return label.length > 24 ? shortenAddress(label, 10) : label;
      } catch (_error) {
        return raw.length > 24 ? shortenAddress(raw, 10) : raw;
      }
    }

    function formatReportEndpointList(endpoints = []) {
      const normalized = Array.isArray(endpoints)
        ? endpoints
          .map((value) => String(value || "").trim())
          .filter(Boolean)
        : [];
      if (!normalized.length) return "--";
      return Array.from(new Set(normalized))
        .map((value) => shortenReportEndpoint(value))
        .join(" | ");
    }

    function formatWatcherModeLabel(mode) {
      const normalized = String(mode || "").trim().toLowerCase();
      if (!normalized) return "";
      if (normalized === "helius-transaction-subscribe") return "Helius transactionSubscribe";
      if (normalized === "standard-ws") return "Standard websocket";
      if (normalized === "rpc-polling") return "RPC polling";
      return String(mode || "").trim();
    }

    function buildWatcherDetail(mode, fallbackReason) {
      const parts = [];
      const modeLabel = formatWatcherModeLabel(mode);
      if (modeLabel) parts.push(`Mode: ${modeLabel}`);
      const note = String(fallbackReason || "").trim();
      if (note) parts.push(note);
      return parts.join(" | ");
    }

    function buildCombinedFollowWatcherCard(actions = [], health = null) {
      const relevantActions = actions.filter((action) => {
        const kind = String(action && action.kind || "").trim().toLowerCase();
        return ["sniper-buy", "sniper-sell", "dev-auto-sell"].includes(kind);
      });
      const actionModes = Array.from(new Set(
        relevantActions
          .map((action) => formatWatcherModeLabel(action && action.watcherMode))
          .filter(Boolean),
      ));
      const healthModes = Array.from(new Set(
        [
          health && health.slotWatcherMode,
          health && health.signatureWatcherMode,
          health && health.marketWatcherMode,
        ]
          .map((mode) => formatWatcherModeLabel(mode))
          .filter(Boolean),
      ));
      const modes = actionModes.length ? actionModes : healthModes;
      const endpointLabel = shortenReportEndpoint(health && health.watchEndpoint);
      if (!modes.length && (!endpointLabel || endpointLabel === "--")) return null;
      const detailParts = [];
      if (modes.length === 1) {
        detailParts.push(`Mode: ${modes[0]}`);
      } else if (modes.length > 1) {
        detailParts.push(`Modes: ${modes.join(" | ")}`);
      }
      return {
        label: "Follow Watcher WS",
        value: endpointLabel && endpointLabel !== "--"
          ? endpointLabel
          : (modes.length === 1 ? modes[0] : "Mixed"),
        detail: detailParts.join(" | "),
      };
    }

    function renderCopyableHash(value, label = "Copy hash") {
      const raw = String(value || "").trim();
      if (!raw) return "--";
      return `
    <button
      type="button"
      class="reports-copy-hash"
      data-copy-value="${escapeHTML(raw)}"
      title="${escapeHTML(label)}"
    >
      ${escapeHTML(shortenAddress(raw, 8))}
    </button>
  `;
    }

    function buildFollowActionMetricItems(action, followJob) {
      const isBuy = String(action && action.kind || "").toLowerCase() === "sniper-buy";
      const route = followActionRouteDetails(action, followJob);
      const observedActionSendSlot = readReportSlotValue(action, "sendObservedSlot", "sendObservedBlockHeight");
      const launchObservedSendSlot = isBuy
        ? readReportSlotValue(followJob, "sendObservedSlot", "sendObservedBlockHeight")
        : null;
      const resolvedConfirmSlot = readReportSlotValue(action, "confirmedObservedSlot", "confirmedObservedBlockHeight");
      const metrics = [
        { label: "Provider", value: formatProviderLabel(route.provider) },
        { label: "Transport", value: route.transportType || "--" },
        { label: "Endpoint Profile", value: route.endpointProfile || "--" },
        { label: "Watcher", value: formatWatcherModeLabel(action && action.watcherMode) || "--", detail: action && action.watcherFallbackReason ? String(action.watcherFallbackReason) : "" },
        { label: "Wallet", value: describeFollowActionWallet(action) },
        { label: "Trigger", value: describeFollowActionTrigger(action) },
        {
          label: "Size",
          value: describeFollowActionSize({
            ...action,
            parentQuoteAsset: followJob && followJob.quoteAsset,
          }),
        },
        { label: "Observed Send Slot", value: observedActionSendSlot != null ? String(observedActionSendSlot) : formatReportSlotValue(launchObservedSendSlot, "launch ") },
        { label: "Observed Confirm Slot", value: formatReportSlotValue(resolvedConfirmSlot) },
        { label: "Observed Slots To Confirm", value: action && action.slotsToConfirm != null ? String(action.slotsToConfirm) : action && action.blocksToConfirm != null ? String(action.blocksToConfirm) : "--" },
        { label: "Endpoint", value: shortenReportEndpoint(action && action.endpoint) },
        { label: "Attempts", value: String(action && action.attemptCount != null ? action.attemptCount : 0) },
      ];
      if (!isBuy) {
        metrics.push(
          { label: "Scheduled", value: formatReportTimestamp(action && action.scheduledForMs) },
          { label: "Started", value: formatReportTimestamp(action && action.submitStartedAtMs) },
          { label: "Submitted", value: formatReportTimestamp(action && action.submittedAtMs) },
          { label: "Confirmed", value: formatReportTimestamp(action && action.confirmedAtMs) },
          { label: "Submit->Confirm", value: formatReportLatencyDelta(action && action.submittedAtMs, action && action.confirmedAtMs) },
        );
      } else {
        metrics.push(
          { label: "Launch->Submit", value: followJob ? formatReportLatencyDelta(followJob.submitAtMs, action && action.submittedAtMs) : "--" },
          { label: "Submit Time", value: formatReportTimestamp(action && action.submittedAtMs) },
        );
      }
      return metrics;
    }

    function renderReportMetricGrid(items = []) {
      const visible = items.filter((item) => item && item.value != null && item.value !== "");
      if (!visible.length) {
        return '<div class="reports-terminal-empty">No metrics available for this section.</div>';
      }
      return `
    <div class="reports-metric-grid">
      ${visible.map((item) => `
        <div class="reports-metric-card${item.tone ? ` is-${escapeHTML(String(item.tone))}` : ""}">
          <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
          <strong class="reports-metric-value">${item.renderValue ? item.renderValue : escapeHTML(String(item.value))}</strong>
          ${item.detail ? `<span class="reports-metric-note">${escapeHTML(String(item.detail))}</span>` : ""}
        </div>
      `).join("")}
    </div>
  `;
    }

    function normalizeReportWarnings(warnings) {
      if (!Array.isArray(warnings)) return [];
      return warnings
        .map((warning) => String(warning || "").trim())
        .filter(Boolean);
    }

    function renderReportWarningsSection(warnings, title = "Warnings") {
      const visible = normalizeReportWarnings(warnings);
      if (!visible.length) return "";
      return `
    <section class="reports-panel-section">
      <div class="reports-panel-title">${escapeHTML(title)}</div>
      ${visible.map((warning) => `<div class="reports-callout is-bad">${escapeHTML(warning)}</div>`).join("")}
    </section>
  `;
    }

    function buildReportsOverviewMarkup() {
      const entry = currentReportsTerminalEntry();
      const payload = currentReportsTerminalPayload();
      const report = currentReportsTerminalReport();
      const execution = currentReportsTerminalExecution() || {};
      const reportWarnings = normalizeReportWarnings(execution.warnings);
      const benchmark = currentReportsTerminalBenchmark() || {};
      const timings = benchmark.timings || execution.timings || {};
      const benchmarkGroups = benchmarkTimingGroupsFromPayload(benchmark, execution);
      const topLevelGroup = benchmarkTimingGroupByKey(benchmarkGroups, "topLevel") || benchmarkGroups[0] || { items: [] };
      const health = report && report.followDaemon && report.followDaemon.health ? report.followDaemon.health : null;
      const job = currentReportsTerminalFollowJob();
      const actions = currentReportsTerminalFollowActions();
      const launchSends = Array.isArray(execution.sent) ? execution.sent : [];
      const benchmarkMode = benchmarkModeLabel(benchmark.mode || (timings && timings.benchmarkMode));
      const launchSpeedCards = buildBenchmarkHeadlineCards(timings);
      const bagsLaunchPhaseSummary = buildBagsLaunchPhaseSummary(report, execution);
      const providerCardLabel = job ? "Launch Provider" : "Provider";
      const transportCardLabel = job ? "Launch Transport" : "Transport";
      const problemCount = actions.filter((action) => ["failed", "cancelled", "expired"].includes(String(action.state || "").toLowerCase())).length;
      const runningCount = actions.filter((action) => ["running", "eligible", "armed", "queued", "sent"].includes(String(action.state || "").toLowerCase())).length;
      const combinedWatcherCard = buildCombinedFollowWatcherCard(actions, health);
      const launchStatusSummary = summarizeLaunchStatuses(launchSends);
      const launchEndpointSummary = launchSends.length
        ? {
          value: formatReportEndpointList(launchSends.map((item) => item && item.endpoint)),
          detail: Array.isArray(launchSends) && launchSends.length > 1 ? `${launchSends.length} tracked tx` : "",
        }
        : null;
      const launchSourceSummary = summarizeLaunchConfirmationSources(launchSends);
      const launchObservedSlotsSummary = summarizeLaunchObservedSlots(launchSends);
      const formatDaemonCapacityValue = (available, max) => {
        if (max == null) return "Uncapped";
        if (available == null) return "--";
        return String(available);
      };
      const overviewCards = [
        { label: "Action", value: entry && entry.action ? entry.action : payload && payload.action ? payload.action : "--" },
        {
          label: "Mint",
          value: entry && entry.mint ? shortenAddress(entry.mint, 6) : report && report.mint ? shortenAddress(report.mint, 6) : "--",
          renderValue: entry && entry.mint
            ? renderCopyableHash(entry.mint, "Copy mint")
            : (report && report.mint ? renderCopyableHash(report.mint, "Copy mint") : "--"),
        },
        { label: providerCardLabel, value: execution.resolvedProvider || execution.provider || "--" },
        { label: transportCardLabel, value: execution.transportType || (entry && entry.transportType) || "--" },
        {
          label: "Backend",
          value: execution.launchpadBackend || "--",
          detail: execution.launchpadRolloutState || "",
        },
        { label: "Signatures", value: entry ? String(entry.signatureCount || 0) : String(Array.isArray(payload && payload.signatures) ? payload.signatures.length : 0) },
        { label: "Follow", value: job ? (job.state || "armed") : "Off" },
        { label: "Selected Wallet", value: job && job.selectedWalletKey ? `Wallet #${walletIndexFromEnvKey(job.selectedWalletKey)}` : "--" },
        { label: "Follow Actions", value: actions.length ? `${actions.length} total` : "0" },
        { label: "Problems", value: String(problemCount) },
        { label: "Running", value: String(runningCount) },
      ]
        .concat(launchStatusSummary ? [{ label: "Launch Status", value: launchStatusSummary.value, detail: launchStatusSummary.detail }] : [])
        .concat(launchEndpointSummary && launchEndpointSummary.value !== "--" ? [{ label: "Launch Endpoint", value: launchEndpointSummary.value, detail: launchEndpointSummary.detail }] : [])
        .concat(launchSourceSummary ? [{ label: "Confirm Path", value: launchSourceSummary.value, detail: launchSourceSummary.detail }] : [])
        .concat(launchObservedSlotsSummary ? [{ label: "Observed Slots To Confirm", value: launchObservedSlotsSummary.value, detail: launchObservedSlotsSummary.detail }] : [])
        .concat(combinedWatcherCard ? [combinedWatcherCard] : []);
      const watcherCards = health
        ? [
          { label: "Slot Watcher", value: health.slotWatcher || "--", detail: buildWatcherDetail(health.slotWatcherMode) },
          { label: "Signature Watcher", value: health.signatureWatcher || "--", detail: buildWatcherDetail(health.signatureWatcherMode) },
          { label: "Market Watcher", value: health.marketWatcher || "--", detail: buildWatcherDetail(health.marketWatcherMode) },
          { label: "Queue Depth", value: String(health.queueDepth != null ? health.queueDepth : "--") },
          { label: "Compile Slots", value: formatDaemonCapacityValue(health.availableCompileSlots, health.maxConcurrentCompiles) },
          { label: "Send Slots", value: formatDaemonCapacityValue(health.availableSendSlots, health.maxConcurrentSends) },
        ]
        : [];
      return `
    <div class="reports-panel-stack">
      <section class="reports-panel-section">
        <div class="reports-panel-title">Overview</div>
        ${renderReportMetricGrid(overviewCards)}
      </section>
      ${renderReportWarningsSection(reportWarnings, "Launch Warnings")}
      ${bagsLaunchPhaseSummary ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Bags Launch Phases</div>
          ${renderReportMetricGrid(bagsLaunchPhaseSummary.cards)}
          <div class="reports-callout is-bad">${escapeHTML(bagsLaunchPhaseSummary.note)}</div>
        </section>
      ` : ""}
      ${benchmarkMode !== "Off" && launchSpeedCards.length ? `
        <section class="reports-panel-section reports-panel-section-launch-speed">
          <div class="reports-panel-title">Launch Speed</div>
          <div class="reports-panel-note">Submission is the steadier execution metric. Confirmation varies more between runs because it depends on provider/RPC observation latency.</div>
          <div class="reports-metric-grid reports-metric-grid-launch-speed">
            ${launchSpeedCards.map((item) => `
              <div class="reports-metric-card${item.tone ? ` is-${escapeHTML(String(item.tone))}` : ""}">
                <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
                <strong class="reports-metric-value">${escapeHTML(String(item.value))}</strong>
                ${item.detail ? `<span class="reports-metric-note">${escapeHTML(String(item.detail))}</span>` : ""}
              </div>
            `).join("")}
          </div>
        </section>
      ` : ""}
      ${benchmarkMode === "Off" ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Benchmarks</div>
          <div class="reports-callout">Benchmark collection was disabled for this report.</div>
        </section>
      ` : `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Stage Totals</div>
          <div class="reports-panel-note">
            ${benchmarkMode ? `Benchmark mode: ${escapeHTML(benchmarkMode)}. ` : ""}Totals are inclusive. Child timings and remainders are broken out on the Benchmarks tab.
          </div>
          ${renderReportMetricGrid(topLevelGroup.items || [])}
        </section>
      `}
      ${watcherCards.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Follow Health</div>
          ${renderReportMetricGrid(watcherCards)}
          ${health && health.lastError ? `<div class="reports-callout is-bad">${escapeHTML(String(health.lastError))}</div>` : ""}
        </section>
      ` : ""}
    </div>
  `;
    }

    function buildReportsActionsMarkup() {
      const report = currentReportsTerminalReport();
      const execution = currentReportsTerminalExecution() || {};
      const reportWarnings = normalizeReportWarnings(execution.warnings);
      const followJob = currentReportsTerminalFollowJob();
      const actions = currentReportsTerminalFollowActions();
      const launchSends = Array.isArray(execution.sent) ? execution.sent : [];
      const bagsLaunchPhaseSummary = buildBagsLaunchPhaseSummary(report, execution);
      if (!launchSends.length && !actions.length) {
        return '<div class="reports-terminal-empty">No action data available in this report.</div>';
      }
      return `
    <div class="reports-panel-stack">
      ${renderReportWarningsSection(reportWarnings, "Launch Warnings")}
      ${bagsLaunchPhaseSummary ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Bags Launch Phases</div>
          <div class="reports-callout is-bad">${escapeHTML(bagsLaunchPhaseSummary.note)}</div>
        </section>
      ` : ""}
      ${launchSends.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Launch Send</div>
          <div class="reports-action-list">
            ${launchSends.map((sent) => `
              <article class="reports-action-card">
                <div class="reports-action-head">
                  <div>
                    <strong>${escapeHTML(formatLaunchTransactionLabel(sent.label || "launch"))}</strong>
                    <div class="reports-action-subtitle">${escapeHTML([
                      execution.resolvedProvider || execution.provider || execution.transportType || "--",
                      sent.endpoint ? `winning ${shortenReportEndpoint(sent.endpoint)}` : "",
                    ].filter(Boolean).join(" | "))}</div>
                  </div>
                  <span class="reports-state-badge ${reportStateClass(sent.confirmationStatus)}">${escapeHTML(sent.confirmationStatus || "sent")}</span>
                </div>
                ${renderReportMetricGrid([
                  { label: "Landed", value: formatLandedValue(sent.confirmationStatus) },
                  { label: "Confirmation Path", value: formatConfirmationSourceLabel(sent.confirmationSource) },
                  { label: "Winning Endpoint", value: shortenReportEndpoint(sent.endpoint) },
                  { label: "Attempted Endpoints", value: formatReportEndpointList(sent.attemptedEndpoints), detail: Array.isArray(sent.attemptedEndpoints) && sent.attemptedEndpoints.length > 1 ? `${sent.attemptedEndpoints.length} attempted` : "" },
                  { label: "Observed Send Slot", value: formatReportSlotValue(readReportSlotValue(sent, "sendObservedSlot", "sendObservedBlockHeight")) },
                  { label: "Confirmed Slot", value: formatReportSlotValue(readReportSlotValue(sent, "confirmedSlot", "confirmedObservedSlot", "confirmedObservedBlockHeight")) },
                  (() => {
                    const observedConfirmSlot = readReportSlotValue(sent, "confirmedObservedSlot", "confirmedObservedBlockHeight");
                    const exactConfirmSlot = readReportSlotValue(sent, "confirmedSlot");
                    if (observedConfirmSlot == null || observedConfirmSlot === exactConfirmSlot) return null;
                    return { label: "Observed Confirm Slot", value: String(observedConfirmSlot) };
                  })(),
                  (() => {
                    const observedSendSlot = readReportSlotValue(sent, "sendObservedSlot", "sendObservedBlockHeight");
                    const observedConfirmSlot = readReportSlotValue(sent, "confirmedObservedSlot", "confirmedObservedBlockHeight");
                    return {
                      label: "Observed Slots To Confirm",
                      value: observedSendSlot != null && observedConfirmSlot != null
                        ? String(Math.max(0, observedConfirmSlot - observedSendSlot))
                        : "--",
                    };
                  })(),
                  { label: "Format", value: sent.format || "--" },
                  { label: "Bundle ID", value: sent.bundleId || "--" },
                ])}
                ${sent.signature ? `<div class="reports-action-links">${renderCopyableHash(sent.signature, "Copy signature")} ${sent.explorerUrl ? `<a class="reports-inline-link" href="${escapeHTML(sent.explorerUrl)}" target="_blank" rel="noreferrer">Open explorer</a>` : ""}</div>` : ""}
              </article>
            `).join("")}
          </div>
        </section>
      ` : ""}
      ${actions.length ? `
        <section class="reports-panel-section">
          <div class="reports-panel-title">Follow Actions</div>
          <div class="reports-action-list">
            ${actions.map((action) => {
              const errorCategory = inferReportErrorCategory(action.lastError);
              const subtitleParts = [
                describeFollowActionRoute(action, followJob),
                describeFollowActionWallet(action),
                describeFollowActionTrigger(action),
                describeFollowActionSize({ ...action, parentQuoteAsset: followJob && followJob.quoteAsset }),
              ].filter((part) => part && part !== "--");
              return `
                <article class="reports-action-card">
                  <div class="reports-action-head">
                    <div>
                      <strong>${escapeHTML(action.kind || action.actionId || "action")}</strong>
                      <div class="reports-action-subtitle">${escapeHTML(subtitleParts.join(" | "))}</div>
                    </div>
                    <span class="reports-state-badge ${reportStateClass(action.state)}">${escapeHTML(action.state || "--")}</span>
                  </div>
                  ${renderReportMetricGrid(buildFollowActionMetricItems(action, followJob))}
                  ${(action.signature || action.explorerUrl) ? `<div class="reports-action-links">${action.signature ? renderCopyableHash(action.signature, "Copy signature") : ""} ${action.explorerUrl ? `<a class="reports-inline-link" href="${escapeHTML(action.explorerUrl)}" target="_blank" rel="noreferrer">Open explorer</a>` : ""}</div>` : ""}
                  ${action.lastError ? `<div class="reports-callout is-bad"><strong>${escapeHTML(errorCategory || "error")}</strong> | ${escapeHTML(String(action.lastError))}</div>` : ""}
                </article>
              `;
            }).join("")}
          </div>
        </section>
      ` : ""}
    </div>
  `;
    }

    function buildReportsBenchmarksMarkup() {
      const benchmark = currentReportsTerminalBenchmark() || {};
      const execution = currentReportsTerminalExecution() || {};
      const autoFee = currentReportsTerminalAutoFee();
      const timings = benchmark.timings || execution.timings || {};
      const benchmarkGroups = benchmarkTimingGroupsFromPayload(benchmark, execution);
      const sent = resolveBenchmarkSentItems(benchmark, execution);
      const benchmarkMode = benchmarkModeLabel(benchmark.mode || (timings && timings.benchmarkMode));
      const launchSpeedCards = buildBenchmarkHeadlineCards(timings);
      const reconciliation = buildBenchmarkReconciliationSections(timings, benchmark.mode || (timings && timings.benchmarkMode));
      if (benchmarkMode === "Off") {
        return `
    <div class="reports-panel-stack">
      <section class="reports-panel-section">
        <div class="reports-panel-title">Benchmark Mode</div>
        <div class="reports-callout">Benchmark collection was disabled for this report.</div>
      </section>
    </div>
  `;
      }
      const timingSectionsMarkup = benchmarkGroups.length
        ? benchmarkGroups.map((group) => `
      <section class="reports-panel-section">
        <div class="reports-panel-title">${escapeHTML(group.label || "Timings")}</div>
        ${group.key === "topLevel"
          ? '<div class="reports-panel-note">Inclusive totals and remainder buckets are separated so the path can be audited.</div>'
          : ""}
        ${renderReportMetricGrid(group.items || [])}
      </section>
    `).join("")
        : '<section class="reports-panel-section"><div class="reports-terminal-empty">No benchmark timing groups are available for this report.</div></section>';
      return `
    <div class="reports-panel-stack">
      ${benchmarkMode ? `<section class="reports-panel-section"><div class="reports-panel-title">Benchmark Mode</div><div class="reports-panel-note">${escapeHTML(benchmarkMode)} benchmark collection is active for this report.</div></section>` : ""}
      ${buildAutoFeeBenchmarkSection(autoFee, benchmarkMode)}
      ${launchSpeedCards.length ? `
        <section class="reports-panel-section reports-panel-section-launch-speed">
          <div class="reports-panel-title">Launch Speed</div>
          <div class="reports-panel-note">Submitted is the steadier execution metric. Confirmed includes the variable provider/RPC confirmation wait.</div>
          <div class="reports-metric-grid reports-metric-grid-launch-speed">
            ${launchSpeedCards.map((item) => `
              <div class="reports-metric-card${item.tone ? ` is-${escapeHTML(String(item.tone))}` : ""}">
                <span class="reports-metric-label">${escapeHTML(item.label || "")}</span>
                <strong class="reports-metric-value">${escapeHTML(String(item.value))}</strong>
                ${item.detail ? `<span class="reports-metric-note">${escapeHTML(String(item.detail))}</span>` : ""}
              </div>
            `).join("")}
          </div>
        </section>
      ` : ""}
      <section class="reports-panel-section">
        <div class="reports-panel-title">End-to-End Composition</div>
        <div class="reports-panel-note">This section shows exactly what the top-line benchmark consists of before you scroll into the lower-level groups.</div>
        ${renderReportMetricGrid(reconciliation.topLevel)}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Client Composition</div>
        ${renderReportMetricGrid(reconciliation.client)}
      </section>
      <section class="reports-panel-section">
        <div class="reports-panel-title">Backend Composition</div>
        <div class="reports-panel-note">If a remainder is non-zero, that is measured time inside the parent total that this report or benchmark mode has not broken into smaller named steps yet.</div>
        ${renderReportMetricGrid(reconciliation.backend)}
      </section>
      ${timingSectionsMarkup}
      <section class="reports-panel-section">
        <div class="reports-panel-title">Chain Benchmark</div>
        ${sent.length ? `
          <div class="reports-action-list">
            ${sent.map((item) => `
              <article class="reports-action-card">
                <div class="reports-action-head">
                  <div>
                    <strong>${escapeHTML(item.label || "tx")}</strong>
                    <div class="reports-action-subtitle">${escapeHTML(item.signature ? shortenAddress(item.signature, 8) : "--")}</div>
                  </div>
                  <span class="reports-state-badge ${reportStateClass(item.confirmationStatus)}">${escapeHTML(item.confirmationStatus || "--")}</span>
                </div>
                ${renderReportMetricGrid([
                  { label: "Landed", value: formatLandedValue(item.confirmationStatus) },
                  { label: "Confirmation Path", value: formatConfirmationSourceLabel(item.confirmationSource) },
                  { label: "Winning Endpoint", value: shortenReportEndpoint(item.endpoint) },
                  { label: "Attempted Endpoints", value: formatReportEndpointList(item.attemptedEndpoints), detail: Array.isArray(item.attemptedEndpoints) && item.attemptedEndpoints.length > 1 ? `${item.attemptedEndpoints.length} attempted` : "" },
                  { label: "Observed Send Slot", value: formatReportSlotValue(readReportSlotValue(item, "sendSlot", "sendBlockHeight", "sendObservedSlot", "sendObservedBlockHeight")) },
                  { label: "Confirmed Slot", value: formatReportSlotValue(readReportSlotValue(item, "confirmedSlot", "confirmedObservedSlot", "confirmedBlockHeight", "confirmedObservedBlockHeight")) },
                  (() => {
                    const observedConfirmSlot = readReportSlotValue(item, "confirmedObservedSlot", "confirmedBlockHeight", "confirmedObservedBlockHeight");
                    const exactConfirmSlot = readReportSlotValue(item, "confirmedSlot");
                    if (observedConfirmSlot == null || observedConfirmSlot === exactConfirmSlot) return null;
                    return { label: "Observed Confirm Slot", value: String(observedConfirmSlot) };
                  })(),
                  { label: "Observed Slots To Confirm", value: computeObservedSlotsToConfirm(item) != null ? String(computeObservedSlotsToConfirm(item)) : "--" },
                  { label: "Bundle ID", value: item.bundleId || "--" },
                ])}
              </article>
            `).join("")}
          </div>
        ` : '<div class="reports-terminal-empty">No chain benchmark entries recorded.</div>'}
      </section>
    </div>
  `;
    }

    function buildBenchmarksPopoutTitle() {
      return "Benchmark Popout";
    }

    function renderBenchmarksPopoutModal() {
      if (!benchmarksPopoutModal || benchmarksPopoutModal.hidden || !benchmarksPopoutBody) return;
      if (benchmarksPopoutTitle) {
        benchmarksPopoutTitle.textContent = buildBenchmarksPopoutTitle();
      }
      const payload = currentReportsTerminalPayload();
      benchmarksPopoutBody.innerHTML = payload
        ? `<div class="benchmarks-popout-content">${buildReportsBenchmarksMarkup()}</div>`
        : '<div class="reports-callout">Structured benchmark data is unavailable for this report.</div>';
    }

    function buildReportsRawMarkup() {
      const state = reportsState();
      return `<pre class="console reports-console">${escapeHTML(state.activeText || "Report is empty.")}</pre>`;
    }

    function buildLaunchHistorySettingsText(launch) {
      const settings = [
        launch.report && launch.report.launchpad ? launch.report.launchpad : "",
        launch.report && launch.report.mode ? launch.report.mode : "",
        launch.quoteAsset || "",
        launch.execution && launch.execution.activePresetId ? launch.execution.activePresetId : "",
        launch.selectedWalletKey ? formatWalletHistoryLabel(launch.selectedWalletKey) : "",
        launch.execution && launch.execution.provider ? launch.execution.provider : "",
        launch.execution && launch.execution.buyProvider ? `buy ${launch.execution.buyProvider}` : "",
        launch.execution && launch.execution.sellProvider ? `sell ${launch.execution.sellProvider}` : "",
      ].filter(Boolean);
      return settings.join(" | ");
    }

    function parseLaunchBuyWalletLabel(rawLabel) {
      const label = String(rawLabel || "").trim();
      const match = label.match(/wallet-(.+)$/i);
      if (!match) return "";
      const suffix = String(match[1] || "").trim();
      if (!suffix || suffix === "primary") return "Wallet #1";
      if (/^\d+$/.test(suffix)) return `Wallet #${suffix}`;
      return `Wallet ${suffix}`;
    }

    function buildLaunchHistorySnipeText(launch) {
      const quoteAsset = launch.quoteAsset || "sol";
      const quoteLabel = getQuoteAssetLabel(quoteAsset);
      const snipes = launch.followLaunch && Array.isArray(launch.followLaunch.snipes)
        ? launch.followLaunch.snipes.filter((entry) => entry && entry.enabled !== false)
        : [];
      if (!snipes.length) return "";
      return snipes.map((entry) => {
        const walletLabel = formatWalletHistoryLabel(entry.walletEnvKey || entry.envKey || "");
        const amount = entry.buyAmountSol ? `${entry.buyAmountSol} ${quoteLabel}` : `-- ${quoteLabel}`;
        const trigger = entry.targetBlockOffset != null
          ? `b${entry.targetBlockOffset}`
          : (entry.submitWithLaunch ? "same-time" : getSniperTriggerSummary(entry).toLowerCase());
        const retry = entry.retryOnFailure ? " | retry once" : "";
        return `${walletLabel || "Wallet"} ${amount} @ ${trigger}${retry}`;
      }).join(" | ");
    }

    function buildLaunchHistoryLaunchBuyFallbackText(launch) {
      const sent = launch.execution && Array.isArray(launch.execution.sent) ? launch.execution.sent : [];
      const labels = sent
        .map((entry) => parseLaunchBuyWalletLabel(entry && entry.label))
        .filter(Boolean);
      if (!labels.length) return "";
      return `launch buys ${labels.join(" | ")}`;
    }

    function buildLaunchHistoryAutoSellText(launch) {
      const devAutoSell = launch.followLaunch && launch.followLaunch.devAutoSell && launch.followLaunch.devAutoSell.enabled;
      if (!devAutoSell) return "";
      const autoSell = launch.followLaunch.devAutoSell;
      const percent = autoSell.percent != null ? autoSell.percent : 100;
      const parts = [`${percent}%`];
      if (autoSell.marketCap && autoSell.marketCap.threshold) {
        const trigger = autoSell.marketCap;
        parts.push(
          `market $${formatMarketCapThresholdForDisplay(trigger.threshold)}${
            (trigger.scanTimeoutSeconds != null || trigger.scanTimeoutMinutes != null)
              ? ` (${trigger.scanTimeoutSeconds != null ? trigger.scanTimeoutSeconds : trigger.scanTimeoutMinutes * 60}s${trigger.timeoutAction ? `, ${trigger.timeoutAction}` : ""})`
              : ""
          }`
        );
      } else if (autoSell.targetBlockOffset != null) {
        parts.push(`confirmed + ${autoSell.targetBlockOffset}`);
      } else if (autoSell.requireConfirmation) {
        parts.push("after confirmation");
      } else {
        const delayMs = autoSell.delayMs != null ? autoSell.delayMs : 0;
        parts.push(delayMs > 0 ? `submit + ${delayMs}ms` : "on submit");
      }
      return parts.join(" | ");
    }

    function buildLaunchHistoryBuyAmountItems(launch) {
      const items = [];
      if (launch.devBuy && launch.devBuy.amount) {
        items.push({
          label: "Dev Buy",
          value: `${launch.devBuy.amount} ${launch.devBuy.mode === "tokens" ? "%" : getDevBuyAssetLabel(launch.launchpad || "pump", launch.quoteAsset || "sol")}`,
        });
      }
      const snipeText = buildLaunchHistorySnipeText(launch);
      if (snipeText) {
        items.push({ label: "Auto Snipe", value: snipeText });
      } else {
        const fallbackSnipeText = buildLaunchHistoryLaunchBuyFallbackText(launch);
        if (fallbackSnipeText) items.push({ label: "Launch Buys", value: fallbackSnipeText });
      }
      const autoSellText = buildLaunchHistoryAutoSellText(launch);
      if (autoSellText) {
        items.push({ label: "Auto Sell", value: autoSellText });
      }
      return items;
    }

    function buildLaunchHistoryBuyAmountsMarkup(launch) {
      const items = buildLaunchHistoryBuyAmountItems(launch);
      if (!items.length) {
        return '<div class="reports-launch-card-copy">No buy actions recorded.</div>';
      }
      return `
    <div class="reports-launch-card-detail-list">
      ${items.map((item) => `
        <div class="reports-launch-card-detail-row">
          <div class="reports-launch-card-detail-key">${escapeHTML(item.label)}</div>
          <div class="reports-launch-card-detail-value">${escapeHTML(item.value)}</div>
        </div>
      `).join("")}
    </div>
  `;
    }

    function formatLaunchHistoryDisplayTime(entry) {
      const writtenAtMs = entry && entry.writtenAtMs != null ? entry.writtenAtMs : 0;
      const numeric = Number(writtenAtMs);
      if (Number.isFinite(numeric) && numeric > 0) {
        try {
          return new Date(numeric).toLocaleString([], {
            month: "short",
            day: "numeric",
            hour: "numeric",
            minute: "2-digit",
          });
        } catch (_error) {
          // Fall back below.
        }
      }
      return entry && entry.displayTime ? String(entry.displayTime) : "";
    }

    function launchPrimarySignature(launch) {
      return launch.payload && Array.isArray(launch.payload.signatures) && launch.payload.signatures[0]
        ? String(launch.payload.signatures[0]).trim()
        : "";
    }

    function launchSolscanUrl(launch) {
      const signature = launchPrimarySignature(launch);
      return signature ? `https://solscan.io/tx/${encodeURIComponent(signature)}` : "";
    }

    function formatFollowStateLabel(value) {
      const normalized = String(value || "").trim();
      if (!normalized) return "Unknown";
      return normalized
        .split("-")
        .filter(Boolean)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ");
    }

    function followStateBadgeTone(state) {
      const normalized = String(state || "").trim().toLowerCase();
      if (["running", "sent", "confirmed", "completed", "stopped"].includes(normalized)) return "is-good";
      if (["failed", "cancelled", "completed-with-failures", "expired"].includes(normalized)) return "is-bad";
      if (["armed", "eligible", "reserved"].includes(normalized)) return "is-warn";
      return "";
    }

    function formatCompactDateTime(value) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric) || numeric <= 0) return "";
      try {
        return new Date(numeric).toLocaleString([], {
          month: "short",
          day: "numeric",
          hour: "numeric",
          minute: "2-digit",
        });
      } catch (_error) {
        return "";
      }
    }

    function summarizeFollowJobProgress(job) {
      const actions = Array.isArray(job && job.actions) ? job.actions : [];
      if (!actions.length) {
        return job && job.cancelRequested
          ? "Cancel requested."
          : "Waiting for follow actions.";
      }
      const counts = actions.reduce((accumulator, action) => {
        const state = String(action && action.state || "").trim().toLowerCase();
        if (state) {
          accumulator[state] = (accumulator[state] || 0) + 1;
        }
        return accumulator;
      }, {});
      const doneCount = (counts.confirmed || 0) + (counts.sent || 0) + (counts.stopped || 0);
      const activeCount = (counts.running || 0) + (counts.eligible || 0);
      const queuedCount = (counts.queued || 0) + (counts.armed || 0);
      const stoppedCount = counts.stopped || 0;
      const failedCount = counts.failed || 0;
      const cancelledCount = counts.cancelled || 0;
      const expiredCount = counts.expired || 0;
      const parts = [`${doneCount}/${actions.length} done`];
      if (activeCount > 0) parts.push(`${activeCount} active`);
      if (queuedCount > 0) parts.push(`${queuedCount} queued`);
      if (stoppedCount > 0) parts.push(`${stoppedCount} stopped`);
      if (failedCount > 0) parts.push(`${failedCount} failed`);
      if (cancelledCount > 0) parts.push(`${cancelledCount} cancelled`);
      if (expiredCount > 0) parts.push(`${expiredCount} expired`);
      if (job && job.cancelRequested) parts.push("cancel requested");
      return parts.join(" | ");
    }

    function buildFollowActionSubtitle(action) {
      const parts = [];
      if (action && action.walletEnvKey) parts.push(`W${walletIndexFromEnvKey(action.walletEnvKey)}`);
      if (action && action.buyAmountSol) parts.push(`${action.buyAmountSol} SOL`);
      if (action && action.sellPercent != null) parts.push(`${action.sellPercent}% sell`);
      if (action && action.targetBlockOffset != null) parts.push(`+${action.targetBlockOffset} slots`);
      if (action && action.submitDelayMs != null && Number(action.submitDelayMs) > 0) parts.push(`${action.submitDelayMs}ms delay`);
      if (action && action.watcherMode) parts.push(formatWatcherModeLabel(action.watcherMode));
      if (action && action.signature) parts.push(shortAddress(action.signature));
      return parts.join(" | ");
    }

    function buildActiveJobActionRouteMarkup(action, followJob) {
      const route = followActionRouteDetails(action, followJob);
      const rows = [
        { label: "Provider", value: formatProviderLabel(route.provider) },
        { label: "Transport", value: route.transportType || "--" },
        { label: "Profile", value: route.endpointProfile || "--" },
      ];
      if (action && action.watcherMode) {
        rows.push({ label: "Watcher", value: formatWatcherModeLabel(action.watcherMode) });
      }
      return `
    <div class="reports-active-job-action-meta">
      ${rows.map((row) => `
        <span class="reports-active-job-action-meta-pill">
          <strong>${escapeHTML(row.label)}</strong>
          <span>${escapeHTML(row.value)}</span>
        </span>
      `).join("")}
    </div>
  `;
    }

    function buildActiveJobLaunchRouteMarkup(job) {
      const plan = job && job.transportPlan && typeof job.transportPlan === "object"
        ? job.transportPlan
        : {};
      const execution = job && job.execution && typeof job.execution === "object"
        ? job.execution
        : {};
      const rows = [
        { label: "Launch Provider", value: formatProviderLabel(plan.resolvedProvider || execution.provider || "") },
        { label: "Launch Transport", value: String(plan.transportType || "--").trim() || "--" },
        { label: "Launch Profile", value: String(plan.resolvedEndpointProfile || execution.endpointProfile || "--").trim() || "--" },
      ];
      return `
    <div class="reports-active-job-route-meta">
      ${rows.map((row) => `
        <span class="reports-active-job-action-meta-pill">
          <strong>${escapeHTML(row.label)}</strong>
          <span>${escapeHTML(row.value)}</span>
        </span>
      `).join("")}
    </div>
  `;
    }

    function buildReportsActiveJobsMarkup() {
      const followJobs = followJobsState();
      const snapshot = getFollowStatusSnapshot();
      const activeJobs = followJobs.jobs.filter((job) => !isTerminalFollowJobState(job && job.state));
      const summaryClassNames = [
        "reports-follow-summary",
        snapshot.offline ? "is-offline" : "",
        snapshot.counts.active > 0 ? "is-active" : "",
        snapshot.counts.issues > 0 ? "is-issues" : "",
      ].filter(Boolean).join(" ");
      if (snapshot.offline) {
        return `
      <div class="reports-panel-stack">
        <div class="reports-active-jobs-header">
          <div class="reports-active-jobs-heading">
            <strong>Jobs</strong>
            <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
          </div>
          <button
            type="button"
            class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger"
            data-follow-cancel-all="1"
            disabled
          >Cancel all</button>
        </div>
        <div class="reports-callout is-bad">${escapeHTML(followJobs.error || "Follow daemon is offline. Live active jobs are temporarily unavailable.")}</div>
      </div>
    `;
      }
      if (!snapshot.configured) {
        return `
      <div class="reports-panel-stack">
        <div class="reports-active-jobs-header">
          <div class="reports-active-jobs-heading">
            <strong>Jobs</strong>
            <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
          </div>
        </div>
        <div class="reports-terminal-empty">Follow daemon is not enabled for this workspace.</div>
      </div>
    `;
      }
      if (!activeJobs.length) {
        return `
      <div class="reports-panel-stack">
        <div class="reports-active-jobs-header">
          <div class="reports-active-jobs-heading">
            <strong>Jobs</strong>
            <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
          </div>
        </div>
        <div class="reports-terminal-empty">No active follow jobs right now.</div>
      </div>
    `;
      }
      return `
    <div class="reports-panel-stack">
      <div class="reports-active-jobs-header">
        <div class="reports-active-jobs-heading">
          <strong>Jobs</strong>
          <span class="${summaryClassNames}">${escapeHTML(buildFollowJobsSummaryText(snapshot))}</span>
        </div>
        <button
          type="button"
          class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger"
          data-follow-cancel-all="1"
          ${snapshot.canCancelAll && !snapshot.offline ? "" : "disabled"}
        >Cancel all</button>
      </div>
      <div class="reports-active-jobs-grid">
        ${activeJobs.map((job) => {
          const createdLabel = formatCompactDateTime(job.createdAtMs || job.updatedAtMs);
          const launchUrl = job.launchSignature ? `https://solscan.io/tx/${encodeURIComponent(job.launchSignature)}` : "";
          return `
            <article class="reports-launch-card reports-active-job-card">
              <div class="reports-action-head">
                <div>
                  <strong class="reports-launch-card-title">${escapeHTML(`${job.launchpad || "launch"} follow job`)}</strong>
                  <div class="reports-launch-card-subtitle">${escapeHTML(createdLabel ? `Created ${createdLabel}` : `Trace ${shortAddress(job.traceId || "")}`)}</div>
                </div>
                <span class="reports-state-badge ${followStateBadgeTone(job.state)}">${escapeHTML(formatFollowStateLabel(job.state))}</span>
              </div>
              <div class="reports-launch-card-chip-row">
                <span class="reports-launch-card-chip">${escapeHTML(job.launchpad || "launch")}</span>
                <span class="reports-launch-card-chip">${escapeHTML(job.quoteAsset || "sol")}</span>
                ${job.cancelRequested ? '<span class="reports-launch-card-chip">cancel requested</span>' : ""}
              </div>
              ${buildActiveJobLaunchRouteMarkup(job)}
              <div class="reports-active-job-meta-grid">
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Wallet</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.selectedWalletKey || "-")}</span>
                </div>
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Mint</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.mint || "-")}</span>
                </div>
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Trace</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.traceId || "-")}</span>
                </div>
                <div class="reports-launch-card-detail-row">
                  <span class="reports-launch-card-detail-key">Launch</span>
                  <span class="reports-launch-card-detail-value">${escapeHTML(job.launchSignature || "-")}</span>
                </div>
              </div>
              <div class="reports-launch-card-section">
                <div class="reports-launch-card-label">Progress</div>
                <div class="reports-launch-card-copy">${escapeHTML(summarizeFollowJobProgress(job))}</div>
                <div class="reports-active-job-action-list">
                  ${(Array.isArray(job.actions) ? job.actions : []).map((action) => `
                    <div class="reports-active-job-action">
                      <div class="reports-active-job-action-copy">
                        <strong>${escapeHTML(formatFollowStateLabel(action.kind || "action"))}</strong>
                        <span>${escapeHTML(buildFollowActionSubtitle(action) || "No extra details.")}</span>
                        ${action && action.watcherFallbackReason ? `<span>${escapeHTML(String(action.watcherFallbackReason))}</span>` : ""}
                        ${buildActiveJobActionRouteMarkup(action, job)}
                      </div>
                      <span class="reports-state-badge ${followStateBadgeTone(action.state)}">${escapeHTML(formatFollowStateLabel(action.state))}</span>
                    </div>
                  `).join("")}
                </div>
              </div>
              ${job.lastError ? `<div class="reports-callout is-bad">${escapeHTML(job.lastError)}</div>` : ""}
              <div class="reports-launch-card-footer">
                ${launchUrl ? `<a class="reports-inline-link" href="${escapeHTML(launchUrl)}" target="_blank" rel="noreferrer">Open launch tx</a>` : '<span class="reports-launch-card-copy">Launch signature not available yet.</span>'}
                <div class="reports-launch-card-actions">
                  <button
                    type="button"
                    class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger"
                    data-follow-cancel-trace-id="${escapeHTML(job.traceId || "")}"
                    ${job.cancelRequested || snapshot.offline ? "disabled" : ""}
                  >Cancel</button>
                </div>
              </div>
            </article>
          `;
        }).join("")}
      </div>
    </div>
  `;
    }

    function buildFollowJobsSummaryText(snapshot = getFollowStatusSnapshot()) {
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

    function logLevelBadgeTone(level) {
      const normalized = String(level || "").trim().toLowerCase();
      if (normalized === "error") return "is-bad";
      if (normalized === "warn" || normalized === "warning") return "is-warn";
      return "is-good";
    }

    function formatActiveLogLevel(level) {
      const normalized = String(level || "").trim().toLowerCase();
      if (!normalized) return "INFO";
      return normalized.toUpperCase();
    }

    function stringifyActiveLogContext(context) {
      if (context == null) return "";
      try {
        return JSON.stringify(context, null, 2);
      } catch (_error) {
        return String(context);
      }
    }

    function summarizeActiveLogContext(context) {
      if (context == null || typeof context !== "object") return "";
      const entries = Object.entries(context)
        .filter(([, value]) => value == null || ["string", "number", "boolean"].includes(typeof value))
        .slice(0, 3)
        .map(([key, value]) => `${key}: ${String(value)}`);
      return entries.join(" | ");
    }

    function buildActiveLogsMarkup() {
      const state = reportsState();
      const activeLogsView = normalizeActiveLogsView(state.activeLogsView);
      const logsState = state.activeLogs && typeof state.activeLogs === "object"
        ? state.activeLogs
        : { live: [], errors: [], error: "", updatedAtMs: 0 };
      const logs = Array.isArray(logsState[activeLogsView]) ? logsState[activeLogsView] : [];
      const updatedLabel = logsState.updatedAtMs ? formatCompactDateTime(logsState.updatedAtMs) : "";
      return `
    <div class="reports-panel-stack">
      <div class="reports-active-jobs-header">
        <div class="reports-active-jobs-heading">
          <strong>Logs</strong>
          <span class="reports-follow-summary ${logsState.error ? "is-issues" : logs.length ? "is-active" : ""}">
            ${escapeHTML(
              logsState.error
                ? logsState.error
                : `${logs.length} ${activeLogsView === "errors" ? "saved error" : "live log"} entr${logs.length === 1 ? "y" : "ies"}${updatedLabel ? ` | Updated ${updatedLabel}` : ""}`
            )}
          </span>
        </div>
        <div class="reports-terminal-tabs reports-active-logs-tabs">
          <button
            type="button"
            class="reports-terminal-tab${activeLogsView === "live" ? " active" : ""}"
            data-active-logs-view="live"
          >Live Logs</button>
          <button
            type="button"
            class="reports-terminal-tab${activeLogsView === "errors" ? " active" : ""}"
            data-active-logs-view="errors"
          >Errors</button>
        </div>
      </div>
      ${logsState.error ? `<div class="reports-callout is-bad">${escapeHTML(logsState.error)}</div>` : ""}
      ${logs.length ? `
        <div class="reports-active-logs-list">
          ${logs.map((entry) => {
            const timestamp = formatCompactDateTime(entry && entry.timestampMs);
            const level = formatActiveLogLevel(entry && entry.level);
            const source = String(entry && entry.source || "engine").trim() || "engine";
            const context = stringifyActiveLogContext(entry && entry.context);
            const contextSummary = summarizeActiveLogContext(entry && entry.context);
            const message = String(entry && entry.message || "No message recorded.");
            return `
              <article class="reports-active-log-entry">
                <div class="reports-active-log-row">
                  <span class="reports-state-badge ${logLevelBadgeTone(level)}">${escapeHTML(level)}</span>
                  <span class="reports-active-log-time">${escapeHTML(timestamp || "Unknown time")}</span>
                  <strong class="reports-active-log-source">${escapeHTML(source)}</strong>
                  <span class="reports-active-log-message">${escapeHTML(message)}</span>
                  ${contextSummary ? `<span class="reports-active-log-context-summary">${escapeHTML(contextSummary)}</span>` : ""}
                  ${entry && entry.persisted ? '<span class="reports-launch-card-chip">saved</span>' : ""}
                </div>
                ${context ? `
                  <details class="reports-active-log-details">
                    <summary>View raw details</summary>
                    <pre class="reports-active-log-context">${escapeHTML(context)}</pre>
                  </details>
                ` : ""}
              </article>
            `;
          }).join("")}
        </div>
      ` : '<div class="reports-terminal-empty">No log entries recorded yet.</div>'}
    </div>
  `;
    }

    function buildReportsLaunchesMarkup() {
      const state = reportsState();
      if (!state.launches.length) {
        return '<div class="reports-terminal-empty">No deployed launches found yet.</div>';
      }
      return `
    <div class="reports-launches-grid">
      ${state.launches.map((launch) => {
        const title = launch.title || "Unknown launch";
        const symbol = launch.symbol || "LAUNCH";
        const activeFollowJob = activeFollowJobForTraceId(launch.traceId);
        const followState = activeFollowJob && activeFollowJob.state
          ? String(activeFollowJob.state)
          : String(launch.followJob && launch.followJob.state || "").trim();
        const imageMarkup = launch.imageUrl
          ? `<img src="${escapeHTML(launch.imageUrl)}" alt="${escapeHTML(title)}" class="reports-launch-card-image">`
          : `<span class="reports-launch-card-image-fallback">${escapeHTML(symbol.slice(0, 4).toUpperCase())}</span>`;
        const solscanUrl = launchSolscanUrl(launch);
        return `
          <article class="reports-launch-card">
            <div class="reports-launch-card-head">
              ${imageMarkup}
              <div>
                <strong class="reports-launch-card-title">${escapeHTML(title)}</strong>
                <span class="reports-launch-card-subtitle">${escapeHTML(symbol)}</span>
                <span class="reports-launch-card-subtitle">${escapeHTML(formatLaunchHistoryDisplayTime(launch.entry) || "Unknown time")}</span>
              </div>
            </div>
            ${launch.report && launch.report.mint ? `
              <button
                type="button"
                class="reports-launch-card-ca"
                data-copy-value="${escapeHTML(launch.report.mint)}"
                title="Copy contract address"
              >
                <span class="reports-launch-card-ca-label">CA</span>
                <span class="reports-launch-card-ca-value">${escapeHTML(launch.report.mint)}</span>
              </button>
            ` : ""}
            <div class="reports-launch-card-chip-row">
              <span class="reports-launch-card-chip">${escapeHTML(launch.report && launch.report.launchpad ? launch.report.launchpad : "launch")}</span>
              <span class="reports-launch-card-chip">${escapeHTML(launch.report && launch.report.mode ? launch.report.mode : "regular")}</span>
              ${launch.followJob && launch.followJob.quoteAsset ? `<span class="reports-launch-card-chip">${escapeHTML(launch.followJob.quoteAsset)}</span>` : ""}
              ${followState ? `<span class="reports-launch-card-chip">${escapeHTML(`follow ${followState}`)}</span>` : ""}
            </div>
            <div class="reports-launch-card-section">
              <div class="reports-launch-card-label">Settings</div>
              <div class="reports-launch-card-copy">${escapeHTML(buildLaunchHistorySettingsText(launch) || "No settings recorded.")}</div>
            </div>
            <div class="reports-launch-card-section">
              <div class="reports-launch-card-label">Buy Amounts</div>
              ${buildLaunchHistoryBuyAmountsMarkup(launch)}
            </div>
            <div class="reports-launch-card-footer">
              ${solscanUrl ? `<a class="reports-inline-link" href="${escapeHTML(solscanUrl)}" target="_blank" rel="noreferrer">Open in Solscan</a>` : '<span class="reports-launch-card-copy">No signature recorded.</span>'}
              <div class="reports-launch-card-actions">
                ${activeFollowJob ? `<button type="button" class="preset-chip compact reports-terminal-chip reports-terminal-chip-danger" data-follow-cancel-trace-id="${escapeHTML(launch.traceId)}">Cancel</button>` : ""}
                <button type="button" class="preset-chip compact reports-terminal-chip" data-report-reuse-id="${escapeHTML(launch.id)}">Reuse</button>
                <button type="button" class="preset-chip compact reports-terminal-chip active" data-report-relaunch-id="${escapeHTML(launch.id)}">Relaunch</button>
              </div>
            </div>
          </article>
        `;
      }).join("")}
    </div>
  `;
    }

    function buildReportsTerminalOutputMarkup() {
      const state = reportsState();
      const view = normalizeReportsTerminalView(state.view);
      if (view === "launches") {
        return `<div class="reports-terminal-content">${buildReportsLaunchesMarkup()}</div>`;
      }
      if (view === "active-jobs") {
        return `<div class="reports-terminal-content">${buildReportsActiveJobsMarkup()}</div>`;
      }
      if (view === "active-logs") {
        return `<div class="reports-terminal-content">${buildActiveLogsMarkup()}</div>`;
      }
      const payload = currentReportsTerminalPayload();
      const tab = normalizeReportsTerminalTab(state.activeTab);
      const tabs = [
        { id: "overview", label: "Overview" },
        { id: "actions", label: "Actions" },
        { id: "benchmarks", label: "Benchmarks" },
        { id: "raw", label: "Raw" },
      ];
      const fallbackMessage = state.activeText || "Structured report data is unavailable for this entry.";
      const content = !payload && tab !== "raw"
        ? `<div class="reports-callout">${escapeHTML(fallbackMessage)}</div>`
        : tab === "actions"
          ? buildReportsActionsMarkup()
          : tab === "benchmarks"
            ? buildReportsBenchmarksMarkup()
            : tab === "raw"
              ? buildReportsRawMarkup()
              : buildReportsOverviewMarkup();
      return `
    <div class="reports-terminal-tabs">
      ${tabs.map((item) => `
        <button
          type="button"
          class="reports-terminal-tab${tab === item.id ? " active" : ""}"
          data-report-tab="${item.id}"
        >
          ${escapeHTML(item.label)}
        </button>
      `).join("")}
      <span class="reports-terminal-tabs-spacer"></span>
      <button
        type="button"
        class="reports-terminal-tab reports-terminal-tab-icon"
        data-benchmark-popout="1"
        title="Open benchmark popout"
        aria-label="Open benchmark popout"
        ${payload ? "" : "disabled"}
      >&#x29C9;</button>
    </div>
    <div class="reports-terminal-content">${content}</div>
  `;
    }

    function renderReportsTerminalOutput() {
      if (!reportsTerminalOutput) return;
      syncReportsTerminalChrome();
      const markup = buildReportsTerminalOutputMarkup();
      writeCachedHTML("reportsOutput", reportsTerminalOutput, markup);
      renderBenchmarksPopoutModal();
    }

    function renderReportsTerminalList() {
      if (!reportsTerminalList) return;
      const state = reportsState();
      syncReportsTerminalChrome();
      if (["launches", "active-jobs", "active-logs"].includes(normalizeReportsTerminalView(state.view))) {
        writeCachedHTML("reportsList", reportsTerminalList, "");
        return;
      }
      if (!state.entries.length) {
        writeCachedHTML("reportsList", reportsTerminalList, '<div class="reports-terminal-empty">No persisted reports found yet.</div>');
        return;
      }
      const markup = state.entries.map((entry) => `
    <button
      type="button"
      class="reports-terminal-item${entry.id === state.activeId ? " active" : ""}"
      data-report-id="${escapeHTML(entry.id)}"
    >
      <span class="reports-terminal-item-title">${escapeHTML(String(entry.action || "unknown"))}</span>
      <span class="reports-terminal-item-meta">${escapeHTML(String(entry.mint || entry.fileName || "Unknown mint"))}</span>
      <span class="reports-terminal-item-meta">${escapeHTML(describeReportEntry(entry) || "No metadata")}</span>
    </button>
  `).join("");
      writeCachedHTML("reportsList", reportsTerminalList, markup);
    }

    return {
      applyFrozenBenchmarkSnapshot,
      captureFrozenBenchmarkSnapshot,
      describeReportEntry,
      normalizeActiveLogsView,
      normalizeReportsTerminalTab,
      normalizeReportsTerminalView,
      renderBenchmarksPopoutModal,
      renderReportsTerminalList,
      renderReportsTerminalOutput,
      reportsTerminalMetaText,
      syncReportsTerminalChrome,
      syncReportsTerminalLayoutHeight,
    };
  }

  global.LaunchDeckReportsPresenters = {
    create: createReportsPresenters,
  };
})(window);
