(function(global) {
    function install(ctx) {
        function formatTimeLeft(seconds) {
            var secondsLeft = Math.max(0, Math.floor(seconds));
            var displaySeconds = secondsLeft % 60;
            var displayMinutes = Math.floor(secondsLeft / 60) % 60;
            var displayHours = Math.floor(secondsLeft / 3600);
            var result = displayHours > 0 ? displayHours + ':' : '';
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
            var incomingHud = hudSource.querySelector('.app-shell-hud');
            var hudTarget = document.querySelector('.app-shell-hud');
            if (incomingHud) {
                var hudParent = hudTarget && (hudTarget.parentNode || hudTarget.parent);
                if (!hudParent) {
                    return;
                }
                hudParent.insertBefore(incomingHud, hudTarget);
                hudTarget.remove();
                return;
            }
            var trimmed = (hudSource.innerHTML || '').trim();
            if (!trimmed) {
                return;
            }
            if (hudTarget) {
                hudTarget.innerHTML = hudSource.innerHTML;
            }
        }

        function applyFragment(html, root) {
            var parser = new window.DOMParser();
            var doc = parser.parseFromString(html, 'text/html');
            var fragmentRoot = root || ctx.pageRoot || document;
            var fragment = doc.getElementById('mining-queue-fragment');
            if (!fragment) {
                throw new Error('missing mining queue fragment');
            }

            var hudSource = doc.getElementById('mining-queue-hud-fragment');
            applyHudFragment(hudSource);

            var dynamicSource = doc.getElementById('mining-queue-dynamic-fragment');
            if (!dynamicSource) {
                throw new Error('missing mining queue dynamic fragment');
            }

            var walletSource = dynamicSource.querySelector('.mining-queue-wallet');
            var walletTarget = fragmentRoot.querySelector('.mining-queue-wallet');
            var walletParent = walletTarget && (walletTarget.parentNode || walletTarget.parent);
            if (walletSource && walletTarget && walletParent) {
                walletParent.insertBefore(walletSource, walletTarget);
                walletTarget.remove();
            } else if (walletSource && walletTarget) {
                walletTarget.outerHTML = walletSource.outerHTML;
            }

            var deck = fragmentRoot.querySelector('.mining-queue-deck');
            fragmentRoot.querySelectorAll('.page-help-hint, .mining-queue-error').forEach(function(node) {
                node.remove();
            });
            var messages = dynamicSource.querySelectorAll('.page-help-hint, .mining-queue-error');
            for (var messageIndex = 0; messageIndex < messages.length; messageIndex += 1) {
                if (deck) {
                    fragmentRoot.insertBefore(messages[messageIndex].cloneNode(true), deck);
                }
            }

            var robotsSource = doc.getElementById('mining-queue-robots-fragment');
            var robotsTarget = fragmentRoot.querySelector('.mining-queue-robots');
            if (robotsSource && robotsTarget) {
                applyRobotsFragment(robotsSource, robotsTarget);
            }

            var configSource = doc.getElementById('mining-queue-clear-config');
            var configTarget = document.getElementById('mining-queue-clear-config');
            if (configSource && configTarget) {
                configTarget.textContent = configSource.textContent;
            }
        }

        function cardRobotId(card) {
            var fromAttr = card.getAttribute('data-robot-id');
            if (fromAttr) {
                return fromAttr;
            }
            var robotInput = card.querySelector('input[name="robotId"]');
            return robotInput && robotInput.value ? String(robotInput.value) : '';
        }

        function mapRobotCards(root) {
            var cards = root.querySelectorAll('.mining-queue-card');
            var map = {};
            var ids = [];
            for (var index = 0; index < cards.length; index += 1) {
                var card = cards[index];
                var robotId = cardRobotId(card);
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
            for (var index = 0; index < leftIds.length; index += 1) {
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
            var incomingByValue = {};
            var incomingOptions = incomingSelect.options || incomingSelect.querySelectorAll('option');
            for (var incomingIndex = 0; incomingIndex < incomingOptions.length; incomingIndex += 1) {
                var incomingOption = incomingOptions[incomingIndex];
                incomingByValue[String(incomingOption.value)] = incomingOption;
            }
            var liveOptions = liveSelect.options || liveSelect.querySelectorAll('option');
            for (var liveIndex = 0; liveIndex < liveOptions.length; liveIndex += 1) {
                var liveOption = liveOptions[liveIndex];
                var match = incomingByValue[String(liveOption.value)];
                if (!match) {
                    continue;
                }
                var blockReason = match.getAttribute('data-block-reason');
                if (blockReason === null || blockReason === '') {
                    liveOption.removeAttribute('data-block-reason');
                } else {
                    liveOption.setAttribute('data-block-reason', blockReason);
                }
            }
        }

        function syncClearButton(liveCard, incomingCard) {
            var liveClear = liveCard.querySelector('.mining-queue-clear-btn');
            var incomingClear = incomingCard.querySelector('.mining-queue-clear-btn');
            if (!liveClear || !incomingClear) {
                return;
            }
            var clearableCount = incomingClear.getAttribute('data-clearable-count');
            if (clearableCount === null) {
                liveClear.removeAttribute('data-clearable-count');
            } else {
                liveClear.setAttribute('data-clearable-count', clearableCount);
            }
            liveClear.disabled = !!incomingClear.disabled;
            var title = incomingClear.getAttribute('title');
            if (title === null || title === '') {
                liveClear.removeAttribute('title');
            } else {
                liveClear.setAttribute('title', title);
            }
        }

        function syncCsrfToken(liveCard, incomingCard) {
            var incomingCsrf = incomingCard.querySelector('input[name="csrfToken"]');
            if (!incomingCsrf || !incomingCsrf.value) {
                return;
            }
            var liveCsrf = liveCard.querySelector('input[name="csrfToken"]');
            if (liveCsrf) {
                liveCsrf.value = incomingCsrf.value;
                liveCsrf.setAttribute('value', incomingCsrf.value);
                return;
            }
            var robotInput = liveCard.querySelector('input[name="robotId"]');
            var created = document.createElement('input');
            created.type = 'hidden';
            created.name = 'csrfToken';
            created.value = incomingCsrf.value;
            created.setAttribute('value', incomingCsrf.value);
            var insertParent = robotInput && (robotInput.parentNode || robotInput.parent);
            if (insertParent) {
                insertParent.insertBefore(created, robotInput);
            } else {
                liveCard.appendChild(created);
            }
        }

        function patchRobotCardActions(liveCard, incomingCard) {
            syncCsrfToken(liveCard, incomingCard);
            var liveSelect = liveCard.querySelector('select.mining-queue-area-select');
            var incomingSelect = incomingCard.querySelector('select.mining-queue-area-select');
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
            var liveCards = mapRobotCards(robotsTarget);
            var incomingCards = mapRobotCards(robotsSource);
            if (!liveCards || !incomingCards || !sameRobotIdSet(liveCards.ids, incomingCards.ids)) {
                robotsTarget.innerHTML = robotsSource.innerHTML;
                return;
            }

            for (var index = 0; index < liveCards.ids.length; index += 1) {
                var robotId = liveCards.ids[index];
                var liveCard = liveCards.map[robotId];
                var incomingCard = incomingCards.map[robotId];
                var liveStatus = liveCard.querySelector('.mining-queue-card-status');
                var incomingStatus = incomingCard.querySelector('.mining-queue-card-status');
                if (!liveStatus || !incomingStatus) {
                    robotsTarget.innerHTML = robotsSource.innerHTML;
                    return;
                }
                liveStatus.innerHTML = incomingStatus.innerHTML;
                patchRobotCardActions(liveCard, incomingCard);
            }
        }

        function formDataToUrlEncoded(formData) {
            var params = new URLSearchParams();
            formData.forEach(function(value, key) {
                params.append(key, value);
            });
            return params.toString();
        }

        function fetchFragment(method, url, body) {
            var scrollEl = document.getElementById('main-content');
            var scrollTop = scrollEl ? scrollEl.scrollTop : 0;
            var options = {
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
            var panels = document.querySelectorAll('tbody.mining-queue-area-panel');
            for (var index = 0; index < panels.length; index += 1) {
                var panel = panels[index];
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
            var form = select.closest('.mining-queue-card');
            if (!form) {
                return;
            }
            var selectedOption = select.options[select.selectedIndex];
            var blockReason = selectedOption ? selectedOption.getAttribute('data-block-reason') : '';
            if (blockReason === null) {
                blockReason = '';
            }
            var disabled = blockReason.length > 0;
            var buttons = form.querySelectorAll('button[name="submitType"][value="add"], button[name="submitType"][value="fill"]');
            for (var buttonIndex = 0; buttonIndex < buttons.length; buttonIndex += 1) {
                var button = buttons[buttonIndex];
                button.disabled = disabled;
                if (disabled) {
                    button.setAttribute('title', blockReason);
                } else {
                    button.removeAttribute('title');
                }
            }
            var hint = form.querySelector('.mining-queue-action-hint');
            if (hint) {
                hint.textContent = blockReason;
                hint.hidden = !disabled;
            }
        }

        function startTimer(cell) {
            var seconds = Number(cell.getAttribute('data-seconds-left'));
            if (!isFinite(seconds)) {
                return;
            }
            var refreshOnComplete = cell.getAttribute('data-refresh-on-complete') === 'true';
            var progressTotal = Number(cell.getAttribute('data-progress-total'));
            function updateProgress(secondsLeft) {
                if (!isFinite(progressTotal) || progressTotal <= 0) {
                    return;
                }
                var run = cell.closest('.mining-queue-run-active');
                if (!run) {
                    return;
                }
                var progressBar = run.querySelector('progress.mining-queue-progress');
                if (!progressBar) {
                    return;
                }
                var elapsed = progressTotal - Math.max(0, secondsLeft);
                var percent = Math.min(100, Math.max(0, (elapsed / progressTotal) * 100));
                progressBar.value = percent.toFixed(1);
            }
            if (seconds <= 0) {
                updateProgress(0);
                if (refreshOnComplete) {
                    ctx.refreshQueue({ forClaim: true });
                }
                return;
            }
            var startTime = Date.now();
            updateProgress(seconds);
            var interval = window.setInterval(function() {
                var secondsLeft = seconds - ((Date.now() - startTime) / 1000);
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
            var target = area.querySelector('a') || area;
            return target.scrollWidth > target.clientWidth + 1;
        }

        function syncQueuedStatusVisibility(row) {
            var area = row.querySelector('.mining-queue-run-area');
            var status = row.querySelector('.mining-queue-status-queued');
            if (!area || !status) {
                return;
            }
            status.classList.remove('mining-queue-status-compact-hidden');
            if (areaNameOverflows(area)) {
                status.classList.add('mining-queue-status-compact-hidden');
            }
        }

        function syncAllQueuedStatusVisibility() {
            var rows = document.querySelectorAll('.mining-queue-run-row');
            for (var rowIndex = 0; rowIndex < rows.length; rowIndex += 1) {
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
            var containers = document.querySelectorAll('.mining-queue-card, .mining-queue-run, .mining-queue-upcoming-list li');
            for (var containerIndex = 0; containerIndex < containers.length; containerIndex += 1) {
                ctx.resizeObserver.observe(containers[containerIndex]);
            }
        }

        function initView(options) {
            var robotAreaSelects = document.querySelectorAll('.mining-queue-card select[name^="miningArea"]');
            for (var selectIndex = 0; selectIndex < robotAreaSelects.length; selectIndex += 1) {
                updateRobotEnqueueState(robotAreaSelects[selectIndex]);
            }

            var forms = document.querySelectorAll('.mining-queue-card');
            for (var formIndex = 0; formIndex < forms.length; formIndex += 1) {
                ctx.updateClearButtonLabel(forms[formIndex]);
            }

            observeQueuedStatusVisibility();

            var cells = document.querySelectorAll('.miningqueuetime[data-seconds-left]');
            for (var cellIndex = 0; cellIndex < cells.length; cellIndex += 1) {
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
