use sqlx::MySqlPool;

use crate::achievements::claim_achievement_step_in_transaction;
use crate::password::{hash_password, is_legacy_password_hash, verify_argon2_password};
use crate::{
    ClaimAchievementStepRequest, CreateUserRejection, CreateUserRequest, CreatedUser,
    UpdateUserAccountRejection, UpdateUserAccountRequest, UpdatedUserAccount, UserRecord,
    VerifiedLogin, VerifyLoginRejection, VerifyLoginRequest, VerifyUserPasswordRequest,
};

pub async fn get_user_by_id(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, String, String, i32, i32)>(
        "SELECT id, username, email, password, achievementPoints, miningQueueSize \
         FROM User \
         WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(id, username, email, password_hash, achievement_points, mining_queue_size)| {
                UserRecord {
                    id,
                    username,
                    email,
                    password_hash,
                    achievement_points,
                    mining_queue_size,
                }
            },
        )
    })
}

pub async fn create_user(
    pool: &MySqlPool,
    request: CreateUserRequest,
) -> Result<Result<CreatedUser, CreateUserRejection>, sqlx::Error> {
    if !valid_username(&request.username) {
        return Ok(Err(CreateUserRejection::InvalidUsername));
    }
    if !valid_email(&request.email) {
        return Ok(Err(CreateUserRejection::InvalidEmail));
    }
    if !valid_password(&request.password) {
        return Ok(Err(CreateUserRejection::InvalidPassword));
    }

    let mut transaction = pool.begin().await?;

    let duplicate_username: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE username = ? LIMIT 1")
            .bind(&request.username)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_username.is_some() {
        transaction.rollback().await?;
        return Ok(Err(CreateUserRejection::DuplicateUsername));
    }

    let duplicate_email: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE email = ? LIMIT 1")
            .bind(&request.email)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_email.is_some() {
        transaction.rollback().await?;
        return Ok(Err(CreateUserRejection::DuplicateEmail));
    }

    let password_hash = hash_password(&request.password);
    let user_result = sqlx::query(
        "INSERT INTO User \
         (username, email, password, achievementPoints, miningQueueSize) \
         VALUES (?, ?, ?, 0, 0)",
    )
    .bind(&request.username)
    .bind(&request.email)
    .bind(password_hash)
    .execute(&mut *transaction)
    .await?;
    let user_id = user_result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO UserAchievement (userId, achievementId, stepsClaimed) \
         VALUES (?, 1, 0)",
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await?;

    match claim_achievement_step_in_transaction(
        &mut transaction,
        ClaimAchievementStepRequest {
            user_id,
            achievement_id: 1,
        },
    )
    .await?
    {
        Ok(_) => {
            transaction.commit().await?;
            Ok(Ok(CreatedUser { user_id }))
        }
        Err(rejection) => {
            transaction.rollback().await?;
            Ok(Err(CreateUserRejection::InitialAchievementRejected(
                rejection,
            )))
        }
    }
}

pub async fn update_user_account(
    pool: &MySqlPool,
    request: UpdateUserAccountRequest,
) -> Result<Result<UpdatedUserAccount, UpdateUserAccountRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let user_exists: Option<i64> = sqlx::query_scalar("SELECT id FROM User WHERE id = ? LIMIT 1")
        .bind(request.user_id)
        .fetch_optional(&mut *transaction)
        .await?;
    if user_exists.is_none() {
        transaction.rollback().await?;
        return Ok(Err(UpdateUserAccountRejection::UnknownUser));
    }
    if !valid_username(&request.username) {
        transaction.rollback().await?;
        return Ok(Err(UpdateUserAccountRejection::InvalidUsername));
    }
    if !valid_email(&request.email) {
        transaction.rollback().await?;
        return Ok(Err(UpdateUserAccountRejection::InvalidEmail));
    }
    if request
        .password
        .as_ref()
        .is_some_and(|password| !valid_password(password))
    {
        transaction.rollback().await?;
        return Ok(Err(UpdateUserAccountRejection::InvalidPassword));
    }

    let duplicate_username: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE username = ? AND id <> ? LIMIT 1")
            .bind(&request.username)
            .bind(request.user_id)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_username.is_some() {
        transaction.rollback().await?;
        return Ok(Err(UpdateUserAccountRejection::DuplicateUsername));
    }

    let duplicate_email: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE email = ? AND id <> ? LIMIT 1")
            .bind(&request.email)
            .bind(request.user_id)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_email.is_some() {
        transaction.rollback().await?;
        return Ok(Err(UpdateUserAccountRejection::DuplicateEmail));
    }

    if let Some(password) = request.password {
        let password_hash = hash_password(&password);
        sqlx::query("UPDATE User SET username = ?, email = ?, password = ? WHERE id = ?")
            .bind(&request.username)
            .bind(&request.email)
            .bind(password_hash)
            .bind(request.user_id)
            .execute(&mut *transaction)
            .await?;
    } else {
        sqlx::query("UPDATE User SET username = ?, email = ? WHERE id = ?")
            .bind(&request.username)
            .bind(&request.email)
            .bind(request.user_id)
            .execute(&mut *transaction)
            .await?;
    }

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;
    Ok(Ok(UpdatedUserAccount {
        user_id: request.user_id,
    }))
}

