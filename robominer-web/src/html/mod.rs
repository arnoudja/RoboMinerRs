mod area_filter;
mod auth_fields;
mod banners;
mod csrf;
mod format;
mod layout;
mod ore_costs;
mod shell;
mod wallet;

#[cfg(test)]
mod assert;
#[cfg(test)]
mod tests;

pub(crate) use area_filter::{AreaFilterOption, render_area_filter_select};
pub(crate) use auth_fields::{render_password_field, render_password_toggle_script};
pub(crate) use banners::{render_claimed_ore_rewards_banner, render_status_banner};
pub(crate) use csrf::inject_csrf_tokens;
pub(crate) use format::{
    escape_html, format_period, format_relative_time_millis, format_utc_millis, selected_attr,
};
pub(crate) use layout::layout;
pub(crate) use ore_costs::{format_ore_shortfall, ore_costs_affordable, render_ore_entry_costs};
pub(crate) use shell::page_footer;
pub(crate) use wallet::{WalletOreLine, WalletStripSection, render_wallet_strip_section};

#[cfg(test)]
pub(crate) use assert::{
    assert_contains_all, assert_html_contains, assert_html_has_class, assert_html_not_contains,
};
