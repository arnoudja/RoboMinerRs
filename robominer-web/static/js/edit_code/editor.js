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
    const lineHeight = editorLineHeightPx(style);
    let paddingTop = parseFloat(style.paddingTop);
    if (!paddingTop || isNaN(paddingTop)) {
        paddingTop = 0;
    }
    // Scroll by visual rows. An earlier wrapped line pushes the target down by
    // each extra row; the target's own wrap stays below this anchor.
    const wrapCounts = measureSourceLineWraps(textarea);
    const visualRowsAbove = wrapCounts
        ? visualRowsBeforeLine(wrapCounts, targetLine)
        : (targetLine - 1);
    textarea.scrollTop = Math.max(0, paddingTop + visualRowsAbove * lineHeight - textarea.clientHeight / 3);
    syncLineNumbersForTextarea(textarea);
}

function editorLineHeightPx(style) {
    let lineHeight = parseFloat(style.lineHeight);
    if (!lineHeight || isNaN(lineHeight)) {
        const fontSize = parseFloat(style.fontSize);
        lineHeight = (fontSize && !isNaN(fontSize) ? fontSize : 14) * 1.45;
    }
    return lineHeight;
}

function editorContentWidth(textarea, style) {
    const paddingLeft = parseFloat(style.paddingLeft) || 0;
    const paddingRight = parseFloat(style.paddingRight) || 0;
    return textarea.clientWidth - paddingLeft - paddingRight;
}

// The mirror has to wrap with the same rules as the textarea. Hardcoded
// white-space or overflow-wrap drifts on tabs and on tokens with no spaces.
function applyMeasuredWrapStyles(element, style, contentWidth) {
    const whiteSpace = style.whiteSpace || 'pre-wrap';
    const overflowWrap = style.overflowWrap || style.wordWrap || 'break-word';
    element.style.whiteSpace = whiteSpace;
    element.style.wordWrap = style.wordWrap || overflowWrap;
    element.style.overflowWrap = overflowWrap;
    if (style.tabSize) {
        element.style.tabSize = style.tabSize;
    }
    if (contentWidth > 0) {
        element.style.width = contentWidth + 'px';
    }
}

function visualRowsBeforeLine(wrapCounts, targetLine) {
    let rows = 0;
    const last = Math.min(Math.max(targetLine - 1, 0), wrapCounts.length);
    for (let index = 0; index < last; index += 1) {
        const parsed = Math.round(wrapCounts[index]);
        rows += parsed >= 1 ? parsed : 1;
    }
    return rows;
}

// Same row count as edit_code_line_count in editor.rs. A trailing newline is an
// empty textarea row; dropping it shifts the gutter once the editor scrolls, so
// the last line of code no longer sits beside its number.
function sourceCodeLineCount(value) {
    if (!value) {
        return 1;
    }
    return value.split('\n').length;
}

function gutterRowLabels(wrapCounts) {
    const rows = [];
    for (let index = 0; index < wrapCounts.length; index += 1) {
        const parsed = Math.round(wrapCounts[index]);
        const visualRows = parsed >= 1 ? parsed : 1;
        rows.push(String(index + 1));
        for (let extra = 1; extra < visualRows; extra += 1) {
            rows.push('');
        }
    }
    return rows;
}

function gutterHtmlForWrapCounts(wrapCounts, lineHeightPx) {
    const rows = gutterRowLabels(wrapCounts);
    if (!(lineHeightPx > 0)) {
        return rows.join('<br>');
    }
    // Each visual row, including a wrap blank, is a fixed box the same height
    // as the textarea line box. <br> blanks are shorter, so scrollTop drifts.
    const height = lineHeightPx + 'px';
    const html = [];
    for (let index = 0; index < rows.length; index += 1) {
        html.push(
            '<div class="edit-code-gutter-row" style="height:' + height + ';line-height:' + height + '">'
            + rows[index]
            + '</div>'
        );
    }
    return html.join('');
}

function wrapCountsFromLineHeights(heights, lineHeight) {
    const safeLineHeight = lineHeight > 0 ? lineHeight : 1;
    const counts = [];
    for (let index = 0; index < heights.length; index += 1) {
        const rows = Math.round(heights[index] / safeLineHeight);
        counts.push(rows >= 1 ? rows : 1);
    }
    return counts;
}

