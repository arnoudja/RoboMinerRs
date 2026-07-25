/// Shop page scripts: shared URL/session helpers, then page logic.
use crate::static_assets::page_scripts_with_url_query_and_session;

const SHOP_PAGE_JS: &str = include_str!("../../static/js/shop/page.js");

pub(super) fn shop_page_script_tag() -> String {
    page_scripts_with_url_query_and_session("js/shop/page.js", SHOP_PAGE_JS)
}
