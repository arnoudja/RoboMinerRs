'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const EDITOR_JS = fs.readFileSync(path.join(__dirname, '..', 'editor.js'), 'utf8');

function loadEditor() {
    const sandbox = { window: {}, console };
    sandbox.window = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(EDITOR_JS, sandbox, { filename: 'editor.js' });
    return sandbox;
}

describe('edit code line gutter', () => {
    it('numbers the last line of code with and without a trailing newline', () => {
        const editor = loadEditor();
        assert.equal(editor.sourceCodeLineCount(''), 1);
        assert.equal(editor.sourceCodeLineCount('mine();'), 1);
        // No trailing newline: the last row is code.
        assert.equal(editor.sourceCodeLineCount('alpha();\nbeta();\ngamma();'), 3);
        // Trailing newline is an empty textarea row. The last line of code is
        // still row 3, and the empty row must be numbered too so scrolling does
        // not slide number 3 onto the blank line.
        assert.equal(editor.sourceCodeLineCount('alpha();\nbeta();\ngamma();\n'), 4);
        assert.equal(editor.sourceCodeLineCount('alpha();\r\nbeta();\r\ngamma();\r\n'), 4);
        assert.equal(editor.sourceCodeLineCount('alpha();\n\nbeta();'), 3);
        assert.equal(editor.sourceCodeLineCount('alpha();\n\nbeta();\n'), 4);
    });

    it('renders a gutter row for every textarea row, including the last', () => {
        const editor = loadEditor();
        const gutter = { innerHTML: '' };
        editor.renderLineNumbers(
            gutter,
            editor.sourceCodeLineCount('alpha();\nbeta();\ngamma();')
        );
        assert.equal(gutter.innerHTML, '1<br>2<br>3');

        editor.renderLineNumbers(
            gutter,
            editor.sourceCodeLineCount('alpha();\nbeta();\ngamma();\n')
        );
        assert.equal(gutter.innerHTML, '1<br>2<br>3<br>4');
    });

    it('inserts blank gutter rows where a source line wraps', () => {
        const editor = loadEditor();
        assert.equal(
            JSON.stringify(editor.wrapCountsFromLineHeights([21, 42, 21], 21)),
            JSON.stringify([1, 2, 1])
        );
        assert.equal(
            JSON.stringify(editor.wrapCountsFromLineHeights([21, 63, 21], 21)),
            JSON.stringify([1, 3, 1])
        );
        assert.equal(
            JSON.stringify(editor.wrapCountsFromLineHeights([21, 0], 21)),
            JSON.stringify([1, 1])
        );
        assert.equal(editor.gutterHtmlForWrapCounts([1, 1, 1]), '1<br>2<br>3');
        // Middle source line uses two visual rows: number, then a blank row.
        assert.equal(editor.gutterHtmlForWrapCounts([1, 2, 1]), '1<br>2<br><br>3');
        // More than one extra visual row.
        assert.equal(editor.gutterHtmlForWrapCounts([1, 3, 1]), '1<br>2<br><br><br>3');

        const gutter = { innerHTML: '' };
        editor.renderLineNumbers(gutter, [1, 2, 1]);
        assert.equal(gutter.innerHTML, '1<br>2<br><br>3');
    });

    it('recomputes blank gutter rows when the text changes or the editor resizes', () => {
        const editor = loadEditor();
        const rowHeights = { one: 21, WRAP: 21, three: 21 };
        const gutter = { innerHTML: '', scrollTop: 0 };
        const listeners = {};
        let resizeCallback = null;
        editor.document = {
            body: {
                appendChild() {},
                removeChild() {}
            },
            createElement() {
                const el = {
                    style: {},
                    textContent: '',
                    setAttribute() {},
                    appendChild() {},
                    removeChild() {}
                };
                Object.defineProperty(el, 'offsetHeight', {
                    get() {
                        return rowHeights[el.textContent] || 21;
                    }
                });
                return el;
            }
        };
        editor.getComputedStyle = function() {
            return {
                lineHeight: '21px',
                fontSize: '14px',
                font: '14px monospace',
                paddingLeft: '0',
                paddingRight: '0',
                letterSpacing: 'normal',
                tabSize: '8'
            };
        };
        editor.ResizeObserver = function(callback) {
            resizeCallback = callback;
            this.observe = function() {};
        };
        const textarea = {
            value: 'one\nWRAP\nthree',
            clientWidth: 80,
            scrollTop: 0,
            closest() {
                return {
                    querySelector() {
                        return gutter;
                    }
                };
            },
            getAttribute() {
                return null;
            },
            setAttribute() {},
            addEventListener(type, listener) {
                listeners[type] = listener;
            }
        };

        editor.attachLineNumberListeners(textarea);
        assert.deepEqual(gutterLabels(gutter.innerHTML), ['1', '2', '3']);
        assertGutterRowsMatchLineHeight(gutter.innerHTML, 21);

        rowHeights.WRAP = 42;
        listeners.input();
        assert.deepEqual(gutterLabels(gutter.innerHTML), ['1', '2', '', '3']);
        assertGutterRowsMatchLineHeight(gutter.innerHTML, 21);

        rowHeights.WRAP = 63;
        resizeCallback();
        assert.deepEqual(gutterLabels(gutter.innerHTML), ['1', '2', '', '', '3']);

        rowHeights.WRAP = 21;
        listeners.input();
        assert.deepEqual(gutterLabels(gutter.innerHTML), ['1', '2', '3']);
    });

    it('inserts blank rows when the first or last line wraps, including three visual rows', () => {
        const editor = loadEditor();
        assert.equal(editor.gutterHtmlForWrapCounts([2, 1, 1]), '1<br><br>2<br>3');
        assert.equal(editor.gutterHtmlForWrapCounts([3, 1, 1]), '1<br><br><br>2<br>3');
        assert.equal(editor.gutterHtmlForWrapCounts([1, 1, 2]), '1<br>2<br>3<br>');
        assert.equal(editor.gutterHtmlForWrapCounts([1, 1, 3]), '1<br>2<br>3<br><br>');

        const layout = installMeasuredEditor(loadEditor(), {
            rowHeights: { FIRST: 63, mid: 21, LAST: 42 }
        });
        layout.textarea.value = 'FIRST\nmid\nLAST';
        editorSync(layout);
        assert.deepEqual(gutterLabels(layout.gutter.innerHTML), ['1', '', '', '2', '3', '']);
        assertGutterRowsMatchLineHeight(layout.gutter.innerHTML, 21);
    });
});

