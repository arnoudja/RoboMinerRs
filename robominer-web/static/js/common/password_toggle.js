(function() {
    function toggleAuthPasswordVisibility(button) {
        var fieldId = button.getAttribute('data-target');
        var input = document.getElementById(fieldId);
        if (!input) {
            return;
        }
        var showing = input.type === 'text';
        input.type = showing ? 'password' : 'text';
        button.textContent = showing ? 'Show' : 'Hide';
        button.setAttribute('aria-pressed', showing ? 'false' : 'true');
        button.setAttribute('aria-label', showing ? 'Show password' : 'Hide password');
    }

    var authPasswordToggles = document.querySelectorAll('.auth-password-toggle');
    for (var index = 0; index < authPasswordToggles.length; index += 1) {
        authPasswordToggles[index].addEventListener('click', function(event) {
            toggleAuthPasswordVisibility(event.currentTarget);
        });
    }
})();
