'use strict';

(function() {
    const form = document.getElementById('signupForm');
    if (!form || !window.crypto || !window.crypto.subtle) {
        return;
    }

    let difficultyBits = parseInt(form.getAttribute('data-pow-difficulty-bits') || '16', 10);
    if (!Number.isFinite(difficultyBits) || difficultyBits < 1) {
        difficultyBits = 16;
    }

    function hex(buffer) {
        return Array.from(new Uint8Array(buffer)).map(function(b) {
            return b.toString(16).padStart(2, '0');
        }).join('');
    }

    function leadingZeroBits(hexDigest) {
        let bits = 0;
        for (let i = 0; i < hexDigest.length; i++) {
            const nibble = parseInt(hexDigest.charAt(i), 16);
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
        const csrf = form.querySelector('input[name="csrfToken"]');
        const nonceInput = form.querySelector('input[name="signupPowNonce"]');
        if (!csrf || !nonceInput) {
            form.submit();
            return;
        }
        const challenge = csrf.value;
        let nonce = 0;

        function step() {
            const candidate = String(nonce);
            const payload = new TextEncoder().encode(challenge + ':' + candidate);
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
