pub const RALLY_ANIMATION_SCRIPT: &str = include_str!("../static/js/rally_animation.js");

#[cfg(test)]
mod tests {
    use super::RALLY_ANIMATION_SCRIPT;

    #[test]
    fn rally_animation_script_exposes_core_viewer_functions() {
        for symbol in [
            "function runanimation(",
            "function rallySetSpeed(",
            "function rallySeekToRatio(",
            "function drawRobot(",
            "function robotColor(",
            "function updateRobotDebugPanel(",
            "function robotCargoFull(",
            "function rallyActionName(",
            "function updateRallySourceHighlight(",
            "function scrollRallySourceLineIntoView(",
        ] {
            assert!(
                RALLY_ANIMATION_SCRIPT.contains(symbol),
                "expected rally animation script to define {symbol}"
            );
        }
    }

    #[test]
    fn rally_animation_script_declares_viewer_highlight_constants() {
        assert!(RALLY_ANIMATION_SCRIPT.contains("RALLY_VIEWER_HIGHLIGHT_PADDING"));
        assert!(RALLY_ANIMATION_SCRIPT.contains("RALLY_VIEWER_HIGHLIGHT_LINE_WIDTH"));
    }
}
