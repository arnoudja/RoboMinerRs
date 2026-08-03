'use strict';

const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const { loadRallyViewer } = require('./load_viewer');

function validPayload(overrides = {}) {
    return {
        v: 2,
        robots: {
            robot: [
                {
                    robotnr: 0,
                    x: 0,
                    y: 0,
                    o: 45,
                    A: 0,
                    B: 0,
                    C: 0,
                    size: 1.5,
                    maxore: 50,
                    maxturns: 100,
                    depotMaxA: 5,
                    depotMaxB: 0,
                    depotMaxC: 0,
                    homeX: 0,
                    homeY: 0,
                    homeSize: 2,
                    DA: 0,
                    DB: 0,
                    DC: 0,
                    locations: [
                        {
                            x: 0,
                            y: 0,
                            o: 45,
                            A: 0,
                            B: 0,
                            C: 0,
                            DA: 0,
                            DB: 0,
                            DC: 0,
                            cpu: [{ l: 1, c: 1, e: 7, r: { k: 'i', v: 4 }, vs: { x: { k: 'i', v: 4 } } }],
                        },
                        {
                            x: 1,
                            y: 0,
                            o: 45,
                            A: 4,
                            B: 0,
                            C: 0,
                            a: 6,
                            cpu: [
                                { l: 1, c: 1, e: 7, r: { k: 'f', v: 1.42 } },
                                { l: 1, c: 9, e: 16, r: { k: 'b', v: 1 } },
                            ],
                        },
                        { A: 0, DA: 4, a: 7, l: 1 },
                    ],
                },
            ],
        },
        ground: {
            sizeX: 8,
            sizeY: 8,
            positions: [
                {
                    x: 0,
                    y: 0,
                    c: [{ A: 8 }, { t: 2, A: 4 }],
                },
                {
                    // Dump onto previously empty cell — no t:0 baseline.
                    x: 3,
                    y: 3,
                    c: [{ t: 5, A: 6 }],
                },
            ],
        },
        oreTypes: { A: { id: 1, max: 8 } },
        ...overrides,
    };
}

