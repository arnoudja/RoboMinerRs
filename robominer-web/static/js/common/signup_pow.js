'use strict';

(function() {
    var form = document.getElementById('signupForm');
    if (!form || !window.crypto || !window.crypto.subtle) {
        return;
    }

    var difficultyBits = parseInt(form.getAttribute('data-pow-difficulty-bits') || '16', 10);
    if (!Number.isFinite(difficultyBits) || difficultyBits < 1) {
        difficultyBits = 16;
    }

    function hex(buffer) {
        return Array.from(new Uint8Array(buffer)).map(function(b) {
            return b.toString(16).padStart(2, '0');
        }).join('');
    }

    function leadingZeroBits(hexDigest) {
        var bits = 0;
        for (var i = 0; i < hexDigest.length; i++) {
            var nibble = parseInt(hexDigest.charAt(i), 16);
            if (nibble === 0) {
                bits += 4;
                continue;
            }
            if (nibble < 2) {
                return bits + 3;
            }
            if (nibble < 4) {
                return bits + 2;
            }
            if (nibble < 8) {
                return bits + 1;
            }
            return bits;
        }
        return bits;
    }

    form.addEventListener('submit', function(event) {
        if (form.dataset.powReady === '1') {
            return;
        }
        event.preventDefault();
        var csrf = form.querySelector('input[name="csrfToken"]');
        var nonceInput = form.querySelector('input[name="signupPowNonce"]');
        if (!csrf || !nonceInput) {
            form.submit();
            return;
        }
        var challenge = csrf.value;
        var nonce = 0;

        function step() {
            var candidate = String(nonce);
            var payload = new TextEncoder().encode(challenge + ':' + candidate);
            return window.crypto.subtle.digest('SHA-256', payload).then(function(digest) {
                if (leadingZeroBits(hex(digest)) >= difficultyBits) {
                    nonceInput.value = candidate;
                    form.dataset.powReady = '1';
                    if (typeof form.requestSubmit === 'function') {
                        form.requestSubmit();
                    } else {
                        form.submit();
                    }
                    return;
                }
                nonce += 1;
                // Yield every batch so the UI stays responsive on slow devices.
                if (nonce % 64 === 0) {
                    return new Promise(function(resolve) {
                        setTimeout(resolve, 0);
                    }).then(step);
                }
                return step();
            });
        }

        step().catch(function() {
            form.submit();
        });
    });
})();
