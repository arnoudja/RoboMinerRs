use crate::password::verify_argon2_password_async;

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
    password: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    Ok(verify_argon2_password_async(password.to_owned(), password_hash.to_owned()).await?)
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
