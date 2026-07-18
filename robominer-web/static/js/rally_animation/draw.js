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
