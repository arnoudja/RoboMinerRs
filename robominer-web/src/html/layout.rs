use super::format::escape_html;
use super::shell::{app_shell_header, page_footer};

pub(crate) fn layout(
    title: &str,
    current_form: &str,
    username: &str,
    hud_markup: Option<&str>,
    body: &str,
) -> String {
    format!(
        r##"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
        <link rel="stylesheet" type="text/css" href="css/robominer.css">
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
        escape_html(title),
        app_shell_header(current_form, username, hud_markup),
        body,
        page_footer(),
        app_dialog_markup(),
        app_dialog_script()
    )
}

fn app_dialog_markup() -> &'static str {
    r#"<div id="robominerDialog" class="robominer-dialog" hidden>
    <button type="button" class="robominer-dialog-backdrop" id="robominerDialogBackdrop" aria-label="Close dialog"></button>
    <div class="robominer-dialog-panel" role="dialog" aria-modal="true" aria-labelledby="robominerDialogTitle">
        <h2 id="robominerDialogTitle" class="robominer-dialog-title">Confirm</h2>
        <p id="robominerDialogMessage" class="robominer-dialog-message"></p>
        <div class="robominer-dialog-actions">
            <button type="button" id="robominerDialogCancel" class="robominer-dialog-btn robominer-dialog-btn-secondary">Cancel</button>
            <button type="button" id="robominerDialogConfirm" class="robominer-dialog-btn robominer-dialog-btn-primary">Confirm</button>
        </div>
    </div>
</div>"#
}

fn app_dialog_script() -> &'static str {
    r#"<script>
(function() {
    var dialog = document.getElementById('robominerDialog');
    var title = document.getElementById('robominerDialogTitle');
    var message = document.getElementById('robominerDialogMessage');
    var cancelButton = document.getElementById('robominerDialogCancel');
    var confirmButton = document.getElementById('robominerDialogConfirm');
    var backdrop = document.getElementById('robominerDialogBackdrop');
    if (!dialog || !title || !message || !cancelButton || !confirmButton || !backdrop) {
        return;
    }

    var pendingCallback = null;
    var alertMode = false;
    var lastFocusedElement = null;

    function finish(result) {
        dialog.hidden = true;
        document.body.classList.remove('robominer-dialog-open');
        var callback = pendingCallback;
        pendingCallback = null;
        alertMode = false;
        if (lastFocusedElement && typeof lastFocusedElement.focus === 'function') {
            lastFocusedElement.focus();
        }
        lastFocusedElement = null;
        if (callback) {
            callback(result);
        }
    }

    function openDialog(options) {
        alertMode = !!options.alert;
        title.textContent = options.title;
        message.textContent = options.message;
        cancelButton.hidden = alertMode;
        confirmButton.textContent = options.confirmLabel;
        pendingCallback = options.onResult;
        lastFocusedElement = document.activeElement;
        dialog.hidden = false;
        document.body.classList.add('robominer-dialog-open');
        confirmButton.focus();
    }

    window.robominerConfirm = function(dialogMessage, onResult) {
        openDialog({
            alert: false,
            title: 'Confirm',
            message: dialogMessage,
            confirmLabel: 'Confirm',
            onResult: onResult
        });
    };

    window.robominerAlert = function(dialogMessage, onDismiss) {
        openDialog({
            alert: true,
            title: 'Notice',
            message: dialogMessage,
            confirmLabel: 'OK',
            onResult: onDismiss || null
        });
    };

    cancelButton.addEventListener('click', function() {
        finish(false);
    });
    backdrop.addEventListener('click', function() {
        if (!alertMode) {
            finish(false);
        }
    });
    confirmButton.addEventListener('click', function() {
        finish(true);
    });
    document.addEventListener('keydown', function(event) {
        if (dialog.hidden) {
            return;
        }
        if (event.key === 'Escape') {
            event.preventDefault();
            finish(alertMode ? true : false);
        }
    });
})();
</script>"#
}
