/// Mining-queue page scripts: shared helpers, clear-wallet preview, then page logic.
use crate::static_assets::script_src_tags;

const URL_QUERY_JS: &str = include_str!("../../static/js/common/url_query.js");
const SESSION_STORE_JS: &str = include_str!("../../static/js/common/session_store.js");
const CLEAR_WALLET_JS: &str = include_str!("../../static/js/mining_queue/clear_wallet.js");
const MINING_QUEUE_PAGE_JS: &str = include_str!("../../static/js/mining_queue/page.js");

pub(super) fn mining_queue_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/common/session_store.js", SESSION_STORE_JS),
        ("js/mining_queue/clear_wallet.js", CLEAR_WALLET_JS),
        ("js/mining_queue/page.js", MINING_QUEUE_PAGE_JS),
    ])
}