pub(crate) async fn touch_user_last_login_time(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE User SET lastLoginTime = NOW() WHERE id = ?")
        .bind(user_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

pub async fn verify_login(
    pool: &MySqlPool,
    request: VerifyLoginRequest,
) -> Result<Result<VerifiedLogin, VerifyLoginRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let Some((user_id, password_hash)) = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, password FROM User WHERE username = ? OR email = ?",
    )
    .bind(&request.login_name)
    .bind(&request.login_name)
    .fetch_optional(&mut *transaction)
    .await?
    else {
        transaction.rollback().await?;
        return Ok(Err(VerifyLoginRejection::UnknownUser));
    };

    if !verify_password_hash(&mut transaction, &request.password, &password_hash).await? {
        transaction.rollback().await?;
        return Ok(Err(VerifyLoginRejection::InvalidPassword));
    }

    upgrade_legacy_password_hash(&mut transaction, user_id, &request.password, &password_hash)
        .await?;

    touch_user_last_login_time(&mut transaction, user_id).await?;

    transaction.commit().await?;
    Ok(Ok(VerifiedLogin { user_id }))
}

pub async fn verify_user_password(
    pool: &MySqlPool,
    request: VerifyUserPasswordRequest,
) -> Result<Result<VerifiedLogin, VerifyLoginRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let Some(password_hash) =
        sqlx::query_scalar::<_, String>("SELECT password FROM User WHERE id = ?")
            .bind(request.user_id)
            .fetch_optional(&mut *transaction)
            .await?
    else {
        transaction.rollback().await?;
        return Ok(Err(VerifyLoginRejection::UnknownUser));
    };

    if !verify_password_hash(&mut transaction, &request.password, &password_hash).await? {
        transaction.rollback().await?;
        return Ok(Err(VerifyLoginRejection::InvalidPassword));
    }

    upgrade_legacy_password_hash(
        &mut transaction,
        request.user_id,
        &request.password,
        &password_hash,
    )
    .await?;

    transaction.commit().await?;
    Ok(Ok(VerifiedLogin {
        user_id: request.user_id,
    }))
}

async fn verify_password_hash(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    password: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    if is_legacy_password_hash(password_hash) {
        return verify_legacy_password_hash(transaction, password, password_hash).await;
    }

    Ok(verify_argon2_password(password, password_hash))
}

async fn verify_legacy_password_hash(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
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
        .fetch_one(&mut **transaction)
        .await?;

    Ok(digest.eq_ignore_ascii_case(expected_digest))
}

async fn upgrade_legacy_password_hash(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    password: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    if !is_legacy_password_hash(password_hash) {
        return Ok(());
    }

    let upgraded_hash = hash_password(password);
    sqlx::query("UPDATE User SET password = ? WHERE id = ?")
        .bind(upgraded_hash)
        .bind(user_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

pub(crate) async fn user_exists(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&mut **transaction)
        .await?;

    Ok(exists.is_some())
}
fn valid_username(username: &str) -> bool {
    (3..=255).contains(&username.len())
        && username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn valid_email(email: &str) -> bool {
    !email.is_empty() && email.contains('@')
}

fn valid_password(password: &str) -> bool {
    password.len() >= 8
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
    fn valid_email_requires_non_empty_address_with_at_sign() {
        assert!(valid_email("player@example.invalid"));
        assert!(!valid_email(""));
        assert!(!valid_email("missing-at.example"));
    }

    #[test]
    fn valid_password_requires_at_least_eight_characters() {
        assert!(valid_password("12345678"));
        assert!(!valid_password("short"));
    }
}
