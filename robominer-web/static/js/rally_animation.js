function rgbToHex(r, g, b) {
    return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
}


/**
 * Load a versioned rally animation payload into the viewer globals.
 * Payload shape (v1): { v, robots, ground, oreTypes }.
 * Legacy executable `var myRobots = …` rows are rejected by the page and never injected.
 */
function applyRallyResultPayload(payload)
{
    if (!payload || payload.v !== 1)
    {
        throw new Error('Unsupported rally result payload version');
    }

    myRobots = payload.robots;
    myGround = payload.ground;
    myOreTypes = payload.oreTypes || {};
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


var RALLY_ACTION_NAMES = {
    0: 'Scan',
    1: 'Wait',
    2: 'Forward',
    3: 'Backward',
    4: 'Rotate R',
    5: 'Rotate L',
    6: 'Mine',
    7: 'Dump'
};


function rallyActionName(actionIndex)
{
    if (typeof actionIndex !== 'number' || isNaN(actionIndex))
    {
        return null;
    }

    return RALLY_ACTION_NAMES[actionIndex] || ('Action ' + actionIndex);
}


function robotLooksIdle(robot, step)
{
    if (typeof robot.a === 'number')
    {
        // Wait only — Scan (0) is productive work, not idle.
        return robot.a === 1;
    }

    if (!robot.locations || step <= 0 || step >= robot.locations.length)
    {
        return false;
    }

    var previous = robot.locations[step - 1];
    var current = robot.locations[step];
    return previous.x === current.x
        && previous.y === current.y
        && previous.o === current.o
        && previous.A === current.A
        && previous.B === current.B
        && previous.C === current.C;
}


function robotCargoFull(robot)
{
    return Math.round(robot.A) + Math.round(robot.B) + Math.round(robot.C) >= robot.maxore;
}


function robotTurnsRemaining(robot, step)
{
    if (typeof robot.maxturns !== 'number' || isNaN(robot.maxturns))
    {
        return null;
    }

    var remaining = Math.floor(robot.maxturns) - Math.floor(step);
    if (remaining < 0)
    {
        remaining = 0;
    }
    return remaining;
}


function updateRobotDebugPanel(robot, step)
{
    var turnsEl = document.getElementById('robotTurns' + robot.robotnr);
    var batteryEl = document.getElementById('robotBattery' + robot.robotnr);
    var batteryFillEl = document.getElementById('robotBatteryFill' + robot.robotnr);
    var remainingTurns = robotTurnsRemaining(robot, step);
    var depleted = remainingTurns === 0;
    var maxTurns = typeof robot.maxturns === 'number' && !isNaN(robot.maxturns)
        ? Math.floor(robot.maxturns)
        : 0;
    var ratio = 0;
    if (remainingTurns !== null && maxTurns > 0)
    {
        ratio = remainingTurns / maxTurns;
    }
    if (turnsEl)
    {
        if (remainingTurns === null || maxTurns <= 0)
        {
            turnsEl.textContent = '—';
        }
        else
        {
            turnsEl.textContent = remainingTurns + ' / ' + maxTurns
                + (depleted ? ' OUT' : '');
        }
    }
    if (batteryFillEl)
    {
        batteryFillEl.style.width = (ratio * 100) + '%';
    }
    if (batteryEl)
    {
        if (remainingTurns === null || maxTurns <= 0)
        {
            batteryEl.setAttribute('aria-valuemax', '0');
            batteryEl.setAttribute('aria-valuenow', '0');
            batteryEl.classList.remove('rally-view-player-battery-low');
        }
        else
        {
            batteryEl.setAttribute('aria-valuemax', String(maxTurns));
            batteryEl.setAttribute('aria-valuenow', String(remainingTurns));
            if (ratio > 0 && ratio <= 0.2)
            {
                batteryEl.classList.add('rally-view-player-battery-low');
            }
            else
            {
                batteryEl.classList.remove('rally-view-player-battery-low');
            }
        }
    }

    var cargoEl = document.getElementById('robotCargo' + robot.robotnr);
    var total = Math.round(robot.A) + Math.round(robot.B) + Math.round(robot.C);
    var full = robotCargoFull(robot);
    if (cargoEl)
    {
        cargoEl.textContent = 'A ' + Math.round(robot.A)
            + ' · B ' + Math.round(robot.B)
            + ' · C ' + Math.round(robot.C)
            + '  (' + total + '/' + robot.maxore + ')'
            + (full ? ' FULL' : '');
    }

    var actionEl = document.getElementById('robotAction' + robot.robotnr);
    var actionName = rallyActionName(robot.a);
    if (actionEl)
    {
        var label = null;
        if (actionName)
        {
            label = actionName;
        }
        else if (robotLooksIdle(robot, step))
        {
            label = 'Idle';
        }

        if (label && typeof robot.l === 'number')
        {
            label += ' · L' + robot.l;
        }

        actionEl.textContent = label || '—';
    }

    var card = document.getElementById('rallyPlayer' + robot.robotnr);
    if (card)
    {
        if (robotLooksIdle(robot, step))
        {
            card.classList.add('rally-view-player-idle');
        }
        else
        {
            card.classList.remove('rally-view-player-idle');
        }

        if (full)
        {
            card.classList.add('rally-view-player-full');
        }
        else
        {
            card.classList.remove('rally-view-player-full');
        }

        if (depleted)
        {
            card.classList.add('rally-view-player-depleted');
        }
        else
        {
            card.classList.remove('rally-view-player-depleted');
        }
    }

    if (typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot)
    {
        updateRallySourceHighlight(robot.l);
        updateRallyEditCodeLink(robot.l);
    }
}


var myRallySourceHighlightLine = null;


function updateRallyEditCodeLink(line)
{
    var link = document.getElementById('rallyEditCodeLink');
    if (!link)
    {
        return;
    }

    var baseHref = link.getAttribute('data-edit-href');
    if (!baseHref)
    {
        return;
    }

    if (typeof line === 'number' && !isNaN(line) && line >= 1)
    {
        link.href = baseHref + '&line=' + encodeURIComponent(String(Math.floor(line)));
    }
    else
    {
        link.href = baseHref;
    }
}


function updateRallySourceHighlight(line)
{
    var sourceCode = document.getElementById('rallySourceCode');
    if (!sourceCode)
    {
        return;
    }

    if (myRallySourceHighlightLine !== null)
    {
        var previous = document.getElementById('rallySourceLine' + myRallySourceHighlightLine);
        if (previous)
        {
            previous.classList.remove('rally-view-source-line-active');
        }
        myRallySourceHighlightLine = null;
    }

    if (typeof line !== 'number' || isNaN(line) || line < 1)
    {
        return;
    }

    var current = document.getElementById('rallySourceLine' + line);
    if (!current)
    {
        return;
    }

    current.classList.add('rally-view-source-line-active');
    myRallySourceHighlightLine = line;
    scrollRallySourceLineIntoView(sourceCode, current);
}


function scrollRallySourceLineIntoView(container, lineEl)
{
    var containerRect = container.getBoundingClientRect();
    var lineRect = lineEl.getBoundingClientRect();
    var above = lineRect.top - containerRect.top;
    var below = lineRect.bottom - containerRect.bottom;

    if (above < 0)
    {
        container.scrollTop += above;
    }
    else if (below > 0)
    {
        container.scrollTop += below;
    }
}


function drawInitialGround(scale)
{
    drawFullGroundAt(0, scale);
}


function groundChangeTime(change)
{
    return typeof change.t === 'undefined' ? 0 : change.t;
}


function findGroundChangeIndex(position, step)
{
    var changes = position.c;
    if (!changes || changes.length === 0)
    {
        return 0;
    }

    var low = 0;
    var high = changes.length - 1;
    var best = 0;
    while (low <= high)
    {
        var mid = (low + high) >> 1;
        if (groundChangeTime(changes[mid]) <= step)
        {
            best = mid;
            low = mid + 1;
        }
        else
        {
            high = mid - 1;
        }
    }
    return best;
}


function drawFullGroundAt(step, scale)
{
    myGround.updatedTo = step;

    myRallyContext.beginPath();
    myRallyContext.rect(0, 0, 600, 600);
    myRallyContext.fillStyle = 'black';
    myRallyContext.fill();

    drawGroundAt(step, scale, 0, 0, myGround.sizeX, myGround.sizeY);
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
            var j = findGroundChangeIndex(myGround.positions[i], step);
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
        if (typeof current.a === 'undefined' && typeof previous.a !== 'undefined')
        {
            current.a = previous.a;
        }
        if (typeof current.l === 'undefined' && typeof previous.l !== 'undefined')
        {
            current.l = previous.l;
        }

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
        myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t1].a;
        myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t1].l;
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
            myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t2].a;
            myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t2].l;
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
            // At the exact start of a segment (including replay start), prefer t1's line so
            // locations[0].l (program entry) is shown instead of jumping to the first action cycle.
            if (dt <= 0 && typeof myRobots.robot[robotNr].locations[t1].l !== 'undefined')
            {
                myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t1].a;
                myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t1].l;
            }
            else if (dt <= 0 && t1 === 0)
            {
                // Legacy replays omit locations[0].l — still show program entry.
                myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t1].a;
                myRobots.robot[robotNr].l = 1;
            }
            else
            {
                myRobots.robot[robotNr].a = myRobots.robot[robotNr].locations[t2].a;
                myRobots.robot[robotNr].l = myRobots.robot[robotNr].locations[t2].l;
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        updateRobotPosition(i, time, stepTime);
        drawRobot(myRobots.robot[i], scale, cycle);
        drawRobotOre(myRobots.robot[i]);
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

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        myRobots.robot[i].updatedTo = 0;
    }
    expandAllRobotLocationDeltas();

    rallyBindTransportControls();
    redrawRallyScene();
}
