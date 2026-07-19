'use strict';

const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ANIMATION_DIR = path.join(__dirname, '..');
const SCRIPT_FILES = ['payload.js', 'draw.js', 'debug.js', 'player.js'];

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
        getElementById(id) {
            return elements.get(id) || null;
        },
        querySelector(sel) {
            if (sel === '.rally-view-stage') {
                return elements.get('rally-view-stage');
            }
            return null;
        },
        createElement(tag) {
            return {
                tagName: tag,
                className: '',
                textContent: '',
                children: [],
                setAttribute() {},
                appendChild(child) {
                    this.children.push(child);
                },
            };
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
            elapsedMs: 0,
            playing: false,
            finished: false,
            speed: 1,
        },
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
