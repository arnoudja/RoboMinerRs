'use strict';

const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const { loadRallyViewer } = require('./load_viewer');

function validPayload() {
    return {
        v: 2,
        robots: {
            robot: [
                {
                    robotnr: 0,
                    x: 0,
                    y: 0,
                    o: 45,
                    cpuspeed: 4,
                    locations: [
                        { cpu: [{ l: 1, c: 1, e: 7 }] },
                        { cpu: [{ l: 1, c: 1, e: 7 }] },
                        { l: 1 },
                    ],
                },
            ],
        },
        ground: { sizeX: 4, sizeY: 4, positions: [] },
    };
}

describe('rally transport helpers', () => {
    it('detects finished playback when elapsed time reaches timeline end', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.rallyRebuildCpuTimeline();

        assert.equal(context.rallyIsPlaybackFinished(), false);
        context.myRallyPlayer.elapsedMs = context.rallyTotalTime();
        assert.equal(context.rallyIsPlaybackFinished(), true);
    });

    it('restart resets elapsed time and finished flag', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.redrawRallyScene = function() {};
        context.myRallyPlayer.elapsedMs = context.rallyTotalTime();
        context.myRallyPlayer.finished = true;

        context.rallyRestart();

        assert.equal(context.myRallyPlayer.elapsedMs, 0);
        assert.equal(context.myRallyPlayer.finished, false);
        assert.equal(context.myRallyPlayer.pausedCpuIndex, null);
    });
});
