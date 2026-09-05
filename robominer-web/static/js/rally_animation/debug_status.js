const RALLY_ACTION_NAMES = {
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


function rallyStatusLabel(status)
{
    switch (status)
    {
        case 'battery':
            return 'Battery depleted';
        case 'scan':
            return 'Scanning';
        case 'cpu':
            return 'Waiting for CPU';
        case 'zero':
            return 'Zero-distance move';
        case 'motion':
            return 'Cannot move';
        case 'wall':
            return 'Blocked by wall';
        case 'robot':
            return 'Blocked by robot';
        case 'wait':
            return 'Wait';
        default:
            return null;
    }
}


function robotLooksIdle(robot, step)
{
    if (robot.s === 'cpu' || robot.s === 'zero' || robot.s === 'motion' || robot.s === 'wait')
    {
        return true;
    }
    if (robot.s === 'scan' || robot.s === 'battery' || robot.s === 'wall' || robot.s === 'robot')
    {
        return false;
    }

    if (typeof robot.a === 'number')
    {
        // Wait only — Scan (0) is productive work, not idle.
        return robot.a === 1;
    }

    if (!robot.locations || step <= 0 || step >= robot.locations.length)
    {
        return false;
    }

    const previous = robot.locations[step - 1];
    const current = robot.locations[step];
    return previous.x === current.x
        && previous.y === current.y
        && previous.o === current.o
        && previous.A === current.A
        && previous.B === current.B
        && previous.C === current.C;
}


function robotLooksBlocked(robot)
{
    return robot.s === 'wall' || robot.s === 'robot';
}


function robotCargoFull(robot)
{
    return Math.round(robot.A) + Math.round(robot.B) + Math.round(robot.C) >= robot.maxore;
}


function robotHasDepot(robot)
{
    function cap(value)
    {
        const n = Number(value);
        return isNaN(n) ? 0 : n;
    }
    return cap(robot.depotMaxA) > 0 || cap(robot.depotMaxB) > 0 || cap(robot.depotMaxC) > 0;
}


function robotTurnsRemaining(robot, step)
{
    if (typeof robot.maxturns !== 'number' || isNaN(robot.maxturns))
    {
        return null;
    }

    let remaining = Math.floor(robot.maxturns) - Math.floor(step);
    if (remaining < 0)
    {
        remaining = 0;
    }
    return remaining;
}


function updateRobotDebugPanel(robot, poseTurn, sourceLine)
{
    const turnsEl = document.getElementById('robotTurns' + robot.robotnr);
    const batteryEl = document.getElementById('robotBattery' + robot.robotnr);
    const batteryFillEl = document.getElementById('robotBatteryFill' + robot.robotnr);
    const remainingTurns = robotTurnsRemaining(robot, poseTurn);
    const depleted = remainingTurns === 0;
    const maxTurns = typeof robot.maxturns === 'number' && !isNaN(robot.maxturns)
        ? Math.floor(robot.maxturns)
        : 0;
    let ratio = 0;
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

    const full = robotCargoFull(robot);
    const depotChartEl = document.getElementById('depotChart' + robot.robotnr);
    if (depotChartEl)
    {
        if (robotHasDepot(robot))
        {
            depotChartEl.removeAttribute('hidden');
        }
        else
        {
            depotChartEl.setAttribute('hidden', '');
        }
    }

    const actionEl = document.getElementById('robotAction' + robot.robotnr);
    const statusLabel = rallyStatusLabel(robot.s);
    const actionName = rallyActionName(robot.a);
    if (actionEl)
    {
        let label = null;
        if (statusLabel)
        {
            label = statusLabel;
        }
        else if (actionName)
        {
            label = actionName;
        }
        else if (robotLooksIdle(robot, poseTurn))
        {
            label = 'Idle';
        }

        // undefined → peer cards use pose robot.l; null/number → highlight authority.
        const lineForLabel = typeof sourceLine === 'undefined' ? robot.l : sourceLine;
        if (label && typeof lineForLabel === 'number')
        {
            label += ' · L' + lineForLabel;
        }

        actionEl.textContent = label || '—';
    }

    const card = document.getElementById('rallyPlayer' + robot.robotnr);
    if (card)
    {
        if (robotLooksIdle(robot, poseTurn))
        {
            card.classList.add('rally-view-player-idle');
        }
        else
        {
            card.classList.remove('rally-view-player-idle');
        }

        if (robotLooksBlocked(robot))
        {
            card.classList.add('rally-view-player-blocked');
        }
        else
        {
            card.classList.remove('rally-view-player-blocked');
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
}


/**
 * Viewer-robot source highlight + edit-link (not per player-card).
 * @param {{l?:number,c?:number,e?:number,r?:{k?:string,v?:number},vs?:Object.<string,{k?:string,v?:number}>}|null|undefined} entry
 * @param {object|null|undefined} viewerRobot Presence check only — highlight uses `entry`, not pose fields.
 */
function updateRallyViewerSourceDebug(entry, viewerRobot)
{
    if (typeof myRallyViewerSlot !== 'number' || !viewerRobot)
    {
        return;
    }
    // Highlight authority is the CPU timeline entry only — never pose robot.l.
    if (entry && typeof entry.l === 'number')
    {
        updateRallySourceHighlight(entry);
        updateRallyEditCodeLink(entry.l);
        return;
    }
    updateRallySourceHighlight(null);
    updateRallyEditCodeLink(null);
}
