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

    var previous = robot.locations[step - 1];
    var current = robot.locations[step];
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
        var n = Number(value);
        return isNaN(n) ? 0 : n;
    }
    return cap(robot.depotMaxA) > 0 || cap(robot.depotMaxB) > 0 || cap(robot.depotMaxC) > 0;
}


function robotDepotMaxTotal(robot)
{
    function cap(value)
    {
        var n = Number(value);
        return isNaN(n) ? 0 : n;
    }
    return cap(robot.depotMaxA) + cap(robot.depotMaxB) + cap(robot.depotMaxC);
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
        cargoEl.textContent = 'Cargo A ' + Math.round(robot.A)
            + ' · B ' + Math.round(robot.B)
            + ' · C ' + Math.round(robot.C)
            + '  (' + total + '/' + robot.maxore + ')'
            + (full ? ' FULL' : '');
    }

    var depotEl = document.getElementById('robotDepot' + robot.robotnr);
    var depotChartEl = document.getElementById('depotChart' + robot.robotnr);
    if (robotHasDepot(robot))
    {
        var depotA = typeof robot.DA === 'number' ? robot.DA : Number(robot.DA) || 0;
        var depotB = typeof robot.DB === 'number' ? robot.DB : Number(robot.DB) || 0;
        var depotC = typeof robot.DC === 'number' ? robot.DC : Number(robot.DC) || 0;
        var depotTotal = Math.round(depotA) + Math.round(depotB) + Math.round(depotC);
        if (depotEl)
        {
            depotEl.removeAttribute('hidden');
            depotEl.textContent = 'Depot A ' + Math.round(depotA)
                + ' · B ' + Math.round(depotB)
                + ' · C ' + Math.round(depotC)
                + '  (' + depotTotal + '/' + robotDepotMaxTotal(robot) + ')';
        }
        if (depotChartEl)
        {
            depotChartEl.removeAttribute('hidden');
        }
    }
    else
    {
        if (depotEl)
        {
            depotEl.setAttribute('hidden', '');
            depotEl.textContent = 'Depot A 0 · B 0 · C 0';
        }
        if (depotChartEl)
        {
            depotChartEl.setAttribute('hidden', '');
        }
    }

    var actionEl = document.getElementById('robotAction' + robot.robotnr);
    var statusLabel = rallyStatusLabel(robot.s);
    var actionName = rallyActionName(robot.a);
    if (actionEl)
    {
        var label = null;
        if (statusLabel)
        {
            label = statusLabel;
        }
        else if (actionName)
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
