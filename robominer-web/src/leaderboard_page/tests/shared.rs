use crate::html::{assert_contains_all, assert_html_not_contains};

use super::super::render::render_leaderboard_page;
use super::super::{LEADERBOARD_PAGE_SIZE, LeaderboardQuery, LeaderboardTab};
use super::fixtures::{
    areas_leaderboard_query, sample_leaderboard_state, sample_leaderboard_state_with_more,
};

#[test]
fn leaderboard_rendering_escapes_dynamic_fields() {
    let html = render_leaderboard_page(
        "User <One>".to_string(),
        None,
        areas_leaderboard_query(),
        &sample_leaderboard_state(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Area <A>".to_string(),
            }],
            vec![robominer_db::LeaderboardMiningAreaScoreRecord {
                mining_area_id: 1,
                robot_name: "Bot <1>".to_string(),
                username: "Owner & Co".to_string(),
                score: 12.34,
                total_runs: 5,
            }],
            vec![robominer_db::LeaderboardTopRobotRecord {
                robot_id: 7,
                robot_name: "Top \"Bot\"".to_string(),
                username: "Top Owner".to_string(),
                ore_per_run: 7.89,
            }],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Player <X>".to_string(),
                achievement_points: 42,
            }],
            None,
        ),
    );

    assert_contains_all(
        &html,
        &[
            "Area &lt;A&gt;",
            "Bot &lt;1&gt;",
            "Owner &amp; Co",
            ">12.3<",
        ],
    );
    assert_html_not_contains(&html, "Top &quot;Bot&quot;");
    assert_html_not_contains(&html, "Player &lt;X&gt;");
    assert_html_not_contains(&html, "User <One>");
}

#[test]
fn leaderboard_rendering_shows_load_more_cross_links_and_metric_hints() {
    let mut area_scores = Vec::new();
    for index in 0..11 {
        area_scores.push(robominer_db::LeaderboardMiningAreaScoreRecord {
            mining_area_id: 1,
            robot_name: format!("Bot {index}"),
            username: format!("Owner {index}"),
            score: 50.0 - f64::from(index),
            total_runs: index + 1,
        });
    }

    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Areas,
            area_id: Some(1),
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state_with_more(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Crystal Cave".to_string(),
            }],
            area_scores,
            vec![],
            vec![],
            None,
            false,
            false,
        ),
    );

    assert_contains_all(
        &html,
        &[
            r#"href="activity?areaId=1">View area rallies</a>"#,
            r#"href="leaderboard?tab=areas&amp;areaId=1&amp;limit=20">Load more entries</a>"#,
            r#"title="Smoothed running score for this robot in this mining area.""#,
        ],
    );

    let robots_html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Robots,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state_with_more(
            vec![],
            vec![],
            vec![robominer_db::LeaderboardTopRobotRecord {
                robot_id: 7,
                robot_name: "Alpha".to_string(),
                username: "Owner".to_string(),
                ore_per_run: 8.0,
            }],
            vec![],
            None,
            true,
            false,
        ),
    );

    assert_contains_all(
        &robots_html,
        &[
            r#"href="miningResults">View mining results</a>"#,
            r#"href="leaderboard?tab=robots&amp;limit=20">Load more entries</a>"#,
            r#"title="Lifetime ore gathered divided by total mining runs.""#,
        ],
    );

    let players_html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Players,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state_with_more(
            vec![],
            vec![],
            vec![],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Champion".to_string(),
                achievement_points: 100,
            }],
            None,
            false,
            true,
        ),
    );

    assert_contains_all(
        &players_html,
        &[
            r#"href="achievements?user=Champion">Champion</a>"#,
            r#"href="achievements">View achievements</a>"#,
            r#"title="Total achievement points claimed across all tracks.""#,
        ],
    );
}
