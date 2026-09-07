(function(global) {
    function install(ctx, view) {
        const MUTATION_MARKERS = [
            'data-mining-queue-remove',
            'data-mining-queue-clear',
            'data-mining-queue-reorder'
        ];

        let activeDrag = null;
        let suppressClickAfterDrag = false;

        function stripMutationFields(form) {
            for (let markerIndex = 0; markerIndex < MUTATION_MARKERS.length; markerIndex += 1) {
                const staleInputs = form.querySelectorAll('input[' + MUTATION_MARKERS[markerIndex] + ']');
                for (let staleIndex = 0; staleIndex < staleInputs.length; staleIndex += 1) {
                    staleInputs[staleIndex].remove();
                }
            }
        }

        function appendHiddenFields(form, markerAttr, fields) {
            stripMutationFields(form);
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

        function applyFieldsToFormData(formData, fields) {
            Object.keys(fields).forEach(function(name) {
                const value = fields[name];
                if (Array.isArray(value)) {
                    if (typeof formData.delete === 'function') {
                        formData.delete(name);
                    }
                    for (let valueIndex = 0; valueIndex < value.length; valueIndex += 1) {
                        formData.append(name, value[valueIndex]);
                    }
                    return;
                }
                if (typeof formData.set === 'function') {
                    formData.set(name, value);
                }
            });
        }

        function submitFormPartial(form, markerAttr, fields) {
            appendHiddenFields(form, markerAttr, fields);
            const formData = new FormData(form);
            applyFieldsToFormData(formData, fields);
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

        function submitQueuedReorder(form, orderedQueueItemIds) {
            submitFormPartial(form, 'data-mining-queue-reorder', {
                submitType: 'reorder',
                orderedQueueItemId: orderedQueueItemIds
            });
        }

        function queuedItemIds(list) {
            const ids = [];
            const items = list.querySelectorAll('.mining-queue-upcoming-item[data-queue-item-id]');
            for (let index = 0; index < items.length; index += 1) {
                ids.push(items[index].getAttribute('data-queue-item-id'));
            }
            return ids;
        }

        function isInteractiveDragTarget(target) {
            if (!target || !target.closest) {
                return false;
            }
            if (target.closest('.mining-queue-drag-handle')) {
                return false;
            }
            return !!target.closest('button, input, a, label');
        }

        function snapshotListChildren(list) {
            const children = [];
            const source = list.children || [];
            for (let index = 0; index < source.length; index += 1) {
                children.push(source[index]);
            }
            return children;
        }

        function restoreDragOrder() {
            if (!activeDrag || !activeDrag.list) {
                return;
            }
            const original = activeDrag.originalOrder;
            for (let index = 0; index < original.length; index += 1) {
                activeDrag.list.appendChild(original[index]);
            }
        }

        function sameIdSet(left, right) {
            if (left.length !== right.length) {
                return false;
            }
            const expected = left.slice().sort();
            const got = right.slice().sort();
            for (let index = 0; index < expected.length; index += 1) {
                if (expected[index] !== got[index]) {
                    return false;
                }
            }
            return true;
        }

        function onDragStart(event) {
            const item = event.target.closest && event.target.closest('.mining-queue-upcoming-item[draggable="true"]');
            if (!item) {
                return;
            }
            if (isInteractiveDragTarget(event.target)) {
                event.preventDefault();
                return;
            }
            const list = item.closest('.mining-queue-upcoming-list-reorderable');
            const form = item.closest('.mining-queue-card');
            activeDrag = {
                item: item,
                list: list,
                form: form,
                originalIds: list ? queuedItemIds(list) : [],
                originalOrder: list ? snapshotListChildren(list) : []
            };
            if (event.dataTransfer) {
                event.dataTransfer.effectAllowed = 'move';
                event.dataTransfer.setData('text/plain', item.getAttribute('data-queue-item-id') || '');
            }
            item.classList.add('mining-queue-upcoming-item-dragging');
        }

        function onDragOver(event) {
            if (!activeDrag) {
                return;
            }
            event.preventDefault();
            if (event.dataTransfer) {
                event.dataTransfer.dropEffect = 'move';
            }
            const list = activeDrag.list;
            if (!list) {
                return;
            }
            const overList = event.target.closest && event.target.closest('.mining-queue-upcoming-list-reorderable');
            if (overList !== list) {
                return;
            }
            const dragging = activeDrag.item;
            const over = event.target.closest && event.target.closest('.mining-queue-upcoming-item');
            if (!dragging || !over || over === dragging || over.parentNode !== list) {
                return;
            }
            const rect = over.getBoundingClientRect ? over.getBoundingClientRect() : { top: 0, height: 0 };
            const before = event.clientY < rect.top + rect.height / 2;
            if (before) {
                list.insertBefore(dragging, over);
            } else if (over.nextSibling) {
                list.insertBefore(dragging, over.nextSibling);
            } else {
                list.appendChild(dragging);
            }
        }

        function finishDrag() {
            if (activeDrag && activeDrag.item && activeDrag.item.classList) {
                activeDrag.item.classList.remove('mining-queue-upcoming-item-dragging');
            }
            activeDrag = null;
        }

        function onDrop(event) {
            if (!activeDrag) {
                return;
            }
            event.preventDefault();
            suppressClickAfterDrag = true;
            const list = activeDrag.list;
            const form = activeDrag.form;
            const originalIds = activeDrag.originalIds;
            const overList = event.target.closest && event.target.closest('.mining-queue-upcoming-list-reorderable');
            if (!list || overList !== list || !form) {
                restoreDragOrder();
                finishDrag();
                return;
            }
            const ids = queuedItemIds(list);
            if (ids.length < 2 || !sameIdSet(ids, originalIds)) {
                restoreDragOrder();
                finishDrag();
                return;
            }
            const unchanged = ids.join(',') === originalIds.join(',');
            finishDrag();
            if (unchanged) {
                return;
            }
            submitQueuedReorder(form, ids);
        }

        function onDragEnd(event) {
            const item = event.target.closest && event.target.closest('.mining-queue-upcoming-item');
            if (item) {
                item.classList.remove('mining-queue-upcoming-item-dragging');
            }
            if (activeDrag) {
                restoreDragOrder();
                finishDrag();
            }
        }

        function consumeDragClickSuppression() {
            if (!suppressClickAfterDrag) {
                return false;
            }
            suppressClickAfterDrag = false;
            return true;
        }

        ctx.updateClearButtonLabel = updateClearButtonLabel;

        return {
            clearQueuedRuns: clearQueuedRuns,
            removeQueuedRun: removeQueuedRun,
            submitFormPartial: submitFormPartial,
            submitQueuedReorder: submitQueuedReorder,
            onDragStart: onDragStart,
            onDragOver: onDragOver,
            onDrop: onDrop,
            onDragEnd: onDragEnd,
            consumeDragClickSuppression: consumeDragClickSuppression
        };
    }

    global.RoboMinerMiningQueueInstall = global.RoboMinerMiningQueueInstall || {};
    global.RoboMinerMiningQueueInstall.actions = install;
})(window);
