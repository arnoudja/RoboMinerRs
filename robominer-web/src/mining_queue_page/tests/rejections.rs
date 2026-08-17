use super::super::robots::format_queue_time_left;
use super::super::{cancel_mining_rejection_message, enqueue_mining_rejection_message};

#[test]
fn mining_queue_time_left_uses_countdown_format() {
    assert_eq!(format_queue_time_left(0), "0:00");
    assert_eq!(format_queue_time_left(60), "1:00");
    assert_eq!(format_queue_time_left(150), "2:30");
    assert_eq!(format_queue_time_left(3_661), "1:01:01");
}

#[test]
fn mining_queue_rejection_messages_match_legacy_copy() {
    assert_eq!(
        enqueue_mining_rejection_message(
            robominer_db::EnqueueMiningRejection::MiningAreaUnavailable
        ),
        "Unable to add to the mining queue: The mining area is not available."
    );
    assert_eq!(
        enqueue_mining_rejection_message(robominer_db::EnqueueMiningRejection::QueueFull),
        "Unable to add to the mining queue: The mining queue is full."
    );
    assert_eq!(
        enqueue_mining_rejection_message(robominer_db::EnqueueMiningRejection::InsufficientFunds),
        "Unable to add to the mining queue: You do not have enough funds to pay the mining costs."
    );
}

#[test]
fn cancel_mining_rejection_messages_match_legacy_copy() {
    assert_eq!(
        cancel_mining_rejection_message(robominer_db::CancelMiningQueueRejection::UnknownQueue),
        "Unknown mining queue item."
    );
    assert_eq!(
        cancel_mining_rejection_message(robominer_db::CancelMiningQueueRejection::NotCancelable),
        "Unable to cancel mining queue item: The mining queue item is not cancelable."
    );
    assert_eq!(
        cancel_mining_rejection_message(robominer_db::CancelMiningQueueRejection::RefundWouldClamp),
        "Unable to cancel mining queue item: refund would exceed your wallet maximum."
    );
}
