'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const LOCAL_TIME_JS = fs.readFileSync(path.join(__dirname, '..', 'local_time.js'), 'utf8');

function loadLocalTime(document) {
    const sandbox = {
        console,
        module: { exports: {} },
        exports: {},
        document,
        window: null,
        globalThis: null,
        Date,
        Number,
    };
    sandbox.window = sandbox;
    sandbox.globalThis = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(LOCAL_TIME_JS, sandbox);
    return sandbox.module.exports;
}

function mockElement(initial) {
    const attrs = { ...(initial.attrs || {}) };
    return {
        textContent: initial.textContent || '',
        getAttribute(name) {
            return Object.prototype.hasOwnProperty.call(attrs, name) ? attrs[name] : null;
        },
        setAttribute(name, value) {
            attrs[name] = String(value);
        },
    };
}

describe('local time helper', () => {
    it('formats a valid ISO instant with toLocaleString', () => {
        const api = loadLocalTime(undefined);
        const formatted = api.formatLocalDateTime('1970-01-01T00:00:00.000Z');
        assert.equal(formatted, new Date('1970-01-01T00:00:00.000Z').toLocaleString());
    });

    it('returns null for invalid ISO values', () => {
        const api = loadLocalTime(undefined);
        assert.equal(api.formatLocalDateTime('not-a-date'), null);
    });

    it('rewrites absolute text and title attributes from data markers', () => {
        const timeEl = mockElement({
            textContent: '1970-01-01 00:00:00 UTC',
            attrs: {
                datetime: '1970-01-01T00:00:00.000Z',
                'data-local-time': '',
            },
        });
        const titleEl = mockElement({
            textContent: '1 hour ago',
            attrs: {
                title: '1970-01-01 00:00:00 UTC',
                'data-local-time-title': '1970-01-01T00:00:00.000Z',
            },
        });
        const document = {
            readyState: 'complete',
            querySelectorAll(selector) {
                if (selector === '[data-local-time]') {
                    return [timeEl];
                }
                if (selector === '[data-local-time-title]') {
                    return [titleEl];
                }
                return [];
            },
            addEventListener() {},
        };

        const api = loadLocalTime(document);
        api.applyLocalTimes(document);

        const expected = new Date('1970-01-01T00:00:00.000Z').toLocaleString();
        assert.equal(timeEl.textContent, expected);
        assert.equal(titleEl.getAttribute('title'), expected);
        assert.equal(titleEl.textContent, '1 hour ago');
    });
});