const CHAR_PX = 10;

function installMeasuredEditor(editor, options) {
    const metrics = Object.assign({
        lineHeight: 21,
        paddingTop: 10,
        paddingLeft: 0,
        paddingRight: 0,
        clientWidth: 80,
        clientHeight: 90,
        whiteSpace: 'pre-wrap',
        overflowWrap: 'break-word',
        tabSize: '4',
        charPx: CHAR_PX
    }, options.metrics || {});
    const rowHeights = options.rowHeights || null;
    const mounted = [];
    const gutter = { innerHTML: options.serverHtml || '', scrollTop: 0, style: {} };
    editor.document = {
        body: {
            appendChild(element) {
                mounted.push(element);
            },
            removeChild() {}
        },
        createElement() {
            const element = {
                style: {},
                textContent: '',
                children: [],
                setAttribute() {},
                appendChild(child) {
                    child.parentElement = this;
                    this.children.push(child);
                },
                removeChild() {}
            };
            Object.defineProperty(element, 'offsetHeight', {
                get() {
                    if (rowHeights) {
                        const key = element.textContent === '\u00a0' ? '' : element.textContent;
                        return Object.prototype.hasOwnProperty.call(rowHeights, key)
                            ? rowHeights[key]
                            : metrics.lineHeight;
                    }
                    return simulatedRowHeight(element, metrics);
                }
            });
            return element;
        }
    };
    editor.getComputedStyle = function() {
        return {
            lineHeight: metrics.lineHeight + 'px',
            fontSize: '14px',
            font: '14px monospace',
            paddingTop: metrics.paddingTop + 'px',
            paddingLeft: metrics.paddingLeft + 'px',
            paddingRight: metrics.paddingRight + 'px',
            letterSpacing: 'normal',
            tabSize: metrics.tabSize,
            whiteSpace: metrics.whiteSpace,
            overflowWrap: metrics.overflowWrap,
            wordWrap: metrics.overflowWrap
        };
    };
    const listeners = {};
    let resizeCallback = null;
    editor.ResizeObserver = function(callback) {
        resizeCallback = callback;
        this.observe = function() {};
    };
    const queuedFrames = [];
    editor.requestAnimationFrame = function(callback) {
        queuedFrames.push(callback);
        return queuedFrames.length;
    };
    let clientWidth = metrics.clientWidth;
    const textarea = {
        value: options.value || '',
        clientHeight: metrics.clientHeight,
        scrollTop: 0,
        selectionStart: 0,
        selectionEnd: 0,
        get clientWidth() {
            return clientWidth;
        },
        set clientWidth(next) {
            clientWidth = next;
        },
        focus() {},
        setSelectionRange(start, end) {
            this.selectionStart = start;
            this.selectionEnd = end;
        },
        closest() {
            return {
                querySelector() {
                    return gutter;
                }
            };
        },
        getAttribute() {
            return null;
        },
        setAttribute() {},
        addEventListener(type, listener) {
            listeners[type] = listener;
        }
    };
    const panel = {
        querySelector(selector) {
            if (selector === 'textarea[name="sourceCode"]') {
                return textarea;
            }
            return null;
        }
    };
    return {
        editor,
        gutter,
        textarea,
        panel,
        listeners,
        metrics,
        mounted,
        queuedFrames,
        setClientWidth(next) {
            clientWidth = next;
        },
        resize() {
            if (resizeCallback) {
                resizeCallback();
            }
        }
    };
}

