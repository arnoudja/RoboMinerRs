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

describe('rally timeline helpers', () => {
    it('rebuilds CPU timeline and reports total turns', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.rallyRebuildCpuTimeline();
        assert.equal(context.rallyTotalTurns(), 3);
        assert.ok(context.rallyTotalCpuSteps() > 0);
    });

    it('maps CPU index to step within turn', () => {
        const { context } = loadRallyViewer();
        assert.equal(context.applyRallyResultPayload(validPayload()), null);
        context.myRallyViewerSlot = 0;
        context.rallyRebuildCpuTimeline();
        const firstTurnIndex = context.rallyFirstCpuIndexForTurn(1);
        const step = context.rallyCpuStepWithinTurn(firstTurnIndex, 1);
        assert.ok(step >= 1);
        assert.ok(step <= context.rallyViewerCpuSpeed());
    });
});
