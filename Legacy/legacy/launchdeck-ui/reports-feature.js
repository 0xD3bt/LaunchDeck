(function initReportsFeature(global) {
  function createReportsFeature(config) {
    const {
      elements,
      storage,
      requestStates,
      renderCache,
      state,
      getResizeState,
      setResizeState,
      constants,
      schedulePopoutAutosize,
      refreshOnVisible,
      renderOutput,
      renderList,
      loadEntry,
      refreshReports,
      getView,
      setView,
      reuseEntry,
      relaunchEntry,
      normalizeTab,
      shortenAddress,
      openPopoutWindow,
    } = config;

    const {
      reportsTerminalSection,
      reportsTerminalList,
      reportsTerminalOutput,
      reportsTerminalMeta,
      reportsTerminalResizeHandle,
      openPopoutButton,
      toggleOutputButton,
      toggleReportsButton,
      reportsRefreshButton,
      reportsTransactionsButton,
      reportsLaunchesButton,
      reportsActiveJobsButton,
      reportsActiveLogsButton,
    } = elements;

    const {
      visibilityKey,
      listWidthKey,
    } = storage;

    const {
      defaultListWidth,
      minListWidth,
      maxListWidth,
    } = constants;

    let eventsBound = false;

    function clampListWidth(value) {
      const numeric = Number(value);
      if (!Number.isFinite(numeric)) return defaultListWidth;
      return Math.min(maxListWidth, Math.max(minListWidth, Math.round(numeric)));
    }

    function getCurrentListWidth() {
      if (!reportsTerminalSection) return defaultListWidth;
      const inlineWidth = reportsTerminalSection.style.getPropertyValue("--reports-terminal-list-width");
      if (inlineWidth) {
        const parsedInlineWidth = Number.parseInt(inlineWidth, 10);
        if (Number.isFinite(parsedInlineWidth)) return clampListWidth(parsedInlineWidth);
      }
      return defaultListWidth;
    }

    function getStoredListWidth() {
      try {
        return clampListWidth(window.localStorage.getItem(listWidthKey));
      } catch (_error) {
        return defaultListWidth;
      }
    }

    function setListWidth(width, { persist = true } = {}) {
      if (!reportsTerminalSection) return;
      const normalized = clampListWidth(width);
      reportsTerminalSection.style.setProperty("--reports-terminal-list-width", `${normalized}px`);
      if (reportsTerminalResizeHandle) {
        reportsTerminalResizeHandle.setAttribute("aria-valuemin", String(minListWidth));
        reportsTerminalResizeHandle.setAttribute("aria-valuemax", String(maxListWidth));
        reportsTerminalResizeHandle.setAttribute("aria-valuenow", String(normalized));
      }
      if (persist) {
        try {
          window.localStorage.setItem(listWidthKey, String(normalized));
        } catch (_error) {
          // Ignore storage failures and keep the UI functional.
        }
      }
      schedulePopoutAutosize();
    }

    function setVisible(isVisible, { persist = true } = {}) {
      document.documentElement.classList.toggle("reports-hidden", !isVisible);
      document.body.classList.toggle("reports-hidden", !isVisible);
      if (reportsTerminalSection) reportsTerminalSection.hidden = !isVisible;
      if (toggleReportsButton) {
        toggleReportsButton.classList.toggle("active", isVisible);
        toggleReportsButton.setAttribute("aria-pressed", String(isVisible));
      }
      if (isVisible) {
        refreshOnVisible().catch((error) => {
          if (reportsTerminalOutput) {
            state.activePayload = null;
            state.activeText = error.message || "Failed to load reports.";
            renderOutput();
          }
        });
      }
      if (persist) {
        try {
          window.localStorage.setItem(visibilityKey, String(isVisible));
        } catch (_error) {
          // Ignore storage failures and keep the UI functional.
        }
      }
      schedulePopoutAutosize();
    }

    function startResize(event) {
      if (!reportsTerminalSection || !reportsTerminalResizeHandle || window.innerWidth <= 680) return;
      setResizeState({
        pointerId: event.pointerId,
        startX: event.clientX,
        startWidth: getCurrentListWidth(),
      });
      reportsTerminalSection.classList.add("is-resizing");
      reportsTerminalResizeHandle.classList.add("is-active");
      reportsTerminalResizeHandle.setPointerCapture(event.pointerId);
      event.preventDefault();
    }

    function updateResize(event) {
      const resizeState = getResizeState();
      if (!resizeState) return;
      const delta = event.clientX - resizeState.startX;
      setListWidth(resizeState.startWidth + delta, { persist: false });
    }

    function finishResize() {
      const resizeState = getResizeState();
      if (!resizeState || !reportsTerminalSection || !reportsTerminalResizeHandle) return;
      const activePointerId = resizeState.pointerId;
      setResizeState(null);
      reportsTerminalSection.classList.remove("is-resizing");
      reportsTerminalResizeHandle.classList.remove("is-active");
      if (typeof activePointerId === "number" && reportsTerminalResizeHandle.hasPointerCapture(activePointerId)) {
        reportsTerminalResizeHandle.releasePointerCapture(activePointerId);
      }
      setListWidth(getCurrentListWidth());
    }

    function handleResizeKeydown(event) {
      if (!reportsTerminalSection || window.innerWidth <= 680) return;
      const step = event.shiftKey ? 40 : 20;
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        setListWidth(getCurrentListWidth() - step);
        return;
      }
      if (event.key === "ArrowRight") {
        event.preventDefault();
        setListWidth(getCurrentListWidth() + step);
        return;
      }
      if (event.key === "Home") {
        event.preventDefault();
        setListWidth(minListWidth);
        return;
      }
      if (event.key === "End") {
        event.preventDefault();
        setListWidth(maxListWidth);
        return;
      }
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        setListWidth(defaultListWidth);
      }
    }

    function bindEvents() {
      if (eventsBound) return;
      eventsBound = true;

      if (openPopoutButton) {
        openPopoutButton.addEventListener("click", openPopoutWindow);
      }
      if (toggleOutputButton) {
        toggleOutputButton.addEventListener("click", () => {
          const outputSection = document.getElementById("output-section");
          if (typeof global.setOutputSectionVisible === "function") {
            global.setOutputSectionVisible(outputSection ? outputSection.hidden : true);
          }
        });
      }
      if (toggleReportsButton) {
        toggleReportsButton.addEventListener("click", () => {
          setVisible(reportsTerminalSection ? reportsTerminalSection.hidden : true);
        });
      }
      if (reportsRefreshButton) {
        reportsRefreshButton.addEventListener("click", async () => {
          try {
            await refreshReports();
          } catch (error) {
            if (reportsTerminalOutput) {
              state.activePayload = null;
              state.activeText = error.message || "Failed to refresh reports.";
              renderOutput();
            }
          }
        });
      }
      if (reportsTransactionsButton) {
        reportsTransactionsButton.addEventListener("click", async () => {
          if (getView() === "transactions") return;
          setView("transactions");
          try {
            await refreshReports({ preserveSelection: true });
          } catch (error) {
            if (reportsTerminalOutput) {
              state.activePayload = null;
              state.activeText = error.message || "Failed to load transactions.";
              renderOutput();
            }
          }
        });
      }
      if (reportsLaunchesButton) {
        reportsLaunchesButton.addEventListener("click", async () => {
          if (getView() === "launches") return;
          setView("launches");
          try {
            await refreshReports({ preserveSelection: false });
          } catch (error) {
            if (reportsTerminalOutput) {
              state.activePayload = null;
              state.activeText = error.message || "Failed to load launches.";
              renderOutput();
            }
          }
        });
      }
      if (reportsActiveJobsButton) {
        reportsActiveJobsButton.addEventListener("click", async () => {
          if (getView() === "active-jobs") return;
          setView("active-jobs");
          try {
            await refreshReports({ preserveSelection: false });
          } catch (error) {
            if (reportsTerminalOutput) {
              state.activePayload = null;
              state.activeText = error.message || "Failed to load active jobs.";
              renderOutput();
            }
          }
        });
      }
      if (reportsActiveLogsButton) {
        reportsActiveLogsButton.addEventListener("click", async () => {
          if (getView() === "active-logs") return;
          setView("active-logs");
          try {
            await refreshReports({ preserveSelection: false });
          } catch (error) {
            if (reportsTerminalOutput) {
              state.activePayload = null;
              state.activeText = error.message || "Failed to load active logs.";
              renderOutput();
            }
          }
        });
      }
      if (reportsTerminalList) {
        reportsTerminalList.addEventListener("click", async (event) => {
          if (getView() !== "transactions") return;
          const button = event.target.closest("[data-report-id]");
          if (!button) return;
          try {
            await loadEntry(button.getAttribute("data-report-id") || "");
          } catch (error) {
            if (reportsTerminalOutput) {
              state.activePayload = null;
              state.activeText = error.message || "Failed to load report.";
              renderOutput();
            }
          }
        });
      }
      if (reportsTerminalOutput) {
        reportsTerminalOutput.addEventListener("click", async (event) => {
          const reuseButton = event.target.closest("[data-report-reuse-id]");
          if (reuseButton) {
            try {
              await reuseEntry(reuseButton.getAttribute("data-report-reuse-id") || "");
            } catch (error) {
              if (reportsTerminalOutput) {
                state.activePayload = null;
                state.activeText = error.message || "Failed to reuse saved launch.";
                renderOutput();
              }
            }
            return;
          }
          const relaunchButton = event.target.closest("[data-report-relaunch-id]");
          if (relaunchButton) {
            try {
              await relaunchEntry(relaunchButton.getAttribute("data-report-relaunch-id") || "");
            } catch (error) {
              if (reportsTerminalOutput) {
                state.activePayload = null;
                state.activeText = error.message || "Failed to relaunch from saved launch.";
                renderOutput();
              }
            }
            return;
          }
          const button = event.target.closest("[data-report-tab]");
          if (button) {
            state.activeTab = normalizeTab(button.getAttribute("data-report-tab"));
            renderOutput();
            return;
          }
          const copyTarget = event.target.closest("[data-copy-value]");
          if (!copyTarget) return;
          const value = copyTarget.getAttribute("data-copy-value") || "";
          if (!value) return;
          try {
            await navigator.clipboard.writeText(value);
            const originalText = copyTarget.textContent || "";
            if (copyTarget.__copyFeedbackTimer) {
              window.clearTimeout(copyTarget.__copyFeedbackTimer);
            }
            copyTarget.classList.add("is-copied");
            copyTarget.textContent = "Copied";
            copyTarget.__copyFeedbackTimer = window.setTimeout(() => {
              copyTarget.classList.remove("is-copied");
              copyTarget.textContent = originalText;
              copyTarget.__copyFeedbackTimer = null;
            }, 900);
          } catch (_error) {
            // Ignore clipboard failures here and keep the terminal subtitle stable.
          }
        });
      }
      if (reportsTerminalResizeHandle) {
        reportsTerminalResizeHandle.addEventListener("pointerdown", startResize);
        reportsTerminalResizeHandle.addEventListener("pointermove", updateResize);
        reportsTerminalResizeHandle.addEventListener("pointerup", finishResize);
        reportsTerminalResizeHandle.addEventListener("pointercancel", finishResize);
        reportsTerminalResizeHandle.addEventListener("dblclick", () => {
          setListWidth(defaultListWidth);
        });
        reportsTerminalResizeHandle.addEventListener("keydown", handleResizeKeydown);
      }
    }

    return {
      bindEvents,
      getCurrentListWidth,
      getStoredListWidth,
      setListWidth,
      setVisible,
    };
  }

  global.ReportsFeature = {
    create: createReportsFeature,
  };
})(window);
