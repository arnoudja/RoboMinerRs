(function(global) {
    function readJson(key, fallback) {
        try {
            var raw = global.sessionStorage.getItem(key);
            if (!raw) {
                return fallback === undefined ? null : fallback;
            }
            return JSON.parse(raw);
        } catch (error) {
            return fallback === undefined ? null : fallback;
        }
    }

    function writeJson(key, value) {
        try {
            global.sessionStorage.setItem(key, JSON.stringify(value));
        } catch (error) {
        }
    }

    global.RoboMinerSessionStore = {
        readJson: readJson,
        writeJson: writeJson
    };
})(window);
