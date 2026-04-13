(function bootstrapRenderUtils(global) {
  function setCachedHTML(cache, key, node, markup) {
    if (!node) return false;
    if (cache[key] === markup) return false;
    cache[key] = markup;
    node.innerHTML = markup;
    return true;
  }

  function setText(node, value) {
    if (!node) return false;
    const nextValue = String(value == null ? "" : value);
    if (node.textContent === nextValue) return false;
    node.textContent = nextValue;
    return true;
  }

  global.LaunchDeckRenderUtils = {
    setCachedHTML,
    setText,
  };
})(window);
