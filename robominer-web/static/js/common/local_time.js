'use strict';

(function (root) {
    function formatLocalDateTime(iso) {
        const date = new Date(iso);
        if (Number.isNaN(date.getTime())) {
            return null;
        }
        return date.toLocaleString();
    }

    function applyLocalTimes(rootNode) {
        const scope = rootNode || document;
        scope.querySelectorAll('[data-local-time]').forEach((element) => {
            const iso = element.getAttribute('datetime') || element.getAttribute('data-local-time');
            if (!iso) {
                return;
            }
            const formatted = formatLocalDateTime(iso);
            if (formatted) {
                element.textContent = formatted;
            }
        });
        scope.querySelectorAll('[data-local-time-title]').forEach((element) => {
            const iso = element.getAttribute('data-local-time-title');
            if (!iso) {
                return;
            }
            const formatted = formatLocalDateTime(iso);
            if (formatted) {
                element.setAttribute('title', formatted);
            }
        });
    }

    const api = {
        formatLocalDateTime,
        applyLocalTimes,
    };

    if (typeof module !== 'undefined' && module.exports) {
        module.exports = api;
    }
    root.RoboMinerLocalTime = api;

    if (typeof document !== 'undefined') {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => applyLocalTimes(document));
        } else {
            applyLocalTimes(document);
        }
    }
})(typeof globalThis !== 'undefined' ? globalThis : window);
