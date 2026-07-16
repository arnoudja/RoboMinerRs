function rgbToHex(r, g, b) {
    return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
}


function smoothen(v1, v2, t, i)
{
    return (v1 * (i - t) + v2 * t) / i;
}


function robotColor(robotNr)
{
    switch (robotNr)
    {
        case 0:
            return '#00a000';

        case 1:
            return '#0000ff';

        case 2:
            return '#ff0000';

        case 3:
            return '#ffff00';
    }
}


function depletedRobotColor(robotNr)
{
    switch (robotNr)
    {
        case 0:
            return '#002000';

        case 1:
            return '#000050';

        case 2:
            return '#400000';

        case 3:
            return '#404000';
    }
}


var RALLY_VIEWER_HIGHLIGHT_PADDING = 4;
var RALLY_VIEWER_HIGHLIGHT_LINE_WIDTH = 3;


function robotCenterPixels(robot, scale)
{
    return {
        x: robot.x * scale + scale / 2.0,
        y: robot.y * scale + scale / 2.0
    };
}


function robotDrawRadiusPixels(robot, scale)
{
    var radius = robot.size * scale / 2.0 + 2;
    if (typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot)
    {
        radius += RALLY_VIEWER_HIGHLIGHT_PADDING + RALLY_VIEWER_HIGHLIGHT_LINE_WIDTH + 2;
    }
    return radius;
}


function drawRobot(robot, scale, turn)
{
    var center = robotCenterPixels(robot, scale);
    var centerX = center.x;
    var centerY = center.y;

    myRallyContext.beginPath();
    myRallyContext.arc(centerX, centerY, robot.size * scale / 2.0, 0, 2.0 * Math.PI, false);
    myRallyContext.fillStyle = turn < robot.maxturns ? robotColor(robot.robotnr) : depletedRobotColor(robot.robotnr);
    myRallyContext.fill();
    myRallyContext.lineWidth = 2;
    myRallyContext.strokeStyle = 'black';
    myRallyContext.stroke();

    if (typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot)
    {
        myRallyContext.beginPath();
        myRallyContext.arc(centerX, centerY, robot.size * scale / 2.0 + RALLY_VIEWER_HIGHLIGHT_PADDING, 0, 2.0 * Math.PI, false);
        myRallyContext.lineWidth = RALLY_VIEWER_HIGHLIGHT_LINE_WIDTH;
        myRallyContext.strokeStyle = '#00e5ff';
        myRallyContext.stroke();
    }

    var orientation = robot.o * Math.PI / 180.0;

    myRallyContext.beginPath();
    myRallyContext.moveTo(centerX, centerY);
    myRallyContext.lineTo(centerX + scale * robot.size * Math.cos(orientation) / 2.0, centerY + scale * robot.size * Math.sin(orientation) / 2.0);
    myRallyContext.lineWidth = 2;
    myRallyContext.strokeStyle = 'black';
    myRallyContext.stroke();
}


function eraseRobot(robot, scale, step)
{
    var center = robotCenterPixels(robot, scale);
    var radius = robotDrawRadiusPixels(robot, scale);
    var orientation = robot.o * Math.PI / 180.0;
    var lineEndX = center.x + scale * robot.size * Math.cos(orientation) / 2.0;
    var lineEndY = center.y + scale * robot.size * Math.sin(orientation) / 2.0;

    var minPxX = Math.min(center.x - radius, lineEndX) - 2;
    var maxPxX = Math.max(center.x + radius, lineEndX) + 2;
    var minPxY = Math.min(center.y - radius, lineEndY) - 2;
    var maxPxY = Math.max(center.y + radius, lineEndY) + 2;

    myRallyContext.clearRect(minPxX, minPxY, maxPxX - minPxX, maxPxY - minPxY);

    var minX = Math.floor(Math.max(0, minPxX / scale));
    var minY = Math.floor(Math.max(0, minPxY / scale));
    var maxX = Math.ceil(Math.min(myGround.sizeX, maxPxX / scale));
    var maxY = Math.ceil(Math.min(myGround.sizeY, maxPxY / scale));

    if (maxX <= minX)
    {
        maxX = minX + 1;
    }
    if (maxY <= minY)
    {
        maxY = minY + 1;
    }

    drawGroundAt(step, scale, minX, minY, maxX, maxY);
}


