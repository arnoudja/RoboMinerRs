/// Robot page scripts: shared panel state, URL helper, then page logic.
use crate::static_assets::page_scripts_with_panel_and_url_query;

const ROBOT_PAGE_JS: &str = include_str!("../../static/js/robot/page.js");

pub(super) fn robot_page_script_tag() -> String {
    page_scripts_with_panel_and_url_query("js/robot/page.js", ROBOT_PAGE_JS)
}
