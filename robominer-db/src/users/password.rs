use sqlx::MySqlPool;

use crate::password::{hash_password_async, is_legacy_password_hash, verify_argon2_password_async};

pub(super) fn valid_username(username: &str) -> bool {
    (3..=255).contains(&username.len())
        && username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

pub(super) fn valid_email(email: &str) -> bool {
    let email = email.trim();
    if !(3..=254).contains(&email.len()) || email.contains(char::is_whitespace) {
        return false;
    }

    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    if local.is_empty()
        || domain.is_empty()
        || local.len() > 64
        || domain.contains('@')
        || local.starts_with('.')
        || local.ends_with('.')
        || local.contains("..")
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
        || domain.starts_with('-')
        || domain.ends_with('-')
    {
        return false;
    }

    if !local
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '%' | '+' | '-'))
    {
        return false;
    }

    if !domain
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'))
    {
        return false;
    }

    let Some((host, tld)) = domain.rsplit_once('.') else {
        return false;
    };
    !host.is_empty() && tld.len() >= 2 && tld.chars().all(|ch| ch.is_ascii_alphabetic())
}

pub(super) fn valid_password(password: &str) -> bool {
    password.len() >= 8
}

pub(super) async fn verify_password_hash(
    pool: &MySqlPool,
    password: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    if is_legacy_password_hash(password_hash) {
        return verify_legacy_password_hash(pool, password, password_hash).await;
    }

    Ok(verify_argon2_password_async(password.to_owned(), password_hash.to_owned()).await?)
}

async fn verify_legacy_password_hash(
    pool: &MySqlPool,
    password: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    let Some(rest) = password_hash.strip_prefix("sha256:") else {
        return Ok(false);
    };
    let Some((salt, expected_digest)) = rest.split_once(':') else {
        return Ok(false);
    };

    let digest: String = sqlx::query_scalar("SELECT SHA2(CONCAT(?, ?), 256)")
        .bind(salt)
        .bind(password)
        .fetch_one(pool)
        .await?;

    Ok(digest.eq_ignore_ascii_case(expected_digest))
}

pub(super) async fn maybe_upgrade_password_hash(
    password: &str,
    password_hash: &str,
) -> Result<Option<String>, sqlx::Error> {
    if !is_legacy_password_hash(password_hash) {
        return Ok(None);
    }

    Ok(Some(hash_password_async(password.to_owned()).await?))
}

pub(super) async fn write_password_hash(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE User SET password = ? WHERE id = ?")
        .bind(password_hash)
        .bind(user_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{valid_email, valid_password, valid_username};

    #[test]
    fn valid_username_requires_three_to_255_alphanumeric_or_separator_chars() {
        assert!(valid_username("abc"));
        assert!(valid_username("user_name-1"));
        assert!(!valid_username("ab"));
        assert!(!valid_username("bad name"));
    }

    #[test]
    fn valid_email_requires_local_and_domain_with_tld() {
        assert!(valid_email("player@example.invalid"));
        assert!(valid_email("a.b+tag@example.com"));
        assert!(valid_email("  user_name%test@sub.example.org  "));

        assert!(!valid_email(""));
        assert!(!valid_email("missing-at.example"));
        assert!(!valid_email("@example.com"));
        assert!(!valid_email("user@"));
        assert!(!valid_email("user@@example.com"));
        assert!(!valid_email("user@example"));
        assert!(!valid_email("user @example.com"));
        assert!(!valid_email("user@exam ple.com"));
        assert!(!valid_email("user@.example.com"));
        assert!(!valid_email("user@example."));
        assert!(!valid_email(".user@example.com"));
        assert!(!valid_email("user.@example.com"));
        assert!(!valid_email("us..er@example.com"));
        assert!(!valid_email("user@-example.com"));
        assert!(!valid_email("user@example.c"));
        assert!(!valid_email("user@example.123"));
    }

    #[test]
    fn valid_password_requires_at_least_eight_characters() {
        assert!(valid_password("12345678"));
        assert!(!valid_password("short"));
    }
}
