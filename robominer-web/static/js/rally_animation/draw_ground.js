function rgbToHex(r, g, b) {
    return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
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
    const changes = position.c;
    if (!changes || changes.length === 0)
    {
        return -1;
    }

    let low = 0;
    let high = changes.length - 1;
    let best = -1;
    while (low <= high)
    {
        const mid = (low + high) >> 1;
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
    const oreAMax = typeof myOreTypes.A !== 'undefined' ? myOreTypes.A.max : 255;
    const oreBMax = typeof myOreTypes.B !== 'undefined' ? myOreTypes.B.max : 255;
    const oreCMax = typeof myOreTypes.C !== 'undefined' ? myOreTypes.C.max : 255;

    myRallyContext.beginPath();
    myRallyContext.rect(fromX * scale, fromY * scale, (tillX - fromX) * scale, (tillY - fromY) * scale);
    myRallyContext.fillStyle = 'black';
    myRallyContext.fill();

    for (let i = 0; i < myGround.positions.length; i++)
    {
        if (myGround.positions[i].x >= fromX && myGround.positions[i].x < tillX &&
            myGround.positions[i].y >= fromY && myGround.positions[i].y < tillY)
        {
            const x = myGround.positions[i].x;
            const y = myGround.positions[i].y;
            const j = findGroundChangeIndex(myGround.positions[i], step);
            myGround.positions[i].lastDrawn = j;
            if (j < 0)
            {
                continue;
            }

            const changes = myGround.positions[i].c[j];
            const oreA = typeof changes.A !== 'undefined' ? changes.A : 0;
            const oreB = typeof changes.B !== 'undefined' ? changes.B : 0;
            const oreC = typeof changes.C !== 'undefined' ? changes.C : 0;

            const oreAIntensity = Math.min(255, Math.floor(oreA * 255 / oreAMax));
            const oreBIntensity = Math.min(255, Math.floor(oreB * 255 / oreBMax));
            const oreCIntensity = Math.min(255, Math.floor(oreC * 255 / oreCMax));

            myRallyContext.beginPath();
            myRallyContext.rect(x * scale, y * scale, scale, scale);
            myRallyContext.fillStyle = rgbToHex(oreAIntensity, oreBIntensity, oreCIntensity);
            myRallyContext.fill();
        }
    }
}
