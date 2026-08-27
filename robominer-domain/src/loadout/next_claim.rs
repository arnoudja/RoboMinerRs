use robominer_db::NextClaimRallyCandidate;

use crate::constants::{RALLY_EXPIRY_START_SECONDS, RALLY_SIZE};

/// Seconds until any mining area has a claimable rally, or `None` if nothing is queued.
pub fn next_claimable_rally_delay_seconds(candidates: &[NextClaimRallyCandidate]) -> Option<u64> {
    next_claimable_rally_delay_seconds_with(candidates, RALLY_SIZE, RALLY_EXPIRY_START_SECONDS)
}

pub(crate) fn next_claimable_rally_delay_seconds_with(
    candidates: &[NextClaimRallyCandidate],
    rally_size: usize,
    expiry_start_seconds: i32,
) -> Option<u64> {
    if candidates.is_empty() || rally_size == 0 {
        return None;
    }

    let mut area_ids: Vec<i64> = candidates.iter().map(|c| c.mining_area_id).collect();
    area_ids.sort_unstable();
    area_ids.dedup();

    let mut min_delay: Option<u64> = None;
    for area_id in area_ids {
        let area: Vec<&NextClaimRallyCandidate> = candidates
            .iter()
            .filter(|c| c.mining_area_id == area_id)
            .collect();
        // Claim readiness keeps one queue head per user (same as mining_rally_queue_rows).
        let distinct = distinct_user_candidates(&area);
        let Some(delay) = delay_for_area(&distinct, rally_size, expiry_start_seconds) else {
            continue;
        };
        min_delay = Some(match min_delay {
            Some(current) => current.min(delay),
            None => delay,
        });
    }
    min_delay
}

fn distinct_user_candidates<'a>(
    candidates: &[&'a NextClaimRallyCandidate],
) -> Vec<&'a NextClaimRallyCandidate> {
    let mut seen_users = Vec::new();
    let mut distinct = Vec::new();
    for candidate in candidates {
        if seen_users.contains(&candidate.user_id) {
            continue;
        }
        seen_users.push(candidate.user_id);
        distinct.push(*candidate);
    }
    distinct
}

fn delay_for_area(
    candidates: &[&NextClaimRallyCandidate],
    rally_size: usize,
    expiry_start_seconds: i32,
) -> Option<u64> {
    if candidates.is_empty() {
        return None;
    }

    let free_now: Vec<&NextClaimRallyCandidate> = candidates
        .iter()
        .copied()
        .filter(|c| c.busy_seconds <= 0)
        .collect();
    if mining_rally_candidates_are_ready(&free_now, rally_size, expiry_start_seconds) {
        return Some(0);
    }

    let mut delays = Vec::new();

    if candidates.len() >= rally_size {
        let mut busies: Vec<i32> = candidates.iter().map(|c| c.busy_seconds.max(0)).collect();
        busies.sort_unstable();
        delays.push(u64::try_from(busies[rally_size - 1]).unwrap_or(u64::MAX));
    }

    for candidate in candidates {
        let busy = u64::try_from(candidate.busy_seconds.max(0)).unwrap_or(u64::MAX);
        let expiry_wait = expiry_wait_seconds(candidate.seconds_left, expiry_start_seconds);
        delays.push(busy.max(expiry_wait));
    }

    delays.into_iter().min()
}

fn mining_rally_candidates_are_ready(
    free: &[&NextClaimRallyCandidate],
    rally_size: usize,
    expiry_start_seconds: i32,
) -> bool {
    if free.is_empty() {
        return false;
    }
    if free.len() >= rally_size {
        return true;
    }
    free.iter()
        .map(|c| c.seconds_left)
        .min()
        .is_some_and(|seconds_left| seconds_left < expiry_start_seconds)
}

fn expiry_wait_seconds(seconds_left: i32, expiry_start_seconds: i32) -> u64 {
    if seconds_left < expiry_start_seconds {
        return 0;
    }
    u64::try_from(seconds_left - expiry_start_seconds + 1).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(area: i64, user: i64, busy: i32, seconds_left: i32) -> NextClaimRallyCandidate {
        NextClaimRallyCandidate {
            mining_area_id: area,
            user_id: user,
            busy_seconds: busy,
            seconds_left,
        }
    }

    #[test]
    fn empty_candidates_yield_none() {
        assert_eq!(next_claimable_rally_delay_seconds_with(&[], 4, 10), None);
    }

    #[test]
    fn already_claim_ready_full_queue_yields_zero() {
        let candidates = [
            candidate(1, 1, 0, 100),
            candidate(1, 2, 0, 90),
            candidate(1, 3, 0, 80),
            candidate(1, 4, 0, 70),
        ];
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(0)
        );
    }

    #[test]
    fn same_user_robots_do_not_form_a_full_rally() {
        let candidates = [
            candidate(1, 1, 0, 100),
            candidate(1, 1, 0, 90),
            candidate(1, 1, 0, 80),
            candidate(1, 1, 0, 70),
        ];
        // Only one distinct user → expiry path: 100 - 10 + 1 = 91.
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(91)
        );
    }

    #[test]
    fn already_claim_ready_via_expiry_yields_zero() {
        let candidates = [candidate(1, 1, 0, 9)];
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(0)
        );
    }

    #[test]
    fn waits_for_nth_robot_to_free_for_full_rally() {
        let candidates = [
            candidate(1, 1, 5, 100),
            candidate(1, 2, 10, 90),
            candidate(1, 3, 15, 80),
            candidate(1, 4, 20, 70),
        ];
        // Fourth distinct user frees at t=20; only then is a full rally available.
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(20)
        );
    }

    #[test]
    fn waits_for_expiry_when_partial_queue() {
        let candidates = [candidate(1, 1, 0, 100)];
        // Ready when seconds_left becomes 9: wait 100 - 10 + 1 = 91.
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(91)
        );
    }

    #[test]
    fn expiry_path_waits_for_busy_robot() {
        let candidates = [candidate(1, 1, 50, 20)];
        // Expiry wait = 20 - 10 + 1 = 11; busy = 50 → max = 50.
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(50)
        );
    }

    #[test]
    fn chooses_sooner_path_across_areas() {
        let candidates = [
            candidate(1, 1, 0, 100), // area 1: expiry in 91s
            candidate(2, 1, 3, 50),
            candidate(2, 2, 3, 50),
            candidate(2, 3, 3, 50),
            candidate(2, 4, 3, 50), // full rally when free in 3s
        ];
        assert_eq!(
            next_claimable_rally_delay_seconds_with(&candidates, 4, 10),
            Some(3)
        );
    }

    #[test]
    fn public_helper_uses_rally_constants() {
        let candidates = [candidate(1, 1, 0, 9)];
        assert_eq!(next_claimable_rally_delay_seconds(&candidates), Some(0));
    }
}
