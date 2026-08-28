-- Migration 012: speed scans of finished, unclaimed MiningQueue rows
-- used by wallet claim passes (claimed = false AND miningEndTime <= NOW()).

ALTER TABLE MiningQueue
    ADD INDEX idx_mining_queue_claimable (claimed, miningEndTime);
