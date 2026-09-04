use crate::assert_sql_safe;
use crate::in_placeholders;

/// Pure tax formula matching the SQL in [`calculate_mining_ore_result_tax_batch`].
///
/// `FLOOR(GREATEST(amount - depot, 0) * tax_rate / 100)
///  + FLOOR(LEAST(depot, amount) * depot_tax_rate / 100)`
///
/// Kept as a Rust twin for unit tests; production tax is applied in SQL.
#[cfg(test)]
pub(crate) fn mining_ore_tax(
    amount: i32,
    depot_amount: i32,
    tax_rate: i32,
    depot_tax_rate: i32,
) -> i32 {
    let above_depot = amount.saturating_sub(depot_amount).max(0);
    let in_depot = depot_amount.min(amount).max(0);
    let container_tax = (i64::from(above_depot) * i64::from(tax_rate) / 100) as i32;
    let depot_tax = (i64::from(in_depot) * i64::from(depot_tax_rate) / 100) as i32;
    container_tax.saturating_add(depot_tax)
}

pub(super) async fn calculate_mining_ore_result_tax_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<(), sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(());
    }

    let placeholders = in_placeholders(queue_ids.len());
    // Keep in sync with [`mining_ore_tax`].
    let query = format!(
        "UPDATE MiningOreResult \
         INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         SET MiningOreResult.tax = \
             FLOOR(GREATEST(MiningOreResult.amount - MiningOreResult.depotAmount, 0) \
                   * MiningArea.taxRate / 100) \
           + FLOOR(LEAST(MiningOreResult.depotAmount, MiningOreResult.amount) \
                   * MiningArea.depotTaxRate / 100) \
         WHERE MiningOreResult.miningQueueId IN ({placeholders})"
    );
    let mut query_builder = sqlx::query(assert_sql_safe(query));
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    query_builder.execute(&mut **transaction).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::mining_ore_tax;

    #[test]
    fn mining_ore_tax_matches_sql_formula() {
        assert_eq!(mining_ore_tax(100, 0, 10, 5), 10);
        assert_eq!(mining_ore_tax(100, 40, 10, 5), 6 + 2);
        assert_eq!(mining_ore_tax(30, 40, 10, 5), 0 + 1);
        assert_eq!(mining_ore_tax(0, 0, 10, 5), 0);
        assert_eq!(mining_ore_tax(7, 3, 15, 20), 0 + 0); // floor(4*15/100)=0, floor(3*20/100)=0
        assert_eq!(mining_ore_tax(200, 50, 25, 10), 37 + 5);
    }
}
