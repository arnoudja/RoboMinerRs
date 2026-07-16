use super::{LEADERBOARD_MAX_LIMIT, LeaderboardQuery};
use crate::html::escape_html;

pub(super) fn leaderboard_activity_area_href(area_id: i64) -> String {
    format!("activity?areaId={area_id}")
}

pub(super) fn render_leaderboard_section_actions(body: &mut String, links: &[(&str, &str)]) {
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

pub(super) fn render_leaderboard_load_more(body: &mut String, query: LeaderboardQuery) {
    if query.limit >= LEADERBOARD_MAX_LIMIT {
        return;
    }

    body.push_str(&format!(
        r#"<p class="leaderboard-load-more-wrap"><a class="leaderboard-load-more-link" href="{}">Load more entries</a></p>"#,
        escape_html(&query.load_more_href()),
    ));
}

pub(super) fn render_leaderboard_empty_state(
    body: &mut String,
    message: &str,
    links: &[(&str, &str)],
) {
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

pub(super) fn render_leaderboard_owner_cell(
    body: &mut String,
    username: &str,
    viewer_username: &str,
) {
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

pub(super) fn render_leaderboard_player_cell(
    body: &mut String,
    username: &str,
    viewer_username: &str,
) {
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

pub(super) fn leaderboard_row_classes(
    rank: usize,
    username: &str,
    viewer_username: &str,
) -> String {
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

pub(super) fn leaderboard_rank_cell_class(rank: usize) -> &'static str {
    match rank {
        1 => "leaderboard-rank leaderboard-rank-1",
        2 => "leaderboard-rank leaderboard-rank-2",
        3 => "leaderboard-rank leaderboard-rank-3",
        _ => "leaderboard-rank",
    }
}
