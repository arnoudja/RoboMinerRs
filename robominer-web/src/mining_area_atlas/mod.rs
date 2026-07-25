mod costs;
mod links;
mod markup;
mod script;

#[cfg(test)]
mod tests;

pub(crate) use links::{
    MiningAreaAtlasLinkTarget, mining_area_atlas_url, render_mining_area_atlas_ore_link,
};
pub(crate) use markup::{MiningAreaAtlasMode, render_mining_area_atlas};
