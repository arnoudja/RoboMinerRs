var RALLY_SIDE_PANEL_ORDER_KEY = 'robominer.rallySidePanelOrder';
var RALLY_SIDE_PANEL_ORDER_PROGRAM = 'program';
var RALLY_SIDE_PANEL_ORDER_PLAYERS = 'players';

function rallyReadSidePanelOrder()
{
    try
    {
        var stored = window.localStorage.getItem(RALLY_SIDE_PANEL_ORDER_KEY);
        if (stored === RALLY_SIDE_PANEL_ORDER_PLAYERS)
        {
            return RALLY_SIDE_PANEL_ORDER_PLAYERS;
        }
    }
    catch (error)
    {
        // Ignore storage access failures (private mode, disabled storage).
    }
    return RALLY_SIDE_PANEL_ORDER_PROGRAM;
}

function rallyWriteSidePanelOrder(order)
{
    if (order !== RALLY_SIDE_PANEL_ORDER_PROGRAM && order !== RALLY_SIDE_PANEL_ORDER_PLAYERS)
    {
        return;
    }
    try
    {
        window.localStorage.setItem(RALLY_SIDE_PANEL_ORDER_KEY, order);
    }
    catch (error)
    {
        // Ignore storage write failures.
    }
}

function rallyUpdateSidePanelOrderButtons(order)
{
    var buttons = document.querySelectorAll('.rally-view-panel-order-button[data-rally-panel]');
    for (var i = 0; i < buttons.length; i++)
    {
        var button = buttons[i];
        var panel = button.getAttribute('data-rally-panel');
        button.disabled = panel === order;
    }
}

function rallyApplySidePanelOrder(order)
{
    var column = document.querySelector('.rally-view-side-column');
    var programPanel = document.getElementById('rallyViewProgramPanel');
    var playersPanel = document.getElementById('rallyViewPlayersPanel');
    if (!column || !programPanel || !playersPanel)
    {
        return;
    }

    var preferred = order || rallyReadSidePanelOrder();
    if (preferred !== RALLY_SIDE_PANEL_ORDER_PLAYERS)
    {
        preferred = RALLY_SIDE_PANEL_ORDER_PROGRAM;
    }

    if (preferred === RALLY_SIDE_PANEL_ORDER_PLAYERS)
    {
        column.appendChild(playersPanel);
        column.appendChild(programPanel);
    }
    else
    {
        column.appendChild(programPanel);
        column.appendChild(playersPanel);
    }

    rallyUpdateSidePanelOrderButtons(preferred);
}

function rallyBindSidePanelOrder()
{
    if (window.__rallySidePanelOrderBound)
    {
        return;
    }
    window.__rallySidePanelOrderBound = true;

    document.addEventListener('click', function(event) {
        var target = event.target;
        if (!target || !target.closest)
        {
            return;
        }
        var button = target.closest('.rally-view-panel-order-button[data-rally-panel]');
        if (!button || button.disabled)
        {
            return;
        }
        var panel = button.getAttribute('data-rally-panel');
        if (panel !== RALLY_SIDE_PANEL_ORDER_PROGRAM && panel !== RALLY_SIDE_PANEL_ORDER_PLAYERS)
        {
            return;
        }
        rallyWriteSidePanelOrder(panel);
        rallyApplySidePanelOrder(panel);
    });
}
