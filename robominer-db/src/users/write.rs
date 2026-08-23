use sqlx::MySqlPool;

use super::password::{
    maybe_upgrade_password_hash, valid_email, valid_password, valid_username, verify_password_hash,
    write_password_hash,
};
use crate::achievements::claim_achievement_step_in_transaction;
use crate::password::hash_password_async;
use crate::{
    ClaimAchievementStepRequest, CreateUserRejection, CreateUserRequest, CreatedUser, DbOutcome,
    UpdateUserAccountRejection, UpdateUserAccountRequest, UpdatedUserAccount, VerifiedLogin,
    VerifyLoginRejection, VerifyLoginRequest, VerifyUserPasswordRequest, db_ok, db_reject,
};

pub async fn create_user(
    pool: &MySqlPool,
    request: CreateUserRequest,
) -> Result<DbOutcome<CreatedUser, CreateUserRejection>, sqlx::Error> {
    if !valid_username(&request.username) {
        return db_reject(CreateUserRejection::InvalidUsername);
    }
    if !valid_email(&request.email) {
        return db_reject(CreateUserRejection::InvalidEmail);
    }
    if !valid_password(&request.password) {
        return db_reject(CreateUserRejection::InvalidPassword);
    }

    // Hash before opening a DB transaction so Argon2 does not hold a pool connection.
    let password_hash = hash_password_async(request.password.clone()).await?;

    let mut transaction = pool.begin().await?;

    let duplicate_username: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE username = ? LIMIT 1")
            .bind(&request.username)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_username.is_some() {
        transaction.rollback().await?;
        return db_reject(CreateUserRejection::DuplicateUsername);
    }

    let duplicate_email: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE email = ? LIMIT 1")
            .bind(&request.email)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_email.is_some() {
        transaction.rollback().await?;
        return db_reject(CreateUserRejection::DuplicateEmail);
    }

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
        DbOutcome::Success(_) => {
            transaction.commit().await?;
            db_ok(CreatedUser {
                user_id,
                session_version: 0,
            })
        }
        DbOutcome::Rejected(rejection) => {
            transaction.rollback().await?;
            db_reject(CreateUserRejection::InitialAchievementRejected(rejection))
        }
    }
}

pub async fn update_user_account(
    pool: &MySqlPool,
    request: UpdateUserAccountRequest,
) -> Result<DbOutcome<UpdatedUserAccount, UpdateUserAccountRejection>, sqlx::Error> {
    if !valid_username(&request.username) {
        return db_reject(UpdateUserAccountRejection::InvalidUsername);
    }
    if !valid_email(&request.email) {
        return db_reject(UpdateUserAccountRejection::InvalidEmail);
    }
    if request
        .password
        .as_ref()
        .is_some_and(|password| !valid_password(password))
    {
        return db_reject(UpdateUserAccountRejection::InvalidPassword);
    }

    // Hash before opening a DB transaction so Argon2 does not hold a pool connection.
    let password_hash = match request.password {
        Some(password) => Some(hash_password_async(password).await?),
        None => None,
    };

    let mut transaction = pool.begin().await?;

    let user_exists: Option<i64> = sqlx::query_scalar("SELECT id FROM User WHERE id = ? LIMIT 1")
        .bind(request.user_id)
        .fetch_optional(&mut *transaction)
        .await?;
    if user_exists.is_none() {
        transaction.rollback().await?;
        return db_reject(UpdateUserAccountRejection::UnknownUser);
    }

    let duplicate_username: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE username = ? AND id <> ? LIMIT 1")
            .bind(&request.username)
            .bind(request.user_id)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_username.is_some() {
        transaction.rollback().await?;
        return db_reject(UpdateUserAccountRejection::DuplicateUsername);
    }

    let duplicate_email: Option<i64> =
        sqlx::query_scalar("SELECT id FROM User WHERE email = ? AND id <> ? LIMIT 1")
            .bind(&request.email)
            .bind(request.user_id)
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_email.is_some() {
        transaction.rollback().await?;
        return db_reject(UpdateUserAccountRejection::DuplicateEmail);
    }

    let password_changed = password_hash.is_some();
    if let Some(password_hash) = password_hash {
        sqlx::query(
            "UPDATE User \
             SET username = ?, email = ?, password = ?, sessionVersion = sessionVersion + 1 \
             WHERE id = ?",
        )
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

    let session_version: i32 = sqlx::query_scalar("SELECT sessionVersion FROM User WHERE id = ?")
        .bind(request.user_id)
        .fetch_one(&mut *transaction)
        .await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;
    db_ok(UpdatedUserAccount {
        user_id: request.user_id,
        session_version,
        password_changed,
    })
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
) -> Result<DbOutcome<VerifiedLogin, VerifyLoginRejection>, sqlx::Error> {
    let Some((user_id, password_hash, session_version)) = sqlx::query_as::<_, (i64, String, i32)>(
        "SELECT id, password, sessionVersion FROM User WHERE username = ? OR email = ?",
    )
    .bind(&request.login_name)
    .bind(&request.login_name)
    .fetch_optional(pool)
    .await?
    else {
        return db_reject(VerifyLoginRejection::UnknownUser);
    };

    if !verify_password_hash(pool, &request.password, &password_hash).await? {
        return db_reject(VerifyLoginRejection::InvalidPassword);
    }

    let upgraded_hash = maybe_upgrade_password_hash(&request.password, &password_hash).await?;

    let mut transaction = pool.begin().await?;
    if let Some(upgraded_hash) = upgraded_hash {
        write_password_hash(&mut transaction, user_id, &upgraded_hash).await?;
    }
    touch_user_last_login_time(&mut transaction, user_id).await?;
    transaction.commit().await?;

    db_ok(VerifiedLogin {
        user_id,
        session_version,
    })
}

pub async fn verify_user_password(
    pool: &MySqlPool,
    request: VerifyUserPasswordRequest,
) -> Result<DbOutcome<VerifiedLogin, VerifyLoginRejection>, sqlx::Error> {
    let Some((password_hash, session_version)) = sqlx::query_as::<_, (String, i32)>(
        "SELECT password, sessionVersion FROM User WHERE id = ?",
    )
    .bind(request.user_id)
    .fetch_optional(pool)
    .await?
    else {
        return db_reject(VerifyLoginRejection::UnknownUser);
    };

    if !verify_password_hash(pool, &request.password, &password_hash).await? {
        return db_reject(VerifyLoginRejection::InvalidPassword);
    }

    let upgraded_hash = maybe_upgrade_password_hash(&request.password, &password_hash).await?;

    if let Some(upgraded_hash) = upgraded_hash {
        let mut transaction = pool.begin().await?;
        write_password_hash(&mut transaction, request.user_id, &upgraded_hash).await?;
        transaction.commit().await?;
    }

    db_ok(VerifiedLogin {
        user_id: request.user_id,
        session_version,
    })
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
