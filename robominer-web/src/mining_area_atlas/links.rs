use crate::html::escape_html;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MiningAreaAtlasLinkTarget {
    StandalonePage,
}

pub(crate) fn mining_area_atlas_url(
    _target: MiningAreaAtlasLinkTarget,
    ore_id: Option<i64>,
    affordable_only: bool,
) -> String {
    let mut params = Vec::new();
    if let Some(ore_id) = ore_id {
        params.push("sort=ore".to_string());
        params.push(format!("oreId={ore_id}"));
    }
    if affordable_only {
        params.push("affordable=1".to_string());
    }
    let base = "miningAreaOverview";
    if params.is_empty() {
        base.to_string()
    } else {
        format!("{base}?{}", params.join("&"))
    }
}

pub(crate) fn mining_area_atlas_url_for_ore(
    ore_id: i64,
    target: MiningAreaAtlasLinkTarget,
) -> String {
    mining_area_atlas_url(target, Some(ore_id), false)
}

pub(crate) fn mining_area_atlas_ore_link_label(ore_name: &str) -> String {
    format!("Areas rich in {ore_name}")
}

pub(crate) fn render_mining_area_atlas_ore_link(
    ore_id: i64,
    ore_name: &str,
    target: MiningAreaAtlasLinkTarget,
    class: &str,
) -> String {
    format!(
        r#"<a class="{class}" href="{}">{}</a>"#,
        escape_html(&mining_area_atlas_url_for_ore(ore_id, target)),
        escape_html(&mining_area_atlas_ore_link_label(ore_name)),
    )
}
