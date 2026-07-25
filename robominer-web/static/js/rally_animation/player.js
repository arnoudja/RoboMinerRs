function rallyUpdateTransportUi(completed, cpuIndex, miningCycle)
{
    var current = document.getElementById('rallyCycleCurrent');
    var total = document.getElementById('rallyCycleTotal');
    if (current)
    {
        current.textContent = miningCycle;
    }
    if (total)
    {
        total.textContent = Math.max(0, rallyTotalMiningCycles() - 1);
    }

    var fill = document.getElementById('rallyProgressFill');
    if (fill)
    {
        fill.style.width = (Math.min(1, Math.max(0, completed)) * 100) + '%';
    }

    var track = document.getElementById('rallyProgressTrack');
    if (track)
    {
        var totalMining = Math.max(0, rallyTotalMiningCycles());
        var totalCpu = Math.max(0, rallyTotalCpuSteps());
        var currentCpu = Math.min(totalCpu, Math.max(0, Math.floor(cpuIndex)));
        var maxMining = Math.max(0, totalMining - 1);
        track.setAttribute('aria-valuemin', '0');
        track.setAttribute('aria-valuemax', String(maxMining));
        track.setAttribute('aria-valuenow', String(miningCycle));
        track.setAttribute(
            'aria-valuetext',
            'Area cycle ' + miningCycle + ' of ' + maxMining +
                (totalCpu > 0 ? (', CPU ' + currentCpu + ' of ' + Math.max(0, totalCpu - 1)) : '')
        );
    }

    var playPause = document.getElementById('rallyPlayPause');
    if (playPause)
    {
        if (myRallyPlayer.playing)
        {
            playPause.textContent = 'Pause';
        }
        else if (myRallyPlayer.finished)
        {
            playPause.textContent = 'Replay';
        }
        else
        {
            playPause.textContent = 'Play';
        }
    }
}


function rallyPaintRobots(scale, cycle, poseTime, stepTime, entry)
{
    for (var i = 0; i < myRobots.robot.length; i++)
    {
        updateRobotPosition(i, poseTime, stepTime);
        drawRobot(myRobots.robot[i], scale, cycle);
        drawRobotOre(myRobots.robot[i]);
        drawRobotDepot(myRobots.robot[i]);
        updateRobotDebugPanel(myRobots.robot[i], cycle, entry);
    }
}


function renderRallyFrame()
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    rallyEnsureCpuTimeline();
    var frame = rallyFrameTiming();
    rallyUpdateTransportUi(frame.completed, frame.cpuIndex, frame.cycle);

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

    rallyEnsureCpuTimeline();
    var frame = rallyFrameTiming();
    rallyUpdateTransportUi(frame.completed, frame.cpuIndex, frame.cycle);

    var scale = myRallyPlayer.scale;
    drawFullGroundAt(frame.cycle, scale);
    drawDepotHomes(scale, frame.cycle);
    rallyPaintRobots(scale, frame.cycle, frame.poseTime, frame.stepTime, frame.entry);
}


function rallyStopPlayback()
{
    myRallyPlayer.playing = false;
    if (myRallyPlayer.frameId !== null)
    {
        cancelAnimationFrame(myRallyPlayer.frameId);
        myRallyPlayer.frameId = null;
    }
    myRallyPlayer.lastFrameTime = null;
}


/**
 * Pause, run a seek mutation, redraw, then resume if playback was active.
 * mutateFn may set finished / pausedCpuIndex / elapsedMs / speed.
 */
function rallyWithPausedSeek(mutateFn, options)
{
    options = options || {};
    var wasPlaying = myRallyPlayer.playing;
    rallyStopPlayback();
    mutateFn(wasPlaying);

    if (options.fullRedraw)
    {
        redrawRallyScene();
    }
    else
    {
        renderRallyFrame();
    }

    if (wasPlaying && !myRallyPlayer.finished)
    {
        rallyPlay();
    }
}


function rallyAnimationLoop(timestamp)
{
    if (!myRallyPlayer.playing)
    {
        return;
    }

    if (myRallyPlayer.lastFrameTime === null)
    {
        myRallyPlayer.lastFrameTime = timestamp;
    }

    var delta = timestamp - myRallyPlayer.lastFrameTime;
    myRallyPlayer.lastFrameTime = timestamp;
    myRallyPlayer.elapsedMs += delta;

    if (myRallyPlayer.elapsedMs >= rallyTotalTime())
    {
        myRallyPlayer.elapsedMs = rallyTotalTime();
        myRallyPlayer.playing = false;
        myRallyPlayer.finished = true;
    }

    renderRallyFrame();

    if (myRallyPlayer.playing)
    {
        myRallyPlayer.frameId = requestAnimFrame(rallyAnimationLoop);
    }
    else
    {
        myRallyPlayer.frameId = null;
        myRallyPlayer.lastFrameTime = null;
    }
}


