use super::LeaderboardQuery;
use super::LeaderboardTab;
use super::render::render_leaderboard_area_filter;
use super::render_shared::{
    leaderboard_activity_area_href, leaderboard_rank_cell_class, leaderboard_row_classes,
    render_leaderboard_empty_state, render_leaderboard_load_more, render_leaderboard_owner_cell,
    render_leaderboard_section_actions,
};
use super::render_sidebar::render_leaderboard_climb_hint;
use crate::html::escape_html;

pub(super) fn render_leaderboard_area_section(
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
