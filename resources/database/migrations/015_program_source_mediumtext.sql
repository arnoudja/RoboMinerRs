-- Migration 015: player program source must fit a compiled size of 24576.
-- The densest such program is about 172 KiB, which does not fit in TEXT.

ALTER TABLE ProgramSource
    MODIFY sourceCode MEDIUMTEXT;

ALTER TABLE Robot
    MODIFY sourceCode MEDIUMTEXT NOT NULL;

ALTER TABLE PendingRobotChanges
    MODIFY sourceCode MEDIUMTEXT NOT NULL;

ALTER TABLE MiningQueue
    MODIFY executedSourceCode MEDIUMTEXT NULL;

ALTER TABLE PoolItem
    MODIFY sourceCode MEDIUMTEXT NOT NULL;