function renderLineNumbers(gutter, lineCountOrWraps, lineHeightPx) {
    let html;
    if (Array.isArray(lineCountOrWraps)) {
        html = gutterHtmlForWrapCounts(lineCountOrWraps, lineHeightPx);
    } else {
        const lineCount = lineCountOrWraps > 0 ? lineCountOrWraps : 1;
        const rows = [];
        for (let line = 1; line <= lineCount; line += 1) {
            rows.push(String(line));
        }
        html = rows.join('<br>');
    }
    if (gutter.innerHTML !== html) {
        gutter.innerHTML = html;
    }
}

function fallbackWrapCounts(value) {
    const count = sourceCodeLineCount(value);
    const wraps = [];
    for (let index = 0; index < count; index += 1) {
        wraps.push(1);
    }
    return wraps;
}

// A source line that is wider than the textarea wraps onto extra visual rows.
// Those rows have no number of their own; the gutter inserts a blank row for
// each one so the next number stays level with the next source line.
function measureSourceLineWraps(textarea) {
    const value = textarea.value || '';
    const lines = value ? value.split('\n') : [''];
    if (typeof document === 'undefined' || !document.body || !window.getComputedStyle) {
        return fallbackWrapCounts(value);
    }
    const style = window.getComputedStyle(textarea);
    const lineHeight = editorLineHeightPx(style);
    const contentWidth = editorContentWidth(textarea, style);
    // Width 0 means the panel is hidden (or not laid out yet). A measurement
    // taken then is all single rows and must not replace a real one.
    if (!(contentWidth > 0)) {
        return null;
    }

    const mirror = document.createElement('div');
    mirror.setAttribute('aria-hidden', 'true');
    mirror.style.position = 'absolute';
    mirror.style.left = '-9999px';
    mirror.style.top = '0';
    mirror.style.visibility = 'hidden';
    mirror.style.boxSizing = 'content-box';
    mirror.style.padding = '0';
    mirror.style.border = '0';
    mirror.style.font = style.font;
    mirror.style.lineHeight = lineHeight + 'px';
    mirror.style.letterSpacing = style.letterSpacing || 'normal';
    applyMeasuredWrapStyles(mirror, style, contentWidth);

    const rowElements = [];
    for (let index = 0; index < lines.length; index += 1) {
        const row = document.createElement('div');
        row.style.boxSizing = 'content-box';
        row.style.padding = '0';
        row.style.border = '0';
        applyMeasuredWrapStyles(row, style, contentWidth);
        row.textContent = lines[index].length ? lines[index] : '\u00a0';
        mirror.appendChild(row);
        rowElements.push(row);
    }
    document.body.appendChild(mirror);
    const heights = [];
    for (let index = 0; index < rowElements.length; index += 1) {
        heights.push(rowElements[index].offsetHeight);
    }
    document.body.removeChild(mirror);
    return wrapCountsFromLineHeights(heights, lineHeight);
}

function syncLineNumbersForTextarea(textarea, allowRetry) {
    const editor = textarea.closest('.edit-code-source-editor');
    if (!editor) {
        return;
    }
    const gutter = editor.querySelector('.edit-code-line-numbers');
    if (!gutter) {
        return;
    }
    const wraps = measureSourceLineWraps(textarea);
    if (!wraps) {
        if (allowRetry !== false) {
            scheduleLineNumberMeasure(textarea);
        }
        return;
    }
    const lineHeight = window.getComputedStyle
        ? editorLineHeightPx(window.getComputedStyle(textarea))
        : 0;
    renderLineNumbers(gutter, wraps, lineHeight);
    gutter.scrollTop = textarea.scrollTop;
}

function scheduleLineNumberMeasure(textarea) {
    if (textarea.lineNumberMeasureQueued) {
        return;
    }
    if (typeof requestAnimationFrame !== 'function') {
        return;
    }
    textarea.lineNumberMeasureQueued = true;
    requestAnimationFrame(function() {
        textarea.lineNumberMeasureQueued = false;
        syncLineNumbersForTextarea(textarea, false);
    });
}

function refreshEditCodeLineNumbers(panel) {
    const textarea = panel && panel.querySelector('textarea[name="sourceCode"]');
    if (!textarea) {
        return;
    }
    syncLineNumbersForTextarea(textarea);
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
    if (typeof ResizeObserver === 'function') {
        const observer = new ResizeObserver(function() {
            syncLineNumbersForTextarea(textarea);
        });
        observer.observe(textarea);
    } else if (window.addEventListener) {
        window.addEventListener('resize', function() {
            syncLineNumbersForTextarea(textarea);
        });
    }
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
