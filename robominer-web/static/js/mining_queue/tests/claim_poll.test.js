'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const CLAIM_POLL_JS = fs.readFileSync(path.join(__dirname, '..', 'claim_poll.js'), 'utf8');

function loadClaimPoll() {
    const sandbox = { window: null, console, document: { querySelector() { return null; } } };
    sandbox.window = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(CLAIM_POLL_JS, sandbox);
    return sandbox;
}

describe('mining queue claim_poll module', () => {
    it('computes wallet credit deltas and signatures', () => {
        const sandbox = loadClaimPoll();
        const ctx = {
            pageRoot: null,
            CREDIT_FEEDBACK_MS: 1000,
            CLAIM_REFRESH_BACKOFF_MS: [1],
            REFRESH_DEBOUNCE_MS: 1,
            claimRefreshAttempt: 0,
            claimRefreshTimer: null,
            claimBaselineSignature: null,
            claimBaselineAmounts: null,
            creditFeedbackTimer: null,
            refreshDebounceTimer: null,
            refreshInFlight: false,
            refreshPending: false,
            buildFragmentUrl() {
                return 'miningQueue?fragment=queue';
            },
            collectQueueQueryParams() {
                return {};
            },
        };
        const view = {
            fetchFragment() {
                return Promise.resolve();
            },
        };
        const claimPoll = sandbox.RoboMinerMiningQueueInstall.claimPoll(ctx, view);
        // Objects created inside vm.createContext are cross-realm; compare via JSON.
        assert.equal(
            JSON.stringify(
                claimPoll.walletCreditDeltas({ Iron: 1 }, { Iron: 4, Gold: 2 })
            ),
            JSON.stringify([
                { oreName: 'Iron', amount: 3 },
                { oreName: 'Gold', amount: 2 },
            ])
        );
        assert.equal(
            JSON.stringify(claimPoll.walletCreditDeltas({ Iron: 5 }, { Iron: 3 })),
            '[]'
        );
        assert.equal(typeof claimPoll.hasFinishingRuns, 'function');
        assert.equal(typeof claimPoll.refreshQueue, 'function');
        assert.equal(typeof claimPoll.performRefresh, 'function');
    });
});
