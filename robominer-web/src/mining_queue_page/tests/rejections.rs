#[test]
fn mining_queue_rejection_messages_match_legacy_copy() {
    assert_eq!(
        robominer_domain::rejection_messages::enqueue_mining_rejection_player_message(
            robominer_db::EnqueueMiningRejection::MiningAreaUnavailable
        ),
        "Unable to add to the mining queue: The mining area is not available."
    );
    assert_eq!(
        robominer_domain::rejection_messages::enqueue_mining_rejection_player_message(
            robominer_db::EnqueueMiningRejection::QueueFull
        ),
        "Unable to add to the mining queue: The mining queue is full."
    );
    assert_eq!(
        robominer_domain::rejection_messages::enqueue_mining_rejection_player_message(
            robominer_db::EnqueueMiningRejection::InsufficientFunds
        ),
        "Unable to add to the mining queue: You do not have enough funds to pay the mining costs."
    );
}

#[test]
fn cancel_mining_rejection_messages_match_legacy_copy() {
    assert_eq!(
        robominer_domain::rejection_messages::cancel_mining_queue_rejection_player_message(
            robominer_db::CancelMiningQueueRejection::UnknownQueue
        ),
        "Unknown mining queue item."
    );
    assert_eq!(
        robominer_domain::rejection_messages::cancel_mining_queue_rejection_player_message(
            robominer_db::CancelMiningQueueRejection::NotCancelable
        ),
        "Unable to cancel mining queue item: The mining queue item is not cancelable."
    );
    assert_eq!(
        robominer_domain::rejection_messages::cancel_mining_queue_rejection_player_message(
            robominer_db::CancelMiningQueueRejection::RefundWouldClamp
        ),
        "Unable to cancel mining queue item: refund would exceed your wallet maximum."
    );
}
