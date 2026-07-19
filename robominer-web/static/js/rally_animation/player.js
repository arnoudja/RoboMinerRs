function updateRobotTo(robotNr, step)
{
    var robot = myRobots.robot[robotNr];
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
        var current = robot.locations[s];
        var previous = robot.locations[s - 1];

        if (typeof current.x === 'undefined')
        {
            current.x = previous.x;
        }
        if (typeof current.y === 'undefined')
        {
            current.y = previous.y;
        }
        if (typeof current.o === 'undefined')
        {
            current.o = previous.o;
        }
        if (typeof current.A === 'undefined')
        {
            current.A = previous.A;
        }
        if (typeof current.B === 'undefined')
        {
            current.B = previous.B;
        }
        if (typeof current.C === 'undefined')
        {
            current.C = previous.C;
        }
        if (typeof current.DA === 'undefined' && typeof previous.DA !== 'undefined')
        {
            current.DA = previous.DA;
        }
        if (typeof current.DB === 'undefined' && typeof previous.DB !== 'undefined')
        {
            current.DB = previous.DB;
        }
        if (typeof current.DC === 'undefined' && typeof previous.DC !== 'undefined')
        {
            current.DC = previous.DC;
        }
        if (typeof current.a === 'undefined' && typeof previous.a !== 'undefined')
        {
            current.a = previous.a;
        }
        if (typeof current.l === 'undefined' && typeof previous.l !== 'undefined')
        {
            current.l = previous.l;
        }
        // Do not fill-forward `s`: productive cycles omit it intentionally.

        robot.updatedTo = s;
    }
}


function updateRobotPosition(robotNr, time, stepTime)
{
    var t1 = Math.floor(time / stepTime);
    var t2 = t1 + 1;

    updateRobotTo(robotNr, t2);

    if (t2 >= myRobots.robot[robotNr].locations.length)
    {
        t1 = myRobots.robot[robotNr].locations.length - 1;
        myRobots.robot[robotNr].x = myRobots.robot[robotNr].locations[t1].x;
        myRobots.robot[robotNr].y = myRobots.robot[robotNr].locations[t1].y;
        myRobots.robot[robotNr].o = myRobots.robot[robotNr].locations[t1].o;
        myRobots.robot[robotNr].A = myRobots.robot[robotNr].locations[t1].A;
        myRobots.robot[robotNr].B = myRobots.robot[robotNr].locations[t1].B;
        myRobots.robot[robotNr].C = myRobots.robot[robotNr].locations[t1].C;
        myRobots.robot[robotNr].DA = myRobots.robot[robotNr].locations[t1].DA;
        myRobots.robot[robotNr].DB = myRobots.robot[robotNr].locations[t1].DB;
        myRobots.robot[robotNr].DC = myRobots.robot[robotNr].locations[t1].DC;
        myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t1].a;
        myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t1].l;
        myRobots.robot[robotNr].s = myRobots.robot[robotNr].locations[t1].s;
    }
    else
    {
        var dt = time % stepTime;
        var timeFraction = typeof myRobots.robot[robotNr].locations[t2].t !== 'undefined' ? myRobots.robot[robotNr].locations[t2].t : 1.0;

        if (dt >= stepTime * timeFraction)
        {
            myRobots.robot[robotNr].x = myRobots.robot[robotNr].locations[t2].x;
            myRobots.robot[robotNr].y = myRobots.robot[robotNr].locations[t2].y;
            myRobots.robot[robotNr].o = myRobots.robot[robotNr].locations[t2].o;
            myRobots.robot[robotNr].A = myRobots.robot[robotNr].locations[t2].A;
            myRobots.robot[robotNr].B = myRobots.robot[robotNr].locations[t2].B;
            myRobots.robot[robotNr].C = myRobots.robot[robotNr].locations[t2].C;
            myRobots.robot[robotNr].DA = myRobots.robot[robotNr].locations[t2].DA;
            myRobots.robot[robotNr].DB = myRobots.robot[robotNr].locations[t2].DB;
            myRobots.robot[robotNr].DC = myRobots.robot[robotNr].locations[t2].DC;
            myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t2].a;
            myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t2].l;
            myRobots.robot[robotNr].s = myRobots.robot[robotNr].locations[t2].s;
        }
        else
        {
            var travelTime = stepTime * timeFraction;
            myRobots.robot[robotNr].x = smoothen(myRobots.robot[robotNr].locations[t1].x, myRobots.robot[robotNr].locations[t2].x, dt, travelTime);
            myRobots.robot[robotNr].y = smoothen(myRobots.robot[robotNr].locations[t1].y, myRobots.robot[robotNr].locations[t2].y, dt, travelTime);
            myRobots.robot[robotNr].o = myRobots.robot[robotNr].locations[t1].o;
            myRobots.robot[robotNr].A = smoothen(myRobots.robot[robotNr].locations[t1].A, myRobots.robot[robotNr].locations[t2].A, dt, travelTime);
            myRobots.robot[robotNr].B = smoothen(myRobots.robot[robotNr].locations[t1].B, myRobots.robot[robotNr].locations[t2].B, dt, travelTime);
            myRobots.robot[robotNr].C = smoothen(myRobots.robot[robotNr].locations[t1].C, myRobots.robot[robotNr].locations[t2].C, dt, travelTime);
            myRobots.robot[robotNr].DA = smoothen(
                typeof myRobots.robot[robotNr].locations[t1].DA !== 'undefined' ? myRobots.robot[robotNr].locations[t1].DA : 0,
                typeof myRobots.robot[robotNr].locations[t2].DA !== 'undefined' ? myRobots.robot[robotNr].locations[t2].DA : 0,
                dt,
                travelTime
            );
            myRobots.robot[robotNr].DB = smoothen(
                typeof myRobots.robot[robotNr].locations[t1].DB !== 'undefined' ? myRobots.robot[robotNr].locations[t1].DB : 0,
                typeof myRobots.robot[robotNr].locations[t2].DB !== 'undefined' ? myRobots.robot[robotNr].locations[t2].DB : 0,
                dt,
                travelTime
            );
            myRobots.robot[robotNr].DC = smoothen(
                typeof myRobots.robot[robotNr].locations[t1].DC !== 'undefined' ? myRobots.robot[robotNr].locations[t1].DC : 0,
                typeof myRobots.robot[robotNr].locations[t2].DC !== 'undefined' ? myRobots.robot[robotNr].locations[t2].DC : 0,
                dt,
                travelTime
            );
            // At the exact start of a segment (including replay start), prefer t1's line so
            // locations[0].l (program entry) is shown instead of jumping to the first action cycle.
            if (dt <= 0 && typeof myRobots.robot[robotNr].locations[t1].l !== 'undefined')
            {
                myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t1].a;
                myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t1].l;
                myRobots.robot[robotNr].s = myRobots.robot[robotNr].locations[t1].s;
            }
            else if (dt <= 0 && t1 === 0)
            {
                // Legacy replays omit locations[0].l — still show program entry.
                myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t1].a;
                myRobots.robot[robotNr].l = 1;
                myRobots.robot[robotNr].s = myRobots.robot[robotNr].locations[t1].s;
            }
            else
            {
                myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t2].a;
                myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t2].l;
                myRobots.robot[robotNr].s = myRobots.robot[robotNr].locations[t2].s;
            }
        }
    }
}


