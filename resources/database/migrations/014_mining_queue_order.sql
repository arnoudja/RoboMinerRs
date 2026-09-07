-- Migration 014: explicit per-robot queue order so queued runs can be reordered
-- without changing auto-increment ids (rally head stays lowest unfinished
-- (queueOrder, id)).

ALTER TABLE MiningQueue
    ADD COLUMN queueOrder INT NOT NULL DEFAULT 0;

UPDATE MiningQueue
SET queueOrder = id;

CREATE INDEX idx_mining_queue_robot_order
    ON MiningQueue (robotId, queueOrder);
