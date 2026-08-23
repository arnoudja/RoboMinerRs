//! Cancel-queue batch helpers for the mining queue page.

#[derive(Debug, Default)]
pub(super) struct CancelBatchResult {
    pub(super) cleared: usize,
    pub(super) skipped: usize,
    pub(super) failed: usize,
    pub(super) last_rejection: Option<robominer_db::CancelMiningQueueRejection>,
}

pub(super) async fn cancel_queued_items(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    mining_queue_ids: &[i64],
    require_refund_fits: bool,
) -> Result<CancelBatchResult, crate::page_context::PageLoadError> {
    let mut batch = CancelBatchResult::default();
    for &mining_queue_id in mining_queue_ids {
        match robominer_db::cancel_mining_queue(
            pool,
            robominer_db::CancelMiningQueueRequest {
                user_id,
                mining_queue_id,
                require_refund_fits,
            },
        )
        .await?
        .into_result()
        {
            Ok(_) => batch.cleared += 1,
            Err(robominer_db::CancelMiningQueueRejection::RefundWouldClamp) => {
                batch.skipped += 1;
            }
            Err(rejection) => {
                batch.failed += 1;
                batch.last_rejection = Some(rejection);
            }
        }
    }
    Ok(batch)
}

pub(super) fn format_cancel_batch_message(batch: &CancelBatchResult) -> Option<String> {
    if batch.cleared == 0 && batch.skipped == 0 && batch.failed == 0 {
        return None;
    }
    if batch.failed == 0 && batch.skipped == 0 {
        return None;
    }

    let mut parts = Vec::new();
    if batch.cleared > 0 {
        parts.push(format!(
            "Cleared {} queued run{}.",
            batch.cleared,
            if batch.cleared == 1 { "" } else { "s" }
        ));
    }
    if batch.skipped > 0 {
        if batch.cleared == 0 {
            parts.push(
                "No queued runs cleared; refunds would exceed your wallet maximum.".to_string(),
            );
        } else {
            parts.push(format!(
                "{} left in queue to avoid losing ore.",
                batch.skipped
            ));
        }
    }
    if batch.failed > 0 {
        let detail = batch
            .last_rejection
            .map(super::cancel_mining_rejection_message)
            .unwrap_or("Unable to cancel mining queue item.");
        if batch.cleared == 0 && batch.skipped == 0 {
            parts.push(detail.to_string());
        } else {
            parts.push(format!(
                "{} could not be canceled ({detail}).",
                batch.failed
            ));
        }
    }
    Some(parts.join(" "))
}

#[cfg(test)]
mod batch_message_tests {
    use super::*;

    #[test]
    fn format_cancel_batch_message_covers_partial_outcomes() {
        assert_eq!(
            format_cancel_batch_message(&CancelBatchResult::default()),
            None
        );
        assert_eq!(
            format_cancel_batch_message(&CancelBatchResult {
                cleared: 2,
                ..CancelBatchResult::default()
            }),
            None
        );
        assert_eq!(
            format_cancel_batch_message(&CancelBatchResult {
                cleared: 2,
                skipped: 1,
                ..CancelBatchResult::default()
            })
            .as_deref(),
            Some("Cleared 2 queued runs. 1 left in queue to avoid losing ore.")
        );
        assert_eq!(
            format_cancel_batch_message(&CancelBatchResult {
                skipped: 2,
                ..CancelBatchResult::default()
            })
            .as_deref(),
            Some("No queued runs cleared; refunds would exceed your wallet maximum.")
        );
        let failed = format_cancel_batch_message(&CancelBatchResult {
            cleared: 1,
            failed: 1,
            last_rejection: Some(robominer_db::CancelMiningQueueRejection::NotCancelable),
            ..CancelBatchResult::default()
        })
        .expect("message");
        assert!(failed.contains("Cleared 1 queued run."));
        assert!(failed.contains("could not be canceled"));
    }
}
