-- Migration 005: per-area top-ore score threshold for rally scoring.
-- Default 30 preserves the legacy formula (30 most-valuable ore → 900 points).

ALTER TABLE MiningArea
    ADD COLUMN scoreOreTarget INT NOT NULL DEFAULT 30;
