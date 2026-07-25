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
