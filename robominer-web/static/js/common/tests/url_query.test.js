'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const URL_QUERY_JS = fs.readFileSync(path.join(__dirname, '..', 'url_query.js'), 'utf8');

describe('url query helper', () => {
    it('builds query strings and detects params', () => {
        const historyCalls = [];
        const sandbox = {
            window: null,
            console,
            location: { search: '?areaId=12&fragment=queue' },
            history: {
                replaceState(_state, _title, url) {
                    historyCalls.push(url);
                },
            },
        };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(URL_QUERY_JS, sandbox);

        assert.equal(sandbox.RoboMinerUrlQuery.getParam('areaId'), '12');
        assert.equal(sandbox.RoboMinerUrlQuery.hasAnyParam(['areaId']), true);
        assert.equal(sandbox.RoboMinerUrlQuery.hasAnyParam(['missing']), false);
        assert.equal(
            sandbox.RoboMinerUrlQuery.buildQueryString({ a: '1', b: '' }),
            'a=1'
        );
        sandbox.RoboMinerUrlQuery.sync('miningQueue', { fragment: 'queue' });
        assert.deepEqual(historyCalls, ['miningQueue?fragment=queue']);
    });
});
