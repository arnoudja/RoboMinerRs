/// Mining-queue page script served from `static/js/mining_queue/page.js`.
use crate::static_assets::script_src_tag;

const MINING_QUEUE_PAGE_JS: &str = include_str!("../../static/js/mining_queue/page.js");

pub(super) fn mining_queue_page_script_tag() -> String {
    script_src_tag("js/mining_queue/page.js", MINING_QUEUE_PAGE_JS)
}
