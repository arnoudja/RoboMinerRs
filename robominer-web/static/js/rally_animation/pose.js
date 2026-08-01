/** Fill sparse location deltas forward from the previous recorded pose. */
function updateRobotTo(robotIndex, step)
{
    var robot = myRobots.robot[robotIndex];
    if (!robot.locations || robot.locations.length === 0)
    {
        return;
    }

    var target = Math.min(step, robot.locations.length - 1);
    var filled = typeof robot.updatedTo === 'number' ? robot.updatedTo : 0;
    if (target <= filled)
    {
        return;
    }

    for (var s = filled + 1; s <= target; s++)
    {
        fillLocationDeltaFromPrevious(robot.locations[s], robot.locations[s - 1]);
        // Do not fill-forward `s`: productive cycles omit it intentionally.
        robot.updatedTo = s;
    }
}


function fillLocationDeltaFromPrevious(current, previous)
{
    copyDefinedField(current, previous, 'x', true);
    copyDefinedField(current, previous, 'y', true);
    copyDefinedField(current, previous, 'o', true);
    copyDefinedField(current, previous, 'A', true);
    copyDefinedField(current, previous, 'B', true);
    copyDefinedField(current, previous, 'C', true);
    copyDefinedField(current, previous, 'DA', false);
    copyDefinedField(current, previous, 'DB', false);
    copyDefinedField(current, previous, 'DC', false);
    copyDefinedField(current, previous, 'a', false);
    // Do not fill-forward `l`: source-line is debug metadata; sparse samples must stay unset
    // so the CPU timeline does not invent sticky highlights.
}


function copyDefinedField(current, previous, field, alwaysIfMissing)
{
    if (typeof current[field] !== 'undefined')
    {
        return;
    }
    if (alwaysIfMissing || typeof previous[field] !== 'undefined')
    {
        current[field] = previous[field];
    }
}


function applyRobotPoseFromLocation(robot, loc)
{
    robot.x = loc.x;
    robot.y = loc.y;
    robot.o = loc.o;
    robot.A = loc.A;
    robot.B = loc.B;
    robot.C = loc.C;
    robot.DA = loc.DA;
    robot.DB = loc.DB;
    robot.DC = loc.DC;
    robot.a = loc.a;
    robot.l = loc.l;
    robot.s = loc.s;
}


function applyRobotPoseInterpolated(robot, loc1, loc2, dt, travelTime)
{
    robot.x = smoothen(loc1.x, loc2.x, dt, travelTime);
    robot.y = smoothen(loc1.y, loc2.y, dt, travelTime);
    robot.o = loc1.o;
    robot.A = smoothen(loc1.A, loc2.A, dt, travelTime);
    robot.B = smoothen(loc1.B, loc2.B, dt, travelTime);
    robot.C = smoothen(loc1.C, loc2.C, dt, travelTime);
    robot.DA = smoothen(
        typeof loc1.DA !== 'undefined' ? loc1.DA : 0,
        typeof loc2.DA !== 'undefined' ? loc2.DA : 0,
        dt,
        travelTime
    );
    robot.DB = smoothen(
        typeof loc1.DB !== 'undefined' ? loc1.DB : 0,
        typeof loc2.DB !== 'undefined' ? loc2.DB : 0,
        dt,
        travelTime
    );
    robot.DC = smoothen(
        typeof loc1.DC !== 'undefined' ? loc1.DC : 0,
        typeof loc2.DC !== 'undefined' ? loc2.DC : 0,
        dt,
        travelTime
    );
}


function applyRobotActionHighlight(robot, loc1, loc2, t1, dt)
{
    // At the exact start of a segment (including replay start), prefer t1's line so
    // locations[0].l (program entry) is shown instead of jumping to the first action cycle.
    if (dt <= 0 && typeof loc1.l !== 'undefined')
    {
        robot.a = loc1.a;
        robot.l = loc1.l;
        robot.s = loc1.s;
    }
    else if (dt <= 0 && t1 === 0)
    {
        // Legacy replays omit locations[0].l — still show program entry.
        robot.a = loc1.a;
        robot.l = 1;
        robot.s = loc1.s;
    }
    else
    {
        robot.a = loc2.a;
        robot.l = loc2.l;
        robot.s = loc2.s;
    }
}


function updateRobotPosition(robotIndex, time, stepTime)
{
    var robot = myRobots.robot[robotIndex];
    if (!(stepTime > 0))
    {
        updateRobotTo(robotIndex, 0);
        if (robot.locations && robot.locations.length > 0)
        {
            applyRobotPoseFromLocation(robot, robot.locations[0]);
        }
        return;
    }

    var t1 = Math.floor(time / stepTime);
    var t2 = t1 + 1;

    updateRobotTo(robotIndex, t2);

    if (t2 >= robot.locations.length)
    {
        applyRobotPoseFromLocation(robot, robot.locations[robot.locations.length - 1]);
        return;
    }

    var loc1 = robot.locations[t1];
    var loc2 = robot.locations[t2];
    var dt = time % stepTime;
    var timeFraction = typeof loc2.t !== 'undefined' ? loc2.t : 1.0;

    if (dt >= stepTime * timeFraction)
    {
        applyRobotPoseFromLocation(robot, loc2);
        return;
    }

    var travelTime = stepTime * timeFraction;
    applyRobotPoseInterpolated(robot, loc1, loc2, dt, travelTime);
    applyRobotActionHighlight(robot, loc1, loc2, t1, dt);
}


function expandAllRobotLocationDeltas()
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        var robot = myRobots.robot[i];
        if (!robot.locations || robot.locations.length === 0)
        {
            continue;
        }
        if (typeof robot.updatedTo !== 'number')
        {
            robot.updatedTo = 0;
        }
        updateRobotTo(i, robot.locations.length - 1);
    }
}
