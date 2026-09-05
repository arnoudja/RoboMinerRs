window.myRallySourceHighlightLine = null;
/** @type {{l?:number,c?:number,e?:number,rKey?:string,rVal?:number,vsKey?:string}|null} */
window.myRallySourceHighlightKey = null;


function rallySourceHighlightKey(line, startCol, endCol, result, variables)
{
    const rKey = result && typeof result.k === 'string' ? result.k : '';
    const rVal = result && typeof result.v === 'number' ? result.v : '';
    let vsKey = '';
    if (variables && typeof variables === 'object')
    {
        vsKey = Object.keys(variables).sort().map(function(name) {
            const entry = variables[name];
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
    const link = document.getElementById('rallyEditCodeLink');
    if (!link)
    {
        return;
    }

    const baseHref = link.getAttribute('data-edit-href');
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
    const code = lineEl.querySelector('.rally-view-source-text');
    if (!code)
    {
        return;
    }
    const text = code.textContent;
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
        return String(Math.round(result.v));
    }
    const kind = typeof result.k === 'string' ? result.k : '?';
    return kind + ':' + result.v;
}


/**
 * Show the selected CPU step's typed return value, or leave empty when absent.
 * @param {{k?:string,v?:number}|null|undefined} result
 */
function updateRallySourceStepResult(result)
{
    const el = document.getElementById('rallySourceStepResult');
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
    const el = document.getElementById('rallySourceVariables');
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

    const names = Object.keys(variables).sort();
    for (let i = 0; i < names.length; i++)
    {
        const name = names[i];
        const formatted = formatRallySourceStepResult(variables[name]);
        if (!formatted)
        {
            continue;
        }

        const row = document.createElement('tr');

        const nameEl = document.createElement('td');
        nameEl.className = 'rally-view-source-var-name';
        nameEl.textContent = name + ':';

        const valueEl = document.createElement('td');
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
    const sourceCode = document.getElementById('rallySourceCode');
    if (!sourceCode)
    {
        myRallySourceHighlightKey = null;
        updateRallySourceStepResult(null);
        updateRallySourceVariables(null);
        return;
    }

    const line = typeof highlight === 'number' ? highlight : (highlight && highlight.l);
    const startCol = typeof highlight === 'object' && highlight ? highlight.c : undefined;
    const endCol = typeof highlight === 'object' && highlight ? highlight.e : undefined;
    const result = typeof highlight === 'object' && highlight ? highlight.r : undefined;
    const variables = typeof highlight === 'object' && highlight ? highlight.vs : undefined;
    const nextKey = rallySourceHighlightKey(line, startCol, endCol, result, variables);
    if (rallySourceHighlightKeysEqual(myRallySourceHighlightKey, nextKey))
    {
        return;
    }

    if (myRallySourceHighlightLine !== null)
    {
        const previous = document.getElementById('rallySourceLine' + myRallySourceHighlightLine);
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

    const current = document.getElementById('rallySourceLine' + line);
    if (!current)
    {
        myRallySourceHighlightKey = nextKey;
        updateRallySourceStepResult(null);
        updateRallySourceVariables(null);
        return;
    }

    current.classList.add('rally-view-source-line-active');
    myRallySourceHighlightLine = line;

    const code = current.querySelector('.rally-view-source-text');
    if (code
        && typeof startCol === 'number'
        && typeof endCol === 'number'
        && startCol >= 1
        && endCol > startCol)
    {
        const text = code.textContent;
        // Columns are 1-based inclusive start, exclusive end over displayed source.
        const start = Math.max(0, Math.min(text.length, Math.floor(startCol) - 1));
        const end = Math.max(start, Math.min(text.length, Math.floor(endCol) - 1));
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
            const token = document.createElement('span');
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
    const containerRect = container.getBoundingClientRect();
    const lineRect = lineEl.getBoundingClientRect();
    const above = lineRect.top - containerRect.top;
    const below = lineRect.bottom - containerRect.bottom;

    if (above < 0)
    {
        container.scrollTop += above;
    }
    else if (below > 0)
    {
        container.scrollTop += below;
    }
}
