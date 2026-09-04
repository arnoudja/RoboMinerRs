(function() {
    const toggle = document.getElementById('app-shell-nav-toggle');
    const label = document.querySelector('label[for="app-shell-nav-toggle"]');
    if (!toggle || !label) {
        return;
    }

    function syncExpanded() {
        label.setAttribute('aria-expanded', toggle.checked ? 'true' : 'false');
    }

    toggle.addEventListener('change', syncExpanded);
    syncExpanded();
})();
