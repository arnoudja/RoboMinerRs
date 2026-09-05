const EDIT_CODE_INDENT = '    ';

function focusSourceLine(panel, lineNumber) {
    const textarea = panel && panel.querySelector('textarea[name="sourceCode"]');
    if (!textarea || typeof lineNumber !== 'number' || isNaN(lineNumber) || lineNumber < 1) {
        return;
    }
    const lines = textarea.value.split('\n');
    if (lines.length === 0) {
        return;
    }
    const targetLine = Math.min(Math.floor(lineNumber), lines.length);
    let start = 0;
    for (let index = 0; index < targetLine - 1; index += 1) {
        start += lines[index].length + 1;
    }
    const end = start + lines[targetLine - 1].length;
    textarea.focus();
    if (typeof textarea.setSelectionRange === 'function') {
        textarea.setSelectionRange(start, end);
    }
    const style = window.getComputedStyle(textarea);
    let lineHeight = parseFloat(style.lineHeight);
    if (!lineHeight || isNaN(lineHeight)) {
        const fontSize = parseFloat(style.fontSize);
        lineHeight = (fontSize && !isNaN(fontSize) ? fontSize : 14) * 1.4;
    }
    let paddingTop = parseFloat(style.paddingTop);
    if (!paddingTop || isNaN(paddingTop)) {
        paddingTop = 0;
    }
    textarea.scrollTop = Math.max(0, paddingTop + (targetLine - 1) * lineHeight - textarea.clientHeight / 3);
    syncLineNumbersForTextarea(textarea);
}

function sourceCodeLineCount(value) {
    if (!value) {
        return 1;
    }
    return value.split('\n').length;
}

function renderLineNumbers(gutter, lineCount) {
    const lines = [];
    for (let line = 1; line <= lineCount; line += 1) {
        lines.push(String(line));
    }
    gutter.innerHTML = lines.join('<br>');
}

function syncLineNumbersForTextarea(textarea) {
    const editor = textarea.closest('.edit-code-source-editor');
    if (!editor) {
        return;
    }
    const gutter = editor.querySelector('.edit-code-line-numbers');
    if (!gutter) {
        return;
    }
    renderLineNumbers(gutter, sourceCodeLineCount(textarea.value));
    gutter.scrollTop = textarea.scrollTop;
}

function attachLineNumberListeners(textarea) {
    if (textarea.getAttribute('data-line-numbers') === 'true') {
        syncLineNumbersForTextarea(textarea);
        return;
    }
    textarea.setAttribute('data-line-numbers', 'true');
    const editor = textarea.closest('.edit-code-source-editor');
    const gutter = editor && editor.querySelector('.edit-code-line-numbers');
    textarea.addEventListener('input', function() {
        syncLineNumbersForTextarea(textarea);
    });
    textarea.addEventListener('scroll', function() {
        if (gutter) {
            gutter.scrollTop = textarea.scrollTop;
        }
    });
    syncLineNumbersForTextarea(textarea);
}

function emitEditCodeInput(textarea) {
    if (typeof InputEvent === 'function') {
        textarea.dispatchEvent(new InputEvent('input', { bubbles: true }));
    } else {
        const event = document.createEvent('Event');
        event.initEvent('input', true, true);
        textarea.dispatchEvent(event);
    }
}

function lineStartIndex(value, index) {
    const start = value.lastIndexOf('\n', Math.max(0, index - 1));
    return start < 0 ? 0 : start + 1;
}

function lineEndIndex(value, index) {
    const end = value.indexOf('\n', index);
    return end < 0 ? value.length : end;
}

function outdentLine(line) {
    if (line.charAt(0) === '\t') {
        return line.substring(1);
    }
    let remove = 0;
    while (remove < EDIT_CODE_INDENT.length && line.charAt(remove) === ' ') {
        remove += 1;
    }
    return remove > 0 ? line.substring(remove) : line;
}

function adjustSelectedLines(textarea, indent) {
    const value = textarea.value;
    const selectionStart = textarea.selectionStart;
    const selectionEnd = textarea.selectionEnd;
    const rangeStart = lineStartIndex(value, selectionStart);
    const rangeEnd = selectionEnd > selectionStart
        ? lineEndIndex(value, Math.max(selectionStart, selectionEnd - 1))
        : lineEndIndex(value, selectionStart);
    const block = value.substring(rangeStart, rangeEnd);
    const lines = block.split('\n');
    const nextLines = [];
    const lineDeltas = [];
    let totalDelta = 0;
    for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        const nextLine = indent ? EDIT_CODE_INDENT + line : outdentLine(line);
        const delta = nextLine.length - line.length;
        lineDeltas.push(delta);
        totalDelta += delta;
        nextLines.push(nextLine);
    }
    if (totalDelta === 0) {
        return;
    }
    const nextBlock = nextLines.join('\n');
    textarea.value = value.substring(0, rangeStart) + nextBlock + value.substring(rangeEnd);

    function mapOffset(offset) {
        const relative = offset - rangeStart;
        if (relative <= 0) {
            return offset;
        }
        let pos = 0;
        let deltaBefore = 0;
        for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
            const lineLength = lines[lineIndex].length;
            const lineEndRel = pos + lineLength;
            if (relative <= lineEndRel || lineIndex === lines.length - 1) {
                const offsetInLine = relative - pos;
                const lineDelta = lineDeltas[lineIndex];
                if (lineDelta < 0) {
                    const removed = -lineDelta;
                    if (offsetInLine <= removed) {
                        return rangeStart + pos + deltaBefore;
                    }
                    return rangeStart + pos + deltaBefore + offsetInLine + lineDelta;
                }
                return rangeStart + pos + deltaBefore + offsetInLine + lineDelta;
            }
            pos = lineEndRel + 1;
            deltaBefore += lineDeltas[lineIndex];
        }
        return offset + totalDelta;
    }

    if (typeof textarea.setSelectionRange === 'function') {
        textarea.setSelectionRange(mapOffset(selectionStart), mapOffset(selectionEnd));
    }
    emitEditCodeInput(textarea);
}

function insertEditCodeIndent(textarea) {
    const value = textarea.value;
    const selectionStart = textarea.selectionStart;
    const selectionEnd = textarea.selectionEnd;
    textarea.value = value.substring(0, selectionStart)
        + EDIT_CODE_INDENT
        + value.substring(selectionEnd);
    const cursor = selectionStart + EDIT_CODE_INDENT.length;
    if (typeof textarea.setSelectionRange === 'function') {
        textarea.setSelectionRange(cursor, cursor);
    }
    emitEditCodeInput(textarea);
}

function handleEditCodeTabKey(event, textarea) {
    if (event.key !== 'Tab' && event.keyCode !== 9) {
        return;
    }
    event.preventDefault();
    const selectionStart = textarea.selectionStart;
    const selectionEnd = textarea.selectionEnd;
    const selected = textarea.value.substring(selectionStart, selectionEnd);
    if (event.shiftKey) {
        adjustSelectedLines(textarea, false);
        return;
    }
    if (selectionStart !== selectionEnd && selected.indexOf('\n') >= 0) {
        adjustSelectedLines(textarea, true);
        return;
    }
    insertEditCodeIndent(textarea);
}
