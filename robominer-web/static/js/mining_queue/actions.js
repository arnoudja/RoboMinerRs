(function(global) {
    function install(ctx, view) {
        function appendHiddenFields(form, markerAttr, fields) {
            var staleInputs = form.querySelectorAll('input[' + markerAttr + '="true"]');
            for (var staleIndex = 0; staleIndex < staleInputs.length; staleIndex += 1) {
                staleInputs[staleIndex].remove();
            }
            Object.keys(fields).forEach(function(name) {
                var value = fields[name];
                var values = Array.isArray(value) ? value : [value];
                for (var valueIndex = 0; valueIndex < values.length; valueIndex += 1) {
                    var input = document.createElement('input');
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
            var formData = new FormData(form);
            return view.fetchFragment('POST', ctx.buildFragmentUrl(), formData).catch(function() {
                form.submit();
            });
        }

        function updateClearButtonLabel(form) {
            var clearButton = form.querySelector('.mining-queue-clear-btn');
            if (!clearButton) {
                return;
            }
            var checked = form.querySelectorAll('.mining-queue-item-check:checked');
            clearButton.textContent = checked.length > 0 ? 'Clear selected' : 'Clear queue';
        }

        function submitQueuedRunRemoval(form, queueItemId) {
            submitFormPartial(form, 'data-mining-queue-remove', {
                selectedQueueItemId: queueItemId,
                submitType: 'remove'
            });
        }

        function removeQueuedRun(button) {
            var form = button.closest('.mining-queue-card');
            if (!form) {
                return;
            }
            var queueItemId = button.getAttribute('data-queue-item-id');
            if (!queueItemId) {
                return;
            }
            var row = button.closest('.mining-queue-run-row');
            var area = row ? row.querySelector('.mining-queue-run-area') : null;
            var areaName = area ? area.textContent.trim() : 'queued run';
            var message = 'Remove queued run in ' + areaName + '?';
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
            var empty = { ores: {}, areaCosts: {}, initialOreWalletMax: 0 };
            var configEl = document.getElementById('mining-queue-clear-config');
            if (!configEl) {
                return empty;
            }
            try {
                var parsed = JSON.parse(configEl.textContent || '{}');
                if (!parsed || typeof parsed !== 'object') {
                    return empty;
                }
                return parsed;
            } catch (error) {
                return empty;
            }
        }

        function submitQueueClear(form, clearMode, selectedQueueItemIds) {
            var fields = {
                submitType: 'clear',
                clearMode: clearMode
            };
            if (selectedQueueItemIds && selectedQueueItemIds.length > 0) {
                fields.selectedQueueItemId = selectedQueueItemIds;
            }
            submitFormPartial(form, 'data-mining-queue-clear', fields);
        }

        function clearQueuedRuns(button) {
            var form = button.closest('.mining-queue-card');
            if (!form || button.disabled) {
                return;
            }
            var checked = form.querySelectorAll('.mining-queue-item-check:checked');
            var selectedOnly = checked.length > 0;
            var targets = selectedOnly
                ? checked
                : form.querySelectorAll('.mining-queue-remove-btn[data-queue-item-id]');
            if (!targets.length) {
                return;
            }
            var clearHelpers = window.RoboMinerMiningQueueClear;
            if (!clearHelpers) {
                return;
            }
            var config = readClearConfig();
            var areaIds = [];
            var selectedQueueItemIds = [];
            for (var targetIndex = 0; targetIndex < targets.length; targetIndex += 1) {
                areaIds.push(targets[targetIndex].getAttribute('data-mining-area-id'));
                if (selectedOnly) {
                    selectedQueueItemIds.push(targets[targetIndex].getAttribute('data-queue-item-id'));
                }
            }
            var wouldLoseOre = clearHelpers.clearingAllWouldLoseOre(config, areaIds);

            function proceed(clearMode) {
                submitQueueClear(form, clearMode, selectedOnly ? selectedQueueItemIds : null);
            }

            if (wouldLoseOre) {
                var lossMessage = selectedOnly
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

            var message = selectedOnly
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
