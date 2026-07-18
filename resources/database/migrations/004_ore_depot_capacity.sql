-- Migration 004: per-ore depot capacity for rally home dumps.
-- depotMaxAllowed starts at 0; achievements raise it via maxDepotReward.

ALTER TABLE UserOreAsset
    ADD COLUMN depotMaxAllowed INT NOT NULL DEFAULT 0;

ALTER TABLE AchievementStep
    ADD COLUMN maxDepotReward INT NOT NULL DEFAULT 0;
