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
    var changes = position.c;
    if (!changes || changes.length === 0)
    {
        return -1;
    }

    var low = 0;
    var high = changes.length - 1;
    var best = -1;
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
            if (j < 0)
            {
                continue;
            }

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
