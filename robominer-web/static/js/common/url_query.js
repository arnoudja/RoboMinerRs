(function(global) {
    function getParam(name) {
        var search = global.location.search;
        if (!search) {
            return null;
        }
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {
            var pair = params[index].split('=');
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
        var search = global.location.search;
        if (!search) {
            return false;
        }
        var wanted = {};
        for (var nameIndex = 0; nameIndex < names.length; nameIndex += 1) {
            wanted[names[nameIndex]] = true;
        }
        var params = search.substring(1).split('&');
        for (var paramIndex = 0; paramIndex < params.length; paramIndex += 1) {
            var paramName = decodeURIComponent(params[paramIndex].split('=')[0]);
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
        var parts = [];
        for (var key in params) {
            if (Object.prototype.hasOwnProperty.call(params, key)) {
                var value = params[key];
                if (value !== null && value !== undefined && value !== '') {
                    parts.push(encodeURIComponent(key) + '=' + encodeURIComponent(String(value)));
                }
            }
        }
        return parts.join('&');
    }

    function replaceQuery(path, params) {
        var query = buildQueryString(params);
        var url = query ? path + '?' + query : path;
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
        replaceQuery: replaceQuery,
        sync: sync
    };
})(window);
