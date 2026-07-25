use crate::animation_script;
use crate::html::{escape_html, layout};
use crate::mining_area_atlas::{MiningAreaAtlasLinkTarget, mining_area_atlas_url};
use crate::rally_pages::{RallyViewBackLink, RallyViewPageState};

use super::payload::{RallyResultPayloadKind, classify_rally_result_payload};
use super::render_deck::{rally_player_color_name, render_rally_view_deck};

pub fn render_rally_view_page(
    username: String,
    hud: Option<&str>,
    state: &RallyViewPageState,
    back_link: Option<RallyViewBackLink<'_>>,
) -> String {
    let payload_kind = classify_rally_result_payload(&state.result_data);
    let replay_available = payload_kind == RallyResultPayloadKind::VersionedJson;

    let mut body = String::from(r#"<div class="rally-view-page">"#);
    render_rally_view_header(&mut body, back_link);
    render_rally_view_context(&mut body, state);
    if replay_available {
        body.push_str(&animation_script::rally_animation_script_tags());
    }
    render_rally_view_deck(&mut body, state, replay_available, payload_kind);
    render_rally_view_quick_links(&mut body, state);
    if replay_available {
        render_rally_view_bootstrap_data(&mut body, &state.result_data, state);
        body.push_str(&animation_script::rally_bootstrap_script_tag());
    }
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
    if let Some(program_source_id) = state.viewer_program_source_id {
        body.push_str(&format!(
            r#"<a class="rally-view-quick-link" href="editCode?nextProgramSourceId={}">Edit code</a>"#,
            program_source_id
        ));
    }
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

fn render_rally_view_bootstrap_data(
    body: &mut String,
    result_data: &str,
    state: &RallyViewPageState,
) {
    // Versioned JSON only — never inject stored resultData as executable script.
    body.push_str(r#"<script type="application/json" id="rally-result-data">"#);
    body.push_str(result_data);
    body.push_str("</script>");

    let mut ore_names = serde_json::Map::new();
    for ore in &state.ores {
        ore_names.insert(
            ore.id.to_string(),
            serde_json::Value::String(ore.ore_name.clone()),
        );
    }
    let config = serde_json::json!({
        "viewerSlot": state.viewer_player_number,
        "oreNames": ore_names,
    });
    body.push_str(r#"<script type="application/json" id="rally-view-config">"#);
    body.push_str(&config.to_string());
    body.push_str("</script>");
}
