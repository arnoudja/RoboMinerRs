use super::LEADERBOARD_SIDEBAR_AREA_STANDINGS;
use super::LeaderboardTab;
use super::render_shared::leaderboard_activity_area_href;
use crate::html::escape_html;

pub(super) fn render_leaderboard_climb_hint(body: &mut String, tab: LeaderboardTab) {
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

pub(super) fn render_leaderboard_sidebar(
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
