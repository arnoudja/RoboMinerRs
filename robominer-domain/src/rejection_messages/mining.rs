pub fn enqueue_mining_rejection_player_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    match rejection {
        robominer_db::EnqueueMiningRejection::UnknownRobot => "Unknown robot",
        robominer_db::EnqueueMiningRejection::UnknownMiningArea => "Unknown mining area",
        robominer_db::EnqueueMiningRejection::MiningAreaUnavailable => {
            "Unable to add to the mining queue: The mining area is not available."
        }
        robominer_db::EnqueueMiningRejection::QueueFull => {
            "Unable to add to the mining queue: The mining queue is full."
        }
        robominer_db::EnqueueMiningRejection::InsufficientFunds => {
            "Unable to add to the mining queue: You do not have enough funds to pay the mining costs."
        }
    }
}

pub fn enqueue_mining_rejection_cli_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    match rejection {
        robominer_db::EnqueueMiningRejection::UnknownRobot => "unknown robot",
        robominer_db::EnqueueMiningRejection::UnknownMiningArea => "unknown mining area",
        robominer_db::EnqueueMiningRejection::MiningAreaUnavailable => {
            "mining area is not available to user"
        }
        robominer_db::EnqueueMiningRejection::QueueFull => "mining queue is full",
        robominer_db::EnqueueMiningRejection::InsufficientFunds => {
            "insufficient funds to pay mining costs"
        }
    }
}

pub fn cancel_mining_queue_rejection_player_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    match rejection {
        robominer_db::CancelMiningQueueRejection::UnknownQueue => "Unknown mining queue item.",
        robominer_db::CancelMiningQueueRejection::WrongOwner => {
            "Unable to cancel mining queue item."
        }
        robominer_db::CancelMiningQueueRejection::NotCancelable => {
            "Unable to cancel mining queue item: The mining queue item is not cancelable."
        }
        robominer_db::CancelMiningQueueRejection::RefundWouldClamp => {
            "Unable to cancel mining queue item: refund would exceed your wallet maximum."
        }
    }
}

pub fn cancel_mining_queue_rejection_cli_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    match rejection {
        robominer_db::CancelMiningQueueRejection::UnknownQueue => "unknown mining queue item",
        robominer_db::CancelMiningQueueRejection::WrongOwner => {
            "mining queue item belongs to another user"
        }
        robominer_db::CancelMiningQueueRejection::NotCancelable => {
            "mining queue item is not cancelable"
        }
        robominer_db::CancelMiningQueueRejection::RefundWouldClamp => {
            "cancel would clamp ore refund past maxAllowed"
        }
    }
}
