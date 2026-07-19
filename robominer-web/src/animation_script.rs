/// Rally replay viewer script, assembled from focused modules under
/// `static/js/rally_animation/`. Served as one inline `<script>` block.
pub const RALLY_ANIMATION_SCRIPT: &str = concat!(
    include_str!("../static/js/rally_animation/payload.js"),
    "\n",
    include_str!("../static/js/rally_animation/draw.js"),
    "\n",
    include_str!("../static/js/rally_animation/debug.js"),
    "\n",
    include_str!("../static/js/rally_animation/player.js"),
);

#[cfg(test)]
mod tests {
    use super::RALLY_ANIMATION_SCRIPT;

    #[test]
    fn rally_animation_script_exposes_core_viewer_functions() {
        for symbol in [
            "function applyRallyResultPayload(",
            "function validateRallyResultPayload(",
            "function showRallyReplayUnavailable(",
            "function runanimation(",
            "function rallySetSpeed(",
            "function rallySeekToRatio(",
            "function rallySeekByCycles(",
            "function rallyTogglePlayPause(",
            "function rallyBindKeyboardControls(",
            "function redrawRallyScene(",
            "function expandAllRobotLocationDeltas(",
            "function findGroundChangeIndex(",
            "function updateRobotTo(",
            "function drawRobot(",
            "function drawRobotDepot(",
            "function drawSideBySideDepotBar(",
            "function drawDepotHomes(",
            "function drawDepotHome(",
            "function robotColorRgba(",
            "function robotColor(",
            "function updateRobotDebugPanel(",
            "function robotCargoFull(",
            "function robotHasDepot(",
            "function robotDepotMaxTotal(",
            "function robotTurnsRemaining(",
            "function rallyActionName(",
            "function updateRallySourceHighlight(",
            "function updateRallyEditCodeLink(",
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

    #[test]
    fn rally_animation_script_is_assembled_from_modules() {
        assert!(RALLY_ANIMATION_SCRIPT.contains("validateRallyResultPayload"));
        assert!(RALLY_ANIMATION_SCRIPT.contains("drawFullGroundAt"));
        assert!(RALLY_ANIMATION_SCRIPT.contains("updateRobotDebugPanel"));
        assert!(RALLY_ANIMATION_SCRIPT.contains("rallyBindTransportControls"));
    }
}