function editorSync(layout) {
    layout.editor.syncLineNumbersForTextarea(layout.textarea);
}

function gutterLabels(html) {
    if (html.indexOf('edit-code-gutter-row') !== -1) {
        const labels = [];
        const pattern = /<div class="edit-code-gutter-row"[^>]*>([^<]*)<\/div>/g;
        let match = pattern.exec(html);
        while (match) {
            labels.push(match[1]);
            match = pattern.exec(html);
        }
        return labels;
    }
    return html.split('<br>');
}

function assertGutterRowsMatchLineHeight(html, lineHeight) {
    const pattern = /<div class="edit-code-gutter-row" style="([^"]*)">/g;
    const styles = [];
    let match = pattern.exec(html);
    while (match) {
        styles.push(match[1]);
        match = pattern.exec(html);
    }
    assert.ok(styles.length > 0, 'gutter rows should be explicit boxes');
    const expected = 'height:' + lineHeight + 'px;line-height:' + lineHeight + 'px';
    for (let index = 0; index < styles.length; index += 1) {
        assert.equal(styles[index], expected);
    }
}

function simulatedRowHeight(row, metrics) {
    const text = row.textContent === '\u00a0' ? '' : row.textContent;
    const mirror = row.parentElement;
    const widthPx = parseFloat(row.style.width || (mirror && mirror.style.width)) || 0;
    const tabRaw = parseFloat(row.style.tabSize || (mirror && mirror.style.tabSize));
    const tabSize = tabRaw > 0 ? tabRaw : 8;
    const whiteSpace = row.style.whiteSpace || (mirror && mirror.style.whiteSpace) || 'pre-wrap';
    const overflowWrap = row.style.overflowWrap || (mirror && mirror.style.overflowWrap) || 'normal';
    return wrapRowsForLine(text, widthPx, tabSize, whiteSpace, overflowWrap, metrics.charPx)
        * metrics.lineHeight;
}

