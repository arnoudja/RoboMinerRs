mod detail;
mod states;

pub use detail::*;
pub use states::*;

/// Newest claimed mining-queue IDs for a user, newest first.
///
/// MySQL rejects `LIMIT` inside an `IN` subquery unless it is wrapped in an
/// extra `FROM (...)` derived table.
pub(crate) const RECENT_CLAIMED_MINING_QUEUE_IDS_FOR_USER: &str = "SELECT id FROM ( \
     SELECT MiningQueue.id \
     FROM MiningQueue \
     INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
     WHERE Robot.userId = ? \
       AND MiningQueue.claimed = TRUE \
     ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC \
     LIMIT ? \
) RecentQueues";
