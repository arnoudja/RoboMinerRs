/// Edit-code page scripts: shared panel state, then page logic.
use crate::static_assets::script_src_tags;

const PANEL_STATE_JS: &str = include_str!("../../static/js/common/panel_state.js");
const EDIT_CODE_PAGE_JS: &str = include_str!("../../static/js/edit_code/page.js");

pub(super) fn edit_code_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/panel_state.js", PANEL_STATE_JS),
        ("js/edit_code/page.js", EDIT_CODE_PAGE_JS),
    ])
}