var myRallyPlayer = {
    scale: 1,
    baseStepTime: 50,
    speed: 1,
    playing: false,
    finished: false,
    elapsedMs: 0,
    frameId: null,
    lastFrameTime: null
};


function rallyHasAnimationData()
{
    return typeof myRobots !== 'undefined' &&
        myRobots.robot &&
        myRobots.robot.length > 0 &&
        myRobots.robot[0].locations;
}


function rallyTotalSteps()
{
    if (!rallyHasAnimationData())
    {
        return 0;
    }
    return myRobots.robot[0].locations.length;
}


function rallyStepTime()
{
    return myRallyPlayer.baseStepTime / myRallyPlayer.speed;
}


function rallyTotalTime()
{
    return rallyTotalSteps() * rallyStepTime();
}


function rallyUpdateTransportUi(completed, cycle)
{
    if (typeof myCycleText !== 'undefined' && myCycleText)
    {
        myCycleText.value = cycle;
    }

    var current = document.getElementById('rallyCycleCurrent');
    var total = document.getElementById('rallyCycleTotal');
    if (current)
    {
        current.textContent = cycle;
    }
    if (total)
    {
        total.textContent = rallyTotalSteps();
    }

    var fill = document.getElementById('rallyProgressFill');
    if (fill)
    {
        fill.style.width = (Math.min(1, Math.max(0, completed)) * 100) + '%';
    }

    var track = document.getElementById('rallyProgressTrack');
    if (track)
    {
        var totalCycles = Math.max(0, rallyTotalSteps());
        var currentCycle = Math.min(totalCycles, Math.max(0, Math.floor(cycle)));
        track.setAttribute('aria-valuemin', '0');
        track.setAttribute('aria-valuemax', String(totalCycles));
        track.setAttribute('aria-valuenow', String(currentCycle));
        track.setAttribute('aria-valuetext', 'Cycle ' + currentCycle + ' of ' + totalCycles);
    }

    if (typeof myProgressContext !== 'undefined' && myProgressContext && typeof myProgressCanvas !== 'undefined' && myProgressCanvas)
    {
        myProgressContext.clearRect(0, 0, myProgressCanvas.width, myProgressCanvas.height);
        myProgressContext.beginPath();
        myProgressContext.rect(0, 0, myProgressCanvas.width * completed, 20);
        myProgressContext.fillStyle = '#00e5ff';
        myProgressContext.fill();
        myProgressContext.lineWidth = 1;
        myProgressContext.strokeStyle = 'black';
        myProgressContext.stroke();
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


function renderRallyFrame()
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    var time = myRallyPlayer.elapsedMs;
    var stepTime = rallyStepTime();
    var totalSteps = rallyTotalSteps();
    var totalTime = rallyTotalTime();
    var completed = totalTime > 0 ? time / totalTime : 0;
    if (completed > 1)
    {
        completed = 1;
    }

    var cycle = Math.floor(time / stepTime);
    if (cycle > totalSteps)
    {
        cycle = totalSteps;
    }

    rallyUpdateTransportUi(completed, cycle);

    var scale = myRallyPlayer.scale;
    for (var i = 0; i < myRobots.robot.length; i++)
    {
        eraseRobot(myRobots.robot[i], scale, cycle);
    }

    drawDepotHomes(scale, cycle);

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        updateRobotPosition(i, time, stepTime);
        drawRobot(myRobots.robot[i], scale, cycle);
        drawRobotOre(myRobots.robot[i]);
        drawRobotDepot(myRobots.robot[i]);
        updateRobotDebugPanel(myRobots.robot[i], cycle);
    }
}


