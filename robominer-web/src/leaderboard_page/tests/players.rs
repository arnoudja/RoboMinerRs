use crate::html::{assert_contains_all, assert_html_not_contains};

use super::super::render::render_leaderboard_page;
use super::super::{LEADERBOARD_PAGE_SIZE, LeaderboardQuery, LeaderboardTab};
use super::fixtures::sample_leaderboard_state;

#[test]
fn leaderboard_rendering_shows_players_tab() {
    let html = render_leaderboard_page(
        "Champion".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Players,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state(
            vec![],
            vec![],
            vec![],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Champion".to_string(),
                achievement_points: 100,
            }],
            None,
        ),
    );

    assert_contains_all(
        &html,
        &[
            r#"href="leaderboard">Top players</a>"#,
            r#"class="leaderboard-section-title">Top players</h2>"#,
            r#"class="leaderboard-table""#,
            "leaderboard-name-self",
            r#"href="achievements">Champion</a>"#,
        ],
    );
    assert_html_not_contains(&html, "leaderboard-table-split");
}

#[test]
fn leaderboard_top_players_link_to_other_player_achievements_overview() {
    let html = render_leaderboard_page(
        "Viewer".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Players,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state(
            vec![],
            vec![],
            vec![],
            vec![
                robominer_db::LeaderboardTopUserRecord {
                    username: "Champion".to_string(),
                    achievement_points: 100,
                },
                robominer_db::LeaderboardTopUserRecord {
                    username: "Player <X>".to_string(),
                    achievement_points: 80,
                },
            ],
            None,
        ),
    );

    assert_contains_all(
        &html,
        &[
            r#"href="achievements?user=Champion">Champion</a>"#,
            r#"href="achievements?user=Player%20%3CX%3E">Player &lt;X&gt;</a>"#,
        ],
    );
    assert_html_not_contains(&html, r#"href="achievements">Champion</a>"#);
}
