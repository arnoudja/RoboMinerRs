/// Mining-results page scripts: shared URL helper, then page logic.
use crate::static_assets::script_src_tags;

const URL_QUERY_JS: &str = include_str!("../../static/js/common/url_query.js");
const MINING_RESULTS_PAGE_JS: &str = include_str!("../../static/js/mining_results/page.js");

pub(super) fn mining_results_page_script_tag() -> String {
    script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/mining_results/page.js", MINING_RESULTS_PAGE_JS),
    ])
}
