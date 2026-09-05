(function() {
    if (typeof rallyApplySidePanelOrder === 'function') {
        rallyApplySidePanelOrder();
    }
    if (typeof rallyBindSidePanelOrder === 'function') {
        rallyBindSidePanelOrder();
    }

    function getOreNameFromConfig(oreId, oreNames) {
        if (!oreNames) {
            return '';
        }
        const key = String(oreId);
        return typeof oreNames[key] === 'string' ? oreNames[key] : '';
    }

    function fillOreLegend(letter, color, oreTypes, oreNames) {
        if (typeof oreTypes[letter] === 'undefined') {
            return;
        }
        const canvas = document.getElementById('oreLegend' + letter + 'Canvas');
        if (!canvas) {
            return;
        }
        const context = canvas.getContext('2d');
        context.beginPath();
        context.rect(0, 0, canvas.width, canvas.height);
        context.fillStyle = color;
        context.fill();
        const nameEl = document.getElementById('oreLegend' + letter + 'Name');
        if (nameEl) {
            nameEl.textContent = getOreNameFromConfig(oreTypes[letter].id, oreNames);
        }
        const item = document.getElementById('oreLegend' + letter);
        if (item) {
            item.style.display = 'flex';
        }
    }

    window.requestAnimFrame = (function() {
        return window.requestAnimationFrame ||
            window.webkitRequestAnimationFrame ||
            window.mozRequestAnimationFrame ||
            window.oRequestAnimationFrame ||
            window.msRequestAnimationFrame ||
            function(callback) {
                window.setTimeout(callback, 1000 / 60);
            };
    })();

    let config = {};
    const configEl = document.getElementById('rally-view-config');
    if (configEl) {
        try {
            config = JSON.parse(configEl.textContent || '{}');
        } catch (error) {
            config = {};
        }
    }

    let rallyPayloadError = null;
    try {
        const rallyResultDataEl = document.getElementById('rally-result-data');
        if (!rallyResultDataEl) {
            rallyPayloadError =
                'This rally replay payload is missing, corrupt, or uses an unsupported version.';
        } else {
            rallyPayloadError = applyRallyResultPayload(JSON.parse(rallyResultDataEl.textContent));
        }
    } catch (error) {
        rallyPayloadError =
            'This rally replay payload is missing, corrupt, or uses an unsupported version.';
    }

    if (rallyPayloadError) {
        showRallyReplayUnavailable(rallyPayloadError);
        return;
    }

    const myRallyViewerSlot =
        typeof config.viewerSlot === 'number' ? config.viewerSlot : null;
    window.myRallyViewerSlot = myRallyViewerSlot;

    const myRallyCanvas = document.getElementById('rallyCanvas');
    const myRallyContext = myRallyCanvas.getContext('2d');
    window.myRallyCanvas = myRallyCanvas;
    window.myRallyContext = myRallyContext;

    const myOreCanvas = [
        document.getElementById('oreCanvas0'),
        document.getElementById('oreCanvas1'),
        document.getElementById('oreCanvas2'),
        document.getElementById('oreCanvas3'),
    ];
    const myOreContext = [
        myOreCanvas[0].getContext('2d'),
        myOreCanvas[1].getContext('2d'),
        myOreCanvas[2].getContext('2d'),
        myOreCanvas[3].getContext('2d'),
    ];
    window.myOreCanvas = myOreCanvas;
    window.myOreContext = myOreContext;

    const myDepotCanvas = [
        document.getElementById('depotCanvas0'),
        document.getElementById('depotCanvas1'),
        document.getElementById('depotCanvas2'),
        document.getElementById('depotCanvas3'),
    ];
    const myDepotContext = [
        myDepotCanvas[0].getContext('2d'),
        myDepotCanvas[1].getContext('2d'),
        myDepotCanvas[2].getContext('2d'),
        myDepotCanvas[3].getContext('2d'),
    ];
    window.myDepotCanvas = myDepotCanvas;
    window.myDepotContext = myDepotContext;

    const myProgressCanvas = document.getElementById('progressCanvas');
    window.myProgressCanvas = myProgressCanvas;
    window.myProgressContext = myProgressCanvas ? myProgressCanvas.getContext('2d') : null;
    window.myTurnText = document.getElementById('turnnr');

    const oreNames = config.oreNames || {};
    fillOreLegend('A', 'red', myOreTypes, oreNames);
    fillOreLegend('B', 'green', myOreTypes, oreNames);
    fillOreLegend('C', 'blue', myOreTypes, oreNames);

    runanimation();
})();