function rallyPause()
{
    rallyStopPlayback();
    renderRallyFrame();
}


function rallyPlay()
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    if (myRallyPlayer.finished)
    {
        rallyRestart();
    }

    // Leave CPU scrub mode so pose interpolates smoothly across mining cycles.
    myRallyPlayer.pausedCpuIndex = null;
    myRallyPlayer.playing = true;
    myRallyPlayer.lastFrameTime = null;
    if (myRallyPlayer.frameId !== null)
    {
        cancelAnimationFrame(myRallyPlayer.frameId);
    }
    myRallyPlayer.frameId = requestAnimFrame(rallyAnimationLoop);
}


function rallyRestart()
{
    rallyStopPlayback();
    myRallyPlayer.elapsedMs = 0;
    myRallyPlayer.finished = false;
    myRallyPlayer.pausedCpuIndex = null;

    if (!rallyHasAnimationData())
    {
        return;
    }

    // Keep expanded location deltas; redraw the scene at cycle 0.
    redrawRallyScene();
}


function rallySeekToRatio(ratio)
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    rallyWithPausedSeek(function() {
        ratio = Math.min(1, Math.max(0, ratio));
        myRallyPlayer.elapsedMs = ratio * rallyTotalTime();
        myRallyPlayer.finished = myRallyPlayer.elapsedMs >= rallyTotalTime();
        // Slider seeks on the mining-cycle clock; clear CPU scrub so pose matches time.
        myRallyPlayer.pausedCpuIndex = null;
    }, { fullRedraw: true });
}


function rallySetSpeed(speed)
{
    var fraction = 0;
    if (rallyHasAnimationData() && rallyTotalTime() > 0)
    {
        fraction = myRallyPlayer.elapsedMs / rallyTotalTime();
    }

    rallyWithPausedSeek(function() {
        myRallyPlayer.speed = speed;
        myRallyPlayer.elapsedMs = fraction * rallyTotalTime();
        myRallyPlayer.finished = myRallyPlayer.elapsedMs >= rallyTotalTime();

        var speedButtons = document.querySelectorAll('.rally-view-speed-button');
        for (var b = 0; b < speedButtons.length; b++)
        {
            var button = speedButtons[b];
            if (Number(button.getAttribute('data-speed')) === speed)
            {
                button.classList.add('rally-view-speed-button-active');
            }
            else
            {
                button.classList.remove('rally-view-speed-button-active');
            }
        }
    }, { fullRedraw: false });
}


/** Step one CPU instruction (paused arrow keys). Syncs pose to that mining cycle. */
function rallySeekByCycles(deltaCycles)
{
    if (!rallyHasAnimationData())
    {
        return;
    }
    rallyEnsureCpuTimeline();

    rallyWithPausedSeek(function() {
        var totalCpu = rallyTotalCpuSteps();
        if (totalCpu <= 0)
        {
            return;
        }

        var currentIndex = myRallyPlayer.pausedCpuIndex;
        if (currentIndex === null)
        {
            currentIndex = rallyCpuIndexAtTime(myRallyPlayer.elapsedMs);
        }
        var targetIndex = Math.min(totalCpu - 1, Math.max(0, currentIndex + deltaCycles));
        myRallyPlayer.pausedCpuIndex = targetIndex;

        var entry = myRallyCpuTimeline && myRallyCpuTimeline[targetIndex]
            ? myRallyCpuTimeline[targetIndex]
            : { miningCycle: targetIndex };
        myRallyPlayer.elapsedMs = (entry.miningCycle || 0) * rallyStepTime();
        myRallyPlayer.finished = myRallyPlayer.elapsedMs >= rallyTotalTime() &&
            targetIndex >= totalCpu - 1;
    }, { fullRedraw: true });
}


