/**
 * Wallet headroom helpers for mining-queue clear preview.
 * Must stay aligned with robominer_db::ore_refund_fits_without_clamp / refund clamp rules.
 */
(function(global) {
    function cloneWallet(config) {
        var wallet = {};
        var ores = (config && config.ores) || {};
        Object.keys(ores).forEach(function(oreId) {
            var ore = ores[oreId] || {};
            wallet[oreId] = {
                amount: Number(ore.amount) || 0,
                maxAllowed: Number(ore.maxAllowed) || 0
            };
        });
        return wallet;
    }

    function areaCostsFor(config, areaId) {
        var areaCosts = (config && config.areaCosts) || {};
        return areaCosts[String(areaId)] || [];
    }

    function initialWalletMax(config) {
        var value = config && Number(config.initialOreWalletMax);
        return isFinite(value) && value > 0 ? value : 0;
    }

    function refundFitsWallet(wallet, costs, initialMax) {
        var projected = {};
        for (var index = 0; index < costs.length; index += 1) {
            var cost = costs[index];
            var oreId = String(cost.oreId);
            var refund = Number(cost.amount) || 0;
            var current = projected[oreId];
            if (!current) {
                if (wallet[oreId]) {
                    current = {
                        amount: wallet[oreId].amount,
                        maxAllowed: wallet[oreId].maxAllowed
                    };
                } else {
                    current = {
                        amount: 0,
                        maxAllowed: Number(initialMax) || 0
                    };
                }
            }
            if (current.amount + refund > current.maxAllowed) {
                return false;
            }
            projected[oreId] = {
                amount: current.amount + refund,
                maxAllowed: current.maxAllowed
            };
        }
        return true;
    }

    function applyRefundToWallet(wallet, costs, initialMax) {
        for (var index = 0; index < costs.length; index += 1) {
            var cost = costs[index];
            var oreId = String(cost.oreId);
            var refund = Number(cost.amount) || 0;
            if (!wallet[oreId]) {
                wallet[oreId] = {
                    amount: 0,
                    maxAllowed: Number(initialMax) || 0
                };
            }
            wallet[oreId].amount = Math.min(
                wallet[oreId].maxAllowed,
                wallet[oreId].amount + refund
            );
        }
    }

    function clearingAllWouldLoseOre(config, areaIds) {
        var wallet = cloneWallet(config);
        var initialMax = initialWalletMax(config);
        for (var index = 0; index < areaIds.length; index += 1) {
            var costs = areaCostsFor(config, areaIds[index]);
            if (!refundFitsWallet(wallet, costs, initialMax)) {
                return true;
            }
            applyRefundToWallet(wallet, costs, initialMax);
        }
        return false;
    }

    global.RoboMinerMiningQueueClear = {
        cloneWallet: cloneWallet,
        areaCostsFor: areaCostsFor,
        initialWalletMax: initialWalletMax,
        refundFitsWallet: refundFitsWallet,
        applyRefundToWallet: applyRefundToWallet,
        clearingAllWouldLoseOre: clearingAllWouldLoseOre
    };
})(typeof window !== 'undefined' ? window : globalThis);
