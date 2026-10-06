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
});