describe('rally animation viewer', () => {
    it('rejects unsupported or incomplete payloads', () => {
        const { context } = loadRallyViewer();
        assert.match(
            context.validateRallyResultPayload({ v: 3, robots: { robot: [] }, ground: {} }),
            /unsupported version/
        );
        assert.equal(
            context.validateRallyResultPayload({
                v: 1,
                robots: { robot: [{ locations: [] }] },
                ground: { sizeX: 1, sizeY: 1, positions: [] },
            }),
            null
        );
        assert.match(
            context.validateRallyResultPayload({
                v: 1,
                robots: { robot: [{ locations: null }] },
                ground: { sizeX: 1, sizeY: 1, positions: [] },
            }),
            /incomplete robot/
        );
        assert.match(
            context.validateRallyResultPayload({
                v: 1,
                robots: { robot: [{ locations: [] }] },
                ground: { sizeX: 1, sizeY: 1 },
            }),
            /missing map/
        );
        assert.equal(context.validateRallyResultPayload(validPayload()), null);
    });

    it('applyRallyResultPayload installs globals only on success', () => {
        const { context } = loadRallyViewer();
        const bad = context.applyRallyResultPayload({ v: 99 });
        assert.match(bad, /unsupported version/);
        assert.equal(context.myRobots, undefined);

        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        assert.equal(context.myRobots.robot.length, 1);
        assert.equal(context.myGround.sizeX, 8);
        assert.equal(context.myOreTypes.A.max, 8);
    });

    it('findGroundChangeIndex skips future-only dump cells (early ore regression)', () => {
        const { context } = loadRallyViewer();
        const emptyUntilDump = { c: [{ t: 5, A: 6 }] };
        assert.equal(context.findGroundChangeIndex(emptyUntilDump, 0), -1);
        assert.equal(context.findGroundChangeIndex(emptyUntilDump, 4), -1);
        assert.equal(context.findGroundChangeIndex(emptyUntilDump, 5), 0);
        assert.equal(context.findGroundChangeIndex(emptyUntilDump, 9), 0);

        const minedThenDumped = { c: [{ A: 8 }, { t: 2, A: 4 }] };
        assert.equal(context.findGroundChangeIndex(minedThenDumped, 0), 0);
        assert.equal(context.findGroundChangeIndex(minedThenDumped, 1), 0);
        assert.equal(context.findGroundChangeIndex(minedThenDumped, 2), 1);
    });

    it('drawGroundAt leaves future dump cells black until their change time', () => {
        const { context, rallyContext } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);

        rallyContext.ops.length = 0;
        context.drawGroundAt(0, 10, 0, 0, 8, 8);
        const fillsAt0 = rallyContext.ops.filter((op) => op.op === 'fill' && op.fillStyle !== 'black');
        // Only the t:0 ore cell at (0,0) should paint non-black ore.
        assert.equal(fillsAt0.length, 1);

        rallyContext.ops.length = 0;
        context.drawGroundAt(5, 10, 0, 0, 8, 8);
        const fillsAt5 = rallyContext.ops.filter((op) => op.op === 'fill' && op.fillStyle !== 'black');
        assert.equal(fillsAt5.length, 2);
    });

    it('detects depot capacity and cargo fullness', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.robotHasDepot({ depotMaxA: 0, depotMaxB: 0, depotMaxC: 0 }), false);
        assert.equal(context.robotHasDepot({ depotMaxA: '5', depotMaxB: 0, depotMaxC: 0 }), true);
        assert.equal(context.robotCargoFull({ A: 20, B: 20, C: 10, maxore: 50 }), true);
        assert.equal(context.robotCargoFull({ A: 10, B: 0, C: 0, maxore: 50 }), false);
        assert.equal(context.robotTurnsRemaining({ maxturns: 10 }, 3), 7);
        assert.equal(context.robotTurnsRemaining({ maxturns: 10 }, 20), 0);
    });

    it('depotHomeSquare uses homeSize and spawn corner fields', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.depotHomeSquare({ depotMaxA: 0 }), null);

        const home = context.depotHomeSquare({
            depotMaxA: 5,
            homeX: 0,
            homeY: 0,
            homeSize: 2,
            size: 1.5,
            robotnr: 0,
        });
        assert.equal(home.x, 0);
        assert.equal(home.y, 0);
        assert.equal(home.side, 2);

        context.myGround = { sizeX: 8, sizeY: 8 };
        const corner = context.depotHomeSquare({
            depotMaxA: 5,
            size: 1.5,
            robotnr: 3,
        });
        assert.equal(corner.x, 6);
        assert.equal(corner.y, 6);
        assert.equal(corner.side, 2);
    });

    it('updateRobotTo fills forward sparse location deltas including depot', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        const robot = context.myRobots.robot[0];
        context.updateRobotTo(0, 2);
        assert.equal(robot.locations[2].x, 1);
        assert.equal(robot.locations[2].y, 0);
        assert.equal(robot.locations[2].A, 0);
        assert.equal(robot.locations[2].DA, 4);
        assert.equal(robot.updatedTo, 2);
    });

    it('drawSideBySideDepotBar fills each unlocked ore against its own capacity', () => {
        const { context } = loadRallyViewer();
        const canvas = { width: 50, height: 200 };
        const ctx = {
            ops: [],
            fillStyle: '',
            beginPath() {},
            rect() {},
            fill() {
                this.ops.push({ op: 'fill', fillStyle: this.fillStyle });
            },
            fillRect(x, y, w, h) {
                this.ops.push({ op: 'fillRect', x, y, w, h, fillStyle: this.fillStyle });
            },
        };

        // Monkey-patch via direct call with recording rects through fill after beginPath/rect pattern
        // drawSideBySideDepotBar uses beginPath/rect/fill, not fillRect.
        const recorded = [];
        ctx.beginPath = function beginPath() {};
        ctx.rect = function rect(x, y, w, h) {
            recorded.push({ x, y, w, h, fillStyle: null, pending: true });
        };
        ctx.fill = function fill() {
            for (let i = recorded.length - 1; i >= 0; i--) {
                if (recorded[i].pending) {
                    recorded[i].fillStyle = this.fillStyle;
                    recorded[i].pending = false;
                    break;
                }
            }
        };

        context.drawSideBySideDepotBar(
            ctx,
            canvas,
            0,
            [5, 2, 0],
            [5, 10, 0]
        );

        const oreFills = recorded.filter(
            (r) => r.fillStyle === 'red' || r.fillStyle === 'green' || r.fillStyle === 'blue'
        );
        assert.equal(oreFills.length, 2);

        const red = oreFills.find((r) => r.fillStyle === 'red');
        const green = oreFills.find((r) => r.fillStyle === 'green');
        assert.ok(red);
        assert.ok(green);
        // A is full (5/5) → full inner height 194; B is 2/10 → ~38px.
        assert.equal(red.h, 194);
        assert.equal(green.h, 38);
        assert.ok(red.x < green.x, 'ore columns should sit side by side');
    });

    it('rally status helpers label blocked and idle states', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.rallyStatusLabel('wall'), 'Blocked by wall');
        assert.equal(context.rallyActionName(7), 'Dump');
        assert.equal(context.robotLooksBlocked({ s: 'robot' }), true);
        assert.equal(context.robotLooksIdle({ a: 1 }), true);
        assert.equal(context.robotLooksIdle({ a: 6 }), false);
    });

    it('exposes core viewer entrypoints from assembled modules', () => {
        const { context } = loadRallyViewer();
        for (const name of [
            'applyRallyResultPayload',
            'runanimation',
            'rallySeekToRatio',
            'rallySeekByCpuSteps',
            'rallyWithPausedSeek',
            'rallyRebuildCpuTimeline',
            'rallyViewerRobot',
            'updateRobotPosition',
            'drawRobot',
            'updateRobotDebugPanel',
            'updateRallyViewerSourceDebug',
            'rallyApplySidePanelOrder',
            'rallyBindSidePanelOrder',
        ]) {
            assert.equal(typeof context[name], 'function', name);
        }
        assert.equal(typeof context.rallySeekByCycles, 'undefined');
        assert.ok(context.RALLY_VIEWER_HIGHLIGHT_PADDING);
    });

    it('binds transport controls only once', () => {
        const { context, document, elements } = loadRallyViewer();
        let playClicks = 0;
        context.rallyTogglePlayPause = function rallyTogglePlayPause() {
            playClicks += 1;
        };
        const listeners = [];
        elements.set('rallyPlayPause', {
            addEventListener(_type, fn) {
                listeners.push(fn);
            },
        });
        document.querySelectorAll = function querySelectorAll() {
            return [];
        };

        context.rallyBindTransportControls();
        context.rallyBindTransportControls();
        assert.equal(listeners.length, 1);
        listeners[0]({ type: 'click' });
        assert.equal(playClicks, 1);
        assert.equal(context.window.__rallyTransportBound, true);
    });

    it('binds keyboard controls only once', () => {
        const { context, document } = loadRallyViewer();
        const keyListeners = [];
        document.addEventListener = function addEventListener(type, fn) {
            if (type === 'keydown') {
                keyListeners.push(fn);
            }
        };
        context.rallyBindKeyboardControls();
        context.rallyBindKeyboardControls();
        assert.equal(keyListeners.length, 1);
        assert.equal(context.window.__rallyKeyboardBound, true);
    });

    it('resolves viewer robot by robotnr and clock length from that robot', () => {
        const { context } = loadRallyViewer();
        const payload = validPayload();
        payload.robots.robot = [
            {
                ...payload.robots.robot[0],
                robotnr: 0,
                locations: [
                    { x: 0, y: 0, o: 0, A: 0, B: 0, C: 0, DA: 0, DB: 0, DC: 0, cpu: [{ l: 9 }] },
                ],
            },
            {
                ...payload.robots.robot[0],
                robotnr: 2,
                locations: [
                    { x: 1, y: 1, o: 0, A: 0, B: 0, C: 0, DA: 0, DB: 0, DC: 0, cpu: [{ l: 1 }] },
                    { x: 2, y: 1, o: 0, A: 0, B: 0, C: 0, DA: 0, DB: 0, DC: 0, l: 1 },
                    { x: 3, y: 1, o: 0, A: 0, B: 0, C: 0, DA: 0, DB: 0, DC: 0, l: 1 },
                ],
            },
        ];
        assert.equal(context.applyRallyResultPayload(payload), null);
        context.myRallyViewerSlot = 2;
        context.rallyRebuildCpuTimeline();
        assert.equal(context.rallyViewerRobot().robotnr, 2);
        assert.equal(context.rallyTotalMiningCycles(), 3);
        assert.equal(context.myRallyCpuTimeline[0].l, 1);
    });

    it('builds a CPU timeline and seeks by CPU vs mining cycle', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.myRallyPlayer.baseStepTime = 50;
        context.myRallyPlayer.speed = 1;
        context.myRallyPlayer.elapsedMs = 0;
        context.myRallyPlayer.playing = false;
        context.myRallyPlayer.pausedCpuIndex = null;
        context.redrawRallyScene = function redrawRallyScene() {};
        context.rallyPause = function rallyPause() {
            context.myRallyPlayer.playing = false;
        };
        context.rallyPlay = function rallyPlay() {
            context.myRallyPlayer.playing = true;
            context.myRallyPlayer.pausedCpuIndex = null;
        };

        assert.equal(context.myRallyCpuTimeline.length, 4);
        // Playback clock is mining cycles; CPU steps are for paused scrubbing.
        assert.equal(context.rallyTotalMiningCycles(), 3);
        assert.equal(context.rallyTotalCpuSteps(), 4);
        assert.equal(context.rallyTotalTime(), 150);
        assert.equal(context.rallyCpuIndexAtTime(0), 0);
        assert.equal(context.rallyPoseTimeForRender(0, { miningCycle: 0 }), 0);
        assert.deepEqual(context.myRallyCpuTimeline[0].r, { k: 'i', v: 4 });
        assert.deepEqual(context.myRallyCpuTimeline[0].vs, { x: { k: 'i', v: 4 } });
        assert.deepEqual(context.myRallyCpuTimeline[1].r, { k: 'f', v: 1.42 });
        assert.deepEqual(context.myRallyCpuTimeline[2].r, { k: 'b', v: 1 });
        // Sticky location carries token span (not return value) from the prior same-line step.
        assert.equal(context.myRallyCpuTimeline[3].r, undefined);
        assert.equal(context.myRallyCpuTimeline[3].c, 9);
        assert.equal(context.myRallyCpuTimeline[3].e, 16);

        context.rallySeekByCpuSteps(1);
        assert.equal(context.myRallyPlayer.pausedCpuIndex, 1);
        assert.equal(context.rallyCpuIndexAtTime(context.myRallyPlayer.elapsedMs), 1);
        assert.equal(
            context.rallyCpuEntryAtTime(context.myRallyPlayer.elapsedMs).miningCycle,
            1
        );
        assert.equal(context.myRallyPlayer.elapsedMs, 50);
        // Expression CPUs on locations[1] show the pre-move pose (start of segment 0→1).
        assert.equal(
            context.rallyPoseTimeForRender(50, context.myRallyCpuTimeline[1]),
            0
        );

        context.rallySeekByCpuSteps(1);
        assert.equal(context.myRallyPlayer.pausedCpuIndex, 2);
        // Last CPU of the cycle (action) shows mid-motion.
        assert.equal(
            context.rallyPoseTimeForRender(50, context.myRallyCpuTimeline[2]),
            25
        );

        context.rallySeekByMiningCycles(1);
        assert.equal(
            context.rallyCpuEntryAtTime(context.myRallyPlayer.elapsedMs).miningCycle,
            2
        );
        assert.equal(context.myRallyPlayer.pausedCpuIndex, 3);
        assert.equal(context.myRallyPlayer.elapsedMs, 100);

        // Paused scrub: transport cycle follows poseTime, not the highlight sample.
        const frame = context.rallyFrameTiming();
        assert.equal(frame.poseCycle, context.rallyMiningCycleAtTime(frame.poseTime));
        assert.equal(frame.entry.miningCycle, 2);
        assert.equal(frame.poseCycle, 1);
        assert.ok(
            Math.abs(frame.completed - frame.poseTime / context.rallyTotalTime()) < 1e-9,
            `scrub completed should track poseTime, got ${frame.completed}`
        );
    });

    it('resumes play from scrub poseTime without jumping', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.myRallyPlayer.baseStepTime = 50;
        context.myRallyPlayer.speed = 1;
        context.redrawRallyScene = function redrawRallyScene() {};
        context.renderRallyFrame = function renderRallyFrame() {};
        context.requestAnimFrame = function requestAnimFrame() {
            return 1;
        };
        context.cancelAnimationFrame = function cancelAnimationFrame() {};

        context.rallySeekByCpuSteps(2);
        assert.equal(context.myRallyPlayer.pausedCpuIndex, 2);
        const scrubFrame = context.rallyFrameTiming();
        const scrubPose = scrubFrame.poseTime;
        assert.equal(scrubPose, 25);
        const scrubCompleted = scrubFrame.completed;

        context.rallyPlay();
        assert.equal(context.myRallyPlayer.pausedCpuIndex, null);
        assert.equal(context.myRallyPlayer.elapsedMs, scrubPose);
        assert.equal(context.myRallyPlayer.playing, true);
        assert.ok(
            Math.abs(context.rallyFrameTiming().completed - scrubCompleted) < 1e-9,
            'Play sync to poseTime must not jump progress backward'
        );
    });

    it('marks finished only at clock end, not mid last-cycle scrub', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.myRallyPlayer.baseStepTime = 50;
        context.redrawRallyScene = function redrawRallyScene() {};
        context.renderRallyFrame = function renderRallyFrame() {};
        context.requestAnimFrame = function requestAnimFrame() {
            return 1;
        };
        context.cancelAnimationFrame = function cancelAnimationFrame() {};

        context.rallySeekByCpuSteps(100);
        assert.equal(context.myRallyPlayer.pausedCpuIndex, 3);
        assert.equal(context.myRallyPlayer.finished, false);

        context.rallySeekByMiningCycles(100);
        assert.equal(context.myRallyPlayer.finished, false);

        // Play from scrub inside last cycle must continue, not restart.
        const scrubPose = context.rallyFrameTiming().poseTime;
        context.rallyPlay();
        assert.equal(context.myRallyPlayer.elapsedMs, scrubPose);
        assert.equal(context.myRallyPlayer.playing, true);
        assert.equal(context.myRallyPlayer.finished, false);

        context.rallySeekToRatio(1);
        assert.equal(context.myRallyPlayer.finished, true);
    });

    it('does not invent a source line for sparse sticky samples after expand', () => {
        const { context } = loadRallyViewer();
        const payload = validPayload();
        payload.robots.robot[0].locations = [
            {
                x: 0,
                y: 0,
                o: 45,
                A: 0,
                B: 0,
                C: 0,
                DA: 0,
                DB: 0,
                DC: 0,
                cpu: [{ l: 1, c: 1, e: 7 }],
            },
            {
                // Sparse dump sample with no l — must not inherit prior line.
                A: 0,
                DA: 4,
            },
        ];
        assert.equal(context.applyRallyResultPayload(payload), null);
        context.expandAllRobotLocationDeltas();
        context.rallyRebuildCpuTimeline();
        assert.equal(context.myRallyCpuTimeline.length, 2);
        assert.equal(context.myRallyCpuTimeline[0].l, 1);
        assert.equal(typeof context.myRobots.robot[0].locations[1].l, 'undefined');
        assert.equal(context.myRallyCpuTimeline[1].l, undefined);
        assert.equal(context.myRallyCpuTimeline[1].c, undefined);
    });

    it('clears source highlight when timeline entry has no line', () => {
        const { context, document, elements } = loadRallyViewer();
        document.body.innerHTML = `
            <div id="rallySourceCode">
              <div class="rally-view-source-line" id="rallySourceLine1">
                <span class="rally-view-source-lineno">1</span>
                <code class="rally-view-source-text">mine();</code>
              </div>
            </div>
        `;
        elements.set('rallyEditCodeLink', {
            href: '/edit',
            getAttribute(name) {
                return name === 'data-edit-href' ? '/edit' : null;
            },
        });
        context.myRallyViewerSlot = 0;
        context.updateRallySourceHighlight({ l: 1, c: 1, e: 5 });
        assert.ok(
            document.getElementById('rallySourceLine1').classList.contains(
                'rally-view-source-line-active'
            )
        );

        const viewerRobot = { robotnr: 0, l: 9 };
        context.updateRallyViewerSourceDebug({ miningCycle: 1 }, viewerRobot);
        assert.equal(
            document.getElementById('rallySourceLine1').classList.contains(
                'rally-view-source-line-active'
            ),
            false
        );
        assert.equal(document.getElementById('rallyEditCodeLink').href, '/edit');
    });

    it('interpolates pose on the mining-cycle clock while playing', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyPlayer.baseStepTime = 50;
        context.myRallyPlayer.speed = 1;
        context.myRallyPlayer.playing = true;
        context.myRallyPlayer.pausedCpuIndex = null;
        context.myRallyPlayer.elapsedMs = 25;
        assert.equal(context.rallyMiningCycleAtTime(25), 0);
        assert.equal(context.rallyPoseTimeForRender(25, { miningCycle: 0 }), 25);
        assert.equal(context.rallyLastCpuIndexForMiningCycle(0), 0);
        assert.equal(context.rallyLastCpuIndexForMiningCycle(1), 2);
        // During segment 0→1, highlight CPUs recorded on locations[1] (destination).
        assert.equal(context.rallyCpuIndexAtTime(25), 2);
        assert.equal(context.rallyCpuEntryAtTime(25).miningCycle, 1);
    });

    it('keeps move token and variables highlighted across sticky pending-motion cycles', () => {
        const { context } = loadRallyViewer();
        const payload = validPayload();
        payload.robots.robot[0].locations = [
            {
                x: 0,
                y: 0,
                o: 45,
                A: 0,
                B: 0,
                C: 0,
                DA: 0,
                DB: 0,
                DC: 0,
                cpu: [
                    {
                        l: 2,
                        c: 12,
                        e: 34,
                        vs: { found: { k: 'b', v: 0 } },
                    },
                ],
            },
            {
                x: 1,
                y: 0,
                o: 45,
                // Continuation chunk: sticky line only (multi-cycle move).
                l: 2,
            },
            {
                x: 2,
                y: 0,
                o: 45,
                l: 2,
            },
        ];
        assert.equal(context.applyRallyResultPayload(payload), null);
        context.rallyRebuildCpuTimeline();

        assert.equal(context.myRallyCpuTimeline.length, 3);
        assert.equal(context.myRallyCpuTimeline[0].c, 12);
        assert.equal(context.myRallyCpuTimeline[0].e, 34);
        assert.deepEqual(context.myRallyCpuTimeline[0].vs, { found: { k: 'b', v: 0 } });

        assert.equal(context.myRallyCpuTimeline[1].l, 2);
        assert.equal(context.myRallyCpuTimeline[1].c, 12);
        assert.equal(context.myRallyCpuTimeline[1].e, 34);
        assert.deepEqual(context.myRallyCpuTimeline[1].vs, { found: { k: 'b', v: 0 } });
        assert.equal(context.myRallyCpuTimeline[1].r, undefined);

        assert.equal(context.myRallyCpuTimeline[2].c, 12);
        assert.equal(context.myRallyCpuTimeline[2].e, 34);
        assert.deepEqual(context.myRallyCpuTimeline[2].vs, { found: { k: 'b', v: 0 } });
    });

    it('highlights a token span within a source line', () => {
        const { context, document } = loadRallyViewer();
        document.body.innerHTML = `
            <div id="rallySourceCode">
              <div class="rally-view-source-line" id="rallySourceLine1">
                <span class="rally-view-source-lineno">1</span>
                <code class="rally-view-source-text">mine(); dumpA();</code>
              </div>
            </div>
            <output id="rallySourceStepResult"></output>
        `;
        context.updateRallySourceHighlight({ l: 1, c: 9, e: 16 });
        const line = document.getElementById('rallySourceLine1');
        assert.ok(line.classList.contains('rally-view-source-line-active'));
        const token = line.querySelector('.rally-view-source-token-active');
        assert.ok(token);
        assert.equal(token.textContent, 'dumpA()');
        assert.equal(document.getElementById('rallySourceStepResult').textContent, '');
    });

    it('formats typed CPU step return values in the Return value field', () => {
        const { context, document } = loadRallyViewer();
        document.body.innerHTML = `
            <div id="rallySourceCode">
              <div class="rally-view-source-line" id="rallySourceLine1">
                <span class="rally-view-source-lineno">1</span>
                <code class="rally-view-source-text">int x = 3 + 4;</code>
              </div>
            </div>
            <output id="rallySourceStepResult"></output>
        `;
        const result = document.getElementById('rallySourceStepResult');

        context.updateRallySourceHighlight({ l: 1, c: 13, e: 14, r: { k: 'i', v: 4 } });
        assert.equal(result.textContent, '4');

        context.updateRallySourceHighlight({ l: 1, c: 13, e: 14, r: { k: 'f', v: 1.42 } });
        assert.equal(result.textContent, '1.42');

        context.updateRallySourceHighlight({ l: 1, c: 13, e: 14, r: { k: 'b', v: 1 } });
        assert.equal(result.textContent, 'true');

        context.updateRallySourceHighlight({ l: 1, c: 13, e: 14, r: { k: 'b', v: 0 } });
        assert.equal(result.textContent, 'false');

        context.updateRallySourceHighlight({ l: 1, c: 13, e: 14, r: { k: 'x', v: 9 } });
        assert.equal(result.textContent, 'x:9');

        context.updateRallySourceHighlight({ l: 1, c: 9, e: 10 });
        assert.equal(result.textContent, '');
    });

    it('shows typed program variables under Return value', () => {
        const { context, document } = loadRallyViewer();
        document.body.innerHTML = `
            <div id="rallySourceCode">
              <div class="rally-view-source-line" id="rallySourceLine1">
                <span class="rally-view-source-lineno">1</span>
                <code class="rally-view-source-text">int count = 3;</code>
              </div>
            </div>
            <output id="rallySourceStepResult"></output>
            <table><tbody id="rallySourceVariables"></tbody></table>
        `;
        const variables = document.getElementById('rallySourceVariables');

        context.updateRallySourceHighlight({
            l: 1,
            c: 1,
            e: 14,
            r: { k: 'i', v: 3 },
            vs: {
                found: { k: 'b', v: 1 },
                count: { k: 'i', v: 3 },
                speed: { k: 'f', v: 1.5 },
            },
        });
        assert.equal(variables.childNodes.length, 3);
        assert.equal(variables.childNodes[0].children[0].textContent, 'count:');
        assert.equal(variables.childNodes[0].children[1].textContent, '3');
        assert.equal(variables.childNodes[1].children[0].textContent, 'found:');
        assert.equal(variables.childNodes[1].children[1].textContent, 'true');
        assert.equal(variables.childNodes[2].children[0].textContent, 'speed:');
        assert.equal(variables.childNodes[2].children[1].textContent, '1.50');

        context.updateRallySourceHighlight({ l: 1, c: 1, e: 4 });
        assert.equal(variables.childNodes.length, 0);
    });

    function installSidePanelDom(document, elements, order = 'players') {
        const children = [];
        const column = {
            className: 'rally-view-side-column',
            get children() {
                return children.slice();
            },
            appendChild(child) {
                const idx = children.indexOf(child);
                if (idx >= 0) {
                    children.splice(idx, 1);
                }
                children.push(child);
                return child;
            },
        };

        function makeButton(panel) {
            return {
                className: 'rally-view-panel-order-button',
                disabled: panel === order,
                attributes: { 'data-rally-panel': panel },
                getAttribute(name) {
                    return this.attributes[name] || null;
                },
                closest(sel) {
                    if (sel === '.rally-view-panel-order-button[data-rally-panel]') {
                        return this;
                    }
                    return null;
                },
            };
        }

        const programButton = makeButton('program');
        const playersButton = makeButton('players');
        const programPanel = {
            id: 'rallyViewProgramPanel',
            orderButton: programButton,
        };
        const playersPanel = {
            id: 'rallyViewPlayersPanel',
            orderButton: playersButton,
        };

        if (order === 'program') {
            children.push(programPanel, playersPanel);
        } else {
            children.push(playersPanel, programPanel);
        }

        elements.set('rallyViewProgramPanel', programPanel);
        elements.set('rallyViewPlayersPanel', playersPanel);

        document.querySelector = function querySelector(sel) {
            if (sel === '.rally-view-side-column') {
                return column;
            }
            if (sel === '.rally-view-stage') {
                return elements.get('rally-view-stage');
            }
            return null;
        };
        document.querySelectorAll = function querySelectorAll(sel) {
            if (sel === '.rally-view-panel-order-button[data-rally-panel]') {
                return [programButton, playersButton];
            }
            return [];
        };

        return { column, programPanel, playersPanel, programButton, playersButton };
    }

    it('defaults to players-first side panel order', () => {
        const { context, document, elements } = loadRallyViewer();
        const { column, programPanel, playersPanel, programButton, playersButton } =
            installSidePanelDom(document, elements, 'program');

        context.rallyApplySidePanelOrder();

        assert.equal(column.children[0], playersPanel);
        assert.equal(column.children[1], programPanel);
        assert.equal(playersButton.disabled, true);
        assert.equal(programButton.disabled, false);
    });

    it('applies program-first side panel order from storage', () => {
        const { context, document, elements } = loadRallyViewer();
        context.localStorage.setItem('robominer.rallySidePanelOrder', 'program');
        const { column, programPanel, playersPanel, programButton, playersButton } =
            installSidePanelDom(document, elements);

        context.rallyApplySidePanelOrder();

        assert.equal(column.children[0], programPanel);
        assert.equal(column.children[1], playersPanel);
        assert.equal(programButton.disabled, true);
        assert.equal(playersButton.disabled, false);
    });

    it('Move up puts program panel first and persists preference', () => {
        const { context, document, elements } = loadRallyViewer();
        const { column, programPanel, playersPanel, programButton, playersButton } =
            installSidePanelDom(document, elements, 'players');
        const clickListeners = [];
        document.addEventListener = function addEventListener(type, fn) {
            if (type === 'click') {
                clickListeners.push(fn);
            }
        };

        context.rallyApplySidePanelOrder('players');
        context.rallyBindSidePanelOrder();
        context.rallyBindSidePanelOrder();
        assert.equal(clickListeners.length, 1);

        clickListeners[0]({ target: programButton });

        assert.equal(
            context.localStorage.getItem('robominer.rallySidePanelOrder'),
            'program'
        );
        assert.equal(column.children[0], programPanel);
        assert.equal(column.children[1], playersPanel);
        assert.equal(programButton.disabled, true);
        assert.equal(playersButton.disabled, false);
    });
});
