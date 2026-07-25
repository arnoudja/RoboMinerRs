/// Mining-results page scripts: shared URL helper, then page logic.
use crate::static_assets::page_scripts_with_url_query;

const MINING_RESULTS_PAGE_JS: &str = include_str!("../../static/js/mining_results/page.js");

pub(super) fn mining_results_page_script_tag() -> String {
    page_scripts_with_url_query("js/mining_results/page.js", MINING_RESULTS_PAGE_JS)
}
