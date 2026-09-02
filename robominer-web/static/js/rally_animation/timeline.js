/**
 * @typedef {{turn:number,l?:number,c?:number,e?:number,r?:{k?:string,v?:number},vs?:Object.<string,{k?:string,v?:number}>}} RallyCpuTimelineEntry
 *
 * Wire contract (see AnimationLocation / AnimationCpuStep in robominer-sim):
 * - locations[m] is the pose after turn m; cpu[] on that sample drove the motion
 *   animated during clock segment [m-1, m).
 * - cpu[].c/e are 1-based half-open [c, e) source columns; omit when unknown.
 * - Emit either sticky l or non-empty cpu per location, not both.
 * - vs is a full locals snapshot (not a delta). r is omitted while an action is still awaiting.
 * - frame.poseTurn is the pose clock (sprite/ground); entry.turn is the highlight sample.
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
var myRallyCpuTurnIndex = { first: [], last: [] };


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


function rallyRecordCpuTurnIndex(turn, timelineIndex)
{
    if (typeof myRallyCpuTurnIndex.first[turn] !== 'number')
    {
        myRallyCpuTurnIndex.first[turn] = timelineIndex;
    }
    myRallyCpuTurnIndex.last[turn] = timelineIndex;
}


function rallyRebuildCpuTimeline()
{
    myRallyCpuTimeline = null;
    myRallyCpuTurnIndex = { first: [], last: [] };
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
                rallyRecordCpuTurnIndex(m, timeline.length);
                timeline.push({
                    turn: m,
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
                turn: m,
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
            rallyRecordCpuTurnIndex(m, timeline.length);
            timeline.push(sticky);
        }
    }
    myRallyCpuTimeline = timeline;
}


function rallyTotalTurns()
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
    return rallyTotalTurns();
}


function rallyStepTime()
{
    return myRallyPlayer.baseStepTime / myRallyPlayer.speed;
}


/**
 * Wall-clock length of the turn timeline (`n * stepTime`).
 * Includes a final hold on the last pose, so scrubbing the last-cycle CPU sample is
 * not `finished` until the clock reaches this end.
 */
function rallyTotalTime()
{
    return rallyTotalTurns() * rallyStepTime();
}


