use crate::html::escape_html;
use crate::rally_pages::RallyViewPageState;

use super::payload::RallyResultPayloadKind;

pub(super) fn render_rally_view_deck(
    body: &mut String,
    state: &RallyViewPageState,
    replay_available: bool,
    payload_kind: RallyResultPayloadKind,
) {
    body.push_str(r#"<div class="rally-view-deck">"#);
    body.push_str(r#"<section class="rally-view-stage" aria-label="Rally map">"#);
    if replay_available {
        body.push_str(r#"<div class="rally-view-canvas-wrap">"#);
        body.push_str(r#"<canvas id="rallyCanvas" width="600" height="600"></canvas>"#);
        body.push_str("</div>");
        body.push_str(r#"<div class="rally-view-transport">"#);
        body.push_str(r#"<div class="rally-view-controls">"#);
        body.push_str(
            r#"<button type="button" class="rally-view-control-button" id="rallyPlayPause" aria-keyshortcuts="Space">Play</button>"#,
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
            r#"<button type="button" class="rally-view-progress-track" id="rallyProgressTrack" role="slider" aria-label="Seek rally replay" aria-valuemin="0" aria-valuemax="0" aria-valuenow="0" aria-valuetext="Cycle 0 of 0" aria-keyshortcuts="ArrowLeft ArrowRight Home End"><span class="rally-view-progress-fill" id="rallyProgressFill"></span></button>"#,
        );
        body.push_str("</div>");
        body.push_str(
            r#"<p class="rally-view-cycle-status">Area cycle <span id="rallyCycleCurrent">0</span> / <span id="rallyCycleTotal">0</span></p>"#,
        );
        body.push_str(
            r#"<p class="rally-view-keyboard-hint">Space play/pause · ← → one CPU cycle (when paused) · Shift+← → next area cycle · Home/End jump</p>"#,
        );
        body.push_str(r#"<input type="hidden" id="cyclenr" value="0" />"#);
        body.push_str(
            r#"<canvas id="progressCanvas" class="rally-view-progress-canvas" width="600" height="50" hidden></canvas>"#,
        );
        body.push_str("</div>");
    } else {
        render_rally_view_replay_unavailable(body, payload_kind);
    }
    body.push_str("</section>");
    body.push_str(r#"<aside class="rally-view-sidebar">"#);
    if state.viewer_player_number.is_some() {
        render_rally_view_source(
            body,
            state.viewer_source_code.as_deref(),
            state.viewer_program_source_id,
        );
    }
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
    body.push_str("</aside></div>");
}

pub(super) fn rally_player_color_name(player_number: i32) -> &'static str {
    match player_number {
        0 => "green",
        1 => "blue",
        2 => "red",
        3 => "yellow",
        _ => "unknown",
    }
}

fn render_rally_view_replay_unavailable(body: &mut String, payload_kind: RallyResultPayloadKind) {
    let detail = match payload_kind {
        RallyResultPayloadKind::LegacyExecutable => {
            "This rally was stored in an older executable format that is no longer played for security reasons."
        }
        RallyResultPayloadKind::Unsupported => {
            "This rally replay payload is missing, corrupt, or uses an unsupported version."
        }
        RallyResultPayloadKind::VersionedJson => "This rally replay is unavailable.",
    };
    body.push_str(r#"<div class="rally-view-replay-unavailable" role="status">"#);
    body.push_str(r#"<p class="rally-view-replay-unavailable-title">Replay unavailable</p>"#);
    body.push_str(&format!(
        r#"<p class="rally-view-replay-unavailable-note">{detail}</p>"#
    ));
    body.push_str("</div>");
}

fn render_rally_view_source(
    body: &mut String,
    source: Option<&str>,
    program_source_id: Option<i64>,
) {
    body.push_str(r#"<section class="rally-view-source" aria-label="Your program">"#);
    body.push_str(r#"<h2 class="rally-view-source-title">Your program</h2>"#);
    match source {
        Some(source) if !source.is_empty() => {
            body.push_str(
                r#"<p class="rally-view-source-note">Highlighted token is the program work running this CPU cycle. Source is the private snapshot from this rally.</p>"#,
            );
            if let Some(program_source_id) = program_source_id {
                render_rally_view_edit_code_link(body, program_source_id, true);
            }
            body.push_str(
                r#"<div class="rally-view-source-code" id="rallySourceCode" role="region" aria-label="Program source">"#,
            );
            for (index, line) in source.lines().enumerate() {
                let line_number = index + 1;
                body.push_str(&format!(
                    r#"<div class="rally-view-source-line" data-line="{line_number}" id="rallySourceLine{line_number}"><span class="rally-view-source-lineno">{line_number}</span><code class="rally-view-source-text">{}</code></div>"#,
                    escape_html(line),
                ));
            }
            body.push_str("</div>");
            body.push_str(
                r#"<div class="rally-view-source-return"><label class="rally-view-source-return-label" for="rallySourceStepResult">Return value</label><output class="rally-view-source-result" id="rallySourceStepResult" for="rallySourceCode" aria-live="polite"></output></div>"#,
            );
            body.push_str(
                r#"<div class="rally-view-source-variables"><div class="rally-view-source-variables-label" id="rallySourceVariablesLabel">Variables</div><table class="rally-view-source-variables-table" aria-labelledby="rallySourceVariablesLabel"><tbody id="rallySourceVariables" aria-live="polite"></tbody></table></div>"#,
            );
        }
        _ => {
            body.push_str(
                r#"<p class="rally-view-source-unavailable">Source snapshot unavailable.</p>"#,
            );
            body.push_str(
                r#"<p class="rally-view-source-note">This rally did not store a private program snapshot, so line highlighting is not shown.</p>"#,
            );
            if let Some(program_source_id) = program_source_id {
                render_rally_view_edit_code_link(body, program_source_id, false);
            }
        }
    }
    body.push_str("</section>");
}

fn render_rally_view_edit_code_link(
    body: &mut String,
    program_source_id: i64,
    track_highlighted_line: bool,
) {
    let href = format!("editCode?nextProgramSourceId={program_source_id}");
    if track_highlighted_line {
        body.push_str(&format!(
            r#"<p class="rally-view-source-edit"><a id="rallyEditCodeLink" class="rally-view-source-edit-link" data-edit-href="{href}" href="{href}">Edit code at highlighted line</a></p>"#,
        ));
        body.push_str(
            r#"<p class="rally-view-source-note">Opens the robot's current linked program in the editor (may differ from this snapshot).</p>"#,
        );
    } else {
        body.push_str(&format!(
            r#"<p class="rally-view-source-edit"><a class="rally-view-source-edit-link" href="{href}">Edit linked program</a></p>"#,
        ));
    }
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
        r#"<article class="rally-view-player rally-view-player-{index}{self_class}" id="rallyPlayer{index}"><header class="rally-view-player-header"><span class="rally-view-player-color" aria-hidden="true"></span><div><p class="rally-view-player-user">{you_badge}{}</p><p class="rally-view-player-robot">{}</p></div></header><div class="rally-view-player-debug"><div class="rally-view-player-battery" id="robotBattery{index}" role="progressbar" aria-label="Battery turns remaining" aria-valuemin="0" aria-valuemax="0" aria-valuenow="0"><div class="rally-view-player-battery-meta"><span class="rally-view-player-battery-caption">Battery</span><span class="rally-view-player-turns" id="robotTurns{index}">—</span></div><div class="rally-view-player-battery-track"><span class="rally-view-player-battery-fill" id="robotBatteryFill{index}"></span></div></div><p class="rally-view-player-action" id="robotAction{index}">—</p></div><div class="rally-view-player-chart"><div class="rally-view-player-chart-bar"><canvas id="oreCanvas{index}" width="50" height="200" aria-label="Cargo"></canvas><span class="rally-view-player-chart-label">Cargo</span></div><div class="rally-view-player-chart-bar" id="depotChart{index}" hidden><canvas id="depotCanvas{index}" width="50" height="200" aria-label="Depot"></canvas><span class="rally-view-player-chart-label">Depot</span></div></div></article>"#,
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
