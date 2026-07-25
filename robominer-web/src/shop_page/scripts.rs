/// Shop page script served from `static/js/shop/page.js`.
use crate::static_assets::script_src_tag;

const SHOP_PAGE_JS: &str = include_str!("../../static/js/shop/page.js");

pub(super) fn shop_page_script_tag() -> String {
    script_src_tag("js/shop/page.js", SHOP_PAGE_JS)
}
