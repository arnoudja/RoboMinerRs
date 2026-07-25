//! Stats, highlights, and memory-usage markup for the robot config panel.

pub(super) fn push_robot_highlight(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<span class="robot-stat-highlight"><span class="robot-stat-highlight-label">{label}</span><span class="robot-stat-highlight-value">{value}{suffix}</span></span>"#,
        ));
    }
}

pub(super) fn add_robot_stat_entry(body: &mut String, label: &str, value: String) {
    body.push_str(&format!(
        r#"<div class="robot-stat"><dt>{label}</dt><dd>{value}</dd></div>"#,
    ));
}

pub(super) fn robot_memory_percent(program_size: i32, memory_size: i32) -> f64 {
    if memory_size <= 0 {
        return 100.0;
    }
    ((program_size as f64 / memory_size as f64) * 100.0).clamp(0.0, 100.0)
}

pub(super) fn render_robot_memory_progress(
    body: &mut String,
    program_size: i32,
    memory_size: i32,
    percent: f64,
    overflow: bool,
) {
    let overflow_class = if overflow { " robot-progress-over" } else { "" };
    body.push_str(&format!(r#"<div class="robot-progress{overflow_class}">"#));
    body.push_str(&format!(
        r#"<div class="robot-progress-heading"><span>Memory used</span><span class="robot-progress-value">{}/{}</span></div>"#,
        program_size, memory_size
    ));
    body.push_str(r#"<div class="robot-progress-track" aria-hidden="true">"#);
    body.push_str(&format!(
        r#"<div class="robot-progress-bar" style="width: {percent:.1}%"></div>"#
    ));
    body.push_str("</div></div>");
}
