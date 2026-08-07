-- Migration 006: move AI opponents from Robot into AIRobot so seeded AI ids
-- cannot collide with player Robot AUTO_INCREMENT rows.
--
-- MariaDB/MySQL refuse DROP COLUMN while a foreign key still uses that column's
-- index, so drop the aiRobotId FK by name before replacing the column.

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

SET @fk_name := (
    SELECT CONSTRAINT_NAME
    FROM information_schema.KEY_COLUMN_USAGE
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'MiningArea'
      AND COLUMN_NAME = 'aiRobotId'
      AND REFERENCED_TABLE_NAME IS NOT NULL
    LIMIT 1
);
SET @drop_fk_sql := IF(
    @fk_name IS NULL,
    'SELECT 1',
    CONCAT('ALTER TABLE MiningArea DROP FOREIGN KEY `', @fk_name, '`')
);
PREPARE drop_fk_stmt FROM @drop_fk_sql;
EXECUTE drop_fk_stmt;
DEALLOCATE PREPARE drop_fk_stmt;

SET @has_new := (
    SELECT COUNT(*)
    FROM information_schema.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'MiningArea'
      AND COLUMN_NAME = 'aiRobotIdNew'
);
SET @add_new_sql := IF(
    @has_new > 0,
    'SELECT 1',
    'ALTER TABLE MiningArea ADD COLUMN aiRobotIdNew INT NULL'
);
PREPARE add_new_stmt FROM @add_new_sql;
EXECUTE add_new_stmt;
DEALLOCATE PREPARE add_new_stmt;

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
