/**
 * @typedef {{miningCycle:number,l?:number,c?:number,e?:number,r?:{k?:string,v?:number},vs?:Object.<string,{k?:string,v?:number}>}} RallyCpuTimelineEntry
 *
 * Wire contract (see AnimationLocation / AnimationCpuStep in robominer-sim):
 * - locations[m] is the pose after mining cycle m; cpu[] on that sample drove the motion
 *   animated during clock segment [m-1, m).
 * - cpu[].c/e are 1-based half-open [c, e) source columns; omit when unknown.
 * - Emit either sticky l or non-empty cpu per location, not both.
 * - vs is a full locals snapshot (not a delta). r is omitted while an action is still awaiting.
 * - frame.poseCycle is the pose clock (sprite/ground); entry.miningCycle is the highlight sample.
 * - Clock length is viewer-robot only; peers with fewer locations freeze at their last pose.
 */

var myRallyPlayer = {
    scale: 1,
    baseStepTime: 50,
    speed: 1,
    playing: false,
    finished: false,
    elapsedMs: 0,
    frameId: null,
    lastFrameTime: null,
    /** When paused, arrow keys scrub this CPU-timeline index; null while playing. */
    pausedCpuIndex: null
};

/** @type {RallyCpuTimelineEntry[]|null} */
var myRallyCpuTimeline = null;
/** @type {{first:number[], last:number[]}} */
var myRallyCpuCycleIndex = { first: [], last: [] };


function rallyHasAnimationData()
{
    if (typeof myRobots === 'undefined' || !myRobots.robot || myRobots.robot.length === 0)
    {
        return false;
    }
    var robot = rallyViewerRobot();
    return !!(robot && robot.locations && robot.locations.length > 0);
}


/**
 * Viewer robot by `robotnr` (matches draw/debug), not array index.
 * @returns {object|null}
 */
function rallyViewerRobot()
{
    if (typeof myRobots === 'undefined' || !myRobots.robot || myRobots.robot.length === 0)
    {
        return null;
    }
    if (typeof myRallyViewerSlot === 'number')
    {
        for (var i = 0; i < myRobots.robot.length; i++)
        {
            if (myRobots.robot[i].robotnr === myRallyViewerSlot)
            {
                return myRobots.robot[i];
            }
        }
    }
    return myRobots.robot[0];
}


function rallyRecordCpuCycleIndex(miningCycle, timelineIndex)
{
    if (typeof myRallyCpuCycleIndex.first[miningCycle] !== 'number')
    {
        myRallyCpuCycleIndex.first[miningCycle] = timelineIndex;
    }
    myRallyCpuCycleIndex.last[miningCycle] = timelineIndex;
}


function rallyRebuildCpuTimeline()
{
    myRallyCpuTimeline = null;
    myRallyCpuCycleIndex = { first: [], last: [] };
    var robot = rallyViewerRobot();
    if (!robot || !robot.locations)
    {
        return;
    }

    var timeline = [];
    for (var m = 0; m < robot.locations.length; m++)
    {
        var loc = robot.locations[m];
        var cpu = loc.cpu;
        if (cpu && cpu.length > 0)
        {
            for (var i = 0; i < cpu.length; i++)
            {
                rallyRecordCpuCycleIndex(m, timeline.length);
                timeline.push({
                    miningCycle: m,
                    l: cpu[i].l,
                    c: cpu[i].c,
                    e: cpu[i].e,
                    r: cpu[i].r,
                    vs: cpu[i].vs
                });
            }
        }
        else
        {
            // Legacy sticky `l` only (older payloads without recorded sticky cpu spans).
            // Carry prior same-line c/e/vs so multi-cycle move/rotate stays highlighted.
            var sticky = {
                miningCycle: m,
                l: loc.l,
                c: undefined,
                e: undefined,
                r: undefined,
                vs: undefined
            };
            for (var j = timeline.length - 1; j >= 0; j--)
            {
                var prev = timeline[j];
                // Only carry columns/vs when this sample has an explicit same-line `l`.
                // Do not invent a line from prior cycles for sparse dump/ore-only samples.
                if (typeof sticky.l === 'number' &&
                    prev.l === sticky.l &&
                    typeof prev.c === 'number' &&
                    typeof prev.e === 'number')
                {
                    sticky.c = prev.c;
                    sticky.e = prev.e;
                    sticky.vs = prev.vs;
                    break;
                }
            }
            rallyRecordCpuCycleIndex(m, timeline.length);
            timeline.push(sticky);
        }
    }
    myRallyCpuTimeline = timeline;
}


function rallyTotalMiningCycles()
{
    var robot = rallyViewerRobot();
    if (!robot || !robot.locations)
    {
        return 0;
    }
    return robot.locations.length;
}


function rallyTotalCpuSteps()
{
    if (myRallyCpuTimeline && myRallyCpuTimeline.length > 0)
    {
        return myRallyCpuTimeline.length;
    }
    return rallyTotalMiningCycles();
}


function rallyStepTime()
{
    return myRallyPlayer.baseStepTime / myRallyPlayer.speed;
}


/**
 * Wall-clock length of the mining-cycle timeline (`n * stepTime`).
 * Includes a final hold on the last pose, so scrubbing the last-cycle CPU sample is
 * not `finished` until the clock reaches this end.
 */
function rallyTotalTime()
{
    return rallyTotalMiningCycles() * rallyStepTime();
}


function rallyMiningCycleAtTime(time)
{
    var stepTime = rallyStepTime();
    var total = rallyTotalMiningCycles();
    if (stepTime <= 0 || total <= 0)
    {
        return 0;
    }
    var index = Math.floor(time / stepTime);
    if (index < 0)
    {
        return 0;
    }
    if (index >= total)
    {
        return total - 1;
    }
    return index;
}


