use std::collections::HashMap;

use super::ShopPageState;
use crate::help_pages;
use crate::html::{escape_html, layout, selected_attr};

use super::catalog::{render_shop_part_compact_card, render_shop_part_detail_panel};
use super::inventory::render_shop_inventory;
use crate::mining_area_atlas::{
    MiningAreaAtlasLinkTarget, mining_area_atlas_url, render_mining_area_atlas_ore_link,
};

pub(super) fn render_shop_page(
    username: String,
    hud: Option<&str>,
    state: &ShopPageState,
) -> String {
    let part_state_map: HashMap<i64, &robominer_db::ShopRobotPartStateRecord> = state
        .part_states
        .iter()
        .map(|state| (state.robot_part_id, state))
        .collect();
    let mut cost_map: HashMap<i64, Vec<&robominer_db::ShopRobotPartCostRecord>> = HashMap::new();
    for cost in &state.costs {
        cost_map.entry(cost.robot_part_id).or_default().push(cost);
    }
    for costs in cost_map.values_mut() {
        costs.sort_by_key(|cost| std::cmp::Reverse(cost.ore_id));
    }
    let ore_amount_map: HashMap<i64, i32> = state
        .ore_assets
        .iter()
        .map(|asset| (asset.ore_id, asset.amount))
        .collect();

    let filter_storage_key = format!(
        "robominer.shop.filterSelections.{}",
        username.replace([' ', '"', '\''], "_")
    );
    let mut body = String::from(&format!(
        r#"<div class="shop-page" data-filter-storage-key="{}">"#,
        escape_html(&filter_storage_key)
    ));
    render_shop_wallet_strip(&mut body, state);
    render_shop_message(&mut body, state);
    render_shop_filters(&mut body, state);

    body.push_str(r#"<div class="shop-deck">"#);
    body.push_str(r#"<section class="shop-catalog" aria-labelledby="shop-catalog-title">"#);
    body.push_str(r#"<h2 id="shop-catalog-title" class="shop-section-title">Catalog</h2>"#);
    body.push_str(r#"<div class="shop-part-cards">"#);

    let mut part_counters: HashMap<(i64, i64), i32> = HashMap::new();
    let mut matching_parts = 0;
    for part in &state.parts {
        let counter = part_counters
            .entry((part.type_id, part.tier_id))
            .or_insert(0);
        *counter += 1;
        if part.type_id == state.selected_part_type_id && part.tier_id == state.selected_tier_id {
            matching_parts += 1;
        }
        let empty_state;
        let part_state = if let Some(part_state) = part_state_map.get(&part.robot_part_id) {
            *part_state
        } else {
            empty_state = robominer_db::ShopRobotPartStateRecord {
                robot_part_id: part.robot_part_id,
                total_owned: 0,
                assigned: 0,
                unassigned: 0,
                can_buy: false,
                can_sell: false,
            };
            &empty_state
        };
        render_shop_part_compact_card(
            &mut body,
            part,
            part_state,
            cost_map
                .get(&part.robot_part_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &ore_amount_map,
            *counter,
            state,
        );
    }

    body.push_str(&format!(
        r#"<p id="shopCatalogEmpty" class="shop-empty shop-catalog-empty"{}>No parts match this category and quality.</p>"#,
        if matching_parts == 0 { "" } else { " hidden" }
    ));
    body.push_str("</div></section>");

    body.push_str(r#"<aside class="shop-detail" aria-labelledby="shop-detail-title">"#);
    body.push_str(r#"<h2 id="shop-detail-title" class="shop-section-title">Part details</h2>"#);
    body.push_str(r#"<div class="shop-detail-panels">"#);
    for part in &state.parts {
        let empty_state;
        let part_state = if let Some(part_state) = part_state_map.get(&part.robot_part_id) {
            *part_state
        } else {
            empty_state = robominer_db::ShopRobotPartStateRecord {
                robot_part_id: part.robot_part_id,
                total_owned: 0,
                assigned: 0,
                unassigned: 0,
                can_buy: false,
                can_sell: false,
            };
            &empty_state
        };
        render_shop_part_detail_panel(
            &mut body,
            part,
            part_state,
            cost_map
                .get(&part.robot_part_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &ore_amount_map,
            state,
        );
    }
    body.push_str("</div></aside></div>");

    render_shop_inventory(&mut body, state, &part_state_map);
    body.push_str(super::scripts::SHOP_PAGE_SCRIPT);
    body.push_str("</div>");

    layout("RoboMiner - Shop", "shop", &username, hud, &body)
}

fn render_shop_wallet_strip(body: &mut String, state: &ShopPageState) {
    body.push_str(r#"<section class="shop-wallet" aria-label="Wallet">"#);
    body.push_str(r#"<div class="shop-wallet-heading">"#);
    body.push_str(r#"<h1 class="shop-page-title">Parts shop</h1>"#);
    body.push_str("</div>");

    if state.ore_assets.is_empty() {
        body.push_str(r#"<p class="shop-wallet-empty">No ore in wallet yet.</p>"#);
    } else {
        body.push_str(r#"<ul class="shop-wallet-list">"#);
        for asset in &state.ore_assets {
            let balance_class = if asset.amount >= asset.max_allowed {
                "shop-wallet-full"
            } else {
                "shop-wallet-ok"
            };
            body.push_str(&format!(
                r#"<li class="shop-wallet-item {balance_class}"><div class="shop-wallet-item-row"><span class="shop-wallet-ore">{}</span><span class="shop-wallet-amount">{}/{}</span></div>{}</li>"#,
                escape_html(&asset.ore_name),
                asset.amount,
                asset.max_allowed,
                render_mining_area_atlas_ore_link(
                    asset.ore_id,
                    &asset.ore_name,
                    MiningAreaAtlasLinkTarget::StandalonePage,
                    "shop-atlas-link",
                ),
            ));
        }
        body.push_str("</ul>");
    }

    body.push_str("</section>");
}

fn render_shop_message(body: &mut String, state: &ShopPageState) {
    let Some(message) = &state.message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        "shop-banner shop-banner-error"
    } else {
        "shop-banner shop-banner-success"
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}

fn render_shop_filters(body: &mut String, state: &ShopPageState) {
    body.push_str(r#"<section class="shop-filters" aria-label="Catalog filters">"#);
    body.push_str(r#"<div class="shop-filter-form">"#);
    body.push_str(r#"<label class="shop-filter-label" for="robotPartTypeId">Category <select id="robotPartTypeId" name="selectedRobotPartTypeId" class="tableitem shop-filter-select">"#);
    for part_type in &state.part_types {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            part_type.id,
            selected_attr(part_type.id == state.selected_part_type_id),
            escape_html(&part_type.type_name)
        ));
    }
    body.push_str("</select></label>");
    body.push_str(r#"<label class="shop-filter-label" for="tierId">Quality <select id="tierId" name="selectedTierId" class="tableitem shop-filter-select">"#);
    for ore in &state.ores {
        body.push_str(&format!(
            r#"<option value="{}"{}>{} quality</option>"#,
            ore.id,
            selected_attr(ore.id == state.selected_tier_id),
            escape_html(&ore.ore_name)
        ));
    }
    body.push_str("</select></label></div>");
    body.push_str(&format!(
        r#"<p class="shop-atlas-helper">Need ore for parts? <a class="shop-atlas-link" href="{}">Compare all areas</a>.</p>"#,
        escape_html(&mining_area_atlas_url(
            MiningAreaAtlasLinkTarget::StandalonePage,
            None,
            false,
        )),
    ));
    body.push_str(&help_pages::render_page_help_hint(
        "What do capacity, power, and weight stats mean?",
        "helpMechanics",
        "Read the mechanics guide",
    ));
    body.push_str("</section>");
}
