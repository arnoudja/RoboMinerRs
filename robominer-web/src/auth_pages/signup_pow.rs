//! Signup proof-of-work: SHA-256(challenge || ":" || nonce) with leading zero bits.

use sha2::{Digest, Sha256};

pub(crate) const POW_NONCE_FIELD: &str = "signupPowNonce";

/// Leading zero bits required in SHA-256(challenge || ":" || nonce).
/// 16 bits is a light interactive cost (~65k hashes average).
pub(crate) const POW_DIFFICULTY_BITS: u32 = 16;

pub(crate) fn verify_solution(challenge: &str, nonce: &str) -> bool {
    if challenge.is_empty() || nonce.is_empty() || nonce.len() > 64 {
        return false;
    }
    let digest = Sha256::digest(format!("{challenge}:{nonce}").as_bytes());
    leading_zero_bits(&digest) >= POW_DIFFICULTY_BITS
}

/// Brute-force a nonce for tests / tooling. Panics if the search bound is exceeded.
#[cfg(any(test, debug_assertions))]
pub(crate) fn solve_challenge(challenge: &str) -> String {
    let mut nonce = 0u64;
    loop {
        let candidate = nonce.to_string();
        if verify_solution(challenge, &candidate) {
            return candidate;
        }
        nonce += 1;
        assert!(
            nonce < 1_000_000,
            "signup PoW search exceeded bound for challenge"
        );
    }
}

fn leading_zero_bits(digest: &[u8]) -> u32 {
    let mut bits = 0u32;
    for byte in digest {
        if *byte == 0 {
            bits += 8;
            continue;
        }
        bits += byte.leading_zeros();
        break;
    }
    bits
}

/// Inline script: solves PoW against the anonymous CSRF token before submit.
pub(crate) fn signup_pow_script() -> String {
    format!(
        r#"<script>
(function () {{
  var form = document.getElementById("signupForm");
  if (!form || !window.crypto || !window.crypto.subtle) {{ return; }}
  var difficultyBits = {POW_DIFFICULTY_BITS};
  function hex(buffer) {{
    return Array.from(new Uint8Array(buffer)).map(function (b) {{
      return b.toString(16).padStart(2, "0");
    }}).join("");
  }}
  function leadingZeroBits(hexDigest) {{
    var bits = 0;
    for (var i = 0; i < hexDigest.length; i++) {{
      var nibble = parseInt(hexDigest.charAt(i), 16);
      if (nibble === 0) {{ bits += 4; continue; }}
      if (nibble < 2) return bits + 3;
      if (nibble < 4) return bits + 2;
      if (nibble < 8) return bits + 1;
      return bits;
    }}
    return bits;
  }}
  form.addEventListener("submit", function (event) {{
    if (form.dataset.powReady === "1") {{ return; }}
    event.preventDefault();
    var csrf = form.querySelector('input[name="csrfToken"]');
    var nonceInput = form.querySelector('input[name="{POW_NONCE_FIELD}"]');
    if (!csrf || !nonceInput) {{ form.submit(); return; }}
    var challenge = csrf.value;
    var nonce = 0;
    function step() {{
      var candidate = String(nonce);
      var payload = new TextEncoder().encode(challenge + ":" + candidate);
      return window.crypto.subtle.digest("SHA-256", payload).then(function (digest) {{
        if (leadingZeroBits(hex(digest)) >= difficultyBits) {{
          nonceInput.value = candidate;
          form.dataset.powReady = "1";
          if (typeof form.requestSubmit === "function") {{ form.requestSubmit(); }}
          else {{ form.submit(); }}
          return;
        }}
        nonce += 1;
        return step();
      }});
    }}
    step().catch(function () {{ form.submit(); }});
  }});
}})();
</script>"#,
        POW_DIFFICULTY_BITS = POW_DIFFICULTY_BITS,
        POW_NONCE_FIELD = POW_NONCE_FIELD,
    )
}

#[cfg(test)]
mod tests {
    use super::{POW_DIFFICULTY_BITS, leading_zero_bits, verify_solution};
    use sha2::{Digest, Sha256};

    #[test]
    fn accepts_bruteforced_nonce_for_challenge() {
        let challenge = "test-challenge-token";
        let mut nonce = 0u64;
        let solution = loop {
            let candidate = nonce.to_string();
            let digest = Sha256::digest(format!("{challenge}:{candidate}").as_bytes());
            if leading_zero_bits(&digest) >= POW_DIFFICULTY_BITS {
                break candidate;
            }
            nonce += 1;
            assert!(nonce < 1_000_000, "PoW search exceeded bound");
        };
        assert!(verify_solution(challenge, &solution));
        assert!(!verify_solution(challenge, "0"));
    }
}
