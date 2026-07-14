-- One-off migration: rename scanSpeed to scanTime and convert scanner values.
-- Apply manually to deployed databases (no automatic migration runner exists).
-- Conversion: scanTime = GREATEST(1, 12 DIV scanSpeed)

ALTER TABLE RobotPart
    CHANGE scanSpeed scanTime INT NOT NULL DEFAULT 0;

UPDATE RobotPart
SET scanTime = GREATEST(1, 12 DIV scanTime)
WHERE typeId = 7 AND scanTime > 0;

ALTER TABLE Robot
    CHANGE scanSpeed scanTime INT NOT NULL DEFAULT 0;

UPDATE Robot
SET scanTime = GREATEST(1, 12 DIV scanTime)
WHERE scanTime > 0;

ALTER TABLE PendingRobotChanges
    CHANGE scanSpeed scanTime INT NOT NULL DEFAULT 0;

UPDATE PendingRobotChanges
SET scanTime = GREATEST(1, 12 DIV scanTime)
WHERE scanTime > 0;
