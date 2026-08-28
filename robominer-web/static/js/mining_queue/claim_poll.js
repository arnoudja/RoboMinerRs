(function(global) {
    function install(ctx, view) {
        function collectVisibleText(node) {
            if (!node) {
                return '';
            }
            var children = node.children;
            if (children && children.length > 0) {
                var parts = [];
                for (var index = 0; index < children.length; index += 1) {
                    var part = collectVisibleText(children[index]);
                    if (part) {
                        parts.push(part);
                    }
                }
                return parts.join(' ').replace(/\s+/g, ' ').trim();
            }
            return String(node.textContent || '').replace(/\s+/g, ' ').trim();
        }

        function parseWalletAmounts(root) {
            var fragmentRoot = root || ctx.pageRoot || document;
            var wallet = fragmentRoot.querySelector('.mining-queue-wallet');
            var amounts = {};
            if (!wallet) {
                return amounts;
            }
            var items = wallet.querySelectorAll('.page-wallet-item');
            for (var index = 0; index < items.length; index += 1) {
                var item = items[index];
                var oreNode = item.querySelector('.page-wallet-ore');
                var amountNode = item.querySelector('.page-wallet-amount');
                if (!oreNode || !amountNode) {
                    continue;
                }
                var oreName = (oreNode.textContent || '').trim();
                var amountText = (amountNode.textContent || '').trim();
                var current = Number(amountText.split('/')[0]);
                if (!oreName || !isFinite(current)) {
                    continue;
                }
                amounts[oreName] = current;
            }
            return amounts;
        }

        function walletSignature(root) {
            var amounts = parseWalletAmounts(root);
            var oreNames = Object.keys(amounts).sort();
            var walletPart = oreNames.map(function(oreName) {
                return oreName + '=' + amounts[oreName];
            }).join(';');
            var fragmentRoot = root || ctx.pageRoot || document;
            var wallet = fragmentRoot.querySelector('.mining-queue-wallet');
            if (!walletPart) {
                walletPart = collectVisibleText(wallet);
            }
            var hud = document.querySelector('.app-shell-hud');
            var hudPart = collectVisibleText(hud);
            return walletPart + '\n' + hudPart;
        }

        function walletCreditDeltas(before, after) {
            var deltas = [];
            if (!after) {
                return deltas;
            }
            Object.keys(after).forEach(function(oreName) {
                var previous = before && isFinite(before[oreName]) ? before[oreName] : 0;
                var next = after[oreName];
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
            var root = ctx.pageRoot || document.querySelector('.mining-queue-page');
            if (!root || !deltas || deltas.length === 0) {
                return;
            }
            root.querySelectorAll('.mining-queue-credit-banner').forEach(function(node) {
                node.remove();
            });
            var parts = deltas.map(function(delta) {
                return '+' + delta.amount + ' ' + delta.oreName;
            });
            var banner = document.createElement('p');
            banner.setAttribute(
                'class',
                'mining-queue-banner mining-queue-banner-success mining-queue-credit-banner'
            );
            banner.setAttribute('role', 'status');
            banner.textContent = 'Added to wallet: ' + parts.join(', ');
            var wallet = root.querySelector('.mining-queue-wallet');
            var walletParent = wallet && (wallet.parentNode || wallet.parent);
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
            var fragmentRoot = root || ctx.pageRoot || document;
            if (fragmentRoot.querySelector('.mining-queue-status-updating')) {
                return true;
            }
            var timers = fragmentRoot.querySelectorAll('.miningqueuetime[data-refresh-on-complete="true"]');
            for (var index = 0; index < timers.length; index += 1) {
                var seconds = Number(timers[index].getAttribute('data-seconds-left'));
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
            var fragmentRoot = root || ctx.pageRoot || document.querySelector('.mining-queue-page');
            ctx.claimBaselineSignature = walletSignature(fragmentRoot);
            ctx.claimBaselineAmounts = parseWalletAmounts(fragmentRoot);
        }

        function scheduleClaimRefreshRetry() {
            if (ctx.claimRefreshAttempt >= ctx.CLAIM_REFRESH_BACKOFF_MS.length) {
                resetClaimRefreshState();
                return;
            }
            var delay = ctx.CLAIM_REFRESH_BACKOFF_MS[ctx.claimRefreshAttempt];
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
            var root = ctx.pageRoot || document.querySelector('.mining-queue-page');
            if (options.forClaim && ctx.claimBaselineSignature === null) {
                captureClaimBaseline(root);
            }
            return view.fetchFragment('GET', ctx.buildFragmentUrl()).then(function() {
                if (!options.forClaim) {
                    return;
                }
                root = ctx.pageRoot || document.querySelector('.mining-queue-page');
                var nextSignature = walletSignature(root);
                if (
                    ctx.claimBaselineSignature !== null
                    && nextSignature !== ctx.claimBaselineSignature
                ) {
                    var deltas = walletCreditDeltas(
                        ctx.claimBaselineAmounts,
                        parseWalletAmounts(root)
                    );
                    showWalletCreditFeedback(deltas);
                    resetClaimRefreshState();
                    return;
                }
                scheduleClaimRefreshRetry();
            }).catch(function() {
                var query = window.RoboMinerUrlQuery.buildQueryString(ctx.collectQueueQueryParams());
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
