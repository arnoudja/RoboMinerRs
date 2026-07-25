/// Shop page scripts: shared URL/session helpers, then page logic.
use crate::static_assets::script_src_tags;

const URL_QUERY_JS: &str = include_str!("../../static/js/common/url_query.js");
const SESSION_STORE_JS: &str = include_str!("../../static/js/common/session_store.js");
const SHOP_PAGE_JS: &str = include_str!("../../static/js/shop/page.js");

pub(super) fn shop_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/common/session_store.js", SESSION_STORE_JS),
        ("js/shop/page.js", SHOP_PAGE_JS),
    ])
}
