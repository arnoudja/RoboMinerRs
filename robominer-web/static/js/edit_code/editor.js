var EDIT_CODE_INDENT = '    ';

function focusSourceLine(panel, lineNumber) {
    var textarea = panel && panel.querySelector('textarea[name="sourceCode"]');
    if (!textarea || typeof lineNumber !== 'number' || isNaN(lineNumber) || lineNumber < 1) {
        return;
    }
    var lines = textarea.value.split('\n');
    if (lines.length === 0) {
        return;
    }
    var targetLine = Math.min(Math.floor(lineNumber), lines.length);
    var start = 0;
    for (var index = 0; index < targetLine - 1; index += 1) {
        start += lines[index].length + 1;
    }
    var end = start + lines[targetLine - 1].length;
    textarea.focus();
    if (typeof textarea.setSelectionRange === 'function') {
        textarea.setSelectionRange(start, end);
    }
    var style = window.getComputedStyle(textarea);
    var lineHeight = parseFloat(style.lineHeight);
    if (!lineHeight || isNaN(lineHeight)) {
        var fontSize = parseFloat(style.fontSize);
        lineHeight = (fontSize && !isNaN(fontSize) ? fontSize : 14) * 1.4;
    }
    var paddingTop = parseFloat(style.paddingTop);
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
    var lines = [];
    for (var line = 1; line <= lineCount; line += 1) {
        lines.push(String(line));
    }
    gutter.innerHTML = lines.join('<br>');
}

function syncLineNumbersForTextarea(textarea) {
    var editor = textarea.closest('.edit-code-source-editor');
    if (!editor) {
        return;
    }
    var gutter = editor.querySelector('.edit-code-line-numbers');
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
    var editor = textarea.closest('.edit-code-source-editor');
    var gutter = editor && editor.querySelector('.edit-code-line-numbers');
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
        var event = document.createEvent('Event');
        event.initEvent('input', true, true);
        textarea.dispatchEvent(event);
    }
}

function lineStartIndex(value, index) {
    var start = value.lastIndexOf('\n', Math.max(0, index - 1));
    return start < 0 ? 0 : start + 1;
}

function lineEndIndex(value, index) {
    var end = value.indexOf('\n', index);
    return end < 0 ? value.length : end;
}

function outdentLine(line) {
    if (line.charAt(0) === '\t') {
        return line.substring(1);
    }
    var remove = 0;
    while (remove < EDIT_CODE_INDENT.length && line.charAt(remove) === ' ') {
        remove += 1;
    }
    return remove > 0 ? line.substring(remove) : line;
}

function adjustSelectedLines(textarea, indent) {
    var value = textarea.value;
    var selectionStart = textarea.selectionStart;
    var selectionEnd = textarea.selectionEnd;
    var rangeStart = lineStartIndex(value, selectionStart);
    var rangeEnd = selectionEnd > selectionStart
        ? lineEndIndex(value, Math.max(selectionStart, selectionEnd - 1))
        : lineEndIndex(value, selectionStart);
    var block = value.substring(rangeStart, rangeEnd);
    var lines = block.split('\n');
    var nextLines = [];
    var lineDeltas = [];
    var totalDelta = 0;
    for (var index = 0; index < lines.length; index += 1) {
        var line = lines[index];
        var nextLine = indent ? EDIT_CODE_INDENT + line : outdentLine(line);
        var delta = nextLine.length - line.length;
        lineDeltas.push(delta);
        totalDelta += delta;
        nextLines.push(nextLine);
    }
    if (totalDelta === 0) {
        return;
    }
    var nextBlock = nextLines.join('\n');
    textarea.value = value.substring(0, rangeStart) + nextBlock + value.substring(rangeEnd);

    function mapOffset(offset) {
        var relative = offset - rangeStart;
        if (relative <= 0) {
            return offset;
        }
        var pos = 0;
        var deltaBefore = 0;
        for (var lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
            var lineLength = lines[lineIndex].length;
            var lineEndRel = pos + lineLength;
            if (relative <= lineEndRel || lineIndex === lines.length - 1) {
                var offsetInLine = relative - pos;
                var lineDelta = lineDeltas[lineIndex];
                if (lineDelta < 0) {
                    var removed = -lineDelta;
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
    var value = textarea.value;
    var selectionStart = textarea.selectionStart;
    var selectionEnd = textarea.selectionEnd;
    textarea.value = value.substring(0, selectionStart)
        + EDIT_CODE_INDENT
        + value.substring(selectionEnd);
    var cursor = selectionStart + EDIT_CODE_INDENT.length;
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
    var selectionStart = textarea.selectionStart;
    var selectionEnd = textarea.selectionEnd;
    var selected = textarea.value.substring(selectionStart, selectionEnd);
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
