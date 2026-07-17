-- Migration 002: snapshot the program that ran for each queue entry at rally persist.
-- Used for private replay source highlighting (not stored in shared RallyResult.resultData).

ALTER TABLE MiningQueue
    ADD COLUMN executedSourceCode TEXT NULL;
