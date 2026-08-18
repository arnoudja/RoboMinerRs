mod detail;
mod states;

pub use detail::*;
pub use states::*;

/// Ranked claimed-queue window used by mining-result read models.
/// `RankedQueue` is claimed for the same robot and counts as more recent when it
/// ended later, or ended at the same time with an id less than or equal to the
/// current queue (legacy tie-break).
const RECENT_CLAIMED_QUEUE_RANK_FILTER: &str = "Robot.userId = ? \
           AND MiningQueue.claimed = TRUE \
           AND (SELECT COUNT(*) \
                FROM MiningQueue RankedQueue \
                WHERE RankedQueue.robotId = MiningQueue.robotId \
                  AND RankedQueue.claimed = TRUE \
                  AND (RankedQueue.miningEndTime > MiningQueue.miningEndTime \
                       OR (RankedQueue.miningEndTime = MiningQueue.miningEndTime \
                           AND RankedQueue.id <= MiningQueue.id))) <= ?";
