-- Migration 009: track claimed run counts for mining-area lifetime averages.

ALTER TABLE MiningAreaLifetimeResult
    ADD COLUMN totalRuns BIGINT NOT NULL DEFAULT 0;

UPDATE MiningAreaLifetimeResult AS lifetime
INNER JOIN (
    SELECT miningAreaId, SUM(totalRuns) AS areaRuns
    FROM RobotMiningAreaScore
    GROUP BY miningAreaId
) AS areaRuns
    ON areaRuns.miningAreaId = lifetime.miningAreaId
SET lifetime.totalRuns = areaRuns.areaRuns
WHERE lifetime.totalContainerSize > 0;

UPDATE MiningAreaLifetimeResult
SET totalRuns = 1
WHERE totalRuns = 0
  AND totalContainerSize > 0;
