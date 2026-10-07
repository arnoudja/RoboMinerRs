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
        assert.equal(gutter.innerHTML, '1<br>2<br>3');

        rowHeights.WRAP = 42;
        listeners.input();
        assert.equal(gutter.innerHTML, '1<br>2<br><br>3');

        rowHeights.WRAP = 63;
        resizeCallback();
        assert.equal(gutter.innerHTML, '1<br>2<br><br><br>3');

        rowHeights.WRAP = 21;
        listeners.input();
        assert.equal(gutter.innerHTML, '1<br>2<br>3');
    });
});
