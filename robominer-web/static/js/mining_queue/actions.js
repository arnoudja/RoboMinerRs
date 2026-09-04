(function(global) {
    function install(ctx, view) {
        function appendHiddenFields(form, markerAttr, fields) {
            const staleInputs = form.querySelectorAll('input[' + markerAttr + '="true"]');
            for (let staleIndex = 0; staleIndex < staleInputs.length; staleIndex += 1) {
                staleInputs[staleIndex].remove();
            }
            Object.keys(fields).forEach(function(name) {
                const value = fields[name];
                const values = Array.isArray(value) ? value : [value];
                for (let valueIndex = 0; valueIndex < values.length; valueIndex += 1) {
                    const input = document.createElement('input');
                    input.type = 'hidden';
                    input.name = name;
                    input.value = values[valueIndex];
                    input.setAttribute(markerAttr, 'true');
                    form.appendChild(input);
                }
            });
        }

        function submitFormPartial(form, markerAttr, fields) {
            appendHiddenFields(form, markerAttr, fields);
            const formData = new FormData(form);
            return view.fetchFragment('POST', ctx.buildFragmentUrl(), formData).catch(function() {
                form.submit();
            });
        }

        function updateClearButtonLabel(form) {
            const clearButton = form.querySelector('.mining-queue-clear-btn');
            if (!clearButton) {
                return;
            }
            const checked = form.querySelectorAll('.mining-queue-item-check:checked');
            clearButton.textContent = checked.length > 0 ? 'Clear selected' : 'Clear queue';
        }

        function submitQueuedRunRemoval(form, queueItemId) {
            submitFormPartial(form, 'data-mining-queue-remove', {
                selectedQueueItemId: queueItemId,
                submitType: 'remove'
            });
        }

        function removeQueuedRun(button) {
            const form = button.closest('.mining-queue-card');
            if (!form) {
                return;
            }
            const queueItemId = button.getAttribute('data-queue-item-id');
            if (!queueItemId) {
                return;
            }
            const row = button.closest('.mining-queue-run-row');
            const area = row ? row.querySelector('.mining-queue-run-area') : null;
            const areaName = area ? area.textContent.trim() : 'queued run';
            const message = 'Remove queued run in ' + areaName + '?';
            if (typeof window.robominerConfirm === 'function') {
                window.robominerConfirm(message, function(confirmed) {
                    if (!confirmed) {
                        return;
                    }
                    submitQueuedRunRemoval(form, queueItemId);
                });
                return;
            }
            if (window.confirm(message)) {
                submitQueuedRunRemoval(form, queueItemId);
            }
        }

        function readClearConfig() {
            const empty = { ores: {}, areaCosts: {}, initialOreWalletMax: 0 };
            const configEl = document.getElementById('mining-queue-clear-config');
            if (!configEl) {
                return empty;
            }
            try {
                const parsed = JSON.parse(configEl.textContent || '{}');
                if (!parsed || typeof parsed !== 'object') {
                    return empty;
                }
                return parsed;
            } catch (error) {
                return empty;
            }
        }

        function submitQueueClear(form, clearMode, selectedQueueItemIds) {
            const fields = {
                submitType: 'clear',
                clearMode: clearMode
            };
            if (selectedQueueItemIds && selectedQueueItemIds.length > 0) {
                fields.selectedQueueItemId = selectedQueueItemIds;
            }
            submitFormPartial(form, 'data-mining-queue-clear', fields);
        }

        function clearQueuedRuns(button) {
            const form = button.closest('.mining-queue-card');
            if (!form || button.disabled) {
                return;
            }
            const checked = form.querySelectorAll('.mining-queue-item-check:checked');
            const selectedOnly = checked.length > 0;
            const targets = selectedOnly
                ? checked
                : form.querySelectorAll('.mining-queue-remove-btn[data-queue-item-id]');
            if (!targets.length) {
                return;
            }
            const clearHelpers = window.RoboMinerMiningQueueClear;
            if (!clearHelpers) {
                return;
            }
            const config = readClearConfig();
            const areaIds = [];
            const selectedQueueItemIds = [];
            for (let targetIndex = 0; targetIndex < targets.length; targetIndex += 1) {
                areaIds.push(targets[targetIndex].getAttribute('data-mining-area-id'));
                if (selectedOnly) {
                    selectedQueueItemIds.push(targets[targetIndex].getAttribute('data-queue-item-id'));
                }
            }
            const wouldLoseOre = clearHelpers.clearingAllWouldLoseOre(config, areaIds);

            function proceed(clearMode) {
                submitQueueClear(form, clearMode, selectedOnly ? selectedQueueItemIds : null);
            }

            if (wouldLoseOre) {
                const lossMessage = selectedOnly
                    ? 'Clearing the selected runs would refund ore past your wallet maximum, so some ore would be lost. Clear selected runs anyway, or only clear runs that fit without losing ore?'
                    : 'Clearing this queue would refund ore past your wallet maximum, so some ore would be lost. Clear all queued runs anyway, or only clear runs that fit without losing ore?';
                if (typeof window.robominerConfirmChoice === 'function') {
                    window.robominerConfirmChoice(
                        lossMessage,
                        {
                            confirmLabel: selectedOnly ? 'Clear selected' : 'Clear all',
                            altLabel: 'Clear without losing ore'
                        },
                        function(result) {
                            if (result === 'confirm') {
                                proceed('all');
                            } else if (result === 'alt') {
                                proceed('safe');
                            }
                        }
                    );
                }
                return;
            }

            const message = selectedOnly
                ? 'Clear selected queued runs for this robot?'
                : 'Clear all queued runs for this robot?';
            if (typeof window.robominerConfirm === 'function') {
                window.robominerConfirm(message, function(confirmed) {
                    if (!confirmed) {
                        return;
                    }
                    proceed('all');
                });
                return;
            }
            if (window.confirm(message)) {
                proceed('all');
            }
        }

        ctx.updateClearButtonLabel = updateClearButtonLabel;

        return {
            clearQueuedRuns: clearQueuedRuns,
            removeQueuedRun: removeQueuedRun,
            submitFormPartial: submitFormPartial,
        };
    }

    global.RoboMinerMiningQueueInstall = global.RoboMinerMiningQueueInstall || {};
    global.RoboMinerMiningQueueInstall.actions = install;
})(window);
