'use strict';

const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ANIMATION_DIR = path.join(__dirname, '..');
const SCRIPT_FILES = [
    'payload.js',
    'draw_ground.js',
    'draw_robots.js',
    'debug_status.js',
    'debug_source.js',
    'timeline.js',
    'pose.js',
    'transport.js',
    'controls.js',
    'side_panels.js',
    'player.js',
];

function createRecordingContext2d() {
    const ops = [];
    return {
        ops,
        fillStyle: '',
        strokeStyle: '',
        lineWidth: 1,
        beginPath() {
            ops.push({ op: 'beginPath' });
        },
        rect(x, y, w, h) {
            ops.push({ op: 'rect', x, y, w, h });
        },
        fill() {
            ops.push({ op: 'fill', fillStyle: this.fillStyle });
        },
        stroke() {
            ops.push({ op: 'stroke', strokeStyle: this.strokeStyle });
        },
        fillRect(x, y, w, h) {
            ops.push({ op: 'fillRect', x, y, w, h, fillStyle: this.fillStyle });
        },
        strokeRect(x, y, w, h) {
            ops.push({ op: 'strokeRect', x, y, w, h, strokeStyle: this.strokeStyle });
        },
        clearRect(x, y, w, h) {
            ops.push({ op: 'clearRect', x, y, w, h });
        },
        arc() {
            ops.push({ op: 'arc' });
        },
        moveTo() {
            ops.push({ op: 'moveTo' });
        },
        lineTo() {
            ops.push({ op: 'lineTo' });
        },
        save() {
            ops.push({ op: 'save' });
        },
        restore() {
            ops.push({ op: 'restore' });
        },
        setLineDash() {
            ops.push({ op: 'setLineDash' });
        },
    };
}

function createCanvas(width, height) {
    return {
        width,
        height,
        getContext() {
            return createRecordingContext2d();
        },
    };
}

function createMemoryLocalStorage() {
    const store = new Map();
    return {
        getItem(key) {
            return store.has(key) ? store.get(key) : null;
        },
        setItem(key, value) {
            store.set(String(key), String(value));
        },
        removeItem(key) {
            store.delete(String(key));
        },
        clear() {
            store.clear();
        },
    };
}

/**
 * Load the assembled rally animation scripts into an isolated VM context.
 * Mirrors include order in animation_script.rs.
 */