function wrapRowsForLine(text, widthPx, tabSize, whiteSpace, overflowWrap, charPx) {
    if (!text || whiteSpace === 'pre' || whiteSpace === 'nowrap' || !(widthPx > 0)) {
        return 1;
    }
    const size = tabSize > 0 ? tabSize : 8;
    let columns = 0;
    for (let index = 0; index < text.length; index += 1) {
        columns += text.charAt(index) === '\t' ? (size - (columns % size)) : 1;
    }
    const textPx = columns * charPx;
    if (textPx <= widthPx) {
        return 1;
    }
    const breaksWords = overflowWrap === 'break-word' || overflowWrap === 'anywhere';
    if (!breaksWords && text.indexOf(' ') === -1 && text.indexOf('\t') === -1) {
        return 1;
    }
    return Math.ceil(textPx / widthPx);
}

function expectedWraps(lines, metrics) {
    const contentWidth = metrics.clientWidth - metrics.paddingLeft - metrics.paddingRight;
    const tabSize = parseFloat(metrics.tabSize);
    return lines.map(function(line) {
        return wrapRowsForLine(
            line,
            contentWidth,
            tabSize,
            metrics.whiteSpace,
            metrics.overflowWrap,
            metrics.charPx
        );
    });
}

describe('edit code line jump', () => {
    it('scrolls to the visual row when an earlier line wraps', () => {
        const layout = installMeasuredEditor(loadEditor(), {
            rowHeights: { one: 42, TWO: 21, three: 21 },
            metrics: { paddingTop: 10, clientHeight: 90, lineHeight: 21 }
        });
        layout.textarea.value = 'one\nTWO\nthree';
        layout.editor.focusSourceLine(layout.panel, 3);
        // Line 1 occupies two visual rows, so line 3 starts on visual row 3 (0-based index 3).
        const lineHeight = 21;
        const visualRowsAbove = 2 + 1;
        assert.equal(
            layout.textarea.scrollTop,
            Math.max(0, 10 + visualRowsAbove * lineHeight - 90 / 3)
        );
        assert.equal(layout.textarea.selectionStart, 8);
        assert.equal(layout.textarea.selectionEnd, 13);
    });

    it('scrolls to the start of a target line that itself wraps', () => {
        const layout = installMeasuredEditor(loadEditor(), {
            rowHeights: { one: 42, TWO: 63, three: 21 },
            metrics: { paddingTop: 10, clientHeight: 90, lineHeight: 21 }
        });
        layout.textarea.value = 'one\nTWO\nthree';
        layout.editor.focusSourceLine(layout.panel, 2);
        // Rows above the target only. The target's own extra rows stay below the scroll anchor.
        assert.equal(
            layout.textarea.scrollTop,
            Math.max(0, 10 + 2 * 21 - 90 / 3)
        );
    });
});

