(function() {
    if (window.RoboMinerAreaFilterBound) {
        return;
    }
    window.RoboMinerAreaFilterBound = true;

    document.addEventListener('change', function(event) {
        var select = event.target.closest('select[data-area-filter-nav="true"]');
        if (!select || !select.value) {
            return;
        }
        window.location = select.value;
    });
})();
