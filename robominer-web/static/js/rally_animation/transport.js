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


/** True when wall-clock has reached the end of the viewer robot's timeline. */
function rallyIsPlaybackFinished()
{
    return rallyTotalTime() > 0 && myRallyPlayer.elapsedMs >= rallyTotalTime();
}


/**
 * Pause, run a seek mutation, redraw, then resume if playback was active.
 * mutateFn may set pausedCpuIndex / elapsedMs / speed; finished is always recomputed.
 */
function rallyWithPausedSeek(mutateFn, options)
{
    options = options || {};
    var wasPlaying = myRallyPlayer.playing;
    rallyStopPlayback();
    mutateFn(wasPlaying);
    myRallyPlayer.finished = rallyIsPlaybackFinished();

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
    }
    if (rallyIsPlaybackFinished())
    {
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
    // Sync wall-clock to the scrub pose first so play does not jump the sprite.
    if (myRallyPlayer.pausedCpuIndex !== null)
    {
        rallyEnsureCpuTimeline();
        var scrubFrame = rallyFrameTiming();
        myRallyPlayer.elapsedMs = scrubFrame.poseTime;
    }
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


/** Step one CPU instruction (paused arrow keys). Syncs pose to that step's motion segment. */
function rallySeekByCpuSteps(deltaSteps)
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
        var targetIndex = Math.min(totalCpu - 1, Math.max(0, currentIndex + deltaSteps));
        myRallyPlayer.pausedCpuIndex = targetIndex;

        var entry = myRallyCpuTimeline && myRallyCpuTimeline[targetIndex]
            ? myRallyCpuTimeline[targetIndex]
            : { miningCycle: targetIndex };
        myRallyPlayer.elapsedMs = (entry.miningCycle || 0) * rallyStepTime();
        // finished is set by rallyWithPausedSeek from clock end only.
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
        // Land on the first CPU step of that mining cycle when scrubbing while paused.
        myRallyPlayer.pausedCpuIndex = wasPlaying
            ? null
            : rallyFirstCpuIndexForMiningCycle(targetMining);
        // finished is set by rallyWithPausedSeek from clock end only.
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
