//! Mining queue enqueue, cancel, reorder, and page read models.
//!
//! Primary entry points: [`enqueue_mining`], [`cancel_mining_queue`],
//! [`reorder_mining_queue`], [`list_mining_queue_states_for_user`].

mod read;
mod write;

pub use read::*;
pub use write::*;

/// `prev` is an unfinished row for the same robot that sorts before `MiningQueue`.
///
/// Unfinished order is `(queueOrder, id)`. Keep this predicate identical in
/// rally claim, next-claim listing, and cancel "earlier row" checks.
pub const EARLIER_UNFINISHED_QUEUE_PRED: &str = "\
prev.robotId = MiningQueue.robotId \
AND prev.miningEndTime IS NULL \
AND (prev.queueOrder < MiningQueue.queueOrder \
     OR (prev.queueOrder = MiningQueue.queueOrder AND prev.id < MiningQueue.id))";

/// Page/state list order for unfinished queue rows.
pub const UNFINISHED_QUEUE_ORDER_BY: &str =
    "MiningQueue.robotId, MiningQueue.queueOrder, MiningQueue.id";
