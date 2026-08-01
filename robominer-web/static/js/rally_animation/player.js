function rallyPaintRobots(scale, poseCycle, poseTime, stepTime, entry)
{
    for (var i = 0; i < myRobots.robot.length; i++)
    {
        var robot = myRobots.robot[i];
        updateRobotPosition(i, poseTime, stepTime);
        drawRobot(robot, scale, poseCycle);
        drawRobotOre(robot);
        drawRobotDepot(robot);
        var isViewer = typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot;
        // Viewer panel line follows CPU-timeline highlight; peers keep pose `robot.l`.
        var sourceLine = isViewer
            ? (entry && typeof entry.l === 'number' ? entry.l : null)
            : undefined;
        updateRobotDebugPanel(robot, poseCycle, sourceLine);
    }
    updateRallyViewerSourceDebug(entry, rallyViewerRobot());
}


function rallyPrepareFrame()
{
    rallyEnsureCpuTimeline();
    var frame = rallyFrameTiming();
    rallyUpdateTransportUi(frame.completed, frame.cpuIndex, frame.poseCycle);
    return frame;
}


function renderRallyFrame()
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    var frame = rallyPrepareFrame();
    var scale = myRallyPlayer.scale;
    for (var i = 0; i < myRobots.robot.length; i++)
    {
        eraseRobot(myRobots.robot[i], scale, frame.poseCycle);
    }

    drawDepotHomes(scale, frame.poseCycle);
    rallyPaintRobots(scale, frame.poseCycle, frame.poseTime, frame.stepTime, frame.entry);
}


function redrawRallyScene()
{
    if (!rallyHasAnimationData() || typeof myGround === 'undefined')
    {
        return;
    }

    var frame = rallyPrepareFrame();
    var scale = myRallyPlayer.scale;
    drawFullGroundAt(frame.poseCycle, scale);
    drawDepotHomes(scale, frame.poseCycle);
    rallyPaintRobots(scale, frame.poseCycle, frame.poseTime, frame.stepTime, frame.entry);
}


function runanimation()
{
    if (!rallyHasAnimationData() || typeof myGround === 'undefined')
    {
        rallyBindTransportControls();
        return;
    }

    var scaleX = 600 / myGround.sizeX;
    var scaleY = 600 / myGround.sizeY;

    myRallyPlayer.scale = scaleX < scaleY ? scaleX : scaleY;
    myRallyPlayer.elapsedMs = 0;
    myRallyPlayer.playing = false;
    myRallyPlayer.finished = false;
    myRallyPlayer.speed = 1;
    myRallyPlayer.pausedCpuIndex = null;
    myRallySourceHighlightKey = null;
    myRallySourceHighlightLine = null;

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }
    expandAllRobotLocationDeltas();
    rallyRebuildCpuTimeline();

    rallyBindTransportControls();
    redrawRallyScene();
}
