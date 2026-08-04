/**
 * App dialogs:
 * - robominerConfirm(message, cb) -> cb(true|false)
 * - robominerConfirmChoice(message, {confirmLabel, altLabel}, cb) -> cb('confirm'|'alt'|false)
 * - robominerAlert(message, cb?) -> cb(true) on dismiss
 */
(function() {
    var dialog = document.getElementById('robominerDialog');
    var title = document.getElementById('robominerDialogTitle');
    var message = document.getElementById('robominerDialogMessage');
    var cancelButton = document.getElementById('robominerDialogCancel');
    var altButton = document.getElementById('robominerDialogAlt');
    var confirmButton = document.getElementById('robominerDialogConfirm');
    var backdrop = document.getElementById('robominerDialogBackdrop');
    if (!dialog || !title || !message || !cancelButton || !altButton || !confirmButton || !backdrop) {
        return;
    }

    var panel = dialog.querySelector('.robominer-dialog-panel');
    if (panel && !panel.getAttribute('aria-describedby')) {
        panel.setAttribute('aria-describedby', 'robominerDialogMessage');
    }

    var pendingCallback = null;
    var alertMode = false;
    var choiceMode = false;
    var lastFocusedElement = null;

    function finish(result) {
        dialog.hidden = true;
        document.body.classList.remove('robominer-dialog-open');
        var callback = pendingCallback;
        pendingCallback = null;
        alertMode = false;
        choiceMode = false;
        altButton.hidden = true;
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
        choiceMode = !!options.choice;
        title.textContent = options.title;
        message.textContent = options.message;
        cancelButton.hidden = alertMode;
        confirmButton.textContent = options.confirmLabel;
        if (choiceMode) {
            altButton.hidden = false;
            altButton.textContent = options.altLabel || 'Other';
        } else {
            altButton.hidden = true;
        }
        pendingCallback = options.onResult;
        lastFocusedElement = document.activeElement;
        dialog.hidden = false;
        document.body.classList.add('robominer-dialog-open');
        confirmButton.focus();
    }

    window.robominerConfirm = function(dialogMessage, onResult) {
        openDialog({
            alert: false,
            choice: false,
            title: 'Confirm',
            message: dialogMessage,
            confirmLabel: 'Confirm',
            onResult: onResult
        });
    };

    window.robominerConfirmChoice = function(dialogMessage, labels, onResult) {
        openDialog({
            alert: false,
            choice: true,
            title: 'Confirm',
            message: dialogMessage,
            confirmLabel: (labels && labels.confirmLabel) || 'Confirm',
            altLabel: (labels && labels.altLabel) || 'Other',
            onResult: onResult
        });
    };

    window.robominerAlert = function(dialogMessage, onDismiss) {
        openDialog({
            alert: true,
            choice: false,
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
    altButton.addEventListener('click', function() {
        if (choiceMode) {
            finish('alt');
        }
    });
    confirmButton.addEventListener('click', function() {
        finish(choiceMode ? 'confirm' : true);
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
