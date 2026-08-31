'use strict';

const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const { createElement, loadMiningQueuePage, buildRobotCard } = require('./page_harness');

describe('mining queue page partial updates', () => {
    it('buildFragmentUrl includes the queue fragment marker and area params', () => {
        const { context, doc } = loadMiningQueuePage();
        const select = createElement('select', { name: 'miningArea1' });
        select.value = '20';
        select.options = [{ value: '20' }];
        doc.page.appendChild(select);
        context.document.querySelectorAll = function(selector) {
            if (selector.indexOf('miningArea') >= 0) {
                return [select];
            }
            return [];
        };

        const url = context.RoboMinerMiningQueuePage.buildFragmentUrl();
        assert.match(url, /fragment=queue/);
        assert.match(url, /miningArea1=20/);
    });

    it('applyFragment swaps hud, wallet, robots, and clear config without touching inspector', () => {
        const { context, doc } = loadMiningQueuePage();
        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    node.innerHTML = '<span id="new-hud">4/8</span>';
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'new-wallet', textContent: 'wallet' }));
                    dynamic.appendChild(wallet);
                    const error = createElement('p', { class: 'error mining-queue-error' });
                    error.textContent = 'new error';
                    dynamic.appendChild(error);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    const robots = createElement('div', { id: id, class: 'mining-queue-robots' });
                    robots.innerHTML = '<form class="mining-queue-card"><span id="new-robot">new</span></form>';
                    return robots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{"ores":{"1":{"amount":3}}}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        assert.match(doc.hud.innerHTML, /new-hud/);
        assert.match(doc.page.querySelector('.mining-queue-wallet').outerHTML, /new-wallet/);
        assert.match(doc.robots.innerHTML, /new-robot/);
        assert.equal(doc.config.textContent, '{"ores":{"1":{"amount":3}}}');
        assert.match(doc.page.querySelector('.mining-queue-error').textContent, /new error/);
        assert.equal(doc.inspector.textContent, 'inspector stays');
    });

    it('applyFragment preserves live area select nodes when robot cards match', () => {
        const { context, doc } = loadMiningQueuePage();
        doc.robots.innerHTML = '';
        doc.robots.appendChild(buildRobotCard({
            robotId: '1',
            statusText: 'old-status',
            areaValue: '20',
            blockReason: 'Queue full',
            clearableCount: '0',
        }));
        const liveSelect = doc.robots.querySelector('select.mining-queue-area-select');
        assert.ok(liveSelect);

        const incomingRobots = createElement('div', {
            id: 'mining-queue-robots-fragment',
            class: 'mining-queue-robots',
        });
        incomingRobots.appendChild(buildRobotCard({
            robotId: '1',
            statusText: 'new-status',
            areaValue: '20',
            blockReason: '',
            clearableCount: '2',
        }));

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    return createElement('div', { id: id });
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'wallet', textContent: 'w' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return incomingRobots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        const afterSelect = doc.robots.querySelector('select.mining-queue-area-select');
        assert.equal(afterSelect, liveSelect, 'area select DOM node must be preserved');
        assert.match(
            doc.robots.querySelector('.mining-queue-card-status').innerHTML,
            /new-status/
        );
        assert.equal(afterSelect.options[0].getAttribute('data-block-reason'), null);
        assert.equal(
            doc.robots.querySelector('.mining-queue-clear-btn').getAttribute('data-clearable-count'),
            '2'
        );
        assert.equal(afterSelect.value, '20');
    });

    it('applyFragment syncs rotated csrfToken onto preserved robot cards', () => {
        const { context, doc } = loadMiningQueuePage();
        doc.robots.innerHTML = '';
        doc.robots.appendChild(buildRobotCard({
            robotId: '1',
            statusText: 'old-status',
            areaValue: '20',
            blockReason: '',
            clearableCount: '0',
            csrfToken: 'csrf-old',
        }));
        const liveSelect = doc.robots.querySelector('select.mining-queue-area-select');
        const liveCsrf = doc.robots.querySelector('input[name="csrfToken"]');
        assert.equal(liveCsrf.value, 'csrf-old');

        const incomingRobots = createElement('div', {
            id: 'mining-queue-robots-fragment',
            class: 'mining-queue-robots',
        });
        incomingRobots.appendChild(buildRobotCard({
            robotId: '1',
            statusText: 'new-status',
            areaValue: '20',
            blockReason: '',
            clearableCount: '1',
            csrfToken: 'csrf-rotated',
        }));

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    return createElement('div', { id: id });
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'wallet', textContent: 'w' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return incomingRobots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        assert.equal(
            doc.robots.querySelector('select.mining-queue-area-select'),
            liveSelect,
            'area select DOM node must stay preserved'
        );
        assert.equal(
            doc.robots.querySelector('input[name="csrfToken"]').value,
            'csrf-rotated',
            'preserved cards must adopt the rotated CSRF token from the fragment'
        );
        assert.equal(liveCsrf.value, 'csrf-rotated');
    });

    it('applyFragment falls back to full robots replace when robot ids differ', () => {
        const { context, doc } = loadMiningQueuePage();
        doc.robots.innerHTML = '';
        doc.robots.appendChild(buildRobotCard({
            robotId: '1',
            statusText: 'old-status',
            areaValue: '20',
            blockReason: '',
            clearableCount: '0',
        }));
        const liveSelect = doc.robots.querySelector('select.mining-queue-area-select');

        const incomingRobots = createElement('div', {
            id: 'mining-queue-robots-fragment',
            class: 'mining-queue-robots',
        });
        incomingRobots.appendChild(buildRobotCard({
            robotId: '2',
            statusText: 'other-robot',
            areaValue: '21',
            blockReason: '',
            clearableCount: '0',
        }));

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    return createElement('div', { id: id });
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'wallet', textContent: 'w' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return incomingRobots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        const afterSelect = doc.robots.querySelector('select.mining-queue-area-select');
        assert.notEqual(afterSelect, liveSelect, 'mismatched robots should replace the deck');
        assert.match(doc.robots.innerHTML, /other-robot/);
        assert.equal(
            doc.robots.querySelector('.mining-queue-card').getAttribute('data-robot-id'),
            '2'
        );
    });

    it('applyFragment replaces nested app-shell-hud without double wrapping', () => {
        const { context, doc } = loadMiningQueuePage();
        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    const nestedHud = createElement('div', { class: 'app-shell-hud' });
                    nestedHud.appendChild(createElement('span', {
                        id: 'new-hud',
                        textContent: '4/8',
                    }));
                    node.appendChild(nestedHud);
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'new-wallet', textContent: 'wallet' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return createElement('div', { id: id, class: 'mining-queue-robots' });
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        const hud = doc.body.querySelector('.app-shell-hud');
        assert.ok(hud);
        assert.match(hud.outerHTML, /new-hud/);
        assert.equal(
            Array.from(doc.body.querySelectorAll('.app-shell-hud')).length,
            1,
            'live header must contain exactly one .app-shell-hud'
        );
    });

    it('applyFragment skips empty HUD markup so the live header is not wiped', () => {
        const { context, doc } = loadMiningQueuePage();
        const original = doc.hud.innerHTML;
        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    return createElement('div', { id: id, innerHTML: '   ' });
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'new-wallet', textContent: 'wallet' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return createElement('div', { id: id, class: 'mining-queue-robots' });
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);
        assert.equal(doc.hud.innerHTML, original);
    });

    it('claim refresh schedules backoff retries while finishing runs remain', async () => {
        const { context, timers, fetches, doc } = loadMiningQueuePage();
        // Match fragment wallet/HUD so a finishing-run poll does not look "already claimed".
        doc.hud.innerHTML = '';
        doc.hud.appendChild(createElement('span', { id: 'hud-amt', textContent: '1' }));
        const seededWallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
        seededWallet.appendChild(createElement('span', { id: 'wallet-amt', textContent: '1' }));
        doc.page.querySelector('.mining-queue-wallet').outerHTML = seededWallet.outerHTML;

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    const nestedHud = createElement('div', { class: 'app-shell-hud' });
                    nestedHud.appendChild(createElement('span', {
                        id: 'hud-amt',
                        textContent: '1',
                    }));
                    node.appendChild(nestedHud);
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'wallet-amt', textContent: '1' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    const robots = createElement('div', { id: id, class: 'mining-queue-robots' });
                    robots.appendChild(createElement('span', {
                        class: 'mining-queue-status mining-queue-status-updating',
                        textContent: 'Finishing rally',
                    }));
                    robots.appendChild(createElement('span', {
                        class: 'miningqueuetime',
                        'data-seconds-left': '0',
                        'data-refresh-on-complete': 'true',
                    }));
                    return robots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };
        context.fetch = function(url) {
            fetches.push({ url: url });
            return Promise.resolve({
                ok: true,
                text() {
                    return Promise.resolve('<fragment>');
                },
            });
        };

        // Ensure fragment application itself installs finishing-run markers.
        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);
        assert.ok(
            context.RoboMinerMiningQueuePage.hasFinishingRuns(doc.page),
            'applyFragment should install finishing-run markers'
        );

        await context.RoboMinerMiningQueuePage.performRefresh({ forClaim: true });
        assert.equal(fetches.length, 1);
        assert.ok(context.RoboMinerMiningQueuePage.hasFinishingRuns(doc.page));

        const retry = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.CLAIM_REFRESH_BACKOFF_MS[0]);
        assert.ok(retry, 'expected claim-refresh backoff timer');
    });

    it('claim refresh keeps backoff when finishing runs leave but wallet is unchanged', async () => {
        const { context, timers, fetches, doc } = loadMiningQueuePage();
        doc.hud.innerHTML = '';
        doc.hud.appendChild(createElement('span', { id: 'hud-amt', textContent: '1' }));
        const oldWallet = doc.page.querySelector('.mining-queue-wallet');
        oldWallet.innerHTML = '';
        oldWallet.appendChild(createElement('span', { id: 'wallet-amt', textContent: '1' }));

        function buildWallet() {
            const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
            wallet.appendChild(createElement('span', { id: 'wallet-amt', textContent: '1' }));
            return wallet;
        }

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    const nestedHud = createElement('div', { class: 'app-shell-hud' });
                    nestedHud.appendChild(createElement('span', {
                        id: 'hud-amt',
                        textContent: '1',
                    }));
                    node.appendChild(nestedHud);
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    dynamic.appendChild(buildWallet());
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    // Finished runs already left the queue list; no finishing markers.
                    return createElement('div', { id: id, class: 'mining-queue-robots' });
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };
        context.fetch = function(url) {
            fetches.push({ url: url });
            return Promise.resolve({
                ok: true,
                text() {
                    return Promise.resolve('<fragment>');
                },
            });
        };

        await context.RoboMinerMiningQueuePage.performRefresh({ forClaim: true });
        assert.equal(fetches.length, 1);
        assert.equal(
            context.RoboMinerMiningQueuePage.hasFinishingRuns(doc.page),
            false,
            'finishing markers should be gone'
        );
        const retry = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.CLAIM_REFRESH_BACKOFF_MS[0]);
        assert.ok(retry, 'expected claim-refresh backoff while wallet signature is unchanged');
    });

    it('claim refresh stops and shows credit banner when wallet signature changes', async () => {
        const { context, timers, fetches, doc } = loadMiningQueuePage();
        doc.hud.innerHTML = '';
        doc.hud.appendChild(createElement('span', { id: 'hud-amt', textContent: '1' }));

        function walletWithAmount(amount) {
            const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
            const item = createElement('li', { class: 'page-wallet-item' });
            item.appendChild(createElement('span', {
                class: 'page-wallet-ore',
                textContent: 'Iron',
            }));
            item.appendChild(createElement('span', {
                class: 'page-wallet-amount',
                textContent: `${amount}/100`,
            }));
            wallet.appendChild(item);
            return wallet;
        }

        const oldWallet = doc.page.querySelector('.mining-queue-wallet');
        oldWallet.parent.appendChild(walletWithAmount(1));
        oldWallet.remove();

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    const nestedHud = createElement('div', { class: 'app-shell-hud' });
                    nestedHud.appendChild(createElement('span', {
                        id: 'hud-amt',
                        textContent: '5',
                    }));
                    node.appendChild(nestedHud);
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    dynamic.appendChild(walletWithAmount(5));
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return createElement('div', { id: id, class: 'mining-queue-robots' });
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };
        context.fetch = function(url) {
            fetches.push({ url: url });
            return Promise.resolve({
                ok: true,
                text() {
                    return Promise.resolve('<fragment>');
                },
            });
        };

        await context.RoboMinerMiningQueuePage.performRefresh({ forClaim: true });
        assert.equal(fetches.length, 1);
        const retry = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.CLAIM_REFRESH_BACKOFF_MS[0]);
        assert.equal(retry, undefined, 'wallet change should stop claim backoff');
        const banner = doc.page.querySelector('.mining-queue-credit-banner');
        assert.ok(banner, 'expected credit feedback banner');
        assert.match(banner.textContent, /Added to wallet: \+4 Iron/);
        const dismiss = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.CREDIT_FEEDBACK_MS);
        assert.ok(dismiss, 'expected credit banner dismiss timer');
    });

    it('formDataToUrlEncoded serializes fields for urlencoded POST bodies', () => {
        const { context } = loadMiningQueuePage();
        const formData = new context.FormData();
        formData.append('submitType', 'add');
        formData.append('robotId', '1');
        formData.append('miningArea1', '20');
        const encoded = context.RoboMinerMiningQueuePage.formDataToUrlEncoded(formData);
        assert.match(encoded, /(^|&)submitType=add(&|$)/);
        assert.match(encoded, /robotId=1/);
        assert.match(encoded, /miningArea1=20/);
    });

    it('refreshQueue debounces multiple timer completions into one fetch', () => {
        const { context, timers, fetches } = loadMiningQueuePage({
            fetch(url) {
                fetches.push({ url: url });
                return Promise.resolve({
                    ok: true,
                    text() {
                        return Promise.resolve([
                            '<div id="mining-queue-fragment">',
                            '<div id="mining-queue-hud-fragment"></div>',
                            '<div id="mining-queue-dynamic-fragment">',
                            '<section class="mining-queue-wallet"></section>',
                            '<div class="mining-queue-robots" id="mining-queue-robots-fragment"></div>',
                            '<script id="mining-queue-clear-config" type="application/json">{}</script>',
                            '</div>',
                            '</div>',
                        ].join(''));
                    },
                });
            },
        });

        context.RoboMinerMiningQueuePage.refreshQueue();
        context.RoboMinerMiningQueuePage.refreshQueue();
        assert.equal(fetches.length, 0);

        const debounced = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.REFRESH_DEBOUNCE_MS);
        assert.ok(debounced);
        debounced.fn();
        assert.equal(fetches.length, 1);
        assert.match(fetches[0].url, /fragment=queue/);
    });
});