function rallyFirstCpuIndexForMiningCycle(miningCycle)
{
    if (!myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        return Math.max(0, miningCycle);
    }
    if (typeof myRallyCpuCycleIndex.first[miningCycle] === 'number')
    {
        return myRallyCpuCycleIndex.first[miningCycle];
    }
    for (var i = 0; i < myRallyCpuTimeline.length; i++)
    {
        if (myRallyCpuTimeline[i].miningCycle === miningCycle)
        {
            return i;
        }
        if (myRallyCpuTimeline[i].miningCycle > miningCycle)
        {
            return Math.max(0, i - 1);
        }
    }
    return myRallyCpuTimeline.length - 1;
}


function rallyLastCpuIndexForMiningCycle(miningCycle)
{
    if (!myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        return Math.max(0, miningCycle);
    }
    if (typeof myRallyCpuCycleIndex.last[miningCycle] === 'number')
    {
        return myRallyCpuCycleIndex.last[miningCycle];
    }
    return rallyFirstCpuIndexForMiningCycle(miningCycle);
}


/**
 * Mining-cycle sample used for source highlight at wall-clock `time`.
 * Recording stores CPUs that produced locations[m] on that same sample, while pose
 * interpolates locations[m-1] → locations[m] during clock cycle m-1. Highlight the
 * destination sample's CPUs so move() lines up with visible travel.
 * At t=0 keep locations[0] so program-entry CPUs are not skipped.
 */
function rallyHighlightMiningCycle(time)
{
    var cycle = rallyMiningCycleAtTime(time);
    var stepTime = rallyStepTime();
    var phase = stepTime > 0 ? time - cycle * stepTime : 0;
    var maxCycle = Math.max(0, rallyTotalMiningCycles() - 1);
    if (phase <= 0 && cycle === 0)
    {
        return 0;
    }
    return Math.min(cycle + 1, maxCycle);
}


function rallyCpuIndexAtTime(time)
{
    if (myRallyPlayer.pausedCpuIndex !== null && !myRallyPlayer.playing)
    {
        var totalCpu = rallyTotalCpuSteps();
        return Math.min(Math.max(0, myRallyPlayer.pausedCpuIndex), Math.max(0, totalCpu - 1));
    }
    var cycle = rallyMiningCycleAtTime(time);
    var stepTime = rallyStepTime();
    var phase = stepTime > 0 ? time - cycle * stepTime : 0;
    var highlightCycle = rallyHighlightMiningCycle(time);

    if (phase <= 0)
    {
        return rallyFirstCpuIndexForMiningCycle(highlightCycle);
    }
    return rallyLastCpuIndexForMiningCycle(highlightCycle);
}


function rallyCpuEntryAtTime(time)
{
    if (!myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        var cycle = rallyMiningCycleAtTime(time);
        return { miningCycle: cycle, l: undefined, c: undefined, e: undefined };
    }
    return myRallyCpuTimeline[rallyCpuIndexAtTime(time)];
}


function rallyPoseTimeForRender(time, entry)
{
    // Continuous play interpolates across mining cycles.
    if (myRallyPlayer.playing || myRallyPlayer.pausedCpuIndex === null)
    {
        return time;
    }
    // Paused CPU scrub: CPUs on locations[m] drove the motion animated in [m-1, m).
    // Show the pre-action pose for expression steps; mid-motion for the last (action) step.
    var miningCycle = entry && typeof entry.miningCycle === 'number' ? entry.miningCycle : 0;
    if (miningCycle <= 0)
    {
        return 0;
    }
    var stepTime = rallyStepTime();
    var segmentStart = (miningCycle - 1) * stepTime;
    var lastIdx = rallyLastCpuIndexForMiningCycle(miningCycle);
    if (myRallyPlayer.pausedCpuIndex === lastIdx)
    {
        return segmentStart + stepTime * 0.5;
    }
    return segmentStart;
}


function rallyEnsureCpuTimeline()
{
    if (!myRallyCpuTimeline)
    {
        rallyRebuildCpuTimeline();
    }
}


function rallyFrameTiming()
{
    var time = myRallyPlayer.elapsedMs;
    var stepTime = rallyStepTime();
    var totalTime = rallyTotalTime();

    var cpuIndex = rallyCpuIndexAtTime(time);
    var entry = rallyCpuEntryAtTime(time);
    // Transport/ground cycle follows the sprite pose clock (not the highlight sample).
    var poseTime = rallyPoseTimeForRender(time, entry);
    var poseCycle = rallyMiningCycleAtTime(poseTime);
    var totalCycles = rallyTotalMiningCycles();
    if (totalCycles <= 0)
    {
        poseCycle = 0;
    }
    else if (poseCycle >= totalCycles)
    {
        poseCycle = totalCycles - 1;
    }

    // While CPU-scrubbed, progress follows poseTime so the bar matches the sprite and
    // Play syncing elapsedMs → poseTime does not jump the fill backward.
    var progressTime = myRallyPlayer.pausedCpuIndex !== null ? poseTime : time;
    var completed = totalTime > 0 ? progressTime / totalTime : 0;
    if (completed > 1)
    {
        completed = 1;
    }
    else if (completed < 0)
    {
        completed = 0;
    }

    return {
        time: time,
        stepTime: stepTime,
        completed: completed,
        cpuIndex: cpuIndex,
        entry: entry,
        poseCycle: poseCycle,
        poseTime: poseTime
    };
}
