//! Rally replay viewer scripts under `static/js/rally_animation/`, linked as static files.

use crate::static_assets::{script_src_tag, script_src_tags};

const PAYLOAD_JS: &str = include_str!("../static/js/rally_animation/payload.js");
const DRAW_GROUND_JS: &str = include_str!("../static/js/rally_animation/draw_ground.js");
const DRAW_ROBOTS_JS: &str = include_str!("../static/js/rally_animation/draw_robots.js");
const DEBUG_STATUS_JS: &str = include_str!("../static/js/rally_animation/debug_status.js");
const DEBUG_SOURCE_JS: &str = include_str!("../static/js/rally_animation/debug_source.js");
const TIMELINE_JS: &str = include_str!("../static/js/rally_animation/timeline.js");
const POSE_JS: &str = include_str!("../static/js/rally_animation/pose.js");
const TRANSPORT_JS: &str = include_str!("../static/js/rally_animation/transport.js");
const CONTROLS_JS: &str = include_str!("../static/js/rally_animation/controls.js");
const SIDE_PANELS_JS: &str = include_str!("../static/js/rally_animation/side_panels.js");
const PLAYER_JS: &str = include_str!("../static/js/rally_animation/player.js");
const BOOTSTRAP_JS: &str = include_str!("../static/js/rally_animation/bootstrap.js");

/// Ordered `<script src>` tags for the rally animation viewer (without bootstrap).
pub fn rally_animation_script_tags() -> String {
    script_src_tags(&[
        ("js/rally_animation/payload.js", PAYLOAD_JS),
        ("js/rally_animation/draw_ground.js", DRAW_GROUND_JS),
        ("js/rally_animation/draw_robots.js", DRAW_ROBOTS_JS),
        ("js/rally_animation/debug_status.js", DEBUG_STATUS_JS),
        ("js/rally_animation/debug_source.js", DEBUG_SOURCE_JS),
        ("js/rally_animation/timeline.js", TIMELINE_JS),
        ("js/rally_animation/pose.js", POSE_JS),
        ("js/rally_animation/transport.js", TRANSPORT_JS),
        ("js/rally_animation/controls.js", CONTROLS_JS),
        ("js/rally_animation/side_panels.js", SIDE_PANELS_JS),
        ("js/rally_animation/player.js", PLAYER_JS),
    ])
}

/// Bootstrap runs after config/result JSON tags are in the DOM.
pub fn rally_bootstrap_script_tag() -> String {
    script_src_tag("js/rally_animation/bootstrap.js", BOOTSTRAP_JS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rally_animation_script_tags_link_all_modules() {
        let tags = rally_animation_script_tags();
        for path in [
            "js/rally_animation/payload.js",
            "js/rally_animation/draw_ground.js",
            "js/rally_animation/draw_robots.js",
            "js/rally_animation/debug_status.js",
            "js/rally_animation/debug_source.js",
            "js/rally_animation/timeline.js",
            "js/rally_animation/pose.js",
            "js/rally_animation/transport.js",
            "js/rally_animation/controls.js",
            "js/rally_animation/side_panels.js",
            "js/rally_animation/player.js",
        ] {
            assert!(
                tags.contains(path),
                "expected rally script tags to include {path}"
            );
        }
        assert!(tags.contains("?v="));
        assert!(rally_bootstrap_script_tag().contains("js/rally_animation/bootstrap.js"));
    }

    #[test]
    fn rally_animation_modules_define_core_entrypoints() {
        assert!(PLAYER_JS.contains("function runanimation("));
        assert!(TRANSPORT_JS.contains("function rallySeekToRatio("));
        assert!(TRANSPORT_JS.contains("function rallyWithPausedSeek("));
        assert!(CONTROLS_JS.contains("function rallyBindKeyboardControls("));
        assert!(SIDE_PANELS_JS.contains("function rallyApplySidePanelOrder("));
        assert!(SIDE_PANELS_JS.contains("function rallyBindSidePanelOrder("));
        assert!(TIMELINE_JS.contains("function rallyRebuildCpuTimeline("));
        assert!(POSE_JS.contains("function updateRobotPosition("));
        assert!(PAYLOAD_JS.contains("function applyRallyResultPayload("));
        assert!(DRAW_ROBOTS_JS.contains("function drawRobot("));
        assert!(DEBUG_STATUS_JS.contains("function updateRobotDebugPanel("));
        assert!(DRAW_ROBOTS_JS.contains("RALLY_VIEWER_HIGHLIGHT_PADDING"));
        assert!(DRAW_GROUND_JS.contains("function drawGroundAt("));
        assert!(DEBUG_SOURCE_JS.contains("function updateRallySourceHighlight("));
        assert!(BOOTSTRAP_JS.contains("runanimation()"));
        assert!(BOOTSTRAP_JS.contains("rallyApplySidePanelOrder()"));
        assert!(BOOTSTRAP_JS.contains("rallyBindSidePanelOrder()"));
    }
}
