var myRallySourceHighlightLine = null;
var myRallySourceHighlightToken = null;


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


function clearRallySourceTokenHighlight(lineEl)
{
    if (!lineEl)
    {
        return;
    }
    var code = lineEl.querySelector('.rally-view-source-text');
    if (!code)
    {
        return;
    }
    var text = code.textContent;
    while (code.firstChild)
    {
        code.removeChild(code.firstChild);
    }
    code.appendChild(document.createTextNode(text));
}


/**
 * Highlight a source line, optionally wrapping columns [c, e) (1-based inclusive start,
 * exclusive end) in a token span.
 * @param {number|{l?:number,c?:number,e?:number}} highlight
 */
function updateRallySourceHighlight(highlight)
{
    var sourceCode = document.getElementById('rallySourceCode');
    if (!sourceCode)
    {
        return;
    }

    var line = typeof highlight === 'number' ? highlight : (highlight && highlight.l);
    var startCol = typeof highlight === 'object' && highlight ? highlight.c : undefined;
    var endCol = typeof highlight === 'object' && highlight ? highlight.e : undefined;

    if (myRallySourceHighlightLine !== null)
    {
        var previous = document.getElementById('rallySourceLine' + myRallySourceHighlightLine);
        if (previous)
        {
            previous.classList.remove('rally-view-source-line-active');
            clearRallySourceTokenHighlight(previous);
        }
        myRallySourceHighlightLine = null;
        myRallySourceHighlightToken = null;
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

    var code = current.querySelector('.rally-view-source-text');
    if (code
        && typeof startCol === 'number'
        && typeof endCol === 'number'
        && startCol >= 1
        && endCol > startCol)
    {
        var text = code.textContent;
        // Columns are 1-based inclusive start, exclusive end over displayed source.
        var start = Math.max(0, Math.min(text.length, Math.floor(startCol) - 1));
        var end = Math.max(start, Math.min(text.length, Math.floor(endCol) - 1));
        if (end > start)
        {
            while (code.firstChild)
            {
                code.removeChild(code.firstChild);
            }
            if (start > 0)
            {
                code.appendChild(document.createTextNode(text.slice(0, start)));
            }
            var token = document.createElement('span');
            token.className = 'rally-view-source-token-active';
            token.textContent = text.slice(start, end);
            code.appendChild(token);
            if (end < text.length)
            {
                code.appendChild(document.createTextNode(text.slice(end)));
            }
            myRallySourceHighlightToken = token;
        }
    }

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
