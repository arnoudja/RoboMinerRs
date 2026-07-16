use super::render_areas::render_leaderboard_area_section;
use super::render_players::render_leaderboard_top_players_section;
use super::render_robots::render_leaderboard_top_robots_section;
use super::render_sidebar::render_leaderboard_sidebar;
use super::{LeaderboardPageState, LeaderboardQuery, LeaderboardTab};
use crate::help_pages;
use crate::html::{escape_html, layout};

pub(super) fn render_leaderboard_page(
    username: String,
    hud: Option<&str>,
    query: LeaderboardQuery,
    state: &LeaderboardPageState,
) -> String {
    let ranked_areas = ranked_mining_areas(&state.mining_areas, &state.mining_area_scores);
    let resolved_query = LeaderboardQuery {
        area_id: query.resolved_area_id(&ranked_areas),
        ..query
    };

    let mut body = String::from(r#"<div class="leaderboard-page">"#);
    render_leaderboard_header(&mut body);
    render_leaderboard_stats(
        &mut body,
        resolved_query.limit,
        ranked_areas.len(),
        state.top_robots.len(),
        state.top_users.len(),
    );
    body.push_str(r#"<div class="leaderboard-deck">"#);
    body.push_str(r#"<div class="leaderboard-main">"#);
    render_leaderboard_tab_filter(&mut body, resolved_query);
    match resolved_query.tab {
        LeaderboardTab::Areas => render_leaderboard_area_section(
            &mut body,
            resolved_query,
            &ranked_areas,
            &state.mining_area_scores,
            &username,
        ),
        LeaderboardTab::Robots => render_leaderboard_top_robots_section(
            &mut body,
            resolved_query,
            &state.top_robots,
            state.has_more_robots,
            &username,
        ),
        LeaderboardTab::Players => render_leaderboard_top_players_section(
            &mut body,
            resolved_query,
            &state.top_users,
            state.has_more_players,
            &username,
        ),
    }
    body.push_str("</div>");
    render_leaderboard_sidebar(
        &mut body,
        state.viewer_standing.as_ref(),
        &state.top_robots,
        &state.mining_area_scores,
        &username,
    );
    body.push_str("</div></div>");

    layout(
        "RoboMiner - Leaderboard",
        "leaderboard",
        &username,
        hud,
        &body,
    )
}

fn ranked_mining_areas<'a>(
    mining_areas: &'a [robominer_db::LeaderboardMiningAreaRecord],
    mining_area_scores: &[robominer_db::LeaderboardMiningAreaScoreRecord],
) -> Vec<&'a robominer_db::LeaderboardMiningAreaRecord> {
    mining_areas
        .iter()
        .filter(|area| {
            mining_area_scores
                .iter()
                .any(|score| score.mining_area_id == area.id)
        })
        .collect()
}

fn render_leaderboard_header(body: &mut String) {
    body.push_str(r#"<header class="leaderboard-header">"#);
    body.push_str(r#"<div class="leaderboard-heading">"#);
    body.push_str(r#"<h1 class="leaderboard-title">Leaderboard</h1>"#);
    body.push_str(
        r#"<p class="leaderboard-subtitle">See who leads each mining area and across the game.</p>"#,
    );
    body.push_str(&help_pages::render_page_help_hint_line(&[
        ("helpTutorial?step=1", "Tutorial"),
        ("achievements", "Achievements"),
        ("miningAreaOverview", "Compare areas"),
    ]));
    body.push_str("</div></header>");
}

fn render_leaderboard_stats(
    body: &mut String,
    limit: i64,
    ranked_area_count: usize,
    top_robot_count: usize,
    top_player_count: usize,
) {
    body.push_str(r#"<dl class="leaderboard-stats">"#);
    body.push_str(&format!(
        r#"<div class="leaderboard-stat"><dt>Showing</dt><dd>Top {limit}</dd></div>"#,
    ));
    body.push_str(&format!(
        r#"<div class="leaderboard-stat"><dt>Areas ranked</dt><dd>{ranked_area_count}</dd></div>"#,
    ));
    body.push_str(&format!(
        r#"<div class="leaderboard-stat"><dt>Top robots</dt><dd>{top_robot_count}</dd></div>"#,
    ));
    body.push_str(&format!(
        r#"<div class="leaderboard-stat"><dt>Top players</dt><dd>{top_player_count}</dd></div>"#,
    ));
    body.push_str("</dl>");
}

fn render_leaderboard_tab_filter(body: &mut String, query: LeaderboardQuery) {
    body.push_str(r#"<nav class="leaderboard-tab-filter" aria-label="Leaderboard views">"#);
    for tab in [
        LeaderboardTab::Areas,
        LeaderboardTab::Robots,
        LeaderboardTab::Players,
    ] {
        let class_name = if tab == query.tab {
            "leaderboard-tab-link leaderboard-tab-link-active"
        } else {
            "leaderboard-tab-link"
        };
        body.push_str(&format!(
            r#"<a class="{class_name}" href="{}">{}</a>"#,
            escape_html(&query.tab_href(tab)),
            escape_html(tab.label()),
        ));
    }
    body.push_str("</nav>");
}

pub(super) fn render_leaderboard_area_filter(
    body: &mut String,
    query: LeaderboardQuery,
    ranked_areas: &[&robominer_db::LeaderboardMiningAreaRecord],
) {
    if ranked_areas.is_empty() {
        return;
    }

    body.push_str(r#"<label class="leaderboard-area-filter" for="leaderboardAreaFilter">Area "#);
    body.push_str(
        r#"<select id="leaderboardAreaFilter" class="tableitem leaderboard-area-filter-select" aria-label="Choose mining area leaderboard" onchange="window.location = this.value;">"#,
    );
    for area in ranked_areas {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            escape_html(&query.area_href(Some(area.id))),
            if query.area_id == Some(area.id) {
                " selected"
            } else {
                ""
            },
            escape_html(&area.area_name),
        ));
    }
    body.push_str("</select></label>");
}
