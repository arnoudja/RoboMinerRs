const RALLY_SIDE_PANEL_ORDER_KEY = 'robominer.rallySidePanelOrder';
const RALLY_SIDE_PANEL_ORDER_PROGRAM = 'program';
const RALLY_SIDE_PANEL_ORDER_PLAYERS = 'players';

function rallyReadSidePanelOrder()
{
    try
    {
        const stored = window.localStorage.getItem(RALLY_SIDE_PANEL_ORDER_KEY);
        if (stored === RALLY_SIDE_PANEL_ORDER_PROGRAM)
        {
            return RALLY_SIDE_PANEL_ORDER_PROGRAM;
        }
    }
    catch (error)
    {
        // Ignore storage access failures (private mode, disabled storage).
    }
    return RALLY_SIDE_PANEL_ORDER_PLAYERS;
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
    const buttons = document.querySelectorAll('.rally-view-panel-order-button[data-rally-panel]');
    for (let i = 0; i < buttons.length; i++)
    {
        const button = buttons[i];
        const panel = button.getAttribute('data-rally-panel');
        button.disabled = panel === order;
    }
}

function rallyApplySidePanelOrder(order)
{
    const column = document.querySelector('.rally-view-side-column');
    const programPanel = document.getElementById('rallyViewProgramPanel');
    const playersPanel = document.getElementById('rallyViewPlayersPanel');
    if (!column || !programPanel || !playersPanel)
    {
        return;
    }

    let preferred = order || rallyReadSidePanelOrder();
    if (preferred !== RALLY_SIDE_PANEL_ORDER_PROGRAM)
    {
        preferred = RALLY_SIDE_PANEL_ORDER_PLAYERS;
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
        const target = event.target;
        if (!target || !target.closest)
        {
            return;
        }
        const button = target.closest('.rally-view-panel-order-button[data-rally-panel]');
        if (!button || button.disabled)
        {
            return;
        }
        const panel = button.getAttribute('data-rally-panel');
        if (panel !== RALLY_SIDE_PANEL_ORDER_PROGRAM && panel !== RALLY_SIDE_PANEL_ORDER_PLAYERS)
        {
            return;
        }
        rallyWriteSidePanelOrder(panel);
        rallyApplySidePanelOrder(panel);
    });
}
