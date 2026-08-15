use std::time::{SystemTime, UNIX_EPOCH};

use crate::html::{escape_html, format_relative_time_millis, format_utc_millis, layout};
use crate::robot_stats_page::RobotStatsPageState;
use crate::static_assets::PageStylesheet;

pub(super) fn render_robot_stats_page(
    username: String,
    hud: Option<&str>,
    state: &RobotStatsPageState,
) -> String {
    render_robot_stats_page_at(username, hud, state, robot_stats_now_millis())
}

fn robot_stats_now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}

pub(super) fn render_robot_stats_page_at(
    username: String,
    hud: Option<&str>,
    state: &RobotStatsPageState,
    now_millis: i64,
) -> String {
    let body = if state.robot_not_found {
        render_robot_stats_not_found()
    } else {
        render_robot_stats_overview(state, now_millis)
    };

    layout(
        "RoboMiner - Robot stats",
        "leaderboard",
        &username,
        hud,
        &body,
        &[PageStylesheet::RobotStats],
    )
}

fn render_robot_stats_not_found() -> String {
    let mut body = String::from(r#"<div class="robot-stats-page">"#);
    body.push_str(r#"<section class="robot-stats-summary" aria-label="Robot statistics">"#);
    body.push_str(r#"<div class="robot-stats-summary-heading">"#);
    body.push_str(r#"<h1 class="robot-stats-page-title">Robot stats</h1>"#);
    body.push_str("</div></section>");
    body.push_str(r#"<p class="robot-stats-empty">Robot not found.</p>"#);
    body.push_str(
        r#"<p class="robot-stats-back"><a class="robot-stats-back-link" href="leaderboard?tab=robots">Back to Top robots</a></p>"#,
    );
    body.push_str("</div>");
    body
}

fn render_robot_stats_overview(state: &RobotStatsPageState, now_millis: i64) -> String {
    let header = state
        .header
        .as_ref()
        .expect("overview render requires robot header");
    let total_ore = state.total_ore_mined();
    let total_tax = state.total_tax();
    let ore_per_run = state
        .ore_per_run()
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "—".to_string());

    let mut body = String::from(r#"<div class="robot-stats-page">"#);
    body.push_str(r#"<section class="robot-stats-summary" aria-label="Robot statistics">"#);
    body.push_str(r#"<div class="robot-stats-summary-heading">"#);
    body.push_str(&format!(
        r#"<h1 class="robot-stats-page-title">{}</h1>"#,
        escape_html(&header.robot_name)
    ));
    body.push_str(&format!(
        r#"<p class="robot-stats-owner">Owned by {}</p>"#,
        escape_html(&header.username)
    ));
    body.push_str("</div>");
    body.push_str(r#"<ul class="robot-stats-summary-list">"#);
    body.push_str(&format!(
        r#"<li class="robot-stats-summary-item"><span class="robot-stats-summary-label">Total runs</span><span class="robot-stats-summary-value">{}</span></li>"#,
        header.total_mining_runs
    ));
    body.push_str(&format!(
        r#"<li class="robot-stats-summary-item"><span class="robot-stats-summary-label">Ore mined</span><span class="robot-stats-summary-value">{}</span></li>"#,
        total_ore
    ));
    body.push_str(&format!(
        r#"<li class="robot-stats-summary-item"><span class="robot-stats-summary-label">Tax paid</span><span class="robot-stats-summary-value">{}</span></li>"#,
        total_tax
    ));
    body.push_str(&format!(
        r#"<li class="robot-stats-summary-item"><span class="robot-stats-summary-label">Ore per run</span><span class="robot-stats-summary-value">{}</span></li>"#,
        escape_html(&ore_per_run)
    ));
    body.push_str("</ul></section>");

    body.push_str(
        r#"<p class="robot-stats-back"><a class="robot-stats-back-link" href="leaderboard?tab=robots">Back to Top robots</a></p>"#,
    );

    render_area_stats_section(&mut body, &state.area_stats);
    render_ore_stats_section(&mut body, &state.ore_stats);
    render_recent_runs_section(&mut body, &state.recent_runs, now_millis);

    body.push_str("</div>");
    body
}

fn render_recent_runs_section(
    body: &mut String,
    recent_runs: &[robominer_db::MiningResultStateRecord],
    now_millis: i64,
) {
    body.push_str(
        r#"<section class="robot-stats-panel" aria-labelledby="robot-stats-runs-title">"#,
    );
    body.push_str(
        r#"<h2 id="robot-stats-runs-title" class="robot-stats-section-title">Latest runs</h2>"#,
    );
    body.push_str(
        r#"<p class="robot-stats-section-hint">Most recent claimed mining runs for this robot.</p>"#,
    );
    if recent_runs.is_empty() {
        body.push_str(r#"<p class="robot-stats-empty">No claimed runs yet.</p>"#);
    } else {
        body.push_str(r#"<table class="robot-stats-table">"#);
        body.push_str(r#"<thead><tr>"#);
        body.push_str(r#"<th scope="col">Area</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Score</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Mined</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Tax</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Net</th>"#);
        body.push_str(r#"<th scope="col">Ended</th>"#);
        body.push_str(r#"<th scope="col">Replay</th>"#);
        body.push_str("</tr></thead><tbody>");
        for run in recent_runs {
            let ended_relative =
                format_relative_time_millis(run.mining_end_time_millis, now_millis);
            let ended_absolute = format_utc_millis(run.mining_end_time_millis);
            body.push_str("<tr>");
            body.push_str(&format!(
                r#"<td>{}</td>"#,
                escape_html(&run.mining_area_name)
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{:.1}</td>"#,
                run.score
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                run.total_ore_mined
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                run.total_tax
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                run.total_reward
            ));
            body.push_str(&format!(
                r#"<td><time datetime="{}" title="{}">{}</time></td>"#,
                escape_html(&ended_absolute),
                escape_html(&ended_absolute),
                escape_html(&ended_relative)
            ));
            if let Some(rally_result_id) = run.rally_result_id {
                body.push_str(&format!(
                    r#"<td><a class="robot-stats-run-link" href="activity?rallyResultId={rally_result_id}">View rally</a></td>"#
                ));
            } else {
                body.push_str(r#"<td class="robot-stats-muted">—</td>"#);
            }
            body.push_str("</tr>");
        }
        body.push_str("</tbody></table>");
    }
    body.push_str("</section>");
}

fn render_area_stats_section(
    body: &mut String,
    area_stats: &[robominer_db::RobotMiningAreaStatRecord],
) {
    body.push_str(
        r#"<section class="robot-stats-panel" aria-labelledby="robot-stats-areas-title">"#,
    );
    body.push_str(
        r#"<h2 id="robot-stats-areas-title" class="robot-stats-section-title">Mining areas</h2>"#,
    );
    body.push_str(
        r#"<p class="robot-stats-section-hint">Runs and smoothed score per mining area.</p>"#,
    );
    if area_stats.is_empty() {
        body.push_str(r#"<p class="robot-stats-empty">No mining area history yet.</p>"#);
    } else {
        body.push_str(r#"<table class="robot-stats-table">"#);
        body.push_str(r#"<thead><tr>"#);
        body.push_str(r#"<th scope="col">Area</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Runs</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Score</th>"#);
        body.push_str("</tr></thead><tbody>");
        for area in area_stats {
            body.push_str("<tr>");
            body.push_str(&format!(r#"<td>{}</td>"#, escape_html(&area.area_name)));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                area.total_runs
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{:.1}</td>"#,
                area.score
            ));
            body.push_str("</tr>");
        }
        body.push_str("</tbody></table>");
    }
    body.push_str("</section>");
}

fn render_ore_stats_section(
    body: &mut String,
    ore_stats: &[robominer_db::RobotLifetimeOreStatRecord],
) {
    body.push_str(
        r#"<section class="robot-stats-panel" aria-labelledby="robot-stats-ores-title">"#,
    );
    body.push_str(
        r#"<h2 id="robot-stats-ores-title" class="robot-stats-section-title">Ore mined</h2>"#,
    );
    body.push_str(
        r#"<p class="robot-stats-section-hint">Lifetime ore gathered and tax paid after claims.</p>"#,
    );
    if ore_stats.is_empty() {
        body.push_str(r#"<p class="robot-stats-empty">No claimed ore totals yet.</p>"#);
    } else {
        body.push_str(r#"<table class="robot-stats-table">"#);
        body.push_str(r#"<thead><tr>"#);
        body.push_str(r#"<th scope="col">Ore</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Mined</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Tax</th>"#);
        body.push_str(r#"<th scope="col" class="robot-stats-col-numeric">Net</th>"#);
        body.push_str("</tr></thead><tbody>");
        for ore in ore_stats {
            let net = ore.amount.saturating_sub(ore.tax);
            body.push_str("<tr>");
            body.push_str(&format!(r#"<td>{}</td>"#, escape_html(&ore.ore_name)));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                ore.amount
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                ore.tax
            ));
            body.push_str(&format!(
                r#"<td class="robot-stats-col-numeric">{}</td>"#,
                net
            ));
            body.push_str("</tr>");
        }
        body.push_str("</tbody></table>");
    }
    body.push_str("</section>");
}
