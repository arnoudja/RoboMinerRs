use super::LeaderboardQuery;
use super::LeaderboardTab;
use super::render_shared::{
    leaderboard_rank_cell_class, leaderboard_row_classes, render_leaderboard_empty_state,
    render_leaderboard_load_more, render_leaderboard_player_cell,
    render_leaderboard_section_actions,
};
use super::render_sidebar::render_leaderboard_climb_hint;

pub(super) fn render_leaderboard_top_players_section(
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
