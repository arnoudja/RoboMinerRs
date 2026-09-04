(function(global) {
    function install(ctx, view) {
        function collectVisibleText(node) {
            if (!node) {
                return '';
            }
            const children = node.children;
            if (children && children.length > 0) {
                const parts = [];
                for (let index = 0; index < children.length; index += 1) {
                    const part = collectVisibleText(children[index]);
                    if (part) {
                        parts.push(part);
                    }
                }
                return parts.join(' ').replace(/\s+/g, ' ').trim();
            }
            return String(node.textContent || '').replace(/\s+/g, ' ').trim();
        }

        function parseWalletAmounts(root) {
            const fragmentRoot = root || ctx.pageRoot || document;
            const wallet = fragmentRoot.querySelector('.mining-queue-wallet');
            const amounts = {};
            if (!wallet) {
                return amounts;
            }
            const items = wallet.querySelectorAll('.page-wallet-item');
            for (let index = 0; index < items.length; index += 1) {
                const item = items[index];
                const oreNode = item.querySelector('.page-wallet-ore');
                const amountNode = item.querySelector('.page-wallet-amount');
                if (!oreNode || !amountNode) {
                    continue;
                }
                const oreName = (oreNode.textContent || '').trim();
                const amountText = (amountNode.textContent || '').trim();
                const current = Number(amountText.split('/')[0]);
                if (!oreName || !isFinite(current)) {
                    continue;
                }
                amounts[oreName] = current;
            }
            return amounts;
        }

        function walletSignature(root) {
            const amounts = parseWalletAmounts(root);
            const oreNames = Object.keys(amounts).sort();
            let walletPart = oreNames.map(function(oreName) {
                return oreName + '=' + amounts[oreName];
            }).join(';');
            const fragmentRoot = root || ctx.pageRoot || document;
            const wallet = fragmentRoot.querySelector('.mining-queue-wallet');
            if (!walletPart) {
                walletPart = collectVisibleText(wallet);
            }
            const hud = document.querySelector('.app-shell-hud');
            const hudPart = collectVisibleText(hud);
            return walletPart + '\n' + hudPart;
        }

        function walletCreditDeltas(before, after) {
            const deltas = [];
            if (!after) {
                return deltas;
            }
            Object.keys(after).forEach(function(oreName) {
                const previous = before && isFinite(before[oreName]) ? before[oreName] : 0;
                const next = after[oreName];
                if (next > previous) {
                    deltas.push({ oreName: oreName, amount: next - previous });
                }
            });
            return deltas;
        }

        function clearCreditFeedbackTimer() {
            if (ctx.creditFeedbackTimer) {
                window.clearTimeout(ctx.creditFeedbackTimer);
                ctx.creditFeedbackTimer = null;
            }
        }

        function showWalletCreditFeedback(deltas) {
            const root = ctx.pageRoot || document.querySelector('.mining-queue-page');
            if (!root || !deltas || deltas.length === 0) {
                return;
            }
            root.querySelectorAll('.mining-queue-credit-banner').forEach(function(node) {
                node.remove();
            });
            const parts = deltas.map(function(delta) {
                return '+' + delta.amount + ' ' + delta.oreName;
            });
            const banner = document.createElement('p');
            banner.setAttribute(
                'class',
                'mining-queue-banner mining-queue-banner-success mining-queue-credit-banner'
            );
            banner.setAttribute('role', 'status');
            banner.textContent = 'Added to wallet: ' + parts.join(', ');
            const wallet = root.querySelector('.mining-queue-wallet');
            const walletParent = wallet && (wallet.parentNode || wallet.parent);
            if (wallet && walletParent) {
                walletParent.insertBefore(banner, wallet);
            } else if (typeof root.appendChild === 'function') {
                root.appendChild(banner);
            }
            clearCreditFeedbackTimer();
            ctx.creditFeedbackTimer = window.setTimeout(function() {
                ctx.creditFeedbackTimer = null;
                if (banner.parentNode) {
                    banner.remove();
                }
            }, ctx.CREDIT_FEEDBACK_MS);
        }

        function hasFinishingRuns(root) {
            const fragmentRoot = root || ctx.pageRoot || document;
            if (fragmentRoot.querySelector('.mining-queue-status-updating')) {
                return true;
            }
            const timers = fragmentRoot.querySelectorAll('.miningqueuetime[data-refresh-on-complete="true"]');
            for (let index = 0; index < timers.length; index += 1) {
                const seconds = Number(timers[index].getAttribute('data-seconds-left'));
                if (!isFinite(seconds) || seconds <= 0) {
                    return true;
                }
            }
            return false;
        }

        function clearClaimRefreshTimer() {
            if (ctx.claimRefreshTimer) {
                window.clearTimeout(ctx.claimRefreshTimer);
                ctx.claimRefreshTimer = null;
            }
        }

        function resetClaimRefreshState() {
            ctx.claimRefreshAttempt = 0;
            ctx.claimBaselineSignature = null;
            ctx.claimBaselineAmounts = null;
            clearClaimRefreshTimer();
        }

        function captureClaimBaseline(root) {
            const fragmentRoot = root || ctx.pageRoot || document.querySelector('.mining-queue-page');
            ctx.claimBaselineSignature = walletSignature(fragmentRoot);
            ctx.claimBaselineAmounts = parseWalletAmounts(fragmentRoot);
        }

        function scheduleClaimRefreshRetry() {
            if (ctx.claimRefreshAttempt >= ctx.CLAIM_REFRESH_BACKOFF_MS.length) {
                resetClaimRefreshState();
                return;
            }
            const delay = ctx.CLAIM_REFRESH_BACKOFF_MS[ctx.claimRefreshAttempt];
            ctx.claimRefreshAttempt += 1;
            clearClaimRefreshTimer();
            ctx.claimRefreshTimer = window.setTimeout(function() {
                ctx.claimRefreshTimer = null;
                performRefresh({ forClaim: true });
            }, delay);
        }

        function performRefresh(options) {
            options = options || {};
            if (ctx.refreshInFlight) {
                ctx.refreshPending = true;
                return Promise.resolve();
            }
            ctx.refreshInFlight = true;
            let root = ctx.pageRoot || document.querySelector('.mining-queue-page');
            if (options.forClaim && ctx.claimBaselineSignature === null) {
                captureClaimBaseline(root);
            }
            return view.fetchFragment('GET', ctx.buildFragmentUrl()).then(function() {
                if (!options.forClaim) {
                    return;
                }
                root = ctx.pageRoot || document.querySelector('.mining-queue-page');
                const nextSignature = walletSignature(root);
                if (
                    ctx.claimBaselineSignature !== null
                    && nextSignature !== ctx.claimBaselineSignature
                ) {
                    const deltas = walletCreditDeltas(
                        ctx.claimBaselineAmounts,
                        parseWalletAmounts(root)
                    );
                    showWalletCreditFeedback(deltas);
                    resetClaimRefreshState();
                    return;
                }
                scheduleClaimRefreshRetry();
            }).catch(function() {
                const query = window.RoboMinerUrlQuery.buildQueryString(ctx.collectQueueQueryParams());
                window.location.replace(query ? 'miningQueue?' + query : 'miningQueue');
            }).finally(function() {
                ctx.refreshInFlight = false;
                if (ctx.refreshPending) {
                    ctx.refreshPending = false;
                    performRefresh(options);
                }
            });
        }

        function refreshQueue(options) {
            options = options || {};
            if (options.forClaim) {
                resetClaimRefreshState();
                captureClaimBaseline(ctx.pageRoot || document.querySelector('.mining-queue-page'));
            }
            if (ctx.refreshDebounceTimer) {
                window.clearTimeout(ctx.refreshDebounceTimer);
            }
            ctx.refreshDebounceTimer = window.setTimeout(function() {
                ctx.refreshDebounceTimer = null;
                performRefresh(options);
            }, ctx.REFRESH_DEBOUNCE_MS);
        }

        return {
            hasFinishingRuns: hasFinishingRuns,
            parseWalletAmounts: parseWalletAmounts,
            performRefresh: performRefresh,
            refreshQueue: refreshQueue,
            showWalletCreditFeedback: showWalletCreditFeedback,
            walletCreditDeltas: walletCreditDeltas,
            walletSignature: walletSignature,
        };
    }

    global.RoboMinerMiningQueueInstall = global.RoboMinerMiningQueueInstall || {};
    global.RoboMinerMiningQueueInstall.claimPoll = install;
})(window);
