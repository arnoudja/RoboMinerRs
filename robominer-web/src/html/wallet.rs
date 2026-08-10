use super::format::escape_html;

/// One ore line in a page wallet strip.
pub(crate) struct WalletOreLine<'a> {
    pub ore_id: i64,
    pub ore_name: &'a str,
    pub amount: i32,
    pub max_allowed: i32,
    pub depot_max_allowed: i32,
}

/// Parameters for [`render_wallet_strip_section`].
pub(crate) struct WalletStripSection<'a> {
    pub section_class: &'a str,
    pub aria_label: &'a str,
    pub heading_class: &'a str,
    pub heading_markup: &'a str,
    pub middle_markup: &'a str,
    pub assets: &'a [WalletOreLine<'a>],
    pub empty_message: &'a str,
    pub wrap_amount_row: bool,
    pub item_row_class: Option<&'a str>,
}

/// Shared wallet strip section wrapper used by shop, mining queue, and similar pages.
pub(crate) fn render_wallet_strip_section(
    body: &mut String,
    section: &WalletStripSection<'_>,
    mut extra_item_html: impl FnMut(&WalletOreLine<'_>) -> String,
) {
    body.push_str(&format!(
        r#"<section class="{}" aria-label="{}">"#,
        section.section_class,
        escape_html(section.aria_label)
    ));
    body.push_str(&format!(r#"<div class="{}">"#, section.heading_class));
    body.push_str(section.heading_markup);
    body.push_str("</div>");
    body.push_str(section.middle_markup);
    render_wallet_ore_list(
        body,
        section.assets,
        section.empty_message,
        section.wrap_amount_row,
        section.item_row_class,
        &mut extra_item_html,
    );
    body.push_str("</section>");
}

fn render_wallet_ore_list(
    body: &mut String,
    assets: &[WalletOreLine<'_>],
    empty_message: &str,
    wrap_amount_row: bool,
    item_row_class: Option<&str>,
    extra_item_html: &mut dyn FnMut(&WalletOreLine<'_>) -> String,
) {
    const CLASS_PREFIX: &str = "page-wallet";

    if assets.is_empty() {
        body.push_str(&format!(
            r#"<p class="{CLASS_PREFIX}-empty">{}</p>"#,
            escape_html(empty_message)
        ));
        return;
    }

    body.push_str(&format!(r#"<ul class="{CLASS_PREFIX}-list">"#));
    for asset in assets {
        let balance_class = if asset.amount >= asset.max_allowed {
            format!("{CLASS_PREFIX}-full")
        } else {
            format!("{CLASS_PREFIX}-ok")
        };
        let amounts = format!(
            r#"<span class="{CLASS_PREFIX}-ore">{}</span><span class="{CLASS_PREFIX}-amount">{}/{}</span>"#,
            escape_html(asset.ore_name),
            asset.amount,
            asset.max_allowed,
        );
        let primary_class = item_row_class.unwrap_or("page-wallet-primary");
        let amounts = if wrap_amount_row {
            format!(r#"<div class="{primary_class}">{amounts}</div>"#)
        } else {
            format!(r#"<div class="{CLASS_PREFIX}-primary">{amounts}</div>"#)
        };
        let depot = if asset.depot_max_allowed > 0 {
            format!(
                r#"<span class="{CLASS_PREFIX}-depot">depot {}</span>"#,
                asset.depot_max_allowed
            )
        } else {
            String::new()
        };
        let extra = extra_item_html(asset);
        body.push_str(&format!(
            r#"<li class="{CLASS_PREFIX}-item {balance_class}">{amounts}{depot}{extra}</li>"#
        ));
    }
    body.push_str("</ul>");
}
