'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const ACTIONS_JS = fs.readFileSync(path.join(__dirname, '..', 'actions.js'), 'utf8');

class FakeFormData {
    constructor(source) {
        this.entries = [];
        if (source && Array.isArray(source.inputs)) {
            source.inputs.forEach((input) => {
                if (input && input.name) {
                    this.entries.push([input.name, String(input.value)]);
                }
            });
            return;
        }
        const fields = source && source.fields ? source.fields : {};
        Object.keys(fields).forEach((name) => {
            const value = fields[name];
            const values = Array.isArray(value) ? value : [value];
            values.forEach((entry) => {
                this.entries.push([name, String(entry)]);
            });
        });
    }

    set(name, value) {
        this.entries = this.entries.filter((entry) => entry[0] !== name);
        this.entries.push([name, String(value)]);
    }

    delete(name) {
        this.entries = this.entries.filter((entry) => entry[0] !== name);
    }

    append(name, value) {
        this.entries.push([name, String(value)]);
    }

    field(name) {
        const matches = this.entries.filter((entry) => entry[0] === name).map((entry) => entry[1]);
        if (matches.length === 0) {
            return undefined;
        }
        return matches.length === 1 ? matches[0] : matches;
    }
}

function loadActions() {
    const timeouts = [];
    const sandbox = {
        window: null,
        console,
        document: {
            getElementById() {
                return null;
            },
            createElement() {
                const attrs = {};
                return {
                    type: '',
                    name: '',
                    value: '',
                    setAttribute(name, value) {
                        attrs[name] = value;
                    },
                    getAttribute(name) {
                        return attrs[name];
                    },
                    attrs,
                };
            },
            querySelectorAll() {
                return [];
            },
        },
        FormData: FakeFormData,
        setTimeout(fn) {
            timeouts.push(fn);
            return timeouts.length;
        },
    };
    sandbox.window = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(ACTIONS_JS, sandbox);
    return { sandbox, timeouts };
}

function installActions(viewOverrides) {
    const { sandbox, timeouts } = loadActions();
    const posted = [];
    const ctx = {
        buildFragmentUrl() {
            return 'miningQueue?fragment=queue';
        },
        updateClearButtonLabel: null,
    };
    const view = Object.assign({
        fetchFragment(method, url, formData) {
            posted.push({ method, url, fields: formData });
            return Promise.resolve();
        },
    }, viewOverrides || {});
    const actions = sandbox.RoboMinerMiningQueueInstall.actions(ctx, view);
    return { sandbox, actions, posted, timeouts };
}

function hiddenInput(name, value, markerAttr) {
    const attrs = {};
    if (markerAttr) {
        attrs[markerAttr] = 'true';
    }
    return {
        name,
        value: String(value),
        attrs,
        getAttribute(attr) {
            return attrs[attr];
        },
        setAttribute(attr, attrValue) {
            attrs[attr] = attrValue;
        },
        remove() {
            this.removed = true;
        },
    };
}

function makeForm(existingInputs) {
    const inputs = existingInputs.slice();
    inputs.forEach((input) => {
        input.remove = function() {
            const index = inputs.indexOf(input);
            if (index !== -1) {
                inputs.splice(index, 1);
            }
        };
    });
    return {
        inputs,
        fields: {},
        querySelectorAll(selector) {
            return inputs.filter((input) => {
                if (selector.indexOf('data-mining-queue-clear') !== -1 && input.attrs['data-mining-queue-clear']) {
                    return true;
                }
                if (selector.indexOf('data-mining-queue-remove') !== -1 && input.attrs['data-mining-queue-remove']) {
                    return true;
                }
                if (selector.indexOf('data-mining-queue-reorder') !== -1 && input.attrs['data-mining-queue-reorder']) {
                    return true;
                }
                return false;
            });
        },
        appendChild(input) {
            inputs.push(input);
            const value = input.value;
            if (this.fields[input.name]) {
                const current = this.fields[input.name];
                this.fields[input.name] = Array.isArray(current) ? current.concat(value) : [current, value];
            } else {
                this.fields[input.name] = value;
            }
        },
    };
}

