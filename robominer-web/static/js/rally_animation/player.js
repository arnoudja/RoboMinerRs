function rallyPaintRobots(scale, poseTurn, poseTime, stepTime, entry)
{
    for (var i = 0; i < myRobots.robot.length; i++)
    {
        var robot = myRobots.robot[i];
        updateRobotPosition(i, poseTime, stepTime);
        drawRobot(robot, scale, poseTurn);
        drawRobotOre(robot);
        drawRobotDepot(robot);
        var isViewer = typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot;
        // Viewer panel line follows CPU-timeline highlight; peers keep pose `robot.l`.
        var sourceLine = isViewer
            ? (entry && typeof entry.l === 'number' ? entry.l : null)
            : undefined;
        updateRobotDebugPanel(robot, poseTurn, sourceLine);
    }
    updateRallyViewerSourceDebug(entry, rallyViewerRobot());
}


function rallyPrepareFrame()
{
    rallyEnsureCpuTimeline();
    var frame = rallyFrameTiming();
    rallyUpdateTransportUi(frame.completed, frame.cpuIndex, frame.poseTurn);
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
        eraseRobot(myRobots.robot[i], scale, frame.poseTurn);
    }

    drawDepotHomes(scale, frame.poseTurn);
    rallyPaintRobots(scale, frame.poseTurn, frame.poseTime, frame.stepTime, frame.entry);
}


function redrawRallyScene()
{
    if (!rallyHasAnimationData() || typeof myGround === 'undefined')
    {
        return;
    }

    var frame = rallyPrepareFrame();
    var scale = myRallyPlayer.scale;
    drawFullGroundAt(frame.poseTurn, scale);
    drawDepotHomes(scale, frame.poseTurn);
    rallyPaintRobots(scale, frame.poseTurn, frame.poseTime, frame.stepTime, frame.entry);
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
