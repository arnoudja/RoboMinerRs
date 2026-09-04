(function(global) {
    function install(ctx) {
        function formatTimeLeft(seconds) {
            const secondsLeft = Math.max(0, Math.floor(seconds));
            const displaySeconds = secondsLeft % 60;
            const displayMinutes = Math.floor(secondsLeft / 60) % 60;
            const displayHours = Math.floor(secondsLeft / 3600);
            let result = displayHours > 0 ? displayHours + ':' : '';
            if (displayMinutes < 10 && displayHours > 0) {
                result += '0';
            }
            result += displayMinutes + ':';
            if (displaySeconds < 10) {
                result += '0';
            }
            return result + displaySeconds;
        }

        function applyHudFragment(hudSource) {
            if (!hudSource) {
                return;
            }
            const incomingHud = hudSource.querySelector('.app-shell-hud');
            const hudTarget = document.querySelector('.app-shell-hud');
            if (incomingHud) {
                const hudParent = hudTarget && (hudTarget.parentNode || hudTarget.parent);
                if (!hudParent) {
                    return;
                }
                hudParent.insertBefore(incomingHud, hudTarget);
                hudTarget.remove();
                return;
            }
            const trimmed = (hudSource.innerHTML || '').trim();
            if (!trimmed) {
                return;
            }
            if (hudTarget) {
                hudTarget.innerHTML = hudSource.innerHTML;
            }
        }

        function applyFragment(html, root) {
            const parser = new window.DOMParser();
            const doc = parser.parseFromString(html, 'text/html');
            const fragmentRoot = root || ctx.pageRoot || document;
            const fragment = doc.getElementById('mining-queue-fragment');
            if (!fragment) {
                throw new Error('missing mining queue fragment');
            }

            const hudSource = doc.getElementById('mining-queue-hud-fragment');
            applyHudFragment(hudSource);

            const dynamicSource = doc.getElementById('mining-queue-dynamic-fragment');
            if (!dynamicSource) {
                throw new Error('missing mining queue dynamic fragment');
            }

            const walletSource = dynamicSource.querySelector('.mining-queue-wallet');
            const walletTarget = fragmentRoot.querySelector('.mining-queue-wallet');
            const walletParent = walletTarget && (walletTarget.parentNode || walletTarget.parent);
            if (walletSource && walletTarget && walletParent) {
                walletParent.insertBefore(walletSource, walletTarget);
                walletTarget.remove();
            } else if (walletSource && walletTarget) {
                walletTarget.outerHTML = walletSource.outerHTML;
            }

            const deck = fragmentRoot.querySelector('.mining-queue-deck');
            fragmentRoot.querySelectorAll('.page-help-hint, .mining-queue-error').forEach(function(node) {
                node.remove();
            });
            const messages = dynamicSource.querySelectorAll('.page-help-hint, .mining-queue-error');
            for (let messageIndex = 0; messageIndex < messages.length; messageIndex += 1) {
                if (deck) {
                    fragmentRoot.insertBefore(messages[messageIndex].cloneNode(true), deck);
                }
            }

            const robotsSource = doc.getElementById('mining-queue-robots-fragment');
            const robotsTarget = fragmentRoot.querySelector('.mining-queue-robots');
            if (robotsSource && robotsTarget) {
                applyRobotsFragment(robotsSource, robotsTarget);
            }

            const configSource = doc.getElementById('mining-queue-clear-config');
            const configTarget = document.getElementById('mining-queue-clear-config');
            if (configSource && configTarget) {
                configTarget.textContent = configSource.textContent;
            }
        }

        function cardRobotId(card) {
            const fromAttr = card.getAttribute('data-robot-id');
            if (fromAttr) {
                return fromAttr;
            }
            const robotInput = card.querySelector('input[name="robotId"]');
            return robotInput && robotInput.value ? String(robotInput.value) : '';
        }

        function mapRobotCards(root) {
            const cards = root.querySelectorAll('.mining-queue-card');
            const map = {};
            const ids = [];
            for (let index = 0; index < cards.length; index += 1) {
                const card = cards[index];
                const robotId = cardRobotId(card);
                if (!robotId) {
                    return null;
                }
                if (Object.prototype.hasOwnProperty.call(map, robotId)) {
                    return null;
                }
                map[robotId] = card;
                ids.push(robotId);
            }
            ids.sort();
            return { map: map, ids: ids };
        }

        function sameRobotIdSet(leftIds, rightIds) {
            if (leftIds.length !== rightIds.length) {
                return false;
            }
            for (let index = 0; index < leftIds.length; index += 1) {
                if (leftIds[index] !== rightIds[index]) {
                    return false;
                }
            }
            return true;
        }

        function syncOptionBlockReasons(liveSelect, incomingSelect) {
            if (!liveSelect || !incomingSelect) {
                return;
            }
            const incomingByValue = {};
            const incomingOptions = incomingSelect.options || incomingSelect.querySelectorAll('option');
            for (let incomingIndex = 0; incomingIndex < incomingOptions.length; incomingIndex += 1) {
                const incomingOption = incomingOptions[incomingIndex];
                incomingByValue[String(incomingOption.value)] = incomingOption;
            }
            const liveOptions = liveSelect.options || liveSelect.querySelectorAll('option');
            for (let liveIndex = 0; liveIndex < liveOptions.length; liveIndex += 1) {
                const liveOption = liveOptions[liveIndex];
                const match = incomingByValue[String(liveOption.value)];
                if (!match) {
                    continue;
                }
                const blockReason = match.getAttribute('data-block-reason');
                if (blockReason === null || blockReason === '') {
                    liveOption.removeAttribute('data-block-reason');
                } else {
                    liveOption.setAttribute('data-block-reason', blockReason);
                }
            }
        }

        function syncClearButton(liveCard, incomingCard) {
            const liveClear = liveCard.querySelector('.mining-queue-clear-btn');
            const incomingClear = incomingCard.querySelector('.mining-queue-clear-btn');
            if (!liveClear || !incomingClear) {
                return;
            }
            const clearableCount = incomingClear.getAttribute('data-clearable-count');
            if (clearableCount === null) {
                liveClear.removeAttribute('data-clearable-count');
            } else {
                liveClear.setAttribute('data-clearable-count', clearableCount);
            }
            liveClear.disabled = !!incomingClear.disabled;
            const title = incomingClear.getAttribute('title');
            if (title === null || title === '') {
                liveClear.removeAttribute('title');
            } else {
                liveClear.setAttribute('title', title);
            }
        }

        function syncCsrfToken(liveCard, incomingCard) {
            const incomingCsrf = incomingCard.querySelector('input[name="csrfToken"]');
            if (!incomingCsrf || !incomingCsrf.value) {
                return;
            }
            const liveCsrf = liveCard.querySelector('input[name="csrfToken"]');
            if (liveCsrf) {
                liveCsrf.value = incomingCsrf.value;
                liveCsrf.setAttribute('value', incomingCsrf.value);
                return;
            }
            const robotInput = liveCard.querySelector('input[name="robotId"]');
            const created = document.createElement('input');
            created.type = 'hidden';
            created.name = 'csrfToken';
            created.value = incomingCsrf.value;
            created.setAttribute('value', incomingCsrf.value);
            const insertParent = robotInput && (robotInput.parentNode || robotInput.parent);
            if (insertParent) {
                insertParent.insertBefore(created, robotInput);
            } else {
                liveCard.appendChild(created);
            }
        }

        function patchRobotCardActions(liveCard, incomingCard) {
            syncCsrfToken(liveCard, incomingCard);
            const liveSelect = liveCard.querySelector('select.mining-queue-area-select');
            const incomingSelect = incomingCard.querySelector('select.mining-queue-area-select');
            syncOptionBlockReasons(liveSelect, incomingSelect);
            syncClearButton(liveCard, incomingCard);
            if (liveSelect) {
                updateRobotEnqueueState(liveSelect);
            }
            if (ctx.updateClearButtonLabel) {
                ctx.updateClearButtonLabel(liveCard);
            }
        }

        function applyRobotsFragment(robotsSource, robotsTarget) {
            const liveCards = mapRobotCards(robotsTarget);
            const incomingCards = mapRobotCards(robotsSource);
            if (!liveCards || !incomingCards || !sameRobotIdSet(liveCards.ids, incomingCards.ids)) {
                robotsTarget.innerHTML = robotsSource.innerHTML;
                return;
            }

            for (let index = 0; index < liveCards.ids.length; index += 1) {
                const robotId = liveCards.ids[index];
                const liveCard = liveCards.map[robotId];
                const incomingCard = incomingCards.map[robotId];
                const liveStatus = liveCard.querySelector('.mining-queue-card-status');
                const incomingStatus = incomingCard.querySelector('.mining-queue-card-status');
                if (!liveStatus || !incomingStatus) {
                    robotsTarget.innerHTML = robotsSource.innerHTML;
                    return;
                }
                liveStatus.innerHTML = incomingStatus.innerHTML;
                patchRobotCardActions(liveCard, incomingCard);
            }
        }

        function formDataToUrlEncoded(formData) {
            const params = new URLSearchParams();
            formData.forEach(function(value, key) {
                params.append(key, value);
            });
            return params.toString();
        }

        function fetchFragment(method, url, body) {
            const scrollEl = document.getElementById('main-content');
            const scrollTop = scrollEl ? scrollEl.scrollTop : 0;
            const options = {
                method: method,
                credentials: 'same-origin'
            };
            if (body) {
                options.body = body instanceof FormData ? formDataToUrlEncoded(body) : body;
                options.headers = {
                    'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8'
                };
            }

            return window.fetch(url, options).then(function(response) {
                if (!response.ok) {
                    throw new Error('mining queue fragment request failed');
                }
                return response.text();
            }).then(function(html) {
                applyFragment(html);
                ctx.init({ skipRestore: true });
                if (scrollEl) {
                    scrollEl.scrollTop = scrollTop;
                }
            });
        }

        function showMiningAreaDetails(areaId) {
            const panels = document.querySelectorAll('tbody.mining-queue-area-panel');
            for (let index = 0; index < panels.length; index += 1) {
                const panel = panels[index];
                if (panel.id === 'miningAreaDetails' + areaId) {
                    panel.classList.add('mining-queue-area-panel-active');
                } else {
                    panel.classList.remove('mining-queue-area-panel-active');
                }
            }
        }

        function syncInspectorArea(areaId) {
            showMiningAreaDetails(areaId);
            window.RoboMinerUrlQuery.sync('miningQueue', ctx.collectQueueQueryParams());
            ctx.writeStoredAreaSelections();
        }

        function updateRobotEnqueueState(select) {
            const form = select.closest('.mining-queue-card');
            if (!form) {
                return;
            }
            const selectedOption = select.options[select.selectedIndex];
            let blockReason = selectedOption ? selectedOption.getAttribute('data-block-reason') : '';
            if (blockReason === null) {
                blockReason = '';
            }
            const disabled = blockReason.length > 0;
            const buttons = form.querySelectorAll('button[name="submitType"][value="add"], button[name="submitType"][value="fill"]');
            for (let buttonIndex = 0; buttonIndex < buttons.length; buttonIndex += 1) {
                const button = buttons[buttonIndex];
                button.disabled = disabled;
                if (disabled) {
                    button.setAttribute('title', blockReason);
                } else {
                    button.removeAttribute('title');
                }
            }
            const hint = form.querySelector('.mining-queue-action-hint');
            if (hint) {
                hint.textContent = blockReason;
                hint.hidden = !disabled;
            }
        }

        function startTimer(cell) {
            const seconds = Number(cell.getAttribute('data-seconds-left'));
            if (!isFinite(seconds)) {
                return;
            }
            const refreshOnComplete = cell.getAttribute('data-refresh-on-complete') === 'true';
            const progressTotal = Number(cell.getAttribute('data-progress-total'));
            function updateProgress(secondsLeft) {
                if (!isFinite(progressTotal) || progressTotal <= 0) {
                    return;
                }
                const run = cell.closest('.mining-queue-run-active');
                if (!run) {
                    return;
                }
                const progressBar = run.querySelector('progress.mining-queue-progress');
                if (!progressBar) {
                    return;
                }
                const elapsed = progressTotal - Math.max(0, secondsLeft);
                const percent = Math.min(100, Math.max(0, (elapsed / progressTotal) * 100));
                progressBar.value = percent.toFixed(1);
            }
            if (seconds <= 0) {
                updateProgress(0);
                if (refreshOnComplete) {
                    ctx.refreshQueue({ forClaim: true });
                }
                return;
            }
            const startTime = Date.now();
            updateProgress(seconds);
            const interval = window.setInterval(function() {
                const secondsLeft = seconds - ((Date.now() - startTime) / 1000);
                if (secondsLeft > 0) {
                    cell.textContent = formatTimeLeft(secondsLeft);
                    updateProgress(secondsLeft);
                    return;
                }
                window.clearInterval(interval);
                cell.textContent = formatTimeLeft(0);
                updateProgress(0);
                if (refreshOnComplete) {
                    ctx.refreshQueue({ forClaim: true });
                }
            }, 200);
            ctx.timerIntervals.push(interval);
            cell.textContent = formatTimeLeft(seconds);
        }

        function areaNameOverflows(area) {
            const target = area.querySelector('a') || area;
            return target.scrollWidth > target.clientWidth + 1;
        }

        function syncQueuedStatusVisibility(row) {
            const area = row.querySelector('.mining-queue-run-area');
            const status = row.querySelector('.mining-queue-status-queued');
            if (!area || !status) {
                return;
            }
            status.classList.remove('mining-queue-status-compact-hidden');
            if (areaNameOverflows(area)) {
                status.classList.add('mining-queue-status-compact-hidden');
            }
        }

        function syncAllQueuedStatusVisibility() {
            const rows = document.querySelectorAll('.mining-queue-run-row');
            for (let rowIndex = 0; rowIndex < rows.length; rowIndex += 1) {
                syncQueuedStatusVisibility(rows[rowIndex]);
            }
        }

        function observeQueuedStatusVisibility() {
            function scheduleSync() {
                window.requestAnimationFrame(function() {
                    window.requestAnimationFrame(syncAllQueuedStatusVisibility);
                });
            }
            scheduleSync();
            if (typeof ResizeObserver === 'undefined') {
                return;
            }
            ctx.resizeObserver = new ResizeObserver(scheduleSync);
            const containers = document.querySelectorAll('.mining-queue-card, .mining-queue-run, .mining-queue-upcoming-list li');
            for (let containerIndex = 0; containerIndex < containers.length; containerIndex += 1) {
                ctx.resizeObserver.observe(containers[containerIndex]);
            }
        }

        function initView(options) {
            const robotAreaSelects = document.querySelectorAll('.mining-queue-card select[name^="miningArea"]');
            for (let selectIndex = 0; selectIndex < robotAreaSelects.length; selectIndex += 1) {
                updateRobotEnqueueState(robotAreaSelects[selectIndex]);
            }

            const forms = document.querySelectorAll('.mining-queue-card');
            for (let formIndex = 0; formIndex < forms.length; formIndex += 1) {
                ctx.updateClearButtonLabel(forms[formIndex]);
            }

            observeQueuedStatusVisibility();

            const cells = document.querySelectorAll('.miningqueuetime[data-seconds-left]');
            for (let cellIndex = 0; cellIndex < cells.length; cellIndex += 1) {
                startTimer(cells[cellIndex]);
            }

            if (!options.skipRestore) {
                try {
                    ctx.restoreAreaSelectionsFromStorage();
                } catch (error) {
                }
            }
        }

        return {
            applyFragment: applyFragment,
            applyHudFragment: applyHudFragment,
            fetchFragment: fetchFragment,
            formDataToUrlEncoded: formDataToUrlEncoded,
            initView: initView,
            syncInspectorArea: syncInspectorArea,
            updateRobotEnqueueState: updateRobotEnqueueState,
        };
    }

    global.RoboMinerMiningQueueInstall = global.RoboMinerMiningQueueInstall || {};
    global.RoboMinerMiningQueueInstall.view = install;
})(window);
