-- Migration 013: durable lifetime depot totals on RobotLifetimeResult.
-- Achievement depot progress must not shrink when claimed MiningQueue history
-- is trimmed (CLAIMED_MINING_QUEUE_RETENTION).

ALTER TABLE RobotLifetimeResult
    ADD COLUMN depotAmount INT NOT NULL DEFAULT 0;

-- Backfill from remaining claimed per-run rows (best-effort for history still present).
INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax, depotAmount)
SELECT MiningQueue.robotId,
       MiningOreResult.oreId,
       0,
       0,
       CAST(SUM(MiningOreResult.depotAmount) AS SIGNED)
FROM MiningOreResult
INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId
WHERE MiningQueue.claimed = true
GROUP BY MiningQueue.robotId, MiningOreResult.oreId
ON DUPLICATE KEY UPDATE
    depotAmount = VALUES(depotAmount);
