use std::collections::HashMap;

use crate::html::EscapedHtml;
use crate::mining_queue_page::{MiningQueueAreaView, MiningQueueDisplayItem, MiningQueueRobotView};

use crate::mining_queue_page::mining_queue_status_description;

const MINING_QUEUE_TRASH_ICON: &str = r#"<svg class="mining-queue-remove-icon" viewBox="0 0 24 24" width="18" height="18" focusable="false" aria-hidden="true"><path fill="currentColor" d="M9 3h6l1 1h4v2H4V4h4l1-1zm1 5h1v10h-1V8zm3 0h1v10h-1V8zM6 8h12l-1 12H7L6 8z"/></svg>"#;

pub(super) fn render_queue_run_row(
    body: &mut String,
    item: &MiningQueueDisplayItem,
    show_remove_button: bool,
    refresh_on_complete: bool,
    progress_total_seconds: Option<i64>,
) {
    body.push_str(r#"<div class="mining-queue-run-row">"#);
    if show_remove_button {
        body.push_str(&format!(
            r#"<input type="checkbox" class="mining-queue-item-check" data-queue-item-id="{}" data-mining-area-id="{}" aria-label="Select queued run in {}"/><button type="button" class="mining-queue-remove-btn" data-queue-item-id="{}" data-mining-area-id="{}" aria-label="Remove queued run in {}">{MINING_QUEUE_TRASH_ICON}</button>"#,
            item.mining_queue_id,
            item.mining_area_id,
            EscapedHtml::from(item.area_name.as_str()),
            item.mining_queue_id,
            item.mining_area_id,
            EscapedHtml::from(item.area_name.as_str())
        ));
    }
    body.push_str(r#"<span class="mining-queue-run-area">"#);
    if active_run_result_link(item.status, item.rally_result_id) {
        body.push_str(&format!(
            r#"<a href="miningResults?rallyResultId={}">{}</a>"#,
            item.rally_result_id.unwrap_or(0),
            EscapedHtml::from(item.area_name.as_str())
        ));
    } else {
        body.push_str(&format!("{}", EscapedHtml::from(item.area_name.as_str())));
    }
    body.push_str("</span>");
    body.push_str(&format!(
        r#"<span class="miningqueuestatus mining-queue-status mining-queue-status-{}">{}</span>"#,
        mining_queue_status_class(item.status),
        mining_queue_status_description(item.status)
    ));
    let progress_total = progress_total_seconds
        .filter(|total| *total > 0)
        .map(|total| {
            if refresh_on_complete {
                total.saturating_add(1)
            } else {
                total
            }
        });
    let time_left_seconds = if refresh_on_complete {
        item.time_left_seconds.saturating_add(1)
    } else {
        item.time_left_seconds
    };
    let progress_attr = progress_total
        .map(|total| format!(r#" data-progress-total="{}""#, total))
        .unwrap_or_default();
    body.push_str(&format!(
        r#"<span class="miningqueuetime mining-queue-run-time" data-seconds-left="{}"{}{}>{}</span>"#,
        time_left_seconds,
        if refresh_on_complete {
            r#" data-refresh-on-complete="true""#
        } else {
            ""
        },
        progress_attr,
        format_queue_time_left(time_left_seconds)
    ));
    body.push_str("</div>");
}

pub(super) fn active_run_result_link(
    status: robominer_db::MiningQueueStatus,
    rally_result_id: Option<i64>,
) -> bool {
    matches!(
        status,
        robominer_db::MiningQueueStatus::Mining | robominer_db::MiningQueueStatus::Recharging
    ) && rally_result_id.is_some()
}

pub(super) fn active_run_progress_total(
    item: &MiningQueueDisplayItem,
    robot: &MiningQueueRobotView,
    area_map: &HashMap<i64, &MiningQueueAreaView>,
) -> Option<i64> {
    match item.status {
        robominer_db::MiningQueueStatus::Mining => area_map
            .get(&item.mining_area_id)
            .map(|area| i64::from(area.mining_time)),
        robominer_db::MiningQueueStatus::Recharging => Some(i64::from(robot.recharge_time)),
        _ => None,
    }
}

pub(super) fn render_run_progress(body: &mut String, time_left_seconds: i64, total_seconds: i64) {
    let percent = if total_seconds > 0 {
        let elapsed = total_seconds.saturating_sub(time_left_seconds.max(0));
        ((elapsed as f64 / total_seconds as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    body.push_str(&format!(
        r#"<progress class="mining-queue-progress" value="{percent:.1}" max="100" aria-hidden="true"></progress>"#
    ));
}

pub(super) fn mining_queue_status_class(status: robominer_db::MiningQueueStatus) -> &'static str {
    match status {
        robominer_db::MiningQueueStatus::Mining => "mining",
        robominer_db::MiningQueueStatus::Recharging => "recharging",
        robominer_db::MiningQueueStatus::Queued => "queued",
        robominer_db::MiningQueueStatus::Updating => "updating",
    }
}

pub(in crate::mining_queue_page) fn format_queue_time_left(seconds: i64) -> String {
    let seconds_left = seconds.max(0);
    let display_seconds = seconds_left % 60;
    let display_minutes = (seconds_left / 60) % 60;
    let display_hours = seconds_left / 3600;

    if display_hours > 0 {
        format!("{display_hours}:{display_minutes:02}:{display_seconds:02}")
    } else {
        format!("{display_minutes}:{display_seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_queue_time_left;

    #[test]
    fn mining_queue_time_left_uses_countdown_format() {
        assert_eq!(format_queue_time_left(0), "0:00");
        assert_eq!(format_queue_time_left(60), "1:00");
        assert_eq!(format_queue_time_left(150), "2:30");
        assert_eq!(format_queue_time_left(3_661), "1:01:01");
    }
}
