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


function robotColorRgba(robotNr, alpha)
{
    switch (robotNr)
    {
        case 0:
            return 'rgba(0, 160, 0, ' + alpha + ')';

        case 1:
            return 'rgba(0, 0, 255, ' + alpha + ')';

        case 2:
            return 'rgba(255, 0, 0, ' + alpha + ')';

        case 3:
            return 'rgba(255, 255, 0, ' + alpha + ')';
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


function depotHomeSquare(robot)
{
    if (!robotHasDepot(robot))
    {
        return null;
    }

    var side = Number(robot.homeSize);
    if (isNaN(side) || side < 1)
    {
        side = Math.ceil(Number(robot.size) || 1);
        if (side < 1)
        {
            side = 1;
        }
    }

    var homeX = Number(robot.homeX);
    var homeY = Number(robot.homeY);
    if (!isNaN(homeX) && !isNaN(homeY))
    {
        return { x: homeX, y: homeY, side: side };
    }

    if (typeof myGround === 'undefined')
    {
        return null;
    }

    switch (robot.robotnr)
    {
        case 0:
            return { x: 0, y: 0, side: side };
        case 1:
            return { x: 0, y: myGround.sizeY - side, side: side };
        case 2:
            return { x: myGround.sizeX - side, y: 0, side: side };
        case 3:
            return { x: myGround.sizeX - side, y: myGround.sizeY - side, side: side };
        default:
            return null;
    }
}


function drawDepotHome(robot, scale, step)
{
    var home = depotHomeSquare(robot);
    if (!home)
    {
        return;
    }

    // Redraw opaque ground first so translucent tint never stacks across frames.
    drawGroundAt(step, scale, home.x, home.y, home.x + home.side, home.y + home.side);

    var x = home.x * scale;
    var y = home.y * scale;
    var size = home.side * scale;

    myRallyContext.fillStyle = robotColorRgba(robot.robotnr, 0.28);
    myRallyContext.fillRect(x, y, size, size);
}


function drawDepotHomes(scale, step)
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    for (var i = 0; i < myRobots.robot.length; i++)
    {
        drawDepotHome(myRobots.robot[i], scale, step);
    }
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


function drawStackedOreBar(context, canvas, robotnr, amountA, amountB, amountC, capacity)
{
    var borderWidth = 3;
    var oreWidth = canvas.width - 2 * borderWidth;
    var oreHeight = canvas.height - 2 * borderWidth;
    var maxCapacity = capacity > 0 ? capacity : 1;
    var oreAHeight = Math.floor(amountA * oreHeight / maxCapacity);
    var oreBHeight = Math.floor((amountA + amountB) * oreHeight / maxCapacity) - oreAHeight;
    var oreCHeight = Math.floor((amountA + amountB + amountC) * oreHeight / maxCapacity) - oreAHeight - oreBHeight;

    context.beginPath();
    context.rect(0, 0, canvas.width, canvas.height);
    context.fillStyle = robotColor(robotnr);
    context.fill();

    context.beginPath();
    context.rect(borderWidth, borderWidth, oreWidth, canvas.height - 2 * borderWidth);
    context.fillStyle = 'black';
    context.fill();

    context.beginPath();
    context.rect(borderWidth, canvas.height - borderWidth - oreAHeight, oreWidth, oreAHeight);
    context.fillStyle = 'red';
    context.fill();

    context.beginPath();
    context.rect(borderWidth, canvas.height - borderWidth - oreAHeight - oreBHeight, oreWidth, oreBHeight);
    context.fillStyle = 'green';
    context.fill();

    context.beginPath();
    context.rect(
        borderWidth,
        canvas.height - borderWidth - oreAHeight - oreBHeight - oreCHeight,
        oreWidth,
        oreCHeight
    );
    context.fillStyle = 'blue';
    context.fill();
}


function drawRobotOre(robot)
{
    var i = robot.robotnr;
    drawStackedOreBar(
        myOreContext[i],
        myOreCanvas[i],
        robot.robotnr,
        robot.A,
        robot.B,
        robot.C,
        robot.maxore
    );
}


function drawRobotDepot(robot)
{
    var i = robot.robotnr;
    if (!myDepotCanvas[i] || !myDepotContext[i] || !robotHasDepot(robot))
    {
        return;
    }

    var depotA = typeof robot.DA === 'number' ? robot.DA : Number(robot.DA) || 0;
    var depotB = typeof robot.DB === 'number' ? robot.DB : Number(robot.DB) || 0;
    var depotC = typeof robot.DC === 'number' ? robot.DC : Number(robot.DC) || 0;
    drawStackedOreBar(
        myDepotContext[i],
        myDepotCanvas[i],
        robot.robotnr,
        depotA,
        depotB,
        depotC,
        robotDepotMaxTotal(robot)
    );
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
