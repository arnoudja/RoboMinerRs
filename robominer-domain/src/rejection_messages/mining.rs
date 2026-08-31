use super::Audience;

pub fn enqueue_mining_rejection_message(
    rejection: robominer_db::EnqueueMiningRejection,
    audience: Audience,
) -> &'static str {
    match (rejection, audience) {
        (robominer_db::EnqueueMiningRejection::UnknownRobot, Audience::Player) => "Unknown robot",
        (robominer_db::EnqueueMiningRejection::UnknownRobot, Audience::Cli) => "unknown robot",
        (robominer_db::EnqueueMiningRejection::UnknownMiningArea, Audience::Player) => {
            "Unknown mining area"
        }
        (robominer_db::EnqueueMiningRejection::UnknownMiningArea, Audience::Cli) => {
            "unknown mining area"
        }
        (robominer_db::EnqueueMiningRejection::MiningAreaUnavailable, Audience::Player) => {
            "Unable to add to the mining queue: The mining area is not available."
        }
        (robominer_db::EnqueueMiningRejection::MiningAreaUnavailable, Audience::Cli) => {
            "mining area is not available to user"
        }
        (robominer_db::EnqueueMiningRejection::QueueFull, Audience::Player) => {
            "Unable to add to the mining queue: The mining queue is full."
        }
        (robominer_db::EnqueueMiningRejection::QueueFull, Audience::Cli) => "mining queue is full",
        (robominer_db::EnqueueMiningRejection::InsufficientFunds, Audience::Player) => {
            "Unable to add to the mining queue: You do not have enough funds to pay the mining costs."
        }
        (robominer_db::EnqueueMiningRejection::InsufficientFunds, Audience::Cli) => {
            "insufficient funds to pay mining costs"
        }
    }
}

pub fn enqueue_mining_rejection_player_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    enqueue_mining_rejection_message(rejection, Audience::Player)
}

pub fn enqueue_mining_rejection_cli_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    enqueue_mining_rejection_message(rejection, Audience::Cli)
}

pub fn cancel_mining_queue_rejection_message(
    rejection: robominer_db::CancelMiningQueueRejection,
    audience: Audience,
) -> &'static str {
    match (rejection, audience) {
        (robominer_db::CancelMiningQueueRejection::UnknownQueue, Audience::Player) => {
            "Unknown mining queue item."
        }
        (robominer_db::CancelMiningQueueRejection::UnknownQueue, Audience::Cli) => {
            "unknown mining queue item"
        }
        (robominer_db::CancelMiningQueueRejection::WrongOwner, Audience::Player) => {
            "Unable to cancel mining queue item."
        }
        (robominer_db::CancelMiningQueueRejection::WrongOwner, Audience::Cli) => {
            "mining queue item belongs to another user"
        }
        (robominer_db::CancelMiningQueueRejection::NotCancelable, Audience::Player) => {
            "Unable to cancel mining queue item: The mining queue item is not cancelable."
        }
        (robominer_db::CancelMiningQueueRejection::NotCancelable, Audience::Cli) => {
            "mining queue item is not cancelable"
        }
        (robominer_db::CancelMiningQueueRejection::RefundWouldClamp, Audience::Player) => {
            "Unable to cancel mining queue item: refund would exceed your wallet maximum."
        }
        (robominer_db::CancelMiningQueueRejection::RefundWouldClamp, Audience::Cli) => {
            "cancel would clamp ore refund past maxAllowed"
        }
    }
}

pub fn cancel_mining_queue_rejection_player_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    cancel_mining_queue_rejection_message(rejection, Audience::Player)
}

pub fn cancel_mining_queue_rejection_cli_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    cancel_mining_queue_rejection_message(rejection, Audience::Cli)
}