function makeItem(id, extraClass) {
    const classTokens = new Set(['mining-queue-upcoming-item']);
    if (extraClass) {
        classTokens.add(extraClass);
    }
    const moveUp = { disabled: false };
    const moveDown = { disabled: false };
    const item = {
        attrs: { 'data-queue-item-id': String(id), draggable: 'true', class: 'mining-queue-upcoming-item' },
        parentNode: null,
        nextSibling: null,
        moveUp,
        moveDown,
        classList: {
            tokens: classTokens,
            add(name) {
                classTokens.add(name);
            },
            remove(name) {
                classTokens.delete(name);
            },
        },
        getAttribute(name) {
            return item.attrs[name];
        },
        querySelector(selector) {
            if (selector.indexOf('mining-queue-move-up') !== -1) {
                return moveUp;
            }
            if (selector.indexOf('mining-queue-move-down') !== -1) {
                return moveDown;
            }
            return null;
        },
        closest(selector) {
            if (selector.indexOf('mining-queue-upcoming-item') !== -1) {
                return item;
            }
            if (selector.indexOf('mining-queue-upcoming-list-reorderable') !== -1) {
                return item.parentNode;
            }
            if (selector.indexOf('mining-queue-card') !== -1) {
                return item.parentNode && item.parentNode.form;
            }
            if (selector === 'button, input, a, label') {
                return null;
            }
            if (selector.indexOf('mining-queue-drag-handle') !== -1) {
                return null;
            }
            return null;
        },
    };
    return item;
}

function makeList(ids) {
    const items = ids.map((id) => makeItem(id));
    const form = makeForm([]);
    const list = {
        children: items,
        form,
        querySelector(selector) {
            if (selector.indexOf('dragging') !== -1) {
                return items.find((item) => item.classList.tokens.has('mining-queue-upcoming-item-dragging')) || null;
            }
            return null;
        },
        querySelectorAll() {
            return items;
        },
        closest(selector) {
            if (selector.indexOf('mining-queue-upcoming-list-reorderable') !== -1) {
                return list;
            }
            if (selector.indexOf('mining-queue-card') !== -1) {
                return form;
            }
            return null;
        },
        insertBefore(node, ref) {
            const from = items.indexOf(node);
            if (from !== -1) {
                items.splice(from, 1);
            }
            const to = items.indexOf(ref);
            items.splice(to < 0 ? items.length : to, 0, node);
            node.parentNode = list;
        },
        appendChild(node) {
            const from = items.indexOf(node);
            if (from !== -1) {
                items.splice(from, 1);
            }
            items.push(node);
            node.parentNode = list;
        },
    };
    items.forEach((item, index) => {
        item.parentNode = list;
        item.nextSibling = items[index + 1] || null;
        item.moveUp.disabled = index === 0;
        item.moveDown.disabled = index === items.length - 1;
    });
    return { list, items, form };
}