function loadRallyViewer(options = {}) {
    const oreCanvas = [
        createCanvas(50, 200),
        createCanvas(50, 200),
        createCanvas(50, 200),
        createCanvas(50, 200),
    ];
    const depotCanvas = [
        createCanvas(50, 200),
        createCanvas(50, 200),
        createCanvas(50, 200),
        createCanvas(50, 200),
    ];
    const rallyContext = createRecordingContext2d();

    const elements = new Map();
    function register(id, el) {
        elements.set(id, el);
        return el;
    }

    for (let i = 0; i < 4; i++) {
        register(`oreCanvas${i}`, oreCanvas[i]);
        register(`depotCanvas${i}`, depotCanvas[i]);
        register(`depotChart${i}`, { hidden: true, removeAttribute() { this.hidden = false; }, setAttribute() { this.hidden = true; } });
        register(`robotTurns${i}`, { textContent: '' });
        register(`robotBattery${i}`, {
            classList: { add() {}, remove() {} },
            setAttribute() {},
        });
        register(`robotBatteryFill${i}`, { style: {} });
        register(`robotAction${i}`, { textContent: '' });
        register(`rallyPlayer${i}`, {
            classList: {
                _set: new Set(),
                add(name) { this._set.add(name); },
                remove(name) { this._set.delete(name); },
            },
        });
    }

    register('rally-view-stage', {
        firstChild: null,
        children: [],
        removeChild() {},
        appendChild(child) {
            this.children.push(child);
            this.firstChild = child;
        },
        querySelector() { return null; },
    });

    const documentStub = {
        body: {
            _html: '',
            set innerHTML(html) {
                this._html = html;
                // Very small HTML subset for source-highlight tests.
                elements.delete('rallySourceCode');
                elements.delete('rallySourceLine1');
                elements.delete('rallySourceStepResult');
                elements.delete('rallySourceVariables');
                const lineMatch = html.match(
                    /id="rallySourceLine1"[\s\S]*?<code class="rally-view-source-text">([^<]*)<\/code>/
                );
                const codeText = lineMatch ? lineMatch[1] : '';
                if (html.includes('id="rallySourceStepResult"')) {
                    register('rallySourceStepResult', {
                        id: 'rallySourceStepResult',
                        textContent: '',
                    });
                }
                if (html.includes('id="rallySourceVariables"')) {
                    const variablesEl = {
                        id: 'rallySourceVariables',
                        childNodes: [],
                        get firstChild() {
                            return this.childNodes[0] || null;
                        },
                        appendChild(child) {
                            this.childNodes.push(child);
                        },
                        removeChild(child) {
                            const idx = this.childNodes.indexOf(child);
                            if (idx >= 0) {
                                this.childNodes.splice(idx, 1);
                            } else if (this.childNodes.length) {
                                this.childNodes.shift();
                            }
                            return child;
                        },
                    };
                    register('rallySourceVariables', variablesEl);
                }
                const codeEl = {
                    className: 'rally-view-source-text',
                    textContent: codeText,
                    firstChild: codeText ? { nodeType: 3 } : null,
                    childNodes: [],
                    appendChild(child) {
                        this.childNodes.push(child);
                        if (child.nodeType === 3 || typeof child.textContent === 'string') {
                            // Rebuild textContent from children when mixed.
                        }
                        this.firstChild = this.childNodes[0] || null;
                        this.textContent = this.childNodes
                            .map((c) => c.textContent || '')
                            .join('');
                    },
                    removeChild() {
                        this.childNodes.shift();
                        this.firstChild = this.childNodes[0] || null;
                        this.textContent = this.childNodes
                            .map((c) => c.textContent || '')
                            .join('');
                    },
                    querySelector() {
                        return null;
                    },
                };
                // Keep textContent in sync when clearing via while(firstChild) removeChild
                const originalRemove = codeEl.removeChild.bind(codeEl);
                codeEl.removeChild = function removeChild(child) {
                    const idx = this.childNodes.indexOf(child);
                    if (idx >= 0) {
                        this.childNodes.splice(idx, 1);
                    } else if (this.childNodes.length) {
                        this.childNodes.shift();
                    }
                    this.firstChild = this.childNodes[0] || null;
                    if (this.childNodes.length === 0 && child && child.nodeType === 3) {
                        // Clearing initial text node: empty the element.
                        this.textContent = '';
                    } else {
                        this.textContent = this.childNodes
                            .map((c) => c.textContent || '')
                            .join('');
                    }
                    return child;
                };
                void originalRemove;

                const lineEl = {
                    id: 'rallySourceLine1',
                    classList: {
                        _set: new Set(),
                        add(name) {
                            this._set.add(name);
                        },
                        remove(name) {
                            this._set.delete(name);
                        },
                        contains(name) {
                            return this._set.has(name);
                        },
                    },
                    querySelector(sel) {
                        if (sel === '.rally-view-source-text') {
                            return codeEl;
                        }
                        if (sel === '.rally-view-source-token-active') {
                            return (
                                codeEl.childNodes.find(
                                    (c) => c.className === 'rally-view-source-token-active'
                                ) || null
                            );
                        }
                        return null;
                    },
                    getBoundingClientRect() {
                        return { top: 0, bottom: 20 };
                    },
                };
                const sourceCode = {
                    id: 'rallySourceCode',
                    getBoundingClientRect() {
                        return { top: 0, bottom: 100 };
                    },
                    scrollTop: 0,
                };
                register('rallySourceCode', sourceCode);
                register('rallySourceLine1', lineEl);
            },
            get innerHTML() {
                return this._html;
            },
        },
        getElementById(id) {
            return elements.get(id) || null;
        },
        querySelector(sel) {
            if (sel === '.rally-view-stage') {
                return elements.get('rally-view-stage');
            }
            return null;
        },
        querySelectorAll() {
            return [];
        },
        addEventListener() {},
        createElement(tag) {
            const el = {
                tagName: tag,
                className: '',
                textContent: '',
                children: [],
                childNodes: [],
                nodeType: 1,
                setAttribute() {},
                appendChild(child) {
                    this.children.push(child);
                    this.childNodes.push(child);
                },
            };
            return el;
        },
        createTextNode(text) {
            return { nodeType: 3, textContent: String(text) };
        },
    };

    const context = {
        console,
        Math,
        Number,
        isNaN,
        parseInt,
        parseFloat,
        Array,
        Object,
        JSON,
        String,
        document: documentStub,
        window: {},
        myRallyContext: rallyContext,
        myOreCanvas: oreCanvas,
        myOreContext: oreCanvas.map((c) => c.getContext('2d')),
        myDepotCanvas: depotCanvas,
        myDepotContext: depotCanvas.map((c) => c.getContext('2d')),
        myRallyViewerSlot: null,
        myRobots: undefined,
        myGround: undefined,
        myOreTypes: undefined,
        myRallyPlayer: {
            scale: 10,
            baseStepTime: 50,
            elapsedMs: 0,
            playing: false,
            finished: false,
            speed: 1,
        },
        localStorage: createMemoryLocalStorage(),
        ...options.globals,
    };

    context.window = context;
    vm.createContext(context);

    for (const file of SCRIPT_FILES) {
        const source = fs.readFileSync(path.join(ANIMATION_DIR, file), 'utf8');
        vm.runInContext(source, context, { filename: file });
    }

    return {
        context,
        document: documentStub,
        elements,
        rallyContext,
        oreCanvas,
        depotCanvas,
    };
}

module.exports = {
    loadRallyViewer,
    createRecordingContext2d,
};
