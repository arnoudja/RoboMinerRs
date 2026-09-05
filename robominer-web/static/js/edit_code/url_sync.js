function syncEditCodeUrl(sourceId) {
    const params = { nextProgramSourceId: sourceId };
    const line = window.RoboMinerUrlQuery.getParam('line');
    if (line) {
        params.line = line;
    }
    window.RoboMinerUrlQuery.sync('editCode', params);
}

function editCodeUrlSourceId() {
    return window.RoboMinerUrlQuery.getParam('nextProgramSourceId');
}

function editCodeUrlLine() {
    const raw = window.RoboMinerUrlQuery.getParam('line');
    if (!raw) {
        return null;
    }
    const line = parseInt(raw, 10);
    if (isNaN(line) || line < 1) {
        return null;
    }
    return line;
}
