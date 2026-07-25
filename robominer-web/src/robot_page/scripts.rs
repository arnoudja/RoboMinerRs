/// Robot page scripts: shared panel state, URL helper, then page logic.
use crate::static_assets::script_src_tags;

const PANEL_STATE_JS: &str = include_str!("../../static/js/common/panel_state.js");
const URL_QUERY_JS: &str = include_str!("../../static/js/common/url_query.js");
const ROBOT_PAGE_JS: &str = include_str!("../../static/js/robot/page.js");

pub(super) fn robot_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/panel_state.js", PANEL_STATE_JS),
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/robot/page.js", ROBOT_PAGE_JS),
    ])
}
