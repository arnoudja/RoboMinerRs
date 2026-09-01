var myRallySourceHighlightLine = null;
/** @type {{l?:number,c?:number,e?:number,rKey?:string,rVal?:number,vsKey?:string}|null} */
var myRallySourceHighlightKey = null;


function rallySourceHighlightKey(line, startCol, endCol, result, variables)
{
    var rKey = result && typeof result.k === 'string' ? result.k : '';
    var rVal = result && typeof result.v === 'number' ? result.v : '';
    var vsKey = '';
    if (variables && typeof variables === 'object')
    {
        vsKey = Object.keys(variables).sort().map(function(name) {
            var entry = variables[name];
            return name + ':' + (entry && entry.k) + '=' + (entry && entry.v);
        }).join('|');
    }
    return {
        l: line,
        c: startCol,
        e: endCol,
        rKey: rKey,
        rVal: rVal,
        vsKey: vsKey
    };
}


function rallySourceHighlightKeysEqual(a, b)
{
    return !!(a && b
        && a.l === b.l
        && a.c === b.c
        && a.e === b.e
        && a.rKey === b.rKey
        && a.rVal === b.rVal
        && a.vsKey === b.vsKey);
}


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
 * Format a typed CPU-step return value for the Return value field.
 * @param {{k?:string,v?:number}|null|undefined} result
 * @returns {string}
 */
function formatRallySourceStepResult(result)
{
    if (!result || typeof result.v !== 'number' || !isFinite(result.v))
    {
        return '';
    }

    if (result.k === 'b')
    {
        return result.v !== 0 ? 'true' : 'false';
    }
    if (result.k === 'f')
    {
        return result.v.toFixed(2);
    }
    if (result.k === 'i')
    {
        return String(Math.trunc(result.v));
    }
    var kind = typeof result.k === 'string' ? result.k : '?';
    return kind + ':' + result.v;
}


/**
 * Show the selected CPU step's typed return value, or leave empty when absent.
 * @param {{k?:string,v?:number}|null|undefined} result
 */
function updateRallySourceStepResult(result)
{
    var el = document.getElementById('rallySourceStepResult');
    if (!el)
    {
        return;
    }
    el.textContent = formatRallySourceStepResult(result);
}


/**
 * Show visible program locals for the selected CPU step, or clear when absent.
 * @param {Object.<string,{k?:string,v?:number}>|null|undefined} variables
 */
function updateRallySourceVariables(variables)
{
    var el = document.getElementById('rallySourceVariables');
    if (!el)
    {
        return;
    }

    while (el.firstChild)
    {
        el.removeChild(el.firstChild);
    }

    if (!variables || typeof variables !== 'object')
    {
        return;
    }

    var names = Object.keys(variables).sort();
    for (var i = 0; i < names.length; i++)
    {
        var name = names[i];
        var formatted = formatRallySourceStepResult(variables[name]);
        if (!formatted)
        {
            continue;
        }

        var row = document.createElement('tr');

        var nameEl = document.createElement('td');
        nameEl.className = 'rally-view-source-var-name';
        nameEl.textContent = name + ':';

        var valueEl = document.createElement('td');
        valueEl.className = 'rally-view-source-var-value';
        valueEl.textContent = formatted;

        row.appendChild(nameEl);
        row.appendChild(valueEl);
        el.appendChild(row);
    }
}


/**
 * Highlight a source line, optionally wrapping columns [c, e) (1-based inclusive start,
 * exclusive end) in a token span. When `r`/`vs` are present, update return value and locals.
 * @param {number|{l?:number,c?:number,e?:number,r?:{k?:string,v?:number},vs?:Object.<string,{k?:string,v?:number}>}} highlight
 */
function updateRallySourceHighlight(highlight)
{
    var sourceCode = document.getElementById('rallySourceCode');
    if (!sourceCode)
    {
        myRallySourceHighlightKey = null;
        updateRallySourceStepResult(null);
        updateRallySourceVariables(null);
        return;
    }

    var line = typeof highlight === 'number' ? highlight : (highlight && highlight.l);
    var startCol = typeof highlight === 'object' && highlight ? highlight.c : undefined;
    var endCol = typeof highlight === 'object' && highlight ? highlight.e : undefined;
    var result = typeof highlight === 'object' && highlight ? highlight.r : undefined;
    var variables = typeof highlight === 'object' && highlight ? highlight.vs : undefined;
    var nextKey = rallySourceHighlightKey(line, startCol, endCol, result, variables);
    if (rallySourceHighlightKeysEqual(myRallySourceHighlightKey, nextKey))
    {
        return;
    }

    if (myRallySourceHighlightLine !== null)
    {
        var previous = document.getElementById('rallySourceLine' + myRallySourceHighlightLine);
        if (previous)
        {
            previous.classList.remove('rally-view-source-line-active');
            clearRallySourceTokenHighlight(previous);
        }
        myRallySourceHighlightLine = null;
    }

    if (typeof line !== 'number' || isNaN(line) || line < 1)
    {
        myRallySourceHighlightKey = nextKey;
        updateRallySourceStepResult(null);
        updateRallySourceVariables(null);
        return;
    }

    var current = document.getElementById('rallySourceLine' + line);
    if (!current)
    {
        myRallySourceHighlightKey = nextKey;
        updateRallySourceStepResult(null);
        updateRallySourceVariables(null);
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
        }
    }

    myRallySourceHighlightKey = nextKey;
    updateRallySourceStepResult(result);
    updateRallySourceVariables(variables);
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
