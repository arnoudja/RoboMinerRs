/// Edit-code page scripts: shared panel state, URL helper, editor modules, then page logic.
use crate::static_assets::{PANEL_STATE_JS, URL_QUERY_JS, script_src_tags};

const EDITOR_JS: &str = include_str!("../../static/js/edit_code/editor.js");
const URL_SYNC_JS: &str = include_str!("../../static/js/edit_code/url_sync.js");
const SAVE_JS: &str = include_str!("../../static/js/edit_code/save.js");
const PAGE_JS: &str = include_str!("../../static/js/edit_code/page.js");

pub(super) fn edit_code_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/panel_state.js", PANEL_STATE_JS),
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/edit_code/editor.js", EDITOR_JS),
        ("js/edit_code/url_sync.js", URL_SYNC_JS),
        ("js/edit_code/save.js", SAVE_JS),
        ("js/edit_code/page.js", PAGE_JS),
    ])
}
