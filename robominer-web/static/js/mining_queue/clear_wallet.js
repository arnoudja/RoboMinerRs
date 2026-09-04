/**
 * Wallet headroom helpers for mining-queue clear preview.
 * Must stay aligned with robominer_db::ore_refund_fits_without_clamp / refund clamp rules.
 */
(function(global) {
    function cloneWallet(config) {
        const wallet = {};
        const ores = (config && config.ores) || {};
        Object.keys(ores).forEach(function(oreId) {
            const ore = ores[oreId] || {};
            wallet[oreId] = {
                amount: Number(ore.amount) || 0,
                maxAllowed: Number(ore.maxAllowed) || 0
            };
        });
        return wallet;
    }

    function areaCostsFor(config, areaId) {
        const areaCosts = (config && config.areaCosts) || {};
        return areaCosts[String(areaId)] || [];
    }

    function initialWalletMax(config) {
        const value = config && Number(config.initialOreWalletMax);
        return isFinite(value) && value > 0 ? value : 0;
    }

    function refundFitsWallet(wallet, costs, initialMax) {
        const projected = {};
        for (let index = 0; index < costs.length; index += 1) {
            const cost = costs[index];
            const oreId = String(cost.oreId);
            const refund = Number(cost.amount) || 0;
            let current = projected[oreId];
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
        for (let index = 0; index < costs.length; index += 1) {
            const cost = costs[index];
            const oreId = String(cost.oreId);
            const refund = Number(cost.amount) || 0;
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
        const wallet = cloneWallet(config);
        const initialMax = initialWalletMax(config);
        for (let index = 0; index < areaIds.length; index += 1) {
            const costs = areaCostsFor(config, areaIds[index]);
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
