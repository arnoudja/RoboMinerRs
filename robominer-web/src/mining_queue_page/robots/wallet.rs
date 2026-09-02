use crate::mining_queue_page::MiningQueuePageState;

pub(in crate::mining_queue_page) fn render_wallet_strip(
    body: &mut String,
    state: &MiningQueuePageState,
) {
    let assets: Vec<_> = state
        .ore_assets
        .iter()
        .map(|asset| crate::html::WalletOreLine {
            ore_id: asset.ore_id,
            ore_name: &asset.ore_name,
            amount: asset.amount,
            max_allowed: asset.max_allowed,
            depot_max_allowed: asset.depot_max_allowed,
        })
        .collect();
    crate::html::render_wallet_strip_section(
        body,
        &crate::html::WalletStripSection {
            section_class: "page-wallet mining-queue-wallet",
            aria_label: "Wallet and queue limits",
            heading_class: "mining-queue-wallet-heading",
            heading_markup: r#"<h1 class="mining-queue-page-title">Mining queue</h1>"#,
            middle_markup: "",
            assets: &assets,
            empty_message: "No ore in wallet yet.",
            wrap_amount_row: false,
            item_row_class: None,
        },
        |_| String::new(),
    );
}
