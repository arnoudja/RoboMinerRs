/// Mining-queue page scripts: shared URL/session helpers, then page logic.
use crate::static_assets::page_scripts_with_url_query_and_session;

const MINING_QUEUE_PAGE_JS: &str = include_str!("../../static/js/mining_queue/page.js");

pub(super) fn mining_queue_page_script_tag() -> String {
    page_scripts_with_url_query_and_session("js/mining_queue/page.js", MINING_QUEUE_PAGE_JS)
}
