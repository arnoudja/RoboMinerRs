mod banners;
mod csrf;
mod format;
mod layout;
mod shell;

#[cfg(test)]
mod tests;

pub(crate) use banners::render_claimed_ore_rewards_banner;
pub(crate) use csrf::inject_csrf_tokens;
pub(crate) use format::{
    escape_html, escape_js_string, format_period, format_relative_time_millis, format_utc_millis,
    selected_attr,
};
pub(crate) use layout::layout;
pub(crate) use shell::page_footer;