describe('edit code gutter scroll sync', () => {
    it('sizes every gutter row, including wrap blanks, to the editor line height', () => {
        const layout = installMeasuredEditor(loadEditor(), {
            rowHeights: { one: 21, WRAP: 42, three: 21 },
            serverHtml: '1<br>2<br>3'
        });
        layout.textarea.value = 'one\nWRAP\nthree';
        layout.editor.attachLineNumberListeners(layout.textarea);
        assert.notEqual(layout.gutter.innerHTML, '1<br>2<br>3');
        assert.deepEqual(gutterLabels(layout.gutter.innerHTML), ['1', '2', '', '3']);
        assertGutterRowsMatchLineHeight(layout.gutter.innerHTML, 21);

        layout.textarea.scrollTop = 84;
        layout.listeners.scroll();
        assert.equal(layout.gutter.scrollTop, layout.textarea.scrollTop);
    });

    it('does not keep a wrap measurement taken at width 0', () => {
        const layout = installMeasuredEditor(loadEditor(), {
            rowHeights: { one: 21, WRAP: 42, three: 21 },
            serverHtml: '1<br>2<br>3'
        });
        layout.textarea.value = 'one\nWRAP\nthree';
        editorSync(layout);
        const aligned = layout.gutter.innerHTML;
        assert.deepEqual(gutterLabels(aligned), ['1', '2', '', '3']);

        layout.setClientWidth(0);
        editorSync(layout);
        assert.equal(layout.gutter.innerHTML, aligned);

        layout.setClientWidth(80);
        layout.resize();
        assert.deepEqual(gutterLabels(layout.gutter.innerHTML), ['1', '2', '', '3']);
        assertGutterRowsMatchLineHeight(layout.gutter.innerHTML, 21);
    });

    it('measures wrap rows after a hidden panel is shown', () => {
        const editor = loadEditor();
        const layout = installMeasuredEditor(editor, {
            rowHeights: { one: 21, WRAP: 63, three: 21 },
            metrics: { clientWidth: 0 },
            serverHtml: '1<br>2<br>3',
            value: 'one\nWRAP\nthree'
        });
        assert.equal(typeof editor.refreshEditCodeLineNumbers, 'function');
        editor.refreshEditCodeLineNumbers(layout.panel);
        assert.equal(layout.gutter.innerHTML, '1<br>2<br>3');
        assert.equal(layout.queuedFrames.length, 1);

        layout.setClientWidth(80);
        layout.queuedFrames[0]();
        assert.deepEqual(gutterLabels(layout.gutter.innerHTML), ['1', '2', '', '', '3']);
        assertGutterRowsMatchLineHeight(layout.gutter.innerHTML, 21);
    });
});

describe('edit code wrap measurement', () => {
    it('wraps tabs and a spaceless token the same way as the textarea', () => {
        const metrics = {
            lineHeight: 21,
            clientWidth: 60,
            paddingLeft: 10,
            paddingRight: 10,
            whiteSpace: 'break-spaces',
            overflowWrap: 'normal',
            tabSize: '2',
            charPx: 10
        };
        const lines = ['ABCDEFGH', '\t\tAB', 'OK'];
        const layout = installMeasuredEditor(loadEditor(), {
            metrics,
            value: lines.join('\n')
        });
        const wraps = layout.editor.measureSourceLineWraps(layout.textarea);
        assert.equal(JSON.stringify(wraps), JSON.stringify(expectedWraps(lines, metrics)));

        const mirror = layout.mounted[0];
        const row = mirror.children[0];
        const contentWidth = metrics.clientWidth - metrics.paddingLeft - metrics.paddingRight;
        assert.equal(mirror.style.whiteSpace, metrics.whiteSpace);
        assert.equal(row.style.whiteSpace, metrics.whiteSpace);
        assert.equal(mirror.style.overflowWrap, metrics.overflowWrap);
        assert.equal(row.style.overflowWrap, metrics.overflowWrap);
        assert.equal(mirror.style.tabSize, metrics.tabSize);
        assert.equal(row.style.tabSize, metrics.tabSize);
        assert.equal(mirror.style.width, contentWidth + 'px');
        assert.equal(row.style.width, contentWidth + 'px');
    });

    it('wraps a long token onto three rows when the textarea breaks words', () => {
        const metrics = {
            lineHeight: 21,
            clientWidth: 50,
            paddingLeft: 5,
            paddingRight: 5,
            whiteSpace: 'pre-wrap',
            overflowWrap: 'anywhere',
            tabSize: '4',
            charPx: 10
        };
        // 12 columns in a 40px content box at 10px each → three visual rows.
        const lines = ['ABCDEFGHIJKL'];
        const layout = installMeasuredEditor(loadEditor(), {
            metrics,
            value: lines[0]
        });
        assert.equal(
            JSON.stringify(layout.editor.measureSourceLineWraps(layout.textarea)),
            JSON.stringify(expectedWraps(lines, metrics))
        );
        assert.equal(JSON.stringify(expectedWraps(lines, metrics)), JSON.stringify([3]));
        assert.equal(layout.mounted[0].children[0].style.overflowWrap, 'anywhere');
    });
});
