use sqlx::MySqlPool;

use crate::{INITIAL_ORE_WALLET_MAX, UserAssetSummaryRecord, UserOreAssetStateRecord};

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrePriceCost {
    ore_id: i64,
    amount: i32,
}

pub async fn list_user_ore_asset_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserOreAssetStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32, i32, i32)>(
        "SELECT UserOreAsset.oreId, Ore.oreName, UserOreAsset.amount, UserOreAsset.maxAllowed, \
                UserOreAsset.depotMaxAllowed \
         FROM UserOreAsset \
         INNER JOIN Ore ON Ore.id = UserOreAsset.oreId \
         WHERE UserOreAsset.userId = ? \
         ORDER BY UserOreAsset.oreId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(ore_id, ore_name, amount, max_allowed, depot_max_allowed)| {
                    UserOreAssetStateRecord {
                        ore_id,
                        ore_name,
                        amount,
                        max_allowed,
                        depot_max_allowed,
                    }
                },
            )
            .collect()
    })
}

/// Per-ore depot capacity for a user (defaults to empty when no asset rows exist).
pub async fn list_user_depot_max_allowed(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<(i64, i32)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32)>(
        "SELECT oreId, depotMaxAllowed \
         FROM UserOreAsset \
         WHERE userId = ? \
         ORDER BY oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn load_user_asset_summary(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<UserAssetSummaryRecord, sqlx::Error> {
    sqlx::query_as::<_, (String, i32, i32, i64)>(
        "SELECT User.username, User.achievementPoints, GREATEST(User.miningQueueSize, 1), \
                (SELECT COUNT(*) FROM Robot WHERE Robot.userId = User.id) \
         FROM User \
         WHERE User.id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(
        |(username, achievement_points, mining_queue_size, robot_count)| UserAssetSummaryRecord {
            username,
            achievement_points,
            mining_queue_size,
            robot_count,
        },
    )
}

pub(crate) async fn list_ore_price_amounts(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ore_price_id: i64,
) -> Result<Vec<OrePriceCost>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i32)>(
        "SELECT oreId, amount FROM OrePriceAmount WHERE orePriceId = ? ORDER BY oreId",
    )
    .bind(ore_price_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(ore_id, amount)| OrePriceCost { ore_id, amount })
        .collect())
}

pub(crate) async fn robot_part_ore_price_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_part_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT orePriceId FROM RobotPart WHERE id = ?")
        .bind(robot_part_id)
        .fetch_optional(&mut **transaction)
        .await
}

pub(crate) async fn can_pay_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<bool, sqlx::Error> {
    for cost in costs {
        let amount: Option<i32> = sqlx::query_scalar(
            "SELECT amount \
             FROM UserOreAsset \
             WHERE userId = ? AND oreId = ? \
             FOR UPDATE",
        )
        .bind(user_id)
        .bind(cost.ore_id)
        .fetch_optional(&mut **transaction)
        .await?;

        if amount.unwrap_or_default() < cost.amount {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) async fn deduct_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<(), sqlx::Error> {
    for cost in costs {
        sqlx::query(
            "UPDATE UserOreAsset \
             SET amount = amount - ? \
             WHERE userId = ? AND oreId = ?",
        )
        .bind(cost.amount)
        .bind(user_id)
        .bind(cost.ore_id)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

pub(crate) async fn refund_half_ore_costs_scaled(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
    count: i32,
) -> Result<(), sqlx::Error> {
    let half_costs: Vec<OrePriceCost> = costs
        .iter()
        .map(|cost| OrePriceCost {
            ore_id: cost.ore_id,
            amount: (cost.amount * count) / 2,
        })
        .collect();
    refund_ore_costs(transaction, user_id, &half_costs).await
}

pub(crate) async fn refund_full_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<(), sqlx::Error> {
    refund_ore_costs(transaction, user_id, costs).await
}

/// Returns true when refunding `costs` would not clamp any ore against `maxAllowed`.
///
/// Missing wallet rows are treated like a new asset capped at [`INITIAL_ORE_WALLET_MAX`].
/// Must stay aligned with the client preview in `mining_queue/clear_wallet.js`.
pub(crate) async fn ore_refund_fits_without_clamp_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<bool, sqlx::Error> {
    use std::collections::HashMap;

    let mut projected: HashMap<i64, (i32, i32)> = HashMap::new();
    for cost in costs {
        let ore_id = cost.ore_id;
        let refund = cost.amount;
        let entry = if let Some(existing) = projected.get(&ore_id).copied() {
            existing
        } else {
            let asset: Option<(i32, i32)> = sqlx::query_as(
                "SELECT amount, maxAllowed \
                 FROM UserOreAsset \
                 WHERE userId = ? AND oreId = ? \
                 FOR UPDATE",
            )
            .bind(user_id)
            .bind(ore_id)
            .fetch_optional(&mut **transaction)
            .await?;
            match asset {
                Some((amount, max_allowed)) => (amount, max_allowed),
                None => (0, INITIAL_ORE_WALLET_MAX),
            }
        };
        let (amount, max_allowed) = entry;
        if amount.saturating_add(refund) > max_allowed {
            return Ok(false);
        }
        projected.insert(ore_id, (amount + refund, max_allowed));
    }
    Ok(true)
}

/// Pool convenience wrapper (locks then rolls back). Prefer the transactional check inside cancel.
pub async fn ore_refund_fits_without_clamp(
    pool: &MySqlPool,
    user_id: i64,
    costs: &[(i64, i32)],
) -> Result<bool, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let ore_costs: Vec<OrePriceCost> = costs
        .iter()
        .map(|&(ore_id, amount)| OrePriceCost { ore_id, amount })
        .collect();
    let fits = ore_refund_fits_without_clamp_tx(&mut transaction, user_id, &ore_costs).await?;
    transaction.rollback().await?;
    Ok(fits)
}

async fn refund_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<(), sqlx::Error> {
    for cost in costs {
        let refund = cost.amount;
        let asset: Option<(i32, i32)> = sqlx::query_as(
            "SELECT amount, maxAllowed \
             FROM UserOreAsset \
             WHERE userId = ? AND oreId = ? \
             FOR UPDATE",
        )
        .bind(user_id)
        .bind(cost.ore_id)
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some((amount, max_allowed)) = asset {
            let new_amount = (amount + refund).min(max_allowed);
            sqlx::query(
                "UPDATE UserOreAsset \
                 SET amount = ? \
                 WHERE userId = ? AND oreId = ?",
            )
            .bind(new_amount)
            .bind(user_id)
            .bind(cost.ore_id)
            .execute(&mut **transaction)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
                 VALUES (?, ?, LEAST(?, ?), ?)",
            )
            .bind(user_id)
            .bind(cost.ore_id)
            .bind(refund)
            .bind(INITIAL_ORE_WALLET_MAX)
            .bind(INITIAL_ORE_WALLET_MAX)
            .execute(&mut **transaction)
            .await?;
        }
    }

    Ok(())
}
