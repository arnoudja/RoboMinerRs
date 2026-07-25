(function(global) {
    function setPanelEnabled(panel, enabled) {
        var fields = panel.querySelectorAll('input, select, textarea, button');
        for (var index = 0; index < fields.length; index += 1) {
            fields[index].disabled = !enabled;
        }
    }

    function panelFormSnapshot(panel, skipNames) {
        var skip = skipNames || [];
        var snapshot = {};
        var fields = panel.querySelectorAll('input[name], select[name], textarea[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name && skip.indexOf(field.name) === -1) {
                snapshot[field.name] = field.value;
            }
        }
        return JSON.stringify(snapshot);
    }

    function isPanelDirty(panel, skipNames) {
        var baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return false;
        }
        return panelFormSnapshot(panel, skipNames) !== baseline;
    }

    function capturePanelBaseline(panel, skipNames) {
        panel.setAttribute('data-form-baseline', panelFormSnapshot(panel, skipNames));
    }

    function restorePanelBaseline(panel, skipNames) {
        var skip = skipNames || [];
        var baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return;
        }
        var snapshot = JSON.parse(baseline);
        var fields = panel.querySelectorAll('input[name], select[name], textarea[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name
                && skip.indexOf(field.name) === -1
                && Object.prototype.hasOwnProperty.call(snapshot, field.name)) {
                field.value = snapshot[field.name];
            }
        }
    }

    global.RoboMinerPanelState = {
        setPanelEnabled: setPanelEnabled,
        panelFormSnapshot: panelFormSnapshot,
        isPanelDirty: isPanelDirty,
        capturePanelBaseline: capturePanelBaseline,
        restorePanelBaseline: restorePanelBaseline
    };
})(window);