function drawRobotOre(robot)
{
    var i = robot.robotnr;
    var borderWidth = 3;
    var oreWidth = myOreCanvas[i].width - 2 * borderWidth;
    var oreHeight = myOreCanvas[i].height - 2 * borderWidth;
    var oreAHeight = Math.floor(robot.A * oreHeight / robot.maxore);
    var oreBHeight = Math.floor((robot.A + robot.B) * oreHeight / robot.maxore) - oreAHeight;
    var oreCHeight = Math.floor((robot.A + robot.B + robot.C) * oreHeight / robot.maxore) - oreAHeight - oreBHeight;

    myOreContext[i].beginPath();
    myOreContext[i].rect(0, 0, myOreCanvas[i].width, myOreCanvas[i].height);
    myOreContext[i].fillStyle = robotColor(robot.robotnr);
    myOreContext[i].fill();

    myOreContext[i].beginPath();
    myOreContext[i].rect(borderWidth, borderWidth, oreWidth, myOreCanvas[i].height - 2 * borderWidth);
    myOreContext[i].fillStyle = 'black';
    myOreContext[i].fill();

    myOreContext[i].beginPath();
    myOreContext[i].rect(borderWidth, myOreCanvas[i].height - borderWidth - oreAHeight, oreWidth, oreAHeight);
    myOreContext[i].fillStyle = 'red';
    myOreContext[i].fill();

    myOreContext[i].beginPath();
    myOreContext[i].rect(borderWidth, myOreCanvas[i].height - borderWidth - oreAHeight - oreBHeight, oreWidth, oreBHeight);
    myOreContext[i].fillStyle = 'green';
    myOreContext[i].fill();

    myOreContext[i].beginPath();
    myOreContext[i].rect(borderWidth, myOreCanvas[i].height - borderWidth - oreAHeight - oreBHeight - oreCHeight, oreWidth, oreCHeight);
    myOreContext[i].fillStyle = 'blue';
    myOreContext[i].fill();
}


function drawInitialGround(scale)
{
    myGround.updatedTo = 0;

    myRallyContext.beginPath();
    myRallyContext.rect(0, 0, 600, 600);
    myRallyContext.fillStyle = 'black';
    myRallyContext.fill();

    var oreAMax = typeof myOreTypes.A !== 'undefined' ? myOreTypes.A.max : 255;
    var oreBMax = typeof myOreTypes.B !== 'undefined' ? myOreTypes.B.max : 255;
    var oreCMax = typeof myOreTypes.C !== 'undefined' ? myOreTypes.C.max : 255;

    for (var i = 0; i < myGround.positions.length; i++)
    {
        myGround.positions[i].lastDrawn = 0;

        var x = myGround.positions[i].x;
        var y = myGround.positions[i].y;

        var changes = myGround.positions[i].c[0];

        var oreA = 0;
        var oreB = 0;
        var oreC = 0;

        if (typeof changes.t === 'undefined' || changes.t === 0)
        {
            oreA = typeof changes.A !== 'undefined' ? changes.A : 0;
            oreB = typeof changes.B !== 'undefined' ? changes.B : 0;
            oreC = typeof changes.C !== 'undefined' ? changes.C : 0;
        }

        var oreAIntensity = Math.min(255, Math.floor(oreA * 255 / oreAMax));
        var oreBIntensity = Math.min(255, Math.floor(oreB * 255 / oreBMax));
        var oreCIntensity = Math.min(255, Math.floor(oreC * 255 / oreCMax));

        myRallyContext.beginPath();
        myRallyContext.rect(x * scale, y * scale, scale, scale);
        myRallyContext.fillStyle = rgbToHex(oreAIntensity, oreBIntensity, oreCIntensity);
        myRallyContext.fill();
    }
}


