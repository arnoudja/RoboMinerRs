-- Migration 010: depot ore total requirements for achievement steps.

CREATE TABLE AchievementStepDepotTotalRequirement
(
    achievementId INT NOT NULL,
    step INT NOT NULL,
    oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
    amount INT NOT NULL,
    PRIMARY KEY (achievementId, step, oreId),
    FOREIGN KEY (achievementId, step) REFERENCES AchievementStep (achievementId, step) ON DELETE CASCADE
);
