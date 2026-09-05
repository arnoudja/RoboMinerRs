function rallyPaintRobots(scale, poseTurn, poseTime, stepTime, entry)
{
    for (let i = 0; i < myRobots.robot.length; i++)
    {
        const robot = myRobots.robot[i];
        updateRobotPosition(i, poseTime, stepTime);
        drawRobot(robot, scale, poseTurn);
        drawRobotOre(robot);
        drawRobotDepot(robot);
        const isViewer = typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot;
        const debugEntry = isViewer ? rallyEntryForViewerDebug(entry, poseTurn) : entry;
        // Viewer panel line follows CPU-timeline highlight; peers keep pose `robot.l`.
        const sourceLine = isViewer
            ? (debugEntry && typeof debugEntry.l === 'number' ? debugEntry.l : null)
            : undefined;
        updateRobotDebugPanel(robot, poseTurn, sourceLine);
    }
    updateRallyViewerSourceDebug(rallyEntryForViewerDebug(entry, poseTurn), rallyViewerRobot());
}


function rallyPrepareFrame()
{
    rallyEnsureCpuTimeline();
    const frame = rallyFrameTiming();
    rallyUpdateTransportUi(frame.completed, frame.cpuIndex, frame.poseTurn, frame.entry);
    return frame;
}


function renderRallyFrame()
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    const frame = rallyPrepareFrame();
    const scale = myRallyPlayer.scale;
    for (let i = 0; i < myRobots.robot.length; i++)
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

    const frame = rallyPrepareFrame();
    const scale = myRallyPlayer.scale;
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

    const scaleX = 600 / myGround.sizeX;
    const scaleY = 600 / myGround.sizeY;

    myRallyPlayer.scale = scaleX < scaleY ? scaleX : scaleY;
    myRallyPlayer.elapsedMs = 0;
    myRallyPlayer.playing = false;
    myRallyPlayer.finished = false;
    myRallyPlayer.speed = 1;
    myRallyPlayer.pausedCpuIndex = null;
    myRallySourceHighlightKey = null;
    myRallySourceHighlightLine = null;

    for (let i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }
    expandAllRobotLocationDeltas();
    rallyRebuildCpuTimeline();

    rallyBindTransportControls();
    redrawRallyScene();
}
