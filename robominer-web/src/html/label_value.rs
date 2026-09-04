use std::fmt::Display;

/// Shared label/value span pair used by achievement rewards, leaderboard standings, etc.
///
/// Callers must pass already-escaped (or trusted) HTML for `label` and `value`.
pub(crate) fn label_value(
    label_class: &str,
    value_class: &str,
    label: impl Display,
    value: impl Display,
) -> String {
    format!(
        r#"<span class="{label_class}">{label}</span><span class="{value_class}">{value}</span>"#
    )
}

#[cfg(test)]
mod tests {
    use super::label_value;

    #[test]
    fn label_value_renders_pair() {
        let html = label_value(
            "achievement-reward-label",
            "achievement-reward-value",
            "Points",
            10,
        );
        assert_eq!(
            html,
            r#"<span class="achievement-reward-label">Points</span><span class="achievement-reward-value">10</span>"#
        );
    }
}
