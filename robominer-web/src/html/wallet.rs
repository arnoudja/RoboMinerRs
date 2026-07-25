use super::format::escape_html;

/// One ore line in a page wallet strip.
pub(crate) struct WalletOreLine<'a> {
    pub ore_id: i64,
    pub ore_name: &'a str,
    pub amount: i32,
    pub max_allowed: i32,
}

/// Shared wallet ore `<ul>` used by shop and mining queue (and similar pages).
pub(crate) fn render_wallet_ore_list(
    body: &mut String,
    class_prefix: &str,
    assets: &[WalletOreLine<'_>],
    empty_message: &str,
    wrap_amount_row: bool,
    mut extra_item_html: impl FnMut(&WalletOreLine<'_>) -> String,
) {
    if assets.is_empty() {
        body.push_str(&format!(
            r#"<p class="{class_prefix}-empty">{}</p>"#,
            escape_html(empty_message)
        ));
        return;
    }

    body.push_str(&format!(r#"<ul class="{class_prefix}-list">"#));
    for asset in assets {
        let balance_class = if asset.amount >= asset.max_allowed {
            format!("{class_prefix}-full")
        } else {
            format!("{class_prefix}-ok")
        };
        let amounts = format!(
            r#"<span class="{class_prefix}-ore">{}</span><span class="{class_prefix}-amount">{}/{}</span>"#,
            escape_html(asset.ore_name),
            asset.amount,
            asset.max_allowed,
        );
        let amounts = if wrap_amount_row {
            format!(r#"<div class="{class_prefix}-item-row">{amounts}</div>"#)
        } else {
            amounts
        };
        let extra = extra_item_html(asset);
        body.push_str(&format!(
            r#"<li class="{class_prefix}-item {balance_class}">{amounts}{extra}</li>"#
        ));
    }
    body.push_str("</ul>");
}
