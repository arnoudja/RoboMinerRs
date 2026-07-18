/**
 * Load a versioned rally animation payload into the viewer globals.
 * Payload shape (v1): { v, robots, ground, oreTypes }.
 * Legacy executable `var myRobots = …` rows are rejected by the page and never injected.
 * Returns null on success; an error string leaves globals unchanged for a graceful unavailable UI.
 */
function validateRallyResultPayload(payload)
{
    if (!payload || payload.v !== 1)
    {
        return 'This rally replay payload is missing, corrupt, or uses an unsupported version.';
    }

    if (!payload.robots || !Array.isArray(payload.robots.robot))
    {
        return 'This rally replay is missing robot animation data.';
    }

    for (var i = 0; i < payload.robots.robot.length; i++)
    {
        if (!payload.robots.robot[i] || !Array.isArray(payload.robots.robot[i].locations))
        {
            return 'This rally replay has incomplete robot animation data.';
        }
    }

    if (!payload.ground
        || typeof payload.ground.sizeX !== 'number'
        || typeof payload.ground.sizeY !== 'number'
        || !Array.isArray(payload.ground.positions))
    {
        return 'This rally replay is missing map animation data.';
    }

    return null;
}


function showRallyReplayUnavailable(detail)
{
    var stage = document.querySelector('.rally-view-stage');
    if (!stage)
    {
        return;
    }

    while (stage.firstChild)
    {
        stage.removeChild(stage.firstChild);
    }

    var wrap = document.createElement('div');
    wrap.className = 'rally-view-replay-unavailable';
    wrap.setAttribute('role', 'status');

    var title = document.createElement('p');
    title.className = 'rally-view-replay-unavailable-title';
    title.textContent = 'Replay unavailable';

    var note = document.createElement('p');
    note.className = 'rally-view-replay-unavailable-note';
    note.textContent = detail
        || 'This rally replay payload is missing, corrupt, or uses an unsupported version.';

    wrap.appendChild(title);
    wrap.appendChild(note);
    stage.appendChild(wrap);
}


function applyRallyResultPayload(payload)
{
    var error = validateRallyResultPayload(payload);
    if (error)
    {
        return error;
    }

    myRobots = payload.robots;
    myGround = payload.ground;
    myOreTypes = payload.oreTypes || {};
    return null;
}
