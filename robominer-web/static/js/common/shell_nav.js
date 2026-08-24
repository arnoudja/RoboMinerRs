(function() {
    var toggle = document.getElementById('app-shell-nav-toggle');
    var label = document.querySelector('label[for="app-shell-nav-toggle"]');
    if (!toggle || !label) {
        return;
    }

    function syncExpanded() {
        label.setAttribute('aria-expanded', toggle.checked ? 'true' : 'false');
    }

    toggle.addEventListener('change', syncExpanded);
    syncExpanded();
})();
