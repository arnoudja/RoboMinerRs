use super::format::escape_html;
use super::shell::{app_shell_header, page_footer};
use crate::static_assets::{PageStylesheet, robominer_stylesheet_tags, script_src_tag};

const APP_DIALOG_JS: &str = include_str!("../../static/js/common/app_dialog.js");

pub(crate) fn layout(
    title: &str,
    current_form: &str,
    username: &str,
    hud_markup: Option<&str>,
    body: &str,
    styles: &[PageStylesheet],
) -> String {
    format!(
        r##"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
        {}
        <title>{}</title>
    </head>
    <body>
        <a class="app-shell-skip" href="#main-content">Skip to content</a>
        <div class="main">
            {}
            <div class="interface" id="main-content">
                {}
            </div>
            {}
        </div>
        {}
        {}
    </body>
</html>"##,
        robominer_stylesheet_tags(styles),
        escape_html(title),
        app_shell_header(current_form, username, hud_markup),
        body,
        page_footer(),
        app_dialog_markup(),
        script_src_tag("js/common/app_dialog.js", APP_DIALOG_JS)
    )
}

fn app_dialog_markup() -> &'static str {
    r#"<div id="robominerDialog" class="robominer-dialog" hidden>
    <button type="button" class="robominer-dialog-backdrop" id="robominerDialogBackdrop" aria-label="Close dialog"></button>
    <div class="robominer-dialog-panel" role="dialog" aria-modal="true" aria-labelledby="robominerDialogTitle" aria-describedby="robominerDialogMessage">
        <h2 id="robominerDialogTitle" class="robominer-dialog-title">Confirm</h2>
        <p id="robominerDialogMessage" class="robominer-dialog-message"></p>
        <div class="robominer-dialog-actions">
            <button type="button" id="robominerDialogCancel" class="robominer-dialog-btn robominer-dialog-btn-secondary">Cancel</button>
            <button type="button" id="robominerDialogAlt" class="robominer-dialog-btn robominer-dialog-btn-secondary" hidden>Other</button>
            <button type="button" id="robominerDialogConfirm" class="robominer-dialog-btn robominer-dialog-btn-primary">Confirm</button>
        </div>
    </div>
</div>"#
}
