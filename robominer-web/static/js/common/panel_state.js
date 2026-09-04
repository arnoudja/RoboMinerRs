(function(global) {
    function setPanelEnabled(panel, enabled) {
        const fields = panel.querySelectorAll('input, select, textarea, button');
        for (let index = 0; index < fields.length; index += 1) {
            fields[index].disabled = !enabled;
        }
    }

    function panelFormSnapshot(panel, skipNames) {
        const skip = skipNames || [];
        const snapshot = {};
        const fields = panel.querySelectorAll('input[name], select[name], textarea[name]');
        for (let index = 0; index < fields.length; index += 1) {
            const field = fields[index];
            if (field.name && skip.indexOf(field.name) === -1) {
                snapshot[field.name] = field.value;
            }
        }
        return JSON.stringify(snapshot);
    }

    function isPanelDirty(panel, skipNames) {
        const baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return false;
        }
        return panelFormSnapshot(panel, skipNames) !== baseline;
    }

    function capturePanelBaseline(panel, skipNames) {
        panel.setAttribute('data-form-baseline', panelFormSnapshot(panel, skipNames));
    }

    function restorePanelBaseline(panel, skipNames) {
        const skip = skipNames || [];
        const baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return;
        }
        const snapshot = JSON.parse(baseline);
        const fields = panel.querySelectorAll('input[name], select[name], textarea[name]');
        for (let index = 0; index < fields.length; index += 1) {
            const field = fields[index];
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