function rallyTurnAtTime(time)
{
    var stepTime = rallyStepTime();
    var total = rallyTotalTurns();
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


function rallyFirstCpuIndexForTurn(turn)
{
    if (!myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        return Math.max(0, turn);
    }
    if (typeof myRallyCpuTurnIndex.first[turn] === 'number')
    {
        return myRallyCpuTurnIndex.first[turn];
    }
    for (var i = 0; i < myRallyCpuTimeline.length; i++)
    {
        if (myRallyCpuTimeline[i].turn === turn)
        {
            return i;
        }
        if (myRallyCpuTimeline[i].turn > turn)
        {
            return Math.max(0, i - 1);
        }
    }
    return myRallyCpuTimeline.length - 1;
}


function rallyLastCpuIndexForTurn(turn)
{
    if (!myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        return Math.max(0, turn);
    }
    if (typeof myRallyCpuTurnIndex.last[turn] === 'number')
    {
        return myRallyCpuTurnIndex.last[turn];
    }
    return rallyFirstCpuIndexForTurn(turn);
}


/**
 * Turn sample used for source highlight at wall-clock `time`.
 * Recording stores CPUs that produced locations[m] on that same sample, while pose
 * interpolates locations[m-1] → locations[m] during clock cycle m-1. Highlight the
 * destination sample's CPUs so move() lines up with visible travel.
 * At t=0 keep locations[0] so program-entry CPUs are not skipped.
 */
function rallyHighlightTurn(time)
{
    var cycle = rallyTurnAtTime(time);
    var stepTime = rallyStepTime();
    var phase = stepTime > 0 ? time - cycle * stepTime : 0;
    var maxCycle = Math.max(0, rallyTotalTurns() - 1);
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
    var cycle = rallyTurnAtTime(time);
    var stepTime = rallyStepTime();
    var phase = stepTime > 0 ? time - cycle * stepTime : 0;
    var highlightCycle = rallyHighlightTurn(time);

    if (phase <= 0)
    {
        return rallyFirstCpuIndexForTurn(highlightCycle);
    }
    return rallyLastCpuIndexForTurn(highlightCycle);
}


function rallyCpuEntryAtTime(time)
{
    if (!myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        var cycle = rallyTurnAtTime(time);
        return { turn: cycle, l: undefined, c: undefined, e: undefined };
    }
    return myRallyCpuTimeline[rallyCpuIndexAtTime(time)];
}


/** True when paused arrow keys are scrubbing the CPU timeline (not continuous play). */
function rallyCpuScrubActive()
{
    return !myRallyPlayer.playing && myRallyPlayer.pausedCpuIndex !== null;
}


function rallyPoseTimeForRender(time, entry)
{
    // Continuous play always interpolates across turns.
    if (myRallyPlayer.playing)
    {
        return time;
    }
    // Paused without CPU scrub: hold the smooth turn clock.
    if (!rallyCpuScrubActive())
    {
        return time;
    }
    // Paused CPU scrub: CPUs on locations[m] drove the motion animated in [m-1, m).
    // Show the pre-action pose for expression steps; mid-motion for the last (action) step.
    var turn = entry && typeof entry.turn === 'number' ? entry.turn : 0;
    if (turn <= 0)
    {
        return 0;
    }
    var stepTime = rallyStepTime();
    var segmentStart = (turn - 1) * stepTime;
    var lastIdx = rallyLastCpuIndexForTurn(turn);
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


function rallyViewerCpuSpeed()
{
    var robot = rallyViewerRobot();
    if (!robot || typeof robot.cpuspeed !== 'number' || isNaN(robot.cpuspeed) || robot.cpuspeed <= 0)
    {
        return 0;
    }
    return Math.floor(robot.cpuspeed);
}


/** 1-based CPU step index within `turn` for global timeline index `cpuIndex`. */
function rallyCpuStepWithinTurn(cpuIndex, turn)
{
    rallyEnsureCpuTimeline();
    if (typeof turn !== 'number' || turn < 0)
    {
        return 0;
    }
    var first = rallyFirstCpuIndexForTurn(turn);
    var last = rallyLastCpuIndexForTurn(turn);
    var stepsInTurn = Math.max(1, last - first + 1);
    var step = cpuIndex - first + 1;
    if (step < 1)
    {
        return 1;
    }
    if (step > stepsInTurn)
    {
        return stepsInTurn;
    }
    return step;
}


/** Line-only highlight for continuous turn playback (no return value / locals). */
function rallyTurnLevelDebugEntry(turn)
{
    var robot = rallyViewerRobot();
    if (!robot || !robot.locations || typeof turn !== 'number' || turn < 0 || turn >= robot.locations.length)
    {
        return null;
    }
    var loc = robot.locations[turn];
    if (typeof loc.l === 'number')
    {
        return { turn: turn, l: loc.l };
    }
    if (loc.cpu && loc.cpu.length > 0 && typeof loc.cpu[0].l === 'number')
    {
        return { turn: turn, l: loc.cpu[0].l };
    }
    return null;
}


/**
 * True when a CPU timeline sample carries token/result/locals debug fields.
 * Pose-only samples (legacy `l` or entry poses before the first micro-step) do not.
 */
function rallyCpuEntryHasDebugDetail(entry)
{
    return !!(entry && (entry.vs || entry.r || typeof entry.c === 'number'));
}


/**
 * When paused on a pose-only sample, prefer the nearest CPU micro-step with locals /
 * return value / token span so the source panel is not empty at t=0 or between turns.
 */
function rallyDetailedCpuEntryNear(entry)
{
    if (!entry || !myRallyCpuTimeline || myRallyCpuTimeline.length === 0)
    {
        return entry;
    }
    if (rallyCpuEntryHasDebugDetail(entry))
    {
        return entry;
    }

    var turn = typeof entry.turn === 'number' ? entry.turn : 0;
    var i;
    for (i = 0; i < myRallyCpuTimeline.length; i++)
    {
        var candidate = myRallyCpuTimeline[i];
        if (candidate.turn < turn)
        {
            continue;
        }
        if (rallyCpuEntryHasDebugDetail(candidate))
        {
            return candidate;
        }
        if (candidate.turn > turn + 1)
        {
            break;
        }
    }
    for (i = myRallyCpuTimeline.length - 1; i >= 0; i--)
    {
        if (rallyCpuEntryHasDebugDetail(myRallyCpuTimeline[i]))
        {
            return myRallyCpuTimeline[i];
        }
    }
    return entry;
}


/**
 * Full CPU detail (token span, return value, variables) while paused or scrubbing.
 * Continuous play keeps a line-only highlight so the panel does not flicker each micro-step.
 */
function rallyEntryForViewerDebug(entry, poseTurn)
{
    if (!myRallyPlayer.playing)
    {
        return rallyDetailedCpuEntryNear(entry);
    }
    return rallyTurnLevelDebugEntry(poseTurn);
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
    var poseTurn = rallyTurnAtTime(poseTime);
    var totalCycles = rallyTotalTurns();
    if (totalCycles <= 0)
    {
        poseTurn = 0;
    }
    else if (poseTurn >= totalCycles)
    {
        poseTurn = totalCycles - 1;
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
        poseTurn: poseTurn,
        poseTime: poseTime
    };
}
