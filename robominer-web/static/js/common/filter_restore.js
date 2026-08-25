/**
 * Shared restore of select filters from sessionStorage + sync into the URL.
 * Used by shop and mining queue page scripts.
 */
(function (global) {
  "use strict";

  function restoreSelectFilters(options) {
    var storageKey = options.storageKey;
    var selectNames = options.selectNames || [];
    var url = options.url || global.RoboMinerUrlQuery;
    var store = options.store || global.RoboMinerSessionStore;
    if (!url || !store || !storageKey) {
      return;
    }

    var saved = store.readObject(storageKey) || {};
    var i;
    for (i = 0; i < selectNames.length; i += 1) {
      var name = selectNames[i];
      var nodes = document.getElementsByName(name);
      if (!nodes || !nodes.length) {
        continue;
      }
      var fromUrl = url.get(name);
      var value = fromUrl != null && fromUrl !== "" ? fromUrl : saved[name];
      if (value == null || value === "") {
        continue;
      }
      var j;
      for (j = 0; j < nodes.length; j += 1) {
        nodes[j].value = String(value);
      }
    }
  }

  function persistSelectFilters(options) {
    var storageKey = options.storageKey;
    var selectNames = options.selectNames || [];
    var store = options.store || global.RoboMinerSessionStore;
    if (!store || !storageKey) {
      return;
    }
    var payload = {};
    var i;
    for (i = 0; i < selectNames.length; i += 1) {
      var name = selectNames[i];
      var node = document.getElementsByName(name)[0];
      if (node) {
        payload[name] = node.value;
      }
    }
    store.writeObject(storageKey, payload);
  }

  global.RoboMinerFilterRestore = {
    restoreSelectFilters: restoreSelectFilters,
    persistSelectFilters: persistSelectFilters
  };
})(window);
