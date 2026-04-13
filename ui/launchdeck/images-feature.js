(function initImagesFeature(global) {
  function createImagesFeature(config) {
    const {
      elements,
      renderCache,
      requestStates,
      getImageLibraryState,
      getActiveImageMenuId,
      setActiveImageMenuId,
      getActiveImageDetailsId,
      setActiveImageDetailsId,
      getImageDetailsTagsState,
      setImageDetailsTagsState,
      getIsEditingNewImageUpload,
      setIsEditingNewImageUpload,
      getImageCategoryModalContext,
      setImageCategoryModalContext,
      getUploadedImage,
      setUploadedImage,
      clearMetadataUploadCache,
      setImagePreview,
      scheduleMetadataPreupload,
      escapeHTML,
      fetchJsonLatest,
    } = config;

    const {
      imageStatus,
      imagePath,
      imagePreview,
      imageEmpty,
      imageLibraryModal,
      imageLibrarySearchInput,
      imageLibraryUploadButton,
      imageLibraryGrid,
      imageLibraryEmpty,
      imageCategoryChips,
      newImageCategoryButton,
      imageItemMenu,
      imageMenuFavorite,
      imageMenuEdit,
      imageMenuDelete,
      imageDetailsModal,
      imageDetailsTitle,
      imageDetailsClose,
      imageDetailsCancel,
      imageDetailsSave,
      imageDetailsName,
      imageDetailsTags,
      imageDetailsAddTag,
      imageDetailsTagList,
      imageDetailsError,
      imageDetailsCategoryRow,
      imageDetailsCategory,
      imageDetailsNewCategory,
      imageCategoryModal,
      imageCategoryClose,
      imageCategoryCancel,
      imageCategorySave,
      imageCategoryName,
      imageCategoryError,
      imageLibraryClose,
      imageInput,
      openImageLibraryButton,
    } = elements;

    let eventsBound = false;

    function resolveImagePreviewUrl(image) {
      if (!image || typeof image !== "object") return "";
      const previewUrl = String(image.previewUrl || "").trim();
      if (previewUrl) return previewUrl;
      const fileName = String(image.fileName || "").trim();
      return fileName ? `/uploads/${encodeURIComponent(fileName)}` : "";
    }

    function hideItemMenu() {
      if (!imageItemMenu) return;
      imageItemMenu.hidden = true;
      imageItemMenu.style.left = "";
      imageItemMenu.style.top = "";
      setActiveImageMenuId("");
    }

    function renderDetailsTags() {
      if (!imageDetailsTagList) return;
      const tags = getImageDetailsTagsState();
      imageDetailsTagList.innerHTML = tags.map((tag, index) => `
        <button type="button" class="image-tag-chip" data-image-tag-index="${index}">
          <span>${escapeHTML(tag)}</span>
          <span class="image-tag-chip-remove">&times;</span>
        </button>
      `).join("");
    }

    function setDetailsError(message = "") {
      if (imageDetailsError) imageDetailsError.textContent = message;
    }

    function normalizeImageTag(value) {
      return String(value || "").trim().replace(/\s+/g, " ").slice(0, 24);
    }

    function normalizeImageCategoryName(value) {
      return String(value || "").trim().replace(/\s+/g, " ").slice(0, 32);
    }

    function renderDetailsCategoryOptions(selectedCategory = "") {
      if (!imageDetailsCategory) return;
      const state = getImageLibraryState();
      const selected = normalizeImageCategoryName(selectedCategory);
      const categories = [...state.categories];
      if (selected && !categories.some((entry) => entry.toLowerCase() === selected.toLowerCase())) {
        categories.push(selected);
        categories.sort((a, b) => a.localeCompare(b));
      }
      imageDetailsCategory.innerHTML = [
        '<option value="">Uncategorized</option>',
        ...categories.map((category) => `<option value="${escapeHTML(category)}">${escapeHTML(category)}</option>`),
      ].join("");
      imageDetailsCategory.value = selected;
    }

    function addDetailTag(rawValue) {
      const value = normalizeImageTag(rawValue);
      if (!value) return false;
      const tags = getImageDetailsTagsState();
      if (tags.some((tag) => tag.toLowerCase() === value.toLowerCase())) return false;
      setImageDetailsTagsState([...tags, value]);
      renderDetailsTags();
      if (imageDetailsTags) imageDetailsTags.value = "";
      return true;
    }

    function setSelectedImage(image) {
      setUploadedImage(image || null);
      clearMetadataUploadCache({ clearInput: true });
      if (!image) {
        imageStatus.textContent = "";
        imagePath.textContent = "";
        setImagePreview("");
        return;
      }
      imageStatus.textContent = "";
      imagePath.textContent = "";
      setImagePreview(resolveImagePreviewUrl(image));
      scheduleMetadataPreupload({ immediate: true });
    }

    function renderCategoryChips() {
      if (!imageCategoryChips) return;
      const state = getImageLibraryState();
      imageCategoryChips.innerHTML = state.categories.map((category) => `
        <button type="button" class="image-category-chip${state.category === category ? " active" : ""}" data-image-category="${escapeHTML(category)}">
          ${escapeHTML(category)}
        </button>
      `).join("");
      document.querySelectorAll("[data-image-category]").forEach((button) => {
        button.classList.toggle("active", button.getAttribute("data-image-category") === state.category);
      });
    }

    function renderLibraryGrid() {
      if (!imageLibraryGrid) return;
      const state = getImageLibraryState();
      const imageTiles = state.images.map((image) => `
        <div class="image-library-item${image.id === state.activeImageId ? " active" : ""}" data-image-id="${escapeHTML(image.id)}" tabindex="0" role="button" aria-label="${escapeHTML(image.name || image.fileName || "image")}">
          <img src="${escapeHTML(resolveImagePreviewUrl(image))}" alt="${escapeHTML(image.name || image.fileName || "image")}">
          <button type="button" class="image-library-item-menu-trigger" data-image-menu-id="${escapeHTML(image.id)}">&hellip;</button>
        </div>
      `);
      imageTiles.push(`
        <button type="button" class="image-library-item image-library-upload-tile" data-image-upload-tile>
          <span>+</span>
        </button>
      `);
      const markup = imageTiles.join("");
      if (global.RenderUtils && global.RenderUtils.setCachedHTML) {
        global.RenderUtils.setCachedHTML(renderCache, "imageGrid", imageLibraryGrid, markup);
      } else {
        imageLibraryGrid.innerHTML = markup;
      }
      const isEmpty = state.images.length === 0;
      imageLibraryGrid.hidden = isEmpty;
      if (imageLibraryEmpty) {
        imageLibraryEmpty.hidden = !isEmpty;
        if (isEmpty) imageLibraryEmpty.textContent = "No images found.";
      }
    }

    async function fetchLibrary() {
      const state = getImageLibraryState();
      const params = new URLSearchParams();
      if (state.search) params.set("search", state.search);
      if (state.category === "favorites") {
        params.set("favoritesOnly", "true");
      } else if (state.category && state.category !== "all") {
        params.set("category", state.category);
      }
      const url = `/api/images?${params.toString()}`;
      const result = fetchJsonLatest
        ? await fetchJsonLatest("images", url, {}, requestStates.images)
        : null;
      if (result && result.aborted) return;
      const response = result ? result.response : await fetch(url);
      const payload = result ? result.payload : await response.json();
      if (result && !result.isLatest) return;
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Failed to load images.");
      }
      state.images = Array.isArray(payload.images) ? payload.images : [];
      state.categories = Array.isArray(payload.categories) ? payload.categories : [];
      const preferredImageId = (getUploadedImage() && getUploadedImage().id) || "";
      if (preferredImageId) {
        const selected = state.images.find((entry) => entry.id === preferredImageId);
        if (selected) {
          state.activeImageId = selected.id;
          if (!getUploadedImage() || getUploadedImage().id !== selected.id) {
            setUploadedImage(selected);
            setImagePreview(resolveImagePreviewUrl(selected));
          }
        }
      }
      renderCategoryChips();
      renderLibraryGrid();
    }

    function showLibraryModal() {
      const state = getImageLibraryState();
      if (imageLibraryModal) imageLibraryModal.hidden = false;
      state.activeImageId = getUploadedImage() && getUploadedImage().id ? getUploadedImage().id : "";
      if (imageLibraryGrid) imageLibraryGrid.hidden = true;
      if (imageLibraryEmpty) {
        imageLibraryEmpty.hidden = false;
        imageLibraryEmpty.textContent = "Loading images...";
      }
      fetchLibrary().catch((error) => {
        imageStatus.textContent = error.message;
      });
    }

    function hideLibraryModal() {
      if (imageLibraryModal) imageLibraryModal.hidden = true;
      hideItemMenu();
    }

    function showDetailsModal(image, options = {}) {
      if (!image) return;
      hideItemMenu();
      setActiveImageDetailsId(image.id);
      setIsEditingNewImageUpload(Boolean(options.isNewUpload));
      setDetailsError("");
      if (imageDetailsName) imageDetailsName.value = image.name || "";
      setImageDetailsTagsState(Array.isArray(image.tags) ? [...image.tags] : []);
      if (imageDetailsTags) imageDetailsTags.value = "";
      renderDetailsTags();
      renderDetailsCategoryOptions(image.category || "");
      if (imageDetailsCategoryRow) imageDetailsCategoryRow.hidden = false;
      if (imageDetailsTitle) imageDetailsTitle.textContent = options.isNewUpload ? "Name Image" : "Edit Image Details";
      if (imageDetailsModal) imageDetailsModal.hidden = false;
    }

    function hideDetailsModal() {
      if (imageDetailsModal) imageDetailsModal.hidden = true;
      setDetailsError("");
      setActiveImageDetailsId("");
      setImageDetailsTagsState([]);
      renderDetailsTags();
      setIsEditingNewImageUpload(false);
    }

    function setCategoryError(message = "") {
      if (imageCategoryError) imageCategoryError.textContent = message;
    }

    function showCategoryModal(context = "library") {
      setImageCategoryModalContext(context);
      setCategoryError("");
      if (imageCategoryName) imageCategoryName.value = "";
      if (imageCategoryModal) imageCategoryModal.hidden = false;
      if (imageCategoryName) imageCategoryName.focus();
    }

    function hideCategoryModal() {
      if (imageCategoryModal) imageCategoryModal.hidden = true;
      if (imageCategoryName) imageCategoryName.value = "";
      setCategoryError("");
    }

    async function createCategory(rawName) {
      const name = normalizeImageCategoryName(rawName);
      if (!name) {
        throw new Error("Category name is required.");
      }
      const response = await fetch("/api/images/categories", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ name }),
      });
      const payload = await response.json();
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Failed to create category.");
      }
      const state = getImageLibraryState();
      state.categories = Array.isArray(payload.categories) ? payload.categories : state.categories;
      renderCategoryChips();
      renderDetailsCategoryOptions(payload.category || name);
      return payload.category || name;
    }

    function openItemMenu(imageId, anchor) {
      const state = getImageLibraryState();
      const image = state.images.find((entry) => entry.id === imageId);
      if (!image || !anchor || !imageItemMenu) return;
      setActiveImageMenuId(imageId);
      imageMenuFavorite.textContent = image.isFavorite ? "Remove Favorite" : "Add to Favorites";
      const rect = anchor.getBoundingClientRect();
      imageItemMenu.style.left = `${Math.max(12, rect.right - 180)}px`;
      imageItemMenu.style.top = `${rect.bottom + 6}px`;
      imageItemMenu.hidden = false;
    }

    async function updateRecord(id, updates) {
      const response = await fetch("/api/images/update", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id, ...updates }),
      });
      const payload = await response.json();
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Failed to update image.");
      }
      const state = getImageLibraryState();
      state.images = Array.isArray(payload.images) ? payload.images : state.images;
      state.categories = Array.isArray(payload.categories) ? payload.categories : state.categories;
      const updated = payload.image || state.images.find((entry) => entry.id === id);
      if (getUploadedImage() && getUploadedImage().id === id && updated) {
        setSelectedImage(updated);
      }
      renderCategoryChips();
      renderDetailsCategoryOptions(updated ? updated.category || "" : imageDetailsCategory ? imageDetailsCategory.value : "");
      renderLibraryGrid();
    }

    async function deleteRecord(id) {
      const response = await fetch("/api/images/delete", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id }),
      });
      const payload = await response.json();
      if (!response.ok || !payload.ok) {
        throw new Error(payload.error || "Failed to delete image.");
      }
      const state = getImageLibraryState();
      state.images = Array.isArray(payload.images) ? payload.images : [];
      state.categories = Array.isArray(payload.categories) ? payload.categories : [];
      if (getUploadedImage() && getUploadedImage().id === id) {
        setSelectedImage(null);
      }
      renderCategoryChips();
      renderDetailsCategoryOptions(imageDetailsCategory ? imageDetailsCategory.value : "");
      renderLibraryGrid();
    }

    function bindEvents() {
      if (eventsBound) return;
      eventsBound = true;

      if (openImageLibraryButton) openImageLibraryButton.addEventListener("click", showLibraryModal);
      if (imageLibraryClose) imageLibraryClose.addEventListener("click", hideLibraryModal);
      if (imageLibraryModal) {
        imageLibraryModal.addEventListener("click", (event) => {
          if (event.target === imageLibraryModal) hideLibraryModal();
        });
      }
      if (imageLibraryUploadButton) {
        imageLibraryUploadButton.addEventListener("click", () => {
          if (imageInput) {
            imageInput.value = "";
            imageInput.click();
          }
        });
      }
      if (imageLibrarySearchInput) {
        imageLibrarySearchInput.addEventListener("input", () => {
          const state = getImageLibraryState();
          state.search = imageLibrarySearchInput.value.trim();
          window.clearTimeout(imageLibrarySearchInput._searchTimer);
          imageLibrarySearchInput._searchTimer = window.setTimeout(() => {
            fetchLibrary().catch((error) => {
              imageStatus.textContent = error.message;
            });
          }, 200);
        });
      }
      if (imageCategoryChips) {
        imageCategoryChips.addEventListener("click", (event) => {
          const chip = event.target.closest("[data-image-category]");
          if (!chip) return;
          const state = getImageLibraryState();
          state.category = chip.getAttribute("data-image-category") || "all";
          fetchLibrary().catch((error) => {
            imageStatus.textContent = error.message;
          });
        });
      }
      if (newImageCategoryButton) {
        newImageCategoryButton.addEventListener("click", () => showCategoryModal("library"));
      }
      if (imageLibraryGrid) {
        imageLibraryGrid.addEventListener("click", async (event) => {
          const uploadTile = event.target.closest("[data-image-upload-tile]");
          if (uploadTile) {
            if (imageInput) {
              imageInput.value = "";
              imageInput.click();
            }
            return;
          }
          const menuTrigger = event.target.closest("[data-image-menu-id]");
          if (menuTrigger) {
            event.stopPropagation();
            openItemMenu(menuTrigger.getAttribute("data-image-menu-id") || "", menuTrigger);
            return;
          }
          const imageButton = event.target.closest("[data-image-id]");
          if (!imageButton) return;
          const state = getImageLibraryState();
          const image = state.images.find((entry) => entry.id === imageButton.getAttribute("data-image-id"));
          if (!image) return;
          state.activeImageId = image.id;
          setSelectedImage(image);
          hideLibraryModal();
        });
      }
      if (imageMenuFavorite) {
        imageMenuFavorite.addEventListener("click", async () => {
          const state = getImageLibraryState();
          const image = state.images.find((entry) => entry.id === getActiveImageMenuId());
          if (!image) return;
          try {
            await updateRecord(image.id, { isFavorite: !image.isFavorite });
            hideItemMenu();
          } catch (error) {
            imageStatus.textContent = error.message;
          }
        });
      }
      if (imageMenuEdit) {
        imageMenuEdit.addEventListener("click", () => {
          const state = getImageLibraryState();
          const image = state.images.find((entry) => entry.id === getActiveImageMenuId());
          if (!image) return;
          hideItemMenu();
          showDetailsModal(image);
        });
      }
      if (imageMenuDelete) {
        imageMenuDelete.addEventListener("click", async () => {
          const state = getImageLibraryState();
          const image = state.images.find((entry) => entry.id === getActiveImageMenuId());
          if (!image) return;
          try {
            await deleteRecord(image.id);
            hideItemMenu();
          } catch (error) {
            imageStatus.textContent = error.message;
          }
        });
      }
      if (imageDetailsClose) imageDetailsClose.addEventListener("click", hideDetailsModal);
      if (imageDetailsCancel) imageDetailsCancel.addEventListener("click", hideDetailsModal);
      if (imageDetailsNewCategory) imageDetailsNewCategory.addEventListener("click", () => showCategoryModal("details"));
      if (imageCategoryClose) imageCategoryClose.addEventListener("click", hideCategoryModal);
      if (imageCategoryCancel) imageCategoryCancel.addEventListener("click", hideCategoryModal);
      if (imageCategoryName) {
        imageCategoryName.addEventListener("keydown", async (event) => {
          if (event.key !== "Enter") return;
          event.preventDefault();
          if (imageCategorySave) imageCategorySave.click();
        });
      }
      if (imageCategorySave) {
        imageCategorySave.addEventListener("click", async () => {
          setCategoryError("");
          imageCategorySave.disabled = true;
          imageCategorySave.textContent = "Creating...";
          try {
            const createdCategory = await createCategory(imageCategoryName ? imageCategoryName.value : "");
            if (getImageCategoryModalContext() === "details") {
              renderDetailsCategoryOptions(createdCategory);
              if (imageDetailsCategory) imageDetailsCategory.value = createdCategory;
            } else {
              const state = getImageLibraryState();
              state.category = createdCategory;
              await fetchLibrary();
            }
            hideCategoryModal();
          } catch (error) {
            setCategoryError(error.message);
          } finally {
            imageCategorySave.disabled = false;
            imageCategorySave.textContent = "Create Category";
          }
        });
      }
      if (imageDetailsAddTag) {
        imageDetailsAddTag.addEventListener("click", () => {
          addDetailTag(imageDetailsTags ? imageDetailsTags.value : "");
        });
      }
      if (imageDetailsTags) {
        imageDetailsTags.addEventListener("keydown", (event) => {
          if (event.key !== "Enter") return;
          event.preventDefault();
          addDetailTag(imageDetailsTags.value);
        });
      }
      if (imageDetailsTagList) {
        imageDetailsTagList.addEventListener("click", (event) => {
          const button = event.target.closest("[data-image-tag-index]");
          if (!button) return;
          const index = Number(button.getAttribute("data-image-tag-index"));
          const tags = [...getImageDetailsTagsState()];
          if (!Number.isFinite(index) || index < 0 || index >= tags.length) return;
          tags.splice(index, 1);
          setImageDetailsTagsState(tags);
          renderDetailsTags();
        });
      }
      if (imageDetailsSave) {
        imageDetailsSave.addEventListener("click", async () => {
          if (!getActiveImageDetailsId()) return;
          setDetailsError("");
          addDetailTag(imageDetailsTags ? imageDetailsTags.value : "");
          imageDetailsSave.disabled = true;
          imageDetailsSave.textContent = "Saving...";
          try {
            await updateRecord(getActiveImageDetailsId(), {
              name: imageDetailsName ? imageDetailsName.value.trim() : "",
              tags: getImageDetailsTagsState(),
              category: imageDetailsCategory ? imageDetailsCategory.value.trim() : "",
            });
            if (getIsEditingNewImageUpload()) {
              imageStatus.textContent = "Image uploaded and selected.";
            }
            hideDetailsModal();
          } catch (error) {
            setDetailsError(error.message);
          } finally {
            imageDetailsSave.disabled = false;
            imageDetailsSave.textContent = "Save Changes";
          }
        });
      }
      document.addEventListener("click", (event) => {
        if (!imageItemMenu.hidden) {
          const clickedMenu = imageItemMenu.contains(event.target);
          const clickedTrigger = event.target.closest("[data-image-menu-id]");
          if (!clickedMenu && !clickedTrigger) hideItemMenu();
        }
      });
    }

    return {
      bindEvents,
      hideItemMenu,
      setSelectedImage,
      fetchLibrary,
      showLibraryModal,
      hideLibraryModal,
      showDetailsModal,
      hideDetailsModal,
      showCategoryModal,
      hideCategoryModal,
      renderCategoryChips,
      renderLibraryGrid,
      setActiveImageId(id) {
        getImageLibraryState().activeImageId = id || "";
      },
    };
  }

  global.ImagesFeature = {
    create: createImagesFeature,
  };
})(window);
