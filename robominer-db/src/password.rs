use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("password hashing should succeed")
        .to_string()
}

pub fn verify_argon2_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn is_legacy_password_hash(password_hash: &str) -> bool {
    password_hash.starts_with("sha256:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_password_hashes_verify_successfully() {
        let hash = hash_password("test-password");

        assert!(hash.starts_with("$argon2"));
        assert!(verify_argon2_password("test-password", &hash));
        assert!(!verify_argon2_password("wrong-password", &hash));
    }

    #[test]
    fn legacy_hashes_are_detected() {
        assert!(is_legacy_password_hash("sha256:abc123:deadbeef"));
    }
}
