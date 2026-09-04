/**
 * Shared restore of select filters from sessionStorage + sync into the URL.
 * Used by shop and mining queue page scripts.
 */
(function (global) {
  "use strict";

  function restoreSelectFilters(options) {
    const storageKey = options.storageKey;
    const selectNames = options.selectNames || [];
    const url = options.url || global.RoboMinerUrlQuery;
    const store = options.store || global.RoboMinerSessionStore;
    if (!url || !store || !storageKey) {
      return;
    }

    const saved = store.readJson(storageKey) || {};
    let i;
    for (i = 0; i < selectNames.length; i += 1) {
      const name = selectNames[i];
      const nodes = document.getElementsByName(name);
      if (!nodes || !nodes.length) {
        continue;
      }
      const fromUrl = url.get(name);
      const value = fromUrl != null && fromUrl !== "" ? fromUrl : saved[name];
      if (value == null || value === "") {
        continue;
      }
      let j;
      for (j = 0; j < nodes.length; j += 1) {
        nodes[j].value = String(value);
      }
    }
  }

  function persistSelectFilters(options) {
    const storageKey = options.storageKey;
    const selectNames = options.selectNames || [];
    const store = options.store || global.RoboMinerSessionStore;
    if (!store || !storageKey) {
      return;
    }
    const payload = {};
    let i;
    for (i = 0; i < selectNames.length; i += 1) {
      const name = selectNames[i];
      const node = document.getElementsByName(name)[0];
      if (node) {
        payload[name] = node.value;
      }
    }
    store.writeJson(storageKey, payload);
  }

  global.RoboMinerFilterRestore = {
    restoreSelectFilters: restoreSelectFilters,
    persistSelectFilters: persistSelectFilters
  };
})(window);
