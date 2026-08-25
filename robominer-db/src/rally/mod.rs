mod claim;
mod cleanup;
mod pending;
mod persist;
mod score;

pub(crate) const CLAIMED_MINING_QUEUE_RETENTION: i64 = 12;

pub use claim::claim_user_results;
pub use cleanup::cleanup_old_claimed_mining_queue_items_for_robot;
pub use pending::reconcile_pending_robot_changes_for_user;
pub use persist::{
    PROCESSING_LEASE_SECONDS, claim_next_mining_rally_queue_for_area, persist_completed_rally,
};
pub use score::updated_robot_mining_area_score;
