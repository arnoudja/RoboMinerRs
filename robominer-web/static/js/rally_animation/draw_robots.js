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


window.RALLY_VIEWER_HIGHLIGHT_PADDING = 4;
window.RALLY_VIEWER_HIGHLIGHT_LINE_WIDTH = 3;


function robotCenterPixels(robot, scale)
{
    return {
        x: robot.x * scale + scale / 2.0,
        y: robot.y * scale + scale / 2.0
    };
}


function robotDrawRadiusPixels(robot, scale)
{
    let radius = robot.size * scale / 2.0 + 2;
    if (typeof myRallyViewerSlot === 'number' && robot.robotnr === myRallyViewerSlot)
    {
        radius += RALLY_VIEWER_HIGHLIGHT_PADDING + RALLY_VIEWER_HIGHLIGHT_LINE_WIDTH + 2;
    }
    return radius;
}


function drawRobot(robot, scale, turn)
{
    const center = robotCenterPixels(robot, scale);
    const centerX = center.x;
    const centerY = center.y;

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

    const orientation = robot.o * Math.PI / 180.0;

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

    let side = Number(robot.homeSize);
    if (isNaN(side) || side < 1)
    {
        side = Math.ceil(Number(robot.size) || 1);
        if (side < 1)
        {
            side = 1;
        }
    }

    const homeX = Number(robot.homeX);
    const homeY = Number(robot.homeY);
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
    const home = depotHomeSquare(robot);
    if (!home)
    {
        return;
    }

    // Redraw opaque ground first so translucent tint never stacks across frames.
    drawGroundAt(step, scale, home.x, home.y, home.x + home.side, home.y + home.side);

    const x = home.x * scale;
    const y = home.y * scale;
    const size = home.side * scale;

    myRallyContext.fillStyle = robotColorRgba(robot.robotnr, 0.28);
    myRallyContext.fillRect(x, y, size, size);
}


function drawDepotHomes(scale, step)
{
    if (!rallyHasAnimationData())
    {
        return;
    }

    for (let i = 0; i < myRobots.robot.length; i++)
    {
        drawDepotHome(myRobots.robot[i], scale, step);
    }
}


function eraseRobot(robot, scale, step)
{
    const center = robotCenterPixels(robot, scale);
    const radius = robotDrawRadiusPixels(robot, scale);
    const orientation = robot.o * Math.PI / 180.0;
    const lineEndX = center.x + scale * robot.size * Math.cos(orientation) / 2.0;
    const lineEndY = center.y + scale * robot.size * Math.sin(orientation) / 2.0;

    const minPxX = Math.min(center.x - radius, lineEndX) - 2;
    const maxPxX = Math.max(center.x + radius, lineEndX) + 2;
    const minPxY = Math.min(center.y - radius, lineEndY) - 2;
    const maxPxY = Math.max(center.y + radius, lineEndY) + 2;

    myRallyContext.clearRect(minPxX, minPxY, maxPxX - minPxX, maxPxY - minPxY);

    const minX = Math.floor(Math.max(0, minPxX / scale));
    const minY = Math.floor(Math.max(0, minPxY / scale));
    let maxX = Math.ceil(Math.min(myGround.sizeX, maxPxX / scale));
    let maxY = Math.ceil(Math.min(myGround.sizeY, maxPxY / scale));

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
    const borderWidth = 3;
    const oreWidth = canvas.width - 2 * borderWidth;
    const oreHeight = canvas.height - 2 * borderWidth;
    const maxCapacity = capacity > 0 ? capacity : 1;
    const oreAHeight = Math.floor(amountA * oreHeight / maxCapacity);
    const oreBHeight = Math.floor((amountA + amountB) * oreHeight / maxCapacity) - oreAHeight;
    const oreCHeight = Math.floor((amountA + amountB + amountC) * oreHeight / maxCapacity) - oreAHeight - oreBHeight;

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


function drawSideBySideDepotBar(context, canvas, robotnr, amounts, capacities)
{
    const colors = ['red', 'green', 'blue'];
    const slots = [];
    for (let s = 0; s < 3; s++)
    {
        const capacity = capacities[s];
        if (!(capacity > 0))
        {
            continue;
        }
        slots.push({
            amount: amounts[s] > 0 ? amounts[s] : 0,
            capacity: capacity,
            color: colors[s]
        });
    }

    const borderWidth = 3;
    const innerWidth = canvas.width - 2 * borderWidth;
    const innerHeight = canvas.height - 2 * borderWidth;

    context.beginPath();
    context.rect(0, 0, canvas.width, canvas.height);
    context.fillStyle = robotColor(robotnr);
    context.fill();

    context.beginPath();
    context.rect(borderWidth, borderWidth, innerWidth, innerHeight);
    context.fillStyle = 'black';
    context.fill();

    if (slots.length === 0)
    {
        return;
    }

    const gap = slots.length > 1 ? 2 : 0;
    const columnWidth = Math.floor((innerWidth - gap * (slots.length - 1)) / slots.length);
    const usedWidth = columnWidth * slots.length + gap * (slots.length - 1);
    const startX = borderWidth + Math.floor((innerWidth - usedWidth) / 2);

    for (let i = 0; i < slots.length; i++)
    {
        const slot = slots[i];
        let fillHeight = Math.floor(slot.amount * innerHeight / slot.capacity);
        if (fillHeight > innerHeight)
        {
            fillHeight = innerHeight;
        }
        const columnX = startX + i * (columnWidth + gap);

        context.beginPath();
        context.rect(
            columnX,
            borderWidth + innerHeight - fillHeight,
            columnWidth,
            fillHeight
        );
        context.fillStyle = slot.color;
        context.fill();
    }
}


function drawRobotOre(robot)
{
    const i = robot.robotnr;
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
    const i = robot.robotnr;
    if (!myDepotCanvas[i] || !myDepotContext[i] || !robotHasDepot(robot))
    {
        return;
    }

    function amount(value)
    {
        const n = Number(value);
        return isNaN(n) ? 0 : n;
    }

    drawSideBySideDepotBar(
        myDepotContext[i],
        myDepotCanvas[i],
        robot.robotnr,
        [amount(robot.DA), amount(robot.DB), amount(robot.DC)],
        [amount(robot.depotMaxA), amount(robot.depotMaxB), amount(robot.depotMaxC)]
    );
}
