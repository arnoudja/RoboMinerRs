'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const CLEAR_WALLET_JS = path.join(__dirname, '..', 'clear_wallet.js');

function loadClearWallet() {
    const context = { console, Math, Number, Object, Array, String, isFinite };
    context.window = context;
    context.globalThis = context;
    vm.createContext(context);
    vm.runInContext(fs.readFileSync(CLEAR_WALLET_JS, 'utf8'), context, {
        filename: 'clear_wallet.js',
    });
    return context.RoboMinerMiningQueueClear;
}

describe('mining queue clear wallet helpers', () => {
    it('detects wallet overflow when clearing all', () => {
        const helpers = loadClearWallet();
        const config = {
            initialOreWalletMax: 5,
            ores: { '1': { amount: 9, maxAllowed: 10 } },
            areaCosts: {
                '10': [{ oreId: 1, amount: 2 }],
                '11': [{ oreId: 1, amount: 1 }],
            },
        };
        assert.equal(helpers.clearingAllWouldLoseOre(config, ['10', '11']), true);
        assert.equal(helpers.clearingAllWouldLoseOre(config, ['11']), false);
    });

    it('skips unsafe items in projection when applying refunds', () => {
        const helpers = loadClearWallet();
        const config = {
            initialOreWalletMax: 5,
            ores: { '1': { amount: 8, maxAllowed: 10 } },
            areaCosts: {
                '20': [{ oreId: 1, amount: 5 }],
                '21': [{ oreId: 1, amount: 1 }],
            },
        };
        const wallet = helpers.cloneWallet(config);
        const initialMax = helpers.initialWalletMax(config);
        assert.equal(
            helpers.refundFitsWallet(wallet, helpers.areaCostsFor(config, '20'), initialMax),
            false
        );
        assert.equal(
            helpers.refundFitsWallet(wallet, helpers.areaCostsFor(config, '21'), initialMax),
            true
        );
        helpers.applyRefundToWallet(wallet, helpers.areaCostsFor(config, '21'), initialMax);
        assert.equal(wallet['1'].amount, 9);
    });
});