describe('mining queue actions module', () => {
    it('registers clear/remove helpers and wires updateClearButtonLabel on ctx', () => {
        const { sandbox } = loadActions();
        const ctx = {
            buildFragmentUrl() {
                return 'miningQueue?fragment=queue';
            },
            updateClearButtonLabel: null,
        };
        const actions = sandbox.RoboMinerMiningQueueInstall.actions(ctx, {
            fetchFragment() {
                return Promise.resolve();
            },
        });
        assert.equal(typeof actions.clearQueuedRuns, 'function');
        assert.equal(typeof actions.removeQueuedRun, 'function');
        assert.equal(typeof actions.submitFormPartial, 'function');
        assert.equal(typeof actions.cardSubmitFormData, 'function');
        assert.equal(typeof actions.submitQueuedReorder, 'function');
        assert.equal(typeof actions.onDragStart, 'function');
        assert.equal(typeof ctx.updateClearButtonLabel, 'function');
    });

    it('submitQueuedReorder posts the queued ids in DOM order', () => {
        const { actions, posted } = installActions();
        const form = makeForm([]);
        actions.submitQueuedReorder(form, ['102', '101']);
        assert.equal(posted.length, 1);
        assert.equal(posted[0].method, 'POST');
        assert.equal(posted[0].fields.field('submitType'), 'reorder');
        assert.deepEqual(posted[0].fields.field('orderedQueueItemId'), ['102', '101']);
    });

    it('submitQueuedReorder ignores leftover clear fields from a previous action', () => {
        const { actions, posted } = installActions();
        const form = makeForm([
            hiddenInput('submitType', 'clear', 'data-mining-queue-clear'),
            hiddenInput('clearMode', 'all', 'data-mining-queue-clear'),
        ]);
        actions.submitQueuedReorder(form, ['102', '101']);
        assert.equal(posted.length, 1);
        assert.equal(posted[0].fields.field('submitType'), 'reorder');
        assert.equal(posted[0].fields.field('clearMode'), undefined);
        assert.deepEqual(posted[0].fields.field('orderedQueueItemId'), ['102', '101']);
    });

    it('cardSubmitFormData lets a move button win over leftover reorder fields', () => {
        const { actions } = installActions();
        const form = makeForm([
            hiddenInput('submitType', 'reorder', 'data-mining-queue-reorder'),
            hiddenInput('orderedQueueItemId', '102', 'data-mining-queue-reorder'),
            hiddenInput('orderedQueueItemId', '101', 'data-mining-queue-reorder'),
        ]);
        const formData = actions.cardSubmitFormData(form, { name: 'moveUp', value: '102' });
        assert.equal(formData.field('submitType'), 'moveUp');
        assert.equal(formData.field('moveQueueItemId'), '102');
        assert.equal(formData.field('orderedQueueItemId'), undefined);
        assert.equal(formData.field('moveUp'), undefined);
    });

    it('cardSubmitFormData keeps add/fill submitType from the clicked button', () => {
        const { actions } = installActions();
        const form = makeForm([
            hiddenInput('submitType', 'reorder', 'data-mining-queue-reorder'),
            hiddenInput('orderedQueueItemId', '101', 'data-mining-queue-reorder'),
        ]);
        const formData = actions.cardSubmitFormData(form, { name: 'submitType', value: 'add' });
        assert.equal(formData.field('submitType'), 'add');
        assert.equal(formData.field('orderedQueueItemId'), undefined);
    });

    it('disables edge move buttons after dropping a queued item', () => {
        const { actions } = installActions();
        const { list, items } = makeList(['101', '102']);
        assert.equal(items[0].moveUp.disabled, true);
        assert.equal(items[0].moveDown.disabled, false);
        assert.equal(items[1].moveUp.disabled, false);
        assert.equal(items[1].moveDown.disabled, true);
        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        list.insertBefore(items[1], items[0]);
        actions.onDrop({
            target: list,
            preventDefault() {},
        });
        assert.equal(items[0].moveUp.disabled, true);
        assert.equal(items[0].moveDown.disabled, false);
        assert.equal(items[1].moveUp.disabled, false);
        assert.equal(items[1].moveDown.disabled, true);
    });

    it('allows dragstart from the drag handle', () => {
        const { actions } = installActions();
        const { items } = makeList(['101', '102']);
        const item = items[0];
        const handle = {
            closest(selector) {
                if (selector.indexOf('mining-queue-drag-handle') !== -1) {
                    return handle;
                }
                if (selector === 'button, input, a, label') {
                    return handle;
                }
                return item.closest(selector);
            },
        };
        let prevented = false;
        const dataTransfer = { effectAllowed: '', setData() {} };
        actions.onDragStart({
            target: handle,
            dataTransfer,
            preventDefault() {
                prevented = true;
            },
        });
        assert.equal(prevented, false);
        assert.equal(dataTransfer.effectAllowed, 'move');
        assert.equal(item.classList.tokens.has('mining-queue-upcoming-item-dragging'), true);
    });

    it('does not post a reorder when the drop lands outside the queued list', () => {
        const { actions, posted } = installActions();
        const { items } = makeList(['101', '102']);
        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        actions.onDrop({
            target: {
                closest() {
                    return null;
                },
            },
            preventDefault() {},
        });
        assert.equal(posted.length, 0);
    });

    it('posts the live list order when dropping onto the queued list', () => {
        const { actions, posted } = installActions();
        const { list, items } = makeList(['101', '102']);
        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        list.insertBefore(items[1], items[0]);
        actions.onDrop({
            target: list,
            preventDefault() {},
        });
        assert.equal(posted.length, 1);
        assert.equal(posted[0].fields.field('submitType'), 'reorder');
        assert.deepEqual(posted[0].fields.field('orderedQueueItemId'), ['102', '101']);
        assert.equal(actions.consumeDragClickSuppression(), false);
    });

    it('does not swallow a later click after a successful queued drop', () => {
        const { actions } = installActions();
        const { list, items } = makeList(['101', '102']);
        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        list.insertBefore(items[1], items[0]);
        actions.onDrop({
            target: list,
            preventDefault() {},
        });
        assert.equal(actions.consumeDragClickSuppression(), false);
    });

    it('suppresses only the same-turn click when a drop lands on a control', () => {
        const { actions, timeouts } = installActions();
        const { items } = makeList(['101', '102']);
        const clearButton = {
            closest(selector) {
                if (selector === 'button, input, a, label') {
                    return clearButton;
                }
                return null;
            },
        };
        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        actions.onDrop({
            target: clearButton,
            preventDefault() {},
        });
        assert.equal(actions.consumeDragClickSuppression(), true);
        assert.equal(actions.consumeDragClickSuppression(), false);

        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        actions.onDrop({
            target: clearButton,
            preventDefault() {},
        });
        timeouts.forEach((fn) => fn());
        assert.equal(actions.consumeDragClickSuppression(), false);
    });

    it('does not post a reorder when the dropped list is missing queue ids', () => {
        const { actions, posted } = installActions();
        const { list, items } = makeList(['101', '102']);
        actions.onDragStart({
            target: items[1],
            dataTransfer: { effectAllowed: '', setData() {} },
            preventDefault() {},
        });
        list.querySelectorAll = function() {
            return [items[0]];
        };
        actions.onDrop({
            target: list,
            preventDefault() {},
        });
        assert.equal(posted.length, 0);
    });
});
