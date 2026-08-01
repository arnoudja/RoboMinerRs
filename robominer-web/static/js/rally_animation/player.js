function rallyPaintRobots(scale, cycle, poseTime, stepTime, entry)
{
    for (var i = 0; i < myRobots.robot.length; i++)
    {
        updateRobotPosition(i, poseTime, stepTime);
        drawRobot(myRobots.robot[i], scale, cycle);
        drawRobotOre(myRobots.robot[i]);
        drawRobotDepot(myRobots.robot[i]);
        updateRobotDebugPanel(myRobots.robot[i], cycle);
    }
    updateRallyViewerSourceDebug(entry, rallyViewerRobot());
}


function rallyPrepareFrame()
{
    rallyEnsureCpuTimeline();
    var frame = rallyFrameTiming();
    rallyUpdateTransportUi(frame.completed, frame.cpuIndex, frame.cycle);
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
        eraseRobot(myRobots.robot[i], scale, frame.cycle);
    }

    drawDepotHomes(scale, frame.cycle);
    rallyPaintRobots(scale, frame.cycle, frame.poseTime, frame.stepTime, frame.entry);
}


function redrawRallyScene()
{
    if (!rallyHasAnimationData() || typeof myGround === 'undefined')
    {
        return;
    }

    var frame = rallyPrepareFrame();
    var scale = myRallyPlayer.scale;
    drawFullGroundAt(frame.cycle, scale);
    drawDepotHomes(scale, frame.cycle);
    rallyPaintRobots(scale, frame.cycle, frame.poseTime, frame.stepTime, frame.entry);
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
