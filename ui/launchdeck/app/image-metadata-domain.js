(function initLaunchDeckImageMetadataDomain(global) {
  function createImageMetadataDomain(config) {
    const {
      elements = {},
      constants = {},
      state = {},
      helpers = {},
      actions = {},
    } = config || {};

    const {
      imagePreview,
      imageEmpty,
      imageStatus,
      imagePath,
      metadataUri,
    } = elements;

    const {
      metadataPreuploadDebounceMs = 500,
    } = constants;

    const {
      getUploadedImage = () => null,
      getImageLibraryState = () => ({
        images: [],
        categories: [],
        activeImageId: "",
      }),
      getMetadataUploadState = () => ({
        debounceTimer: null,
        inFlightPromise: null,
        inFlightFingerprint: "",
        completedFingerprint: "",
        latestScheduledFingerprint: "",
        lastCanPreupload: false,
        staleWhileUploading: false,
        autoRetryFailures: 0,
        autoRetryDisabled: false,
        lastAlertedWarning: "",
      }),
    } = state;

    const {
      readForm = () => ({}),
      canPreuploadMetadata = () => false,
      hasFreshPreuploadedMetadata = () => false,
      launchpadHandlesOwnMetadata = () => false,
      metadataFingerprintFromForm = () => "",
    } = helpers;

    const {
      fetchImageLibrary = async () => {},
      showImageDetailsModal = () => {},
      setSelectedImageInFeature = () => {},
    } = actions;

    function metadataUploadState() {
      return getMetadataUploadState();
    }

    function imageLibraryState() {
      return getImageLibraryState();
    }

    function setImagePreview(previewUrl) {
      if (!imagePreview || !imageEmpty) return;
      if (!previewUrl) {
        imagePreview.removeAttribute("src");
        imagePreview.hidden = true;
        imageEmpty.hidden = false;
        return;
      }
      imagePreview.src = previewUrl;
      imagePreview.hidden = false;
      imageEmpty.hidden = true;
    }

    function setSelectedImage(image) {
      setSelectedImageInFeature(image);
    }

    function hasAttachedImage() {
      return Boolean(
        getUploadedImage()
        || (metadataUri && metadataUri.value)
        || (imagePreview && !imagePreview.hidden && imagePreview.src),
      );
    }

    function clearMetadataUploadDebounce() {
      const uploadState = metadataUploadState();
      if (!uploadState.debounceTimer) return;
      global.clearTimeout(uploadState.debounceTimer);
      uploadState.debounceTimer = null;
    }

    function clearMetadataUploadCache({ clearInput = false } = {}) {
      const uploadState = metadataUploadState();
      clearMetadataUploadDebounce();
      uploadState.completedFingerprint = "";
      uploadState.latestScheduledFingerprint = "";
      uploadState.lastCanPreupload = false;
      uploadState.autoRetryFailures = 0;
      uploadState.autoRetryDisabled = false;
      uploadState.lastAlertedWarning = "";
      if (clearInput && metadataUri) {
        metadataUri.value = "";
      }
    }

    function markMetadataUploadDirty() {
      const formValues = readForm();
      if (hasFreshPreuploadedMetadata(formValues)) return;
      const uploadState = metadataUploadState();
      uploadState.completedFingerprint = "";
      uploadState.autoRetryFailures = 0;
      uploadState.autoRetryDisabled = false;
      uploadState.lastAlertedWarning = "";
      if (metadataUri) {
        metadataUri.value = "";
      }
    }

    function currentMetadataRetryDelayMs() {
      return metadataUploadState().autoRetryFailures >= 2
        ? metadataPreuploadDebounceMs * 2
        : metadataPreuploadDebounceMs;
    }

    function surfaceMetadataWarning(warning) {
      const message = String(warning || "").trim();
      if (!message) return;
      if (imageStatus) imageStatus.textContent = message;
      const uploadState = metadataUploadState();
      if (uploadState.lastAlertedWarning === message) return;
      uploadState.lastAlertedWarning = message;
      global.alert(message);
    }

    async function uploadMetadataForCurrentForm(source = "background") {
      const formValues = readForm();
      if (launchpadHandlesOwnMetadata(formValues)) {
        return "";
      }
      if (!canPreuploadMetadata(formValues)) {
        if (source === "send") {
          throw new Error("Select an image and fill in both name and ticker before deploying.");
        }
        return "";
      }
      const fingerprint = metadataFingerprintFromForm(formValues);
      if (hasFreshPreuploadedMetadata(formValues)) {
        return metadataUri ? metadataUri.value : "";
      }
      const uploadState = metadataUploadState();
      if (uploadState.inFlightPromise) {
        if (uploadState.inFlightFingerprint === fingerprint) {
          return uploadState.inFlightPromise;
        }
        uploadState.staleWhileUploading = true;
        uploadState.latestScheduledFingerprint = fingerprint;
        if (source !== "send") {
          await uploadState.inFlightPromise.catch(() => "");
          if (hasFreshPreuploadedMetadata(readForm())) {
            return metadataUri ? metadataUri.value : "";
          }
        }
      }

      uploadState.inFlightFingerprint = fingerprint;
      uploadState.latestScheduledFingerprint = fingerprint;
      if (imageStatus) {
        imageStatus.textContent = source === "send" ? "Preparing metadata..." : "Uploading metadata...";
      }
      if (imagePath) imagePath.textContent = "";

      const request = global.fetch("/api/metadata/upload", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          form: formValues,
        }),
      })
        .then(async (response) => {
          const payload = await response.json();
          if (!response.ok || !payload.ok) {
            throw new Error(payload.error || "Metadata upload failed.");
          }
          const liveForm = readForm();
          const liveFingerprint = canPreuploadMetadata(liveForm) ? metadataFingerprintFromForm(liveForm) : "";
          if (liveFingerprint === fingerprint) {
            if (metadataUri) metadataUri.value = payload.metadataUri || "";
            uploadState.completedFingerprint = fingerprint;
            uploadState.autoRetryFailures = 0;
            uploadState.autoRetryDisabled = false;
            if (imageStatus) {
              imageStatus.textContent = payload.metadataWarning ? payload.metadataWarning : "Metadata ready.";
            }
          } else {
            uploadState.staleWhileUploading = true;
          }
          surfaceMetadataWarning(payload.metadataWarning);
          return payload.metadataUri || "";
        })
        .catch((error) => {
          if (source === "background") {
            uploadState.autoRetryFailures += 1;
            uploadState.autoRetryDisabled = uploadState.autoRetryFailures >= 4;
            if (imageStatus) {
              if (uploadState.autoRetryDisabled) {
                imageStatus.textContent = `${error.message} Auto retry paused until deploy.`;
              } else {
                const nextDelayMs = currentMetadataRetryDelayMs();
                imageStatus.textContent = `${error.message} Retrying in ${nextDelayMs}ms.`;
              }
            }
          }
          throw error;
        })
        .finally(() => {
          if (uploadState.inFlightPromise === request) {
            uploadState.inFlightPromise = null;
            uploadState.inFlightFingerprint = "";
          }
          if (
            uploadState.staleWhileUploading
            && uploadState.latestScheduledFingerprint
            && uploadState.latestScheduledFingerprint !== uploadState.completedFingerprint
          ) {
            uploadState.staleWhileUploading = false;
            scheduleMetadataPreupload({ immediate: true });
          } else {
            uploadState.staleWhileUploading = false;
          }
        });

      uploadState.inFlightPromise = request;
      return request;
    }

    function scheduleMetadataPreupload({ immediate = false } = {}) {
      clearMetadataUploadDebounce();
      if (!getUploadedImage()) return;
      const formValues = readForm();
      const uploadState = metadataUploadState();
      if (launchpadHandlesOwnMetadata(formValues)) {
        uploadState.lastCanPreupload = false;
        uploadState.completedFingerprint = "";
        uploadState.latestScheduledFingerprint = "";
        uploadState.autoRetryFailures = 0;
        uploadState.autoRetryDisabled = false;
        if (imageStatus) imageStatus.textContent = "Bags uploads metadata during launch.";
        if (imagePath) imagePath.textContent = "";
        return;
      }
      if (!canPreuploadMetadata(formValues)) {
        uploadState.lastCanPreupload = false;
        markMetadataUploadDirty();
        if (imageStatus) imageStatus.textContent = "Waiting for name and ticker to pre-upload metadata.";
        if (imagePath) imagePath.textContent = "";
        return;
      }
      const becameReady = !uploadState.lastCanPreupload;
      uploadState.lastCanPreupload = true;
      const fingerprint = metadataFingerprintFromForm(formValues);
      uploadState.latestScheduledFingerprint = fingerprint;
      if (hasFreshPreuploadedMetadata(formValues)) return;
      if (uploadState.inFlightPromise && uploadState.inFlightFingerprint === fingerprint) {
        return;
      }
      if (uploadState.autoRetryDisabled) {
        if (imageStatus) imageStatus.textContent = "Metadata auto retry paused until deploy.";
        if (imagePath) imagePath.textContent = "";
        return;
      }
      const delayMs = immediate || becameReady ? 0 : currentMetadataRetryDelayMs();
      uploadState.debounceTimer = global.setTimeout(() => {
        uploadState.debounceTimer = null;
        uploadMetadataForCurrentForm("background").catch(() => {});
      }, delayMs);
    }

    async function ensureMetadataReadyForAction(action) {
      const formValues = readForm();
      if (!formValues.imageFileName) return;
      if (launchpadHandlesOwnMetadata(formValues)) {
        if (!String(formValues.name || "").trim() || !String(formValues.symbol || "").trim()) {
          throw new Error(
            action === "send"
              ? "Select an image and fill in both name and ticker before deploying."
              : `Select an image and fill in both name and ticker before ${action}.`,
          );
        }
        return;
      }
      if (hasFreshPreuploadedMetadata(formValues)) return;
      if (canPreuploadMetadata(formValues)) {
        await uploadMetadataForCurrentForm(action === "send" ? "send" : "action");
        return;
      }
      const uploadState = metadataUploadState();
      if (uploadState.inFlightPromise) {
        await uploadState.inFlightPromise.catch(() => "");
        if (hasFreshPreuploadedMetadata(readForm())) {
          return;
        }
      }
      throw new Error(
        action === "send"
          ? "Select an image and fill in both name and ticker before deploying."
          : `Select an image and fill in both name and ticker before ${action}.`,
      );
    }

    async function uploadSelectedImage(file) {
      const formData = new global.FormData();
      formData.append("file", file, file.name);
      const response = await global.fetch("/api/upload-image", {
        method: "POST",
        body: formData,
      });

      const payload = await response.json();
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Image upload failed.");
      }

      if (imageStatus) imageStatus.textContent = "Image uploaded to library.";
      if (imagePath) imagePath.textContent = "";
      imageLibraryState().activeImageId = payload.id || "";
      try {
        await fetchImageLibrary();
      } catch (error) {
        if (imageStatus) imageStatus.textContent = error.message;
      }
      showImageDetailsModal(payload, { isNewUpload: true });
    }

    async function ensureTestImageSelected() {
      let availableImages = Array.isArray(imageLibraryState().images) ? [...imageLibraryState().images] : [];
      if (!availableImages.length) {
        try {
          const response = await global.fetch("/api/images");
          const payload = await response.json();
          if (response.ok && payload.ok) {
            imageLibraryState().images = Array.isArray(payload.images) ? payload.images : [];
            imageLibraryState().categories = Array.isArray(payload.categories) ? payload.categories : [];
            availableImages = imageLibraryState().images;
          }
        } catch (_error) {
          // Fall through when the library fetch fails.
        }
      }

      const preferred =
        availableImages.find((entry) => entry && entry.fileName === "solana-mark.png")
        || availableImages.find((entry) => entry && entry.previewUrl === "/images/solana-mark.png")
        || availableImages[0];

      if (!preferred) return false;
      imageLibraryState().activeImageId = preferred.id || "";
      setSelectedImage(preferred);
      return true;
    }

    function resolvePreviewUrl(image) {
      return String(image && image.previewUrl || "").trim()
        || (image && image.fileName ? `/uploads/${encodeURIComponent(image.fileName)}` : "");
    }

    async function selectImportedImage(image) {
      if (!image) return;
      const importedPreviewUrl = resolvePreviewUrl(image);
      imageLibraryState().activeImageId = image.id || "";
      setSelectedImage(image);
      if (importedPreviewUrl) {
        setImagePreview(importedPreviewUrl);
      }
      try {
        await fetchImageLibrary();
        const refreshedImportedImage = imageLibraryState().images.find((entry) => entry.id === imageLibraryState().activeImageId);
        if (refreshedImportedImage) {
          setSelectedImage(refreshedImportedImage);
        } else if (importedPreviewUrl) {
          setImagePreview(importedPreviewUrl);
        }
      } catch (_error) {
        // Keep the imported image selected even if the library refresh fails.
        if (importedPreviewUrl) {
          setImagePreview(importedPreviewUrl);
        }
      }
    }

    function restoreLaunchHistoryImage(launch) {
      const libraryState = imageLibraryState();
      setSelectedImage(null);
      libraryState.activeImageId = "";
      clearMetadataUploadCache({ clearInput: true });
      if (metadataUri) metadataUri.value = launch && launch.metadataUri ? launch.metadataUri : "";
      setImagePreview(launch && launch.imageUrl ? launch.imageUrl : "");
      if (imageStatus) {
        imageStatus.textContent = launch && launch.metadataUri ? "Restored image from saved launch metadata." : "";
      }
      if (imagePath) imagePath.textContent = "";
    }

    return {
      canPreuploadMetadata,
      clearMetadataUploadCache,
      ensureMetadataReadyForAction,
      ensureTestImageSelected,
      hasAttachedImage,
      hasFreshPreuploadedMetadata,
      launchpadHandlesOwnMetadata,
      markMetadataUploadDirty,
      metadataFingerprintFromForm,
      restoreLaunchHistoryImage,
      scheduleMetadataPreupload,
      selectImportedImage,
      setImagePreview,
      setSelectedImage,
      surfaceMetadataWarning,
      uploadSelectedImage,
    };
  }

  global.LaunchDeckImageMetadataDomain = {
    create: createImageMetadataDomain,
  };
})(window);
