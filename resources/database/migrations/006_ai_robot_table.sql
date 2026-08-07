-- Migration 006: move AI opponents from Robot into AIRobot so seeded AI ids
-- cannot collide with player Robot AUTO_INCREMENT rows.
--
-- Do not use PREPARE/EXECUTE here: sqlx applies migrations over the binary
-- protocol, which rejects those statements (MySQL error 1295).
-- Before this script runs, the migrate runner drops any MiningArea.aiRobotId
-- foreign key and ensures MiningArea.aiRobotIdNew exists.

CREATE TABLE IF NOT EXISTS AIRobot
(
    id INT AUTO_INCREMENT PRIMARY KEY,
    robotName VARCHAR(255) NOT NULL,
    sourceCode TEXT NOT NULL,
    maxOre INT NOT NULL,
    miningSpeed INT NOT NULL,
    maxTurns INT NOT NULL,
    cpuSpeed INT NOT NULL,
    forwardSpeed DOUBLE NOT NULL,
    backwardSpeed DOUBLE NOT NULL,
    rotateSpeed INT NOT NULL,
    robotSize DOUBLE NOT NULL,
    scanTime INT NOT NULL DEFAULT 0,
    scanDistance INT NOT NULL DEFAULT 0
);

INSERT INTO AIRobot (
    id, robotName, sourceCode, maxOre, miningSpeed, maxTurns, cpuSpeed,
    forwardSpeed, backwardSpeed, rotateSpeed, robotSize, scanTime, scanDistance
)
SELECT DISTINCT
    Robot.id,
    Robot.robotName,
    Robot.sourceCode,
    Robot.maxOre,
    Robot.miningSpeed,
    Robot.maxTurns,
    Robot.cpuSpeed,
    Robot.forwardSpeed,
    Robot.backwardSpeed,
    Robot.rotateSpeed,
    Robot.robotSize,
    Robot.scanTime,
    Robot.scanDistance
FROM Robot
INNER JOIN MiningArea ON MiningArea.aiRobotId = Robot.id
WHERE NOT EXISTS (SELECT 1 FROM AIRobot WHERE AIRobot.id = Robot.id);

UPDATE MiningArea
SET aiRobotIdNew = aiRobotId
WHERE aiRobotIdNew IS NULL;

ALTER TABLE MiningArea
    DROP COLUMN aiRobotId;

ALTER TABLE MiningArea
    CHANGE aiRobotIdNew aiRobotId INT NOT NULL,
    ADD FOREIGN KEY (aiRobotId) REFERENCES AIRobot (id);

-- Only remove Robot rows that mining areas still treat as AI opponents.
DELETE Robot
FROM Robot
INNER JOIN MiningArea ON MiningArea.aiRobotId = Robot.id
INNER JOIN AIRobot ON AIRobot.id = Robot.id;
