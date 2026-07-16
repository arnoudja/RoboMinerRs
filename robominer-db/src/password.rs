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

/// Hash on a blocking thread so Argon2 does not stall the Tokio runtime.
pub async fn hash_password_async(password: String) -> String {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .expect("password hashing task should not panic")
}

/// Verify on a blocking thread so Argon2 does not stall the Tokio runtime.
pub async fn verify_argon2_password_async(password: String, password_hash: String) -> bool {
    tokio::task::spawn_blocking(move || verify_argon2_password(&password, &password_hash))
        .await
        .expect("password verification task should not panic")
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

    #[tokio::test(flavor = "current_thread")]
    async fn async_wrappers_match_sync_hash_verify() {
        let hash = hash_password_async("async-password".to_string()).await;
        assert!(hash.starts_with("$argon2"));
        assert!(verify_argon2_password_async("async-password".to_string(), hash.clone()).await);
        assert!(!verify_argon2_password_async("wrong-password".to_string(), hash).await);
    }

    #[test]
    fn legacy_hashes_are_detected() {
        assert!(is_legacy_password_hash("sha256:abc123:deadbeef"));
    }
}
