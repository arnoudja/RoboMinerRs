use super::LeaderboardQuery;
use super::LeaderboardTab;
use super::render_shared::{
    leaderboard_rank_cell_class, leaderboard_row_classes, render_leaderboard_empty_state,
    render_leaderboard_load_more, render_leaderboard_owner_cell,
    render_leaderboard_section_actions,
};
use super::render_sidebar::render_leaderboard_climb_hint;
use crate::html::escape_html;

pub(super) fn render_leaderboard_top_robots_section(
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
