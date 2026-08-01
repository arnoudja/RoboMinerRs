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

/** Flat CPU-instruction timeline for the viewer robot: [{miningCycle,l,c,e}, ...] */
var myRallyCpuTimeline = null;


function rallyHasAnimationData()
{
    return typeof myRobots !== 'undefined' &&
        myRobots.robot &&
        myRobots.robot.length > 0 &&
        myRobots.robot[0].locations;
}


function rallyViewerRobot()
{
    if (!rallyHasAnimationData())
    {
        return null;
    }
    if (typeof myRallyViewerSlot === 'number' && myRobots.robot[myRallyViewerSlot])
    {
        return myRobots.robot[myRallyViewerSlot];
    }
    return myRobots.robot[0];
}


function rallyRebuildCpuTimeline()
{
    myRallyCpuTimeline = null;
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
            timeline.push({
                miningCycle: m,
                l: loc.l,
                c: undefined,
                e: undefined,
                r: undefined,
                vs: undefined
            });
        }
    }
    myRallyCpuTimeline = timeline;
}


function rallyTotalMiningCycles()
{
    if (!rallyHasAnimationData())
    {
        return 0;
    }
    return myRobots.robot[0].locations.length;
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
    var found = -1;
    for (var i = 0; i < myRallyCpuTimeline.length; i++)
    {
        if (myRallyCpuTimeline[i].miningCycle === miningCycle)
        {
            found = i;
        }
        else if (myRallyCpuTimeline[i].miningCycle > miningCycle)
        {
            break;
        }
    }
    if (found >= 0)
    {
        return found;
    }
    return rallyFirstCpuIndexForMiningCycle(miningCycle);
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
    // At the exact start of a mining cycle (incl. replay start), show the first CPU step.
    if (phase <= 0)
    {
        return rallyFirstCpuIndexForMiningCycle(cycle);
    }
    return rallyLastCpuIndexForMiningCycle(cycle);
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
    // Paused CPU scrub: show the recorded pose for that mining cycle (locations[m]).
    var miningCycle = entry && typeof entry.miningCycle === 'number' ? entry.miningCycle : 0;
    return miningCycle * rallyStepTime();
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
    var completed = totalTime > 0 ? time / totalTime : 0;
    if (completed > 1)
    {
        completed = 1;
    }

    var cpuIndex = rallyCpuIndexAtTime(time);
    var entry = rallyCpuEntryAtTime(time);
    var cycle = entry.miningCycle;
    if (myRallyPlayer.playing || myRallyPlayer.pausedCpuIndex === null)
    {
        cycle = rallyMiningCycleAtTime(time);
    }
    if (cycle > rallyTotalMiningCycles())
    {
        cycle = rallyTotalMiningCycles();
    }

    return {
        time: time,
        stepTime: stepTime,
        completed: completed,
        cpuIndex: cpuIndex,
        entry: entry,
        cycle: cycle,
        poseTime: rallyPoseTimeForRender(time, entry)
    };
}
