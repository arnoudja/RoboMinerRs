(function() {
    function toggleAuthPasswordVisibility(button) {
        const fieldId = button.getAttribute('data-target');
        const input = document.getElementById(fieldId);
        if (!input) {
            return;
        }
        const showing = input.type === 'text';
        input.type = showing ? 'password' : 'text';
        button.textContent = showing ? 'Show' : 'Hide';
        button.setAttribute('aria-pressed', showing ? 'false' : 'true');
        button.setAttribute('aria-label', showing ? 'Show password' : 'Hide password');
    }

    const authPasswordToggles = document.querySelectorAll('.auth-password-toggle');
    for (let index = 0; index < authPasswordToggles.length; index += 1) {
        authPasswordToggles[index].addEventListener('click', function(event) {
            toggleAuthPasswordVisibility(event.currentTarget);
        });
    }
})();