function drawGroundAt(step, scale, fromX, fromY, tillX, tillY)
{
    var oreAMax = typeof myOreTypes.A !== 'undefined' ? myOreTypes.A.max : 255;
    var oreBMax = typeof myOreTypes.B !== 'undefined' ? myOreTypes.B.max : 255;
    var oreCMax = typeof myOreTypes.C !== 'undefined' ? myOreTypes.C.max : 255;

    myRallyContext.beginPath();
    myRallyContext.rect(fromX * scale, fromY * scale, (tillX - fromX) * scale, (tillY - fromY) * scale);
    myRallyContext.fillStyle = 'black';
    myRallyContext.fill();

    for (var i = 0; i < myGround.positions.length; i++)
    {
        if (myGround.positions[i].x >= fromX && myGround.positions[i].x < tillX &&
            myGround.positions[i].y >= fromY && myGround.positions[i].y < tillY)
        {
            var x = myGround.positions[i].x;
            var y = myGround.positions[i].y;

            var j = myGround.positions[i].lastDrawn;

            while (myGround.positions[i].c.length > (j + 1) &&
                   myGround.positions[i].c[j + 1].t <= step)
            {
                j++;
            }

            myGround.positions[i].lastDrawn = j;

            var changes = myGround.positions[i].c[j];
            var oreA = typeof changes.A !== 'undefined' ? changes.A : 0;
            var oreB = typeof changes.B !== 'undefined' ? changes.B : 0;
            var oreC = typeof changes.C !== 'undefined' ? changes.C : 0;

            var oreAIntensity = Math.min(255, Math.floor(oreA * 255 / oreAMax));
            var oreBIntensity = Math.min(255, Math.floor(oreB * 255 / oreBMax));
            var oreCIntensity = Math.min(255, Math.floor(oreC * 255 / oreCMax));

            myRallyContext.beginPath();
            myRallyContext.rect(x * scale, y * scale, scale, scale);
            myRallyContext.fillStyle = rgbToHex(oreAIntensity, oreBIntensity, oreCIntensity);
            myRallyContext.fill();
        }
    }
}


function updateRobotTo(robotNr, step)
{
    if (step > myRobots.robot[robotNr].updatedTo && step < myRobots.robot[robotNr].locations.length)
    {
        updateRobotTo(robotNr, step - 1);

        if (typeof myRobots.robot[robotNr].locations[step].x === 'undefined')
        {
            myRobots.robot[robotNr].locations[step].x = myRobots.robot[robotNr].locations[step - 1].x;
        }
        if (typeof myRobots.robot[robotNr].locations[step].y === 'undefined')
        {
            myRobots.robot[robotNr].locations[step].y = myRobots.robot[robotNr].locations[step - 1].y;
        }
        if (typeof myRobots.robot[robotNr].locations[step].o === 'undefined')
        {
            myRobots.robot[robotNr].locations[step].o = myRobots.robot[robotNr].locations[step - 1].o;
        }
        if (typeof myRobots.robot[robotNr].locations[step].A === 'undefined')
        {
            myRobots.robot[robotNr].locations[step].A = myRobots.robot[robotNr].locations[step - 1].A;
        }
        if (typeof myRobots.robot[robotNr].locations[step].B === 'undefined')
        {
            myRobots.robot[robotNr].locations[step].B = myRobots.robot[robotNr].locations[step - 1].B;
        }
        if (typeof myRobots.robot[robotNr].locations[step].C === 'undefined')
        {
            myRobots.robot[robotNr].locations[step].C = myRobots.robot[robotNr].locations[step - 1].C;
        }

        myRobots.robot[robotNr].updatedTo = step;
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        updateRobotPosition(i, time, stepTime);
        drawRobot(myRobots.robot[i], scale, cycle);
        drawRobotOre(myRobots.robot[i]);
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }

    if (typeof myGround !== 'undefined')
    {
        myGround.updatedTo = 0;
        for (var g = 0; g < myGround.positions.length; g++)
        {
            myGround.positions[g].lastDrawn = 0;
        }
    }

    drawInitialGround(myRallyPlayer.scale);
    renderRallyFrame();
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }

    if (typeof myGround !== 'undefined')
    {
        myGround.updatedTo = 0;
        for (var g = 0; g < myGround.positions.length; g++)
        {
            myGround.positions[g].lastDrawn = 0;
        }
    }

    drawInitialGround(myRallyPlayer.scale);
    renderRallyFrame();

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


function rallyBindTransportControls()
{
    var playPause = document.getElementById('rallyPlayPause');
    if (playPause)
    {
        playPause.addEventListener('click', function() {
            if (myRallyPlayer.playing)
            {
                rallyPause();
            }
            else
            {
                rallyPlay();
            }
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
            var rect = track.getBoundingClientRect();
            if (rect.width <= 0)
            {
                return;
            }
            rallySeekToRatio((event.clientX - rect.left) / rect.width);
        });
    }
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

    drawInitialGround(myRallyPlayer.scale);

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
        drawRobot(myRobots.robot[i], myRallyPlayer.scale, 0);
        drawRobotOre(myRobots.robot[i]);
    }

    rallyBindTransportControls();
    rallyUpdateTransportUi(0, 0);
    renderRallyFrame();
}
