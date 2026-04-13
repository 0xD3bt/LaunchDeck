(function initLaunchDeckFeeRouting(global) {
  const SUPPORTED_SOCIAL_RECIPIENT_TYPES = ["github", "twitter", "x", "kick", "tiktok"];

  function createFeeRouting(config) {
    const {
      getLaunchpad,
      normalizeLaunchpad,
      escapeHTML,
    } = config;

    function normalizeRecipientType(type, { allowAgent = false } = {}) {
      const normalized = String(type || "").trim().toLowerCase();
      if (allowAgent && normalized === "agent") return "agent";
      if (normalized === "wallet") return "wallet";
      if (normalized === "x") return "twitter";
      return SUPPORTED_SOCIAL_RECIPIENT_TYPES.includes(normalized) ? normalized : "wallet";
    }

    function launchpadSupportsExtendedSocialRecipients(launchpad = getLaunchpad()) {
      return normalizeLaunchpad(launchpad) === "bagsapp";
    }

    function supportedRecipientTypesForLaunchpad(launchpad = getLaunchpad(), { allowAgent = false } = {}) {
      const types = ["wallet", "github"];
      if (launchpadSupportsExtendedSocialRecipients(launchpad)) {
        types.push("twitter", "kick", "tiktok");
      }
      if (allowAgent) {
        types.unshift("agent");
      }
      return types;
    }

    function isRecipientTypeSupportedForLaunchpad(type, launchpad = getLaunchpad(), { allowAgent = false } = {}) {
      return supportedRecipientTypesForLaunchpad(launchpad, { allowAgent }).includes(normalizeRecipientType(type, { allowAgent }));
    }

    function isSocialRecipientType(type) {
      return SUPPORTED_SOCIAL_RECIPIENT_TYPES.includes(normalizeRecipientType(type));
    }

    function recipientTypeLabel(type) {
      switch (normalizeRecipientType(type)) {
        case "github":
          return "GitHub";
        case "twitter":
        case "x":
          return "X";
        case "kick":
          return "Kick";
        case "tiktok":
          return "TikTok";
        default:
          return "Wallet";
      }
    }

    function recipientTypeIconSrc(type) {
      switch (normalizeRecipientType(type)) {
        case "github":
          return "/images/recipient-github.png";
        case "twitter":
        case "x":
          return "/images/recipient-x.png";
        case "kick":
          return "/images/recipient-kick.png";
        case "tiktok":
          return "/images/recipient-tiktok.png";
        default:
          return "/images/recipient-wallet.png";
      }
    }

    function recipientTypeIconMarkup(type) {
      const normalized = normalizeRecipientType(type);
      return `<span class="recipient-platform-icon recipient-platform-icon-${normalized}" aria-hidden="true"><img class="recipient-platform-icon-image" src="${recipientTypeIconSrc(normalized)}" alt=""></span>`;
    }

    function recipientTypeTabsMarkup() {
      return ["wallet", "github", "twitter", "kick", "tiktok"]
        .map((type) => `
      <button
        type="button"
        class="recipient-type-tab"
        data-type="${type}"
        title="${escapeHTML(recipientTypeLabel(type))}"
        aria-label="${escapeHTML(recipientTypeLabel(type))}"
      >
        ${recipientTypeIconMarkup(type)}
        <span class="recipient-type-tab-label">${escapeHTML(recipientTypeLabel(type))}</span>
      </button>
    `)
        .join("");
    }

    function syncRecipientTypeTabVisibility(row) {
      if (!row) return;
      row.querySelectorAll(".recipient-type-tab").forEach((button) => {
        if (!button.dataset.type) return;
        const allowed = isRecipientTypeSupportedForLaunchpad(button.dataset.type, getLaunchpad(), {
          allowAgent: row.dataset.locked === "true",
        });
        button.hidden = !allowed && button.dataset.type !== row.dataset.type;
      });
    }

    function recipientTargetPlaceholder(type) {
      switch (normalizeRecipientType(type)) {
        case "github":
          return "GitHub username or user id";
        case "twitter":
        case "x":
          return "X username";
        case "kick":
          return "Kick username";
        case "tiktok":
          return "TikTok username";
        default:
          return "Wallet address";
      }
    }

    function recipientDisplayValueFromEntry(entry) {
      if (!entry) return "";
      return isSocialRecipientType(entry.type)
        ? String(entry.githubUsername || entry.githubUserId || "").trim().replace(/^@+/, "")
        : String(entry.address || "").trim();
    }

    return {
      isRecipientTypeSupportedForLaunchpad,
      isSocialRecipientType,
      launchpadSupportsExtendedSocialRecipients,
      normalizeRecipientType,
      recipientDisplayValueFromEntry,
      recipientTargetPlaceholder,
      recipientTypeIconMarkup,
      recipientTypeIconSrc,
      recipientTypeLabel,
      recipientTypeTabsMarkup,
      supportedRecipientTypesForLaunchpad,
      syncRecipientTypeTabVisibility,
    };
  }

  global.LaunchDeckFeeRouting = {
    createFeeRouting,
  };
})(window);
