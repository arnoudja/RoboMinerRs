(function(global) {
    function getParam(name) {
        const search = global.location.search;
        if (!search) {
            return null;
        }
        const params = search.substring(1).split('&');
        for (let index = 0; index < params.length; index += 1) {
            const pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === name && pair[1]) {
                return decodeURIComponent(pair[1]);
            }
        }
        return null;
    }

    function hasAnyParam(names) {
        if (!names || !names.length) {
            return false;
        }
        const search = global.location.search;
        if (!search) {
            return false;
        }
        const wanted = {};
        for (let nameIndex = 0; nameIndex < names.length; nameIndex += 1) {
            wanted[names[nameIndex]] = true;
        }
        const params = search.substring(1).split('&');
        for (let paramIndex = 0; paramIndex < params.length; paramIndex += 1) {
            const paramName = decodeURIComponent(params[paramIndex].split('=')[0]);
            if (wanted[paramName]) {
                return true;
            }
        }
        return false;
    }

    function buildQueryString(params) {
        if (params === null || params === undefined) {
            return '';
        }
        if (typeof params === 'string') {
            return params;
        }
        const parts = [];
        for (const key in params) {
            if (Object.prototype.hasOwnProperty.call(params, key)) {
                const value = params[key];
                if (value !== null && value !== undefined && value !== '') {
                    parts.push(encodeURIComponent(key) + '=' + encodeURIComponent(String(value)));
                }
            }
        }
        return parts.join('&');
    }

    function replaceQuery(path, params) {
        const query = buildQueryString(params);
        const url = query ? path + '?' + query : path;
        if (global.history && global.history.replaceState) {
            global.history.replaceState(null, '', url);
        }
    }

    function sync(path, params) {
        replaceQuery(path, params);
    }

    global.RoboMinerUrlQuery = {
        getParam: getParam,
        hasAnyParam: hasAnyParam,
        buildQueryString: buildQueryString,
        replaceQuery: replaceQuery,
        sync: sync
    };
})(window);
