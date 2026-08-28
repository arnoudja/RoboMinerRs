/// Mining-queue page scripts: shared helpers, clear-wallet preview, then page logic.
use crate::static_assets::{FILTER_RESTORE_JS, SESSION_STORE_JS, URL_QUERY_JS, script_src_tags};

const CLEAR_WALLET_JS: &str = include_str!("../../static/js/mining_queue/clear_wallet.js");
const MINING_QUEUE_VIEW_JS: &str = include_str!("../../static/js/mining_queue/view.js");
const MINING_QUEUE_CLAIM_POLL_JS: &str = include_str!("../../static/js/mining_queue/claim_poll.js");
const MINING_QUEUE_ACTIONS_JS: &str = include_str!("../../static/js/mining_queue/actions.js");
const MINING_QUEUE_PAGE_JS: &str = include_str!("../../static/js/mining_queue/page.js");

pub(super) fn mining_queue_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/common/session_store.js", SESSION_STORE_JS),
        ("js/common/filter_restore.js", FILTER_RESTORE_JS),
        ("js/mining_queue/clear_wallet.js", CLEAR_WALLET_JS),
        ("js/mining_queue/view.js", MINING_QUEUE_VIEW_JS),
        ("js/mining_queue/claim_poll.js", MINING_QUEUE_CLAIM_POLL_JS),
        ("js/mining_queue/actions.js", MINING_QUEUE_ACTIONS_JS),
        ("js/mining_queue/page.js", MINING_QUEUE_PAGE_JS),
    ])
}
