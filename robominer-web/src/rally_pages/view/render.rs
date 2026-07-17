use crate::animation_script;
use crate::html::{escape_html, escape_js_string, layout};
use crate::mining_area_atlas::{MiningAreaAtlasLinkTarget, mining_area_atlas_url};
use crate::rally_pages::{RallyViewBackLink, RallyViewPageState};

pub fn render_rally_view_page(
    username: String,
    hud: Option<&str>,
    state: &RallyViewPageState,
    back_link: Option<RallyViewBackLink<'_>>,
) -> String {
    let mut ore_cases = String::new();
    for ore in &state.ores {
        ore_cases.push_str(&format!(
            "                        case {}:\n                            return '{}';\n",
            ore.id,
            escape_js_string(&ore.ore_name)
        ));
    }

    let mut body = String::from(r#"<div class="rally-view-page">"#);
    render_rally_view_header(&mut body, back_link);
    render_rally_view_context(&mut body, state);
    body.push_str("<script>");
    body.push_str(animation_script::RALLY_ANIMATION_SCRIPT);
    body.push_str("</script>");
    render_rally_view_deck(&mut body, state);
    render_rally_view_quick_links(&mut body, state);
    render_rally_view_bootstrap_script(&mut body, &ore_cases, &state.result_data, state);
    body.push_str("</div>");

    layout(
        "RoboMiner - Rally replay",
        "miningResults",
        &username,
        hud,
        &body,
    )
}

fn render_rally_view_header(body: &mut String, back_link: Option<RallyViewBackLink<'_>>) {
    body.push_str(r#"<header class="rally-view-header">"#);
    body.push_str(r#"<div class="rally-view-heading">"#);
    body.push_str(r#"<h1 class="rally-view-title">Rally replay</h1>"#);
    body.push_str(
        r#"<p class="rally-view-subtitle">Watch robots compete move by move on the mining map.</p>"#,
    );
    body.push_str("</div>");
    match back_link {
        Some(RallyViewBackLink::MiningResults(query)) => {
            body.push_str(&format!(
                r#"<a class="rally-view-back-link" href="miningResults?{}">Back to results</a>"#,
                escape_html(query)
            ));
        }
        Some(RallyViewBackLink::Activity(feed_query)) => {
            body.push_str(&format!(
                r#"<a class="rally-view-back-link" href="{}">Back to activity</a>"#,
                escape_html(&feed_query.href()),
            ));
        }
        None => {}
    }
    body.push_str("</header>");
}

fn rally_player_color_name(player_number: i32) -> &'static str {
    match player_number {
        0 => "green",
        1 => "blue",
        2 => "red",
        3 => "yellow",
        _ => "unknown",
    }
}

fn render_rally_view_context(body: &mut String, state: &RallyViewPageState) {
    body.push_str(r#"<section class="rally-view-context" aria-label="Rally context">"#);
    body.push_str(r#"<dl class="rally-view-context-stats">"#);
    body.push_str(&format!(
        r#"<div class="rally-view-context-item"><dt>Area</dt><dd>{}</dd></div>"#,
        escape_html(&state.mining_area_name),
    ));

    if let Some(player_number) = state.viewer_player_number {
        let robot_name = state
            .viewer_robot_name
            .as_deref()
            .filter(|name| !name.is_empty())
            .map(escape_html)
            .unwrap_or_else(|| "Your robot".to_string());
        body.push_str(&format!(
            r#"<div class="rally-view-context-item rally-view-context-item-self"><dt>Your robot</dt><dd>{} · {} slot</dd></div>"#,
            robot_name,
            rally_player_color_name(player_number),
        ));
    }

    if let Some(score) = state.viewer_score {
        body.push_str(&format!(
            r#"<div class="rally-view-context-item"><dt>Score</dt><dd>{:.1}</dd></div>"#,
            score
        ));
    }

    if state.viewer_result_claimed && state.viewer_total_reward.is_some() {
        body.push_str(&format!(
            r#"<div class="rally-view-context-item"><dt>Net payout</dt><dd class="rally-view-context-payout">+{}</dd></div>"#,
            state.viewer_total_reward.unwrap_or(0)
        ));
    }

    body.push_str("</dl></section>");
}

fn render_rally_view_quick_links(body: &mut String, state: &RallyViewPageState) {
    let Some(robot_id) = state.viewer_robot_id else {
        return;
    };

    body.push_str(r#"<nav class="rally-view-quick-links" aria-label="Rally quick links">"#);
    body.push_str(&format!(
        r#"<a class="rally-view-quick-link" href="miningQueue?robotId={}">Mining queue</a>"#,
        robot_id
    ));
    body.push_str(&format!(
        r#"<a class="rally-view-quick-link" href="robot?robotId={}">Robot workshop</a>"#,
        robot_id
    ));
    body.push_str(r#"<a class="rally-view-quick-link" href="shop">Shop parts</a>"#);
    body.push_str(&format!(
        r#"<a class="rally-view-quick-link" href="{}">Compare areas</a>"#,
        escape_html(&mining_area_atlas_url(
            MiningAreaAtlasLinkTarget::StandalonePage,
            None,
            false,
        )),
    ));
    body.push_str("</nav>");
}

fn render_rally_view_deck(body: &mut String, state: &RallyViewPageState) {
    body.push_str(r#"<div class="rally-view-deck">"#);
    body.push_str(r#"<section class="rally-view-stage" aria-label="Rally map">"#);
    body.push_str(r#"<div class="rally-view-canvas-wrap">"#);
    body.push_str(r#"<canvas id="rallyCanvas" width="600" height="600"></canvas>"#);
    body.push_str("</div>");
    body.push_str(r#"<div class="rally-view-transport">"#);
    body.push_str(r#"<div class="rally-view-controls">"#);
    body.push_str(
        r#"<button type="button" class="rally-view-control-button" id="rallyPlayPause">Play</button>"#,
    );
    body.push_str(
        r#"<button type="button" class="rally-view-control-button" id="rallyRestart">Restart</button>"#,
    );
    body.push_str(r#"<div class="rally-view-speed" aria-label="Playback speed">"#);
    body.push_str(
        r#"<button type="button" class="rally-view-speed-button" data-speed="0.1">0.1×</button>"#,
    );
    body.push_str(
        r#"<button type="button" class="rally-view-speed-button rally-view-speed-button-active" data-speed="1">1×</button>"#,
    );
    body.push_str(
        r#"<button type="button" class="rally-view-speed-button" data-speed="2">2×</button>"#,
    );
    body.push_str(
        r#"<button type="button" class="rally-view-speed-button" data-speed="4">4×</button>"#,
    );
    body.push_str("</div></div>");
    body.push_str(r#"<div class="rally-view-progress">"#);
    body.push_str(
        r#"<button type="button" class="rally-view-progress-track" id="rallyProgressTrack" aria-label="Seek rally replay"><span class="rally-view-progress-fill" id="rallyProgressFill"></span></button>"#,
    );
    body.push_str("</div>");
    body.push_str(
        r#"<p class="rally-view-cycle-status">Cycle <span id="rallyCycleCurrent">0</span> / <span id="rallyCycleTotal">0</span></p>"#,
    );
    body.push_str(r#"<input type="hidden" id="cyclenr" value="0" />"#);
    body.push_str(
        r#"<canvas id="progressCanvas" class="rally-view-progress-canvas" width="600" height="50" hidden></canvas>"#,
    );
    body.push_str("</div></section>");
    body.push_str(r#"<aside class="rally-view-sidebar">"#);
    body.push_str(r#"<h2 class="rally-view-sidebar-title">Players</h2>"#);
    body.push_str(r#"<div class="rally-view-players">"#);
    for index in 0..4 {
        let is_viewer = state
            .viewer_player_number
            .is_some_and(|player_number| player_number == i32::try_from(index).unwrap_or(-1));
        render_rally_view_player(
            body,
            index,
            &state.slots[index].0,
            &state.slots[index].1,
            is_viewer,
        );
    }
    body.push_str("</div>");
    render_rally_view_legend(body);
    if state.viewer_player_number.is_some() {
        render_rally_view_source(body, state.viewer_source_code.as_deref().unwrap_or(""));
    }
    body.push_str("</aside></div>");
}

fn render_rally_view_source(body: &mut String, source: &str) {
    body.push_str(r#"<section class="rally-view-source" aria-label="Your program">"#);
    body.push_str(r#"<h2 class="rally-view-source-title">Your program</h2>"#);
    body.push_str(
        r#"<p class="rally-view-source-note">Highlighted line is the statement running in the replay. Source is the private snapshot from this rally when available.</p>"#,
    );
    body.push_str(r#"<pre class="rally-view-source-code" id="rallySourceCode">"#);
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        body.push_str(&format!(
            r#"<span class="rally-view-source-line" data-line="{line_number}" id="rallySourceLine{line_number}"><span class="rally-view-source-lineno">{line_number}</span><span class="rally-view-source-text">{}</span></span>"#,
            escape_html(line),
        ));
    }
    body.push_str("</pre></section>");
}

fn render_rally_view_player(
    body: &mut String,
    index: usize,
    robot_name: &str,
    username: &str,
    is_viewer: bool,
) {
    let self_class = if is_viewer {
        " rally-view-player-self"
    } else {
        ""
    };
    let you_badge = if is_viewer {
        r#"<span class="rally-view-player-you">You</span>"#
    } else {
        ""
    };
    body.push_str(&format!(
        r#"<article class="rally-view-player rally-view-player-{index}{self_class}" id="rallyPlayer{index}"><header class="rally-view-player-header"><span class="rally-view-player-color" aria-hidden="true"></span><div><p class="rally-view-player-user">{you_badge}{}</p><p class="rally-view-player-robot">{}</p></div></header><div class="rally-view-player-debug"><p class="rally-view-player-cargo" id="robotCargo{index}">A 0 · B 0 · C 0</p><p class="rally-view-player-action" id="robotAction{index}">—</p></div><div class="rally-view-player-chart"><canvas id="oreCanvas{index}" width="50" height="200"></canvas></div></article>"#,
        escape_html(username),
        escape_html(robot_name),
    ));
}

fn render_rally_view_legend(body: &mut String) {
    body.push_str(r#"<section class="rally-view-legend" aria-label="Map ore types">"#);
    body.push_str(r#"<h2 class="rally-view-legend-title">Map ore</h2>"#);
    body.push_str(r#"<ul class="rally-view-legend-list">"#);
    body.push_str(
        r#"<li id="oreLegendA" class="rally-view-legend-item"><canvas id="oreLegendACanvas" width="25" height="25"></canvas><span id="oreLegendAName">OreA</span></li>"#,
    );
    body.push_str(
        r#"<li id="oreLegendB" class="rally-view-legend-item"><canvas id="oreLegendBCanvas" width="25" height="25"></canvas><span id="oreLegendBName">OreB</span></li>"#,
    );
    body.push_str(
        r#"<li id="oreLegendC" class="rally-view-legend-item"><canvas id="oreLegendCCanvas" width="25" height="25"></canvas><span id="oreLegendCName">OreC</span></li>"#,
    );
    body.push_str("</ul></section>");
}

fn render_rally_view_bootstrap_script(
    body: &mut String,
    ore_cases: &str,
    result_data: &str,
    state: &RallyViewPageState,
) {
    let viewer_slot = state
        .viewer_player_number
        .map(|player_number| player_number.to_string())
        .unwrap_or_else(|| "null".to_string());
    body.push_str(&format!(
        r#"<script>
                function getOreName(oreId) {{

                    switch (oreId) {{
{ore_cases}                        default:
                            return '';
                    }}
                }}

                window.requestAnimFrame = (function(callback) {{
                    return window.requestAnimationFrame || window.webkitRequestAnimationFrame || window.mozRequestAnimationFrame || window.oRequestAnimationFrame || window.msRequestAnimationFrame ||
                            function(callback) {{
                                window.setTimeout(callback, 1000 / 60);
                            }};
                    }})();
                {result_data}

                var myRallyViewerSlot = {viewer_slot};

                var myRallyCanvas = document.getElementById('rallyCanvas');
                var myRallyContext = myRallyCanvas.getContext('2d');

                var myOreCanvas = [ document.getElementById('oreCanvas0'), document.getElementById('oreCanvas1'), document.getElementById('oreCanvas2'), document.getElementById('oreCanvas3') ];
                var myOreContext = [ myOreCanvas[0].getContext('2d'), myOreCanvas[1].getContext('2d'), myOreCanvas[2].getContext('2d'), myOreCanvas[3].getContext('2d') ];

                var myProgressCanvas = document.getElementById('progressCanvas');
                var myProgressContext = myProgressCanvas ? myProgressCanvas.getContext('2d') : null;
                var myCycleText = document.getElementById('cyclenr');

                if (typeof myOreTypes.A !== 'undefined') {{
                    var canvas = document.getElementById('oreLegendACanvas');
                    var context = canvas.getContext('2d');
                    context.beginPath();
                    context.rect(0, 0, canvas.width, canvas.height);
                    context.fillStyle = 'red';
                    context.fill();
                    document.getElementById('oreLegendAName').textContent = getOreName(myOreTypes.A.id);
                    document.getElementById('oreLegendA').style.display = 'flex';
                }}

                if (typeof myOreTypes.B !== 'undefined') {{
                    var canvas = document.getElementById('oreLegendBCanvas');
                    var context = canvas.getContext('2d');
                    context.beginPath();
                    context.rect(0, 0, canvas.width, canvas.height);
                    context.fillStyle = 'green';
                    context.fill();
                    document.getElementById('oreLegendBName').textContent = getOreName(myOreTypes.B.id);
                    document.getElementById('oreLegendB').style.display = 'flex';
                }}

                if (typeof myOreTypes.C !== 'undefined') {{
                    var canvas = document.getElementById('oreLegendCCanvas');
                    var context = canvas.getContext('2d');
                    context.beginPath();
                    context.rect(0, 0, canvas.width, canvas.height);
                    context.fillStyle = 'blue';
                    context.fill();
                    document.getElementById('oreLegendCName').textContent = getOreName(myOreTypes.C.id);
                    document.getElementById('oreLegendC').style.display = 'flex';
                }}

                runanimation();
            </script>"#,
    ));
}
