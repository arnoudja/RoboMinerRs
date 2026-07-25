use crate::static_assets::page_scripts_with_url_query;

const MINING_AREA_ATLAS_PAGE_JS: &str = include_str!("../../static/js/mining_area_atlas/page.js");

pub(crate) fn render_mining_area_atlas_script(body: &mut String) {
    body.push_str(&page_scripts_with_url_query(
        "js/mining_area_atlas/page.js",
        MINING_AREA_ATLAS_PAGE_JS,
    ));
}
