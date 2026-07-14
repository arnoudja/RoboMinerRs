use super::{LEADERBOARD_MAX_LIMIT, LEADERBOARD_SIDEBAR_AREA_STANDINGS};
use crate::help_pages;
use crate::html::{escape_html, layout};
use crate::leaderboard_page::{LeaderboardPageState, LeaderboardQuery, LeaderboardTab};

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

fn render_leaderboard_area_filter(
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

fn render_leaderboard_area_section(
    body: &mut String,
    query: LeaderboardQuery,
    ranked_areas: &[&robominer_db::LeaderboardMiningAreaRecord],
    mining_area_scores: &[robominer_db::LeaderboardMiningAreaScoreRecord],
    viewer_username: &str,
) {
    body.push_str(r#"<section class="leaderboard-panel">"#);
    if ranked_areas.is_empty() {
        render_leaderboard_empty_state(
            body,
            "No area scores yet. Queue mining runs to start climbing the board.",
            &[
                ("miningQueue", "Add runs to your mining queue"),
                ("helpTutorial?step=1", "Follow the tutorial"),
            ],
        );
        render_leaderboard_climb_hint(body, LeaderboardTab::Areas);
        body.push_str("</section>");
        return;
    }

    let Some(area_id) = query.area_id else {
        render_leaderboard_empty_state(body, "Choose a mining area to view its leaderboard.", &[]);
        render_leaderboard_climb_hint(body, LeaderboardTab::Areas);
        body.push_str("</section>");
        return;
    };

    let area_name = ranked_areas
        .iter()
        .find(|area| area.id == area_id)
        .map(|area| area.area_name.as_str())
        .unwrap_or("Mining area");
    let mut rows: Vec<_> = mining_area_scores
        .iter()
        .filter(|score| score.mining_area_id == area_id)
        .collect();
    let has_more = rows.len() as i64 > query.limit;
    rows.truncate(query.limit as usize);

    body.push_str(&format!(
        r#"<h2 class="leaderboard-section-title">{}</h2>"#,
        escape_html(area_name),
    ));
    body.push_str(
        r#"<p class="leaderboard-section-hint">Best recorded score per robot in this area.</p>"#,
    );
    render_leaderboard_area_filter(body, query, ranked_areas);
    render_leaderboard_section_actions(
        body,
        &[
            (
                &leaderboard_activity_area_href(area_id),
                "View area rallies",
            ),
            ("miningQueue", "Queue a run"),
        ],
    );
    render_leaderboard_area_score_table(body, &rows, area_id, viewer_username);
    if has_more {
        render_leaderboard_load_more(body, query);
    }
    render_leaderboard_climb_hint(body, LeaderboardTab::Areas);
    body.push_str("</section>");
}

fn leaderboard_activity_area_href(area_id: i64) -> String {
    format!("activity?areaId={area_id}")
}

fn render_leaderboard_section_actions(body: &mut String, links: &[(&str, &str)]) {
    body.push_str(r#"<p class="leaderboard-section-actions">"#);
    for (index, (href, label)) in links.iter().enumerate() {
        if index > 0 {
            body.push_str(" · ");
        }
        body.push_str(&format!(
            r#"<a class="leaderboard-section-action" href="{}">{}</a>"#,
            escape_html(href),
            escape_html(label),
        ));
    }
    body.push_str("</p>");
}

fn render_leaderboard_load_more(body: &mut String, query: LeaderboardQuery) {
    if query.limit >= LEADERBOARD_MAX_LIMIT {
        return;
    }

    body.push_str(&format!(
        r#"<p class="leaderboard-load-more-wrap"><a class="leaderboard-load-more-link" href="{}">Load more entries</a></p>"#,
        escape_html(&query.load_more_href()),
    ));
}

fn render_leaderboard_area_score_table(
    body: &mut String,
    rows: &[&robominer_db::LeaderboardMiningAreaScoreRecord],
    area_id: i64,
    viewer_username: &str,
) {
    if rows.is_empty() {
        render_leaderboard_empty_state(
            body,
            "No scores recorded for this area yet.",
            &[
                ("miningQueue", "Queue a run"),
                (
                    &leaderboard_activity_area_href(area_id),
                    "View area rallies",
                ),
            ],
        );
        return;
    }

    body.push_str(r#"<table class="leaderboard-table">"#);
    body.push_str(r#"<thead><tr>"#);
    body.push_str(r#"<th scope="col" class="leaderboard-col-rank">Rank</th>"#);
    body.push_str(r#"<th scope="col">Robot</th>"#);
    body.push_str(r#"<th scope="col">Owner</th>"#);
    body.push_str(
        r#"<th scope="col" class="leaderboard-col-score" title="Best single-run score recorded in this mining area.">Score</th>"#,
    );
    body.push_str("</tr></thead><tbody>");
    for (index, score) in rows.iter().enumerate() {
        let rank = index + 1;
        body.push_str(&format!(
            r#"<tr class="{}">"#,
            leaderboard_row_classes(rank, score.username.as_str(), viewer_username),
        ));
        body.push_str(&format!(
            r#"<td class="{}">#{}</td>"#,
            leaderboard_rank_cell_class(rank),
            rank,
        ));
        body.push_str(&format!(
            r#"<td class="leaderboard-name">{}</td>"#,
            escape_html(&score.robot_name),
        ));
        render_leaderboard_owner_cell(body, &score.username, viewer_username);
        body.push_str(&format!(
            r#"<td class="leaderboard-score"><span class="leaderboard-score-value">{:.1}</span><span class="leaderboard-score-meta">{} runs</span></td>"#,
            score.score,
            score.total_runs,
        ));
        body.push_str("</tr>");
    }
    body.push_str("</tbody></table>");
}

fn render_leaderboard_top_robots_section(
    body: &mut String,
    query: LeaderboardQuery,
    top_robots: &[robominer_db::LeaderboardTopRobotRecord],
    has_more: bool,
    viewer_username: &str,
) {
    body.push_str(r#"<section class="leaderboard-panel">"#);
    body.push_str(r#"<h2 class="leaderboard-section-title">Top robots</h2>"#);
    body.push_str(
        r#"<p class="leaderboard-section-hint">Highest average ore per run across all mining.</p>"#,
    );
    render_leaderboard_section_actions(
        body,
        &[
            ("miningResults", "View mining results"),
            ("miningQueue", "Queue more runs"),
        ],
    );
    if top_robots.is_empty() {
        render_leaderboard_empty_state(
            body,
            "No robot averages to show yet. Finish mining runs to appear here.",
            &[
                ("miningQueue", "Queue mining runs"),
                ("miningResults", "View mining results"),
            ],
        );
    } else {
        body.push_str(r#"<table class="leaderboard-table">"#);
        body.push_str(r#"<thead><tr>"#);
        body.push_str(r#"<th scope="col" class="leaderboard-col-rank">Rank</th>"#);
        body.push_str(r#"<th scope="col">Robot</th>"#);
        body.push_str(r#"<th scope="col">Owner</th>"#);
        body.push_str(
            r#"<th scope="col" class="leaderboard-col-score" title="Lifetime ore gathered divided by total mining runs.">Ore per run</th>"#,
        );
        body.push_str("</tr></thead><tbody>");
        for (index, robot) in top_robots.iter().enumerate() {
            let rank = index + 1;
            body.push_str(&format!(
                r#"<tr class="{}">"#,
                leaderboard_row_classes(rank, robot.username.as_str(), viewer_username),
            ));
            body.push_str(&format!(
                r#"<td class="{}">#{}</td>"#,
                leaderboard_rank_cell_class(rank),
                rank,
            ));
            body.push_str(&format!(
                r#"<td class="leaderboard-name">{}</td>"#,
                escape_html(&robot.robot_name),
            ));
            render_leaderboard_owner_cell(body, &robot.username, viewer_username);
            body.push_str(&format!(
                r#"<td class="leaderboard-score"><span class="leaderboard-score-value">{:.1}</span></td></tr>"#,
                robot.ore_per_run,
            ));
        }
        body.push_str("</tbody></table>");
        if has_more {
            render_leaderboard_load_more(body, query);
        }
    }
    render_leaderboard_climb_hint(body, LeaderboardTab::Robots);
    body.push_str("</section>");
}

fn render_leaderboard_top_players_section(
    body: &mut String,
    query: LeaderboardQuery,
    top_users: &[robominer_db::LeaderboardTopUserRecord],
    has_more: bool,
    viewer_username: &str,
) {
    body.push_str(r#"<section class="leaderboard-panel">"#);
    body.push_str(r#"<h2 class="leaderboard-section-title">Top players</h2>"#);
    body.push_str(r#"<p class="leaderboard-section-hint">Most achievement points earned.</p>"#);
    render_leaderboard_section_actions(body, &[("achievements", "View achievements")]);
    if top_users.is_empty() {
        render_leaderboard_empty_state(
            body,
            "No player standings yet. Claim achievements to climb the board.",
            &[
                ("achievements", "View achievements"),
                ("helpTutorial?step=1", "Follow the tutorial"),
            ],
        );
    } else {
        body.push_str(r#"<table class="leaderboard-table">"#);
        body.push_str(r#"<thead><tr>"#);
        body.push_str(r#"<th scope="col" class="leaderboard-col-rank">Rank</th>"#);
        body.push_str(r#"<th scope="col">Player</th>"#);
        body.push_str(
            r#"<th scope="col" class="leaderboard-col-score" title="Total achievement points claimed across all tracks.">Points</th>"#,
        );
        body.push_str("</tr></thead><tbody>");
        for (index, user) in top_users.iter().enumerate() {
            let rank = index + 1;
            body.push_str(&format!(
                r#"<tr class="{}">"#,
                leaderboard_row_classes(rank, user.username.as_str(), viewer_username),
            ));
            body.push_str(&format!(
                r#"<td class="{}">#{}</td>"#,
                leaderboard_rank_cell_class(rank),
                rank,
            ));
            render_leaderboard_player_cell(body, &user.username, viewer_username);
            body.push_str(&format!(
                r#"<td class="leaderboard-score"><span class="leaderboard-score-value">{}</span></td></tr>"#,
                user.achievement_points,
            ));
        }
        body.push_str("</tbody></table>");
        if has_more {
            render_leaderboard_load_more(body, query);
        }
    }
    render_leaderboard_climb_hint(body, LeaderboardTab::Players);
    body.push_str("</section>");
}

fn render_leaderboard_empty_state(body: &mut String, message: &str, links: &[(&str, &str)]) {
    body.push_str(r#"<div class="leaderboard-empty-state">"#);
    body.push_str(&format!(r#"<p>{}</p>"#, escape_html(message)));
    if !links.is_empty() {
        body.push_str(r#"<p class="leaderboard-empty-actions">"#);
        for (index, (href, label)) in links.iter().enumerate() {
            if index > 0 {
                body.push_str(" · ");
            }
            body.push_str(&format!(
                r#"<a class="leaderboard-section-action" href="{}">{}</a>"#,
                escape_html(href),
                escape_html(label),
            ));
        }
        body.push_str("</p>");
    }
    body.push_str("</div>");
}

fn render_leaderboard_climb_hint(body: &mut String, tab: LeaderboardTab) {
    let (summary, links): (&str, &[(&str, &str)]) = match tab {
        LeaderboardTab::Areas => (
            "Improve your robot program, queue more runs in this area, and study rival replays to raise your area score.",
            &[
                ("editCode", "Edit code"),
                ("miningQueue", "Mining queue"),
                ("activity", "Activity replays"),
            ],
        ),
        LeaderboardTab::Robots => (
            "Average ore per run rises when robots finish more successful mining runs with tuned programs.",
            &[
                ("editCode", "Edit code"),
                ("miningQueue", "Mining queue"),
                ("miningResults", "Mining results"),
            ],
        ),
        LeaderboardTab::Players => (
            "Achievement points come from completing tracks and claiming rewards across the game.",
            &[
                ("achievements", "Achievements"),
                ("helpTutorial?step=1", "Tutorial"),
            ],
        ),
    };

    body.push_str(r#"<aside class="leaderboard-climb-hint" aria-label="How to climb">"#);
    body.push_str(r#"<h3 class="leaderboard-climb-title">How to climb</h3>"#);
    body.push_str(&format!(
        r#"<p class="leaderboard-climb-copy">{}</p>"#,
        escape_html(summary)
    ));
    body.push_str(r#"<p class="leaderboard-climb-links">"#);
    for (index, (href, label)) in links.iter().enumerate() {
        if index > 0 {
            body.push_str(" · ");
        }
        body.push_str(&format!(
            r#"<a class="leaderboard-section-action" href="{}">{}</a>"#,
            escape_html(href),
            escape_html(label),
        ));
    }
    body.push_str("</p></aside>");
}

fn area_leader_score(
    scores: &[robominer_db::LeaderboardMiningAreaScoreRecord],
    area_id: i64,
) -> Option<f64> {
    scores
        .iter()
        .filter(|score| score.mining_area_id == area_id)
        .map(|score| score.score)
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

fn viewer_climb_insight(
    standing: &robominer_db::LeaderboardViewerStandingRecord,
    scores: &[robominer_db::LeaderboardMiningAreaScoreRecord],
) -> Option<String> {
    let mut best: Option<(&robominer_db::LeaderboardViewerAreaStandingRecord, f64)> = None;
    for area in &standing.area_standings {
        if area.rank <= 1 {
            continue;
        }
        let Some(leader_score) = area_leader_score(scores, area.mining_area_id) else {
            continue;
        };
        let gap = leader_score - area.score;
        if gap <= 0.0 {
            continue;
        }
        if best.map(|(_, best_gap)| gap < best_gap).unwrap_or(true) {
            best = Some((area, gap));
        }
    }

    best.map(|(area, gap)| {
        format!(
            "{} · {:.1} behind leader (#{})",
            area.area_name, gap, area.rank
        )
    })
}

fn render_leaderboard_metric_glossary(body: &mut String) {
    body.push_str(r#"<section class="leaderboard-sidebar-panel">"#);
    body.push_str(r#"<h2 class="leaderboard-section-title">How rankings work</h2>"#);
    body.push_str(r#"<dl class="leaderboard-metric-glossary">"#);
    body.push_str(
        r#"<div class="leaderboard-metric-item"><dt>Area score</dt><dd>Best single-run score per robot in a mining area.</dd></div>"#,
    );
    body.push_str(
        r#"<div class="leaderboard-metric-item"><dt>Ore per run</dt><dd>Lifetime ore gathered divided by total mining runs.</dd></div>"#,
    );
    body.push_str(
        r#"<div class="leaderboard-metric-item"><dt>Achievement points</dt><dd>Total points claimed from completed achievement tracks.</dd></div>"#,
    );
    body.push_str("</dl></section>");
}

fn render_leaderboard_sidebar(
    body: &mut String,
    viewer_standing: Option<&robominer_db::LeaderboardViewerStandingRecord>,
    top_robots: &[robominer_db::LeaderboardTopRobotRecord],
    mining_area_scores: &[robominer_db::LeaderboardMiningAreaScoreRecord],
    viewer_username: &str,
) {
    body.push_str(r#"<aside class="leaderboard-sidebar" aria-label="Leaderboard sidebar">"#);
    render_leaderboard_metric_glossary(body);
    if let Some(standing) = viewer_standing {
        render_leaderboard_sidebar_standings(
            body,
            standing,
            top_robots,
            mining_area_scores,
            viewer_username,
        );
    }
    body.push_str("</aside>");
}

fn render_leaderboard_sidebar_standings(
    body: &mut String,
    standing: &robominer_db::LeaderboardViewerStandingRecord,
    top_robots: &[robominer_db::LeaderboardTopRobotRecord],
    mining_area_scores: &[robominer_db::LeaderboardMiningAreaScoreRecord],
    viewer_username: &str,
) {
    body.push_str(r#"<section class="leaderboard-sidebar-panel">"#);
    body.push_str(r#"<h2 class="leaderboard-section-title">Your standings</h2>"#);
    body.push_str(r#"<ul class="leaderboard-standing-list">"#);
    body.push_str(&format!(
        r#"<li class="leaderboard-standing-item"><span class="leaderboard-standing-label">Achievement rank</span><span class="leaderboard-standing-value"><a class="leaderboard-standing-link" href="achievements">#{} · {} pts</a></span></li>"#,
        standing.achievement_rank,
        standing.achievement_points,
    ));

    if let Some(insight) = viewer_climb_insight(standing, mining_area_scores) {
        body.push_str(&format!(
            r#"<li class="leaderboard-standing-item leaderboard-standing-climb"><span class="leaderboard-standing-label">Closest to #1</span><span class="leaderboard-standing-value">{}</span></li>"#,
            escape_html(&insight),
        ));
    }

    if let Some((index, robot)) = top_robots
        .iter()
        .enumerate()
        .find(|(_, robot)| robot.username == viewer_username)
    {
        body.push_str(&format!(
            r#"<li class="leaderboard-standing-item"><span class="leaderboard-standing-label">Top robot list</span><span class="leaderboard-standing-value">#{} · {} ({:.1} ore/run)</span></li>"#,
            index + 1,
            escape_html(&robot.robot_name),
            robot.ore_per_run,
        ));
    }

    for area in standing
        .area_standings
        .iter()
        .take(LEADERBOARD_SIDEBAR_AREA_STANDINGS)
    {
        body.push_str(&format!(
            r#"<li class="leaderboard-standing-item"><span class="leaderboard-standing-label"><a class="leaderboard-standing-link" href="{}">{}</a></span><span class="leaderboard-standing-value">#{} · {:.1} with {}</span></li>"#,
            escape_html(&leaderboard_activity_area_href(area.mining_area_id)),
            escape_html(&area.area_name),
            area.rank,
            area.score,
            escape_html(&area.robot_name),
        ));
    }

    if standing.area_standings.is_empty() {
        body.push_str(
            r#"<li class="leaderboard-standing-item"><span class="leaderboard-standing-label">Area scores</span><span class="leaderboard-standing-value">No ranked runs yet</span></li>"#,
        );
    }

    body.push_str("</ul></section>");
}

fn render_leaderboard_owner_cell(body: &mut String, username: &str, viewer_username: &str) {
    let is_viewer = viewer_is_row_owner(username, viewer_username);
    let you_badge = if is_viewer {
        r#"<span class="leaderboard-you-badge">You</span>"#
    } else {
        ""
    };
    body.push_str(&format!(
        r#"<td class="leaderboard-owner{}">{}{}</td>"#,
        if is_viewer {
            " leaderboard-owner-self"
        } else {
            ""
        },
        escape_html(username),
        you_badge,
    ));
}

fn render_leaderboard_player_cell(body: &mut String, username: &str, viewer_username: &str) {
    let is_viewer = viewer_is_row_owner(username, viewer_username);
    let you_badge = if is_viewer {
        r#"<span class="leaderboard-you-badge">You</span>"#
    } else {
        ""
    };
    body.push_str(&format!(
        r#"<td class="leaderboard-name{}"><a class="leaderboard-row-link" href="achievements">{}</a>{}</td>"#,
        if is_viewer {
            " leaderboard-name-self"
        } else {
            ""
        },
        escape_html(username),
        you_badge,
    ));
}

fn viewer_is_row_owner(username: &str, viewer_username: &str) -> bool {
    !viewer_username.is_empty() && username == viewer_username
}

fn leaderboard_row_classes(rank: usize, username: &str, viewer_username: &str) -> String {
    format!(
        "leaderboard-row{}{}",
        leaderboard_row_rank_class(rank),
        if viewer_is_row_owner(username, viewer_username) {
            " leaderboard-row-self"
        } else {
            ""
        }
    )
}

fn leaderboard_row_rank_class(rank: usize) -> &'static str {
    match rank {
        1 => " leaderboard-row-rank-1",
        2 => " leaderboard-row-rank-2",
        3 => " leaderboard-row-rank-3",
        _ => "",
    }
}

fn leaderboard_rank_cell_class(rank: usize) -> &'static str {
    match rank {
        1 => "leaderboard-rank leaderboard-rank-1",
        2 => "leaderboard-rank leaderboard-rank-2",
        3 => "leaderboard-rank leaderboard-rank-3",
        _ => "leaderboard-rank",
    }
}
