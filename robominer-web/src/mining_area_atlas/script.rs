use crate::static_assets::script_src_tags;

const URL_QUERY_JS: &str = include_str!("../../static/js/common/url_query.js");
const MINING_AREA_ATLAS_PAGE_JS: &str = include_str!("../../static/js/mining_area_atlas/page.js");

pub(crate) fn render_mining_area_atlas_script(body: &mut String) {
    body.push_str(&script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/mining_area_atlas/page.js", MINING_AREA_ATLAS_PAGE_JS),
    ]));
}