function redrawRallyScene()
{
    if (!rallyHasAnimationData() || typeof myGround === 'undefined')
    {
        return;
    }

    var time = myRallyPlayer.elapsedMs;
    var stepTime = rallyStepTime();
    var totalSteps = rallyTotalSteps();
    var totalTime = rallyTotalTime();
    var completed = totalTime > 0 ? time / totalTime : 0;
    if (completed > 1)
    {
        completed = 1;
    }

    var cycle = Math.floor(time / stepTime);
    if (cycle > totalSteps)
    {
        cycle = totalSteps;
    }

    rallyUpdateTransportUi(completed, cycle);

    var scale = myRallyPlayer.scale;
    drawFullGroundAt(cycle, scale);
    drawDepotHomes(scale, cycle);

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        updateRobotPosition(i, time, stepTime);
        drawRobot(myRobots.robot[i], scale, cycle);
        drawRobotOre(myRobots.robot[i]);
        drawRobotDepot(myRobots.robot[i]);
        updateRobotDebugPanel(myRobots.robot[i], cycle);
    }
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
    myRallyPlayer.playing = false;
    if (myRallyPlayer.frameId !== null)
    {
        cancelAnimationFrame(myRallyPlayer.frameId);
        myRallyPlayer.frameId = null;
    }
    myRallyPlayer.lastFrameTime = null;

    var totalTime = rallyTotalTime();
    var completed = totalTime > 0 ? myRallyPlayer.elapsedMs / totalTime : 0;
    var cycle = Math.min(rallyTotalSteps(), Math.floor(myRallyPlayer.elapsedMs / rallyStepTime()));
    rallyUpdateTransportUi(completed, cycle);
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
    rallyPause();
    myRallyPlayer.elapsedMs = 0;
    myRallyPlayer.finished = false;

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

    var wasPlaying = myRallyPlayer.playing;
    rallyPause();
    ratio = Math.min(1, Math.max(0, ratio));
    myRallyPlayer.elapsedMs = ratio * rallyTotalTime();
    myRallyPlayer.finished = myRallyPlayer.elapsedMs >= rallyTotalTime();

    // Avoid resetting filled deltas / ground cursors: expand only what is still
    // missing, then paint the full frame at the seek target.
    redrawRallyScene();

    if (wasPlaying && !myRallyPlayer.finished)
    {
        rallyPlay();
    }
}


function rallySetSpeed(speed)
{
    var fraction = 0;
    if (rallyHasAnimationData() && rallyTotalTime() > 0)
    {
        fraction = myRallyPlayer.elapsedMs / rallyTotalTime();
    }

    var wasPlaying = myRallyPlayer.playing;
    rallyPause();
    myRallyPlayer.speed = speed;
    myRallyPlayer.elapsedMs = fraction * rallyTotalTime();
    myRallyPlayer.finished = myRallyPlayer.elapsedMs >= rallyTotalTime();
    renderRallyFrame();

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

    if (wasPlaying && !myRallyPlayer.finished)
    {
        rallyPlay();
    }
}


function rallySeekByCycles(deltaCycles)
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    var totalTime = rallyTotalTime();
    var stepTime = rallyStepTime();
    if (totalTime <= 0 || stepTime <= 0)
    {
        return;
    }

    rallySeekToRatio((myRallyPlayer.elapsedMs + deltaCycles * stepTime) / totalTime);
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
            rallySeekByCycles(event.shiftKey ? -10 : -1);
            return;
        }

        if (key === 'ArrowRight')
        {
            event.preventDefault();
            rallySeekByCycles(event.shiftKey ? 10 : 1);
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }
    expandAllRobotLocationDeltas();

    rallyBindTransportControls();
    redrawRallyScene();
}
