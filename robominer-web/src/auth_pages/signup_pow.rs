//! Signup proof-of-work: SHA-256(challenge || ":" || nonce) with leading zero bits.

use sha2::{Digest, Sha256};

use crate::static_assets::script_src_tag;

pub(crate) const POW_NONCE_FIELD: &str = "signupPowNonce";

/// Leading zero bits required in SHA-256(challenge || ":" || nonce).
/// 16 bits is a light interactive cost (~65k hashes average).
pub(crate) const POW_DIFFICULTY_BITS: u32 = 16;

const SIGNUP_POW_JS: &str = include_str!("../../static/js/common/signup_pow.js");

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

/// External script tag for the signup PoW solver (CSP `script-src 'self'`).
pub(crate) fn signup_pow_script() -> String {
    script_src_tag("js/common/signup_pow.js", SIGNUP_POW_JS)
}

#[cfg(test)]
mod tests {
    use super::{POW_DIFFICULTY_BITS, SIGNUP_POW_JS, leading_zero_bits, verify_solution};
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

    #[test]
    fn static_script_defaults_match_server_difficulty() {
        assert!(
            SIGNUP_POW_JS.contains(&format!("|| '{POW_DIFFICULTY_BITS}'")),
            "signup_pow.js default difficulty must stay in sync with POW_DIFFICULTY_BITS"
        );
    }
}