function rallySeekByMiningCycles(deltaMiningCycles)
{
    if (!rallyHasAnimationData())
    {
        return;
    }
    rallyEnsureCpuTimeline();

    rallyWithPausedSeek(function(wasPlaying) {
        var maxMining = Math.max(0, rallyTotalMiningCycles() - 1);
        var currentMining = rallyMiningCycleAtTime(myRallyPlayer.elapsedMs);
        if (myRallyPlayer.pausedCpuIndex !== null && myRallyCpuTimeline &&
            myRallyCpuTimeline[myRallyPlayer.pausedCpuIndex])
        {
            currentMining = myRallyCpuTimeline[myRallyPlayer.pausedCpuIndex].miningCycle;
        }

        var targetMining = Math.min(maxMining, Math.max(0, currentMining + deltaMiningCycles));
        myRallyPlayer.elapsedMs = targetMining * rallyStepTime();
        myRallyPlayer.finished = targetMining >= maxMining &&
            myRallyPlayer.elapsedMs >= rallyTotalTime();
        // Land on the first CPU step of that mining cycle when scrubbing while paused.
        myRallyPlayer.pausedCpuIndex = wasPlaying
            ? null
            : rallyFirstCpuIndexForMiningCycle(targetMining);
    }, { fullRedraw: true });
}


function rallyTogglePlayPause()
{
    if (myRallyPlayer.playing)
    {
        rallyPause();
    }
    else
    {
        rallyPlay();
    }
}


function rallyIsTypingTarget(target)
{
    if (!target || !target.tagName)
    {
        return false;
    }

    var tag = target.tagName.toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select')
    {
        return true;
    }

    return !!target.isContentEditable;
}


function rallyBindKeyboardControls()
{
    if (window.__rallyKeyboardBound)
    {
        return;
    }
    window.__rallyKeyboardBound = true;

    document.addEventListener('keydown', function(event) {
        if (!rallyHasAnimationData())
        {
            return;
        }
        if (rallyIsTypingTarget(event.target))
        {
            return;
        }
        if (event.altKey || event.ctrlKey || event.metaKey)
        {
            return;
        }

        var key = event.key;
        if (key === ' ' || key === 'Spacebar')
        {
            // Let focused control buttons keep native Space activation, but treat
            // the seek slider as play/pause (keyboard click coords are unreliable).
            var onControl = event.target && event.target.closest
                && event.target.closest('button, a, [role="button"]');
            var onSeekSlider = event.target && event.target.id === 'rallyProgressTrack';
            if (onControl && !onSeekSlider)
            {
                return;
            }
            event.preventDefault();
            rallyTogglePlayPause();
            return;
        }

        if (key === 'ArrowLeft')
        {
            event.preventDefault();
            if (event.shiftKey || myRallyPlayer.playing)
            {
                rallySeekByMiningCycles(-1);
            }
            else
            {
                rallySeekByCycles(-1);
            }
            return;
        }

        if (key === 'ArrowRight')
        {
            event.preventDefault();
            if (event.shiftKey || myRallyPlayer.playing)
            {
                rallySeekByMiningCycles(1);
            }
            else
            {
                rallySeekByCycles(1);
            }
            return;
        }

        if (key === 'Home')
        {
            event.preventDefault();
            rallySeekToRatio(0);
            return;
        }

        if (key === 'End')
        {
            event.preventDefault();
            rallySeekToRatio(1);
        }
    });
}


function rallyBindTransportControls()
{
    var playPause = document.getElementById('rallyPlayPause');
    if (playPause)
    {
        playPause.addEventListener('click', function() {
            rallyTogglePlayPause();
        });
    }

    var restart = document.getElementById('rallyRestart');
    if (restart)
    {
        restart.addEventListener('click', rallyRestart);
    }

    var speedButtons = document.querySelectorAll('.rally-view-speed-button');
    for (var i = 0; i < speedButtons.length; i++)
    {
        speedButtons[i].addEventListener('click', function(event) {
            var speed = Number(event.currentTarget.getAttribute('data-speed'));
            if (speed > 0)
            {
                rallySetSpeed(speed);
            }
        });
    }

    var track = document.getElementById('rallyProgressTrack');
    if (track)
    {
        track.addEventListener('click', function(event) {
            // Keyboard-activated clicks have detail 0 and unusable coordinates.
            if (event.detail === 0)
            {
                return;
            }
            var rect = track.getBoundingClientRect();
            if (rect.width <= 0)
            {
                return;
            }
            rallySeekToRatio((event.clientX - rect.left) / rect.width);
        });
    }

    rallyBindKeyboardControls();
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }
    expandAllRobotLocationDeltas();
    rallyRebuildCpuTimeline();

    rallyBindTransportControls();
    redrawRallyScene();
}
