'use strict';

const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ANIMATION_DIR = path.join(__dirname, '..');
const GOLDEN_PAYLOAD = JSON.parse(
    fs.readFileSync(
        path.join(__dirname, '../../../../../resources/rally_animation/golden_payload_v2.json'),
        'utf8'
    )
);

function loadContractAndPayload() {
    const context = { console };
    vm.createContext(context);
    vm.runInContext(
        fs.readFileSync(path.join(ANIMATION_DIR, 'generated/contract.js'), 'utf8'),
        context,
        { filename: 'generated/contract.js' }
    );
    vm.runInContext(
        fs.readFileSync(path.join(ANIMATION_DIR, 'payload.js'), 'utf8'),
        context,
        { filename: 'payload.js' }
    );
    return context;
}

test('generated contract lists supported payload versions', () => {
    const ctx = loadContractAndPayload();
    assert.strictEqual(ctx.ANIMATION_PAYLOAD_CURRENT_VERSION, 2);
    assert.deepEqual([...ctx.ANIMATION_PAYLOAD_SUPPORTED_VERSIONS], [1, 2]);
});

test('golden v2 payload passes viewer validation', () => {
    const ctx = loadContractAndPayload();
    assert.strictEqual(ctx.validateRallyResultPayload(GOLDEN_PAYLOAD), null);
});
