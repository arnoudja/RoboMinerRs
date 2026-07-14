use crate::html::layout;
use crate::mining_area_atlas::{MiningAreaAtlasMode, render_mining_area_atlas};
use crate::mining_area_overview_page::MiningAreaOverviewPageState;

pub(super) fn render_mining_area_overview_page(
    username: String,
    hud: Option<&str>,
    state: &MiningAreaOverviewPageState,
) -> String {
    let mut body = String::from(r#"<div class="mining-area-atlas-page">"#);
    render_mining_area_atlas(
        &mut body,
        MiningAreaAtlasMode::StandalonePage,
        &state.ores,
        &state.areas,
        &state.percentages,
        &state.costs,
        &state.ore_assets,
    );
    body.push_str("</div>");

    layout(
        "RoboMiner - Mining area atlas",
        "miningAreaOverview",
        &username,
        hud,
        &body,
    )
}
