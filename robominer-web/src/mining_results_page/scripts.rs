/// Mining-results page script served from `static/js/mining_results/page.js`.
use crate::static_assets::script_src_tag;

const MINING_RESULTS_PAGE_JS: &str = include_str!("../../static/js/mining_results/page.js");

pub(super) fn mining_results_page_script_tag() -> String {
    script_src_tag("js/mining_results/page.js", MINING_RESULTS_PAGE_JS)
}
