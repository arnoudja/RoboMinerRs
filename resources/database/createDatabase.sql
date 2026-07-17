SET storage_engine=InnoDB;

drop view if exists TopRobotsView;

drop table if exists SchemaMigration;
drop table if exists PoolItemMiningTotals;
drop table if exists PoolItem;
drop table if exists Pool;
drop table if exists UserAchievement;
drop table if exists AchievementStepMiningScoreRequirement;
drop table if exists AchievementStepMiningTotalRequirement;
drop table if exists AchievementPredecessor;
drop table if exists AchievementStep;
drop table if exists Achievement;
drop table if exists RobotLifetimeResult;
drop table if exists RobotActionsDone;
drop table if exists MiningOreResult;
drop table if exists MiningQueue;
drop table if exists RobotMiningAreaScore;
drop table if exists RallyResult;
drop table if exists UserMiningArea;
drop table if exists MiningAreaLifetimeResult;
drop table if exists MiningAreaOreSupply;
drop table if exists MiningArea;
drop table if exists PendingRobotChanges;
drop table if exists Robot;
drop table if exists UserRobotPartAsset;
drop table if exists RobotPart;
drop table if exists RobotPartType;
drop table if exists ProgramSource;
drop table if exists UserOreAsset;
drop table if exists User;
drop table if exists OrePriceAmount;
drop table if exists OrePrice;
drop table if exists Ore;


create table Ore
(
id INT AUTO_INCREMENT PRIMARY KEY,
oreName VARCHAR(255) NOT NULL
);

create table OrePrice
(
id INT AUTO_INCREMENT PRIMARY KEY,
description VARCHAR(255) NOT NULL
);

create table OrePriceAmount
(
orePriceId INT NOT NULL REFERENCES OrePrice (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
amount INT NOT NULL,
PRIMARY KEY (orePriceId, oreId)
);


create table User
(
id INT AUTO_INCREMENT PRIMARY KEY,
username VARCHAR(255) NOT NULL UNIQUE,
email VARCHAR(255) NOT NULL UNIQUE,
password VARCHAR(255) NOT NULL,
achievementPoints INT NOT NULL DEFAULT 0,
miningQueueSize INT NOT NULL DEFAULT 0,
lastLoginTime TIMESTAMP NOT NULL DEFAULT NOW(),
INDEX (username),
INDEX (email),
INDEX (achievementPoints),
INDEX (lastLoginTime)
);


create table UserOreAsset
(
userId INT NOT NULL REFERENCES User (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
amount INT NOT NULL DEFAULT 0,
maxAllowed INT NOT NULL,
PRIMARY KEY (userId, oreId)
);


create table ProgramSource
(
id INT AUTO_INCREMENT PRIMARY KEY,
userId INT NOT NULL REFERENCES User (id) ON DELETE CASCADE,
sourceName VARCHAR(255) NOT NULL,
sourceCode TEXT,
verified BOOL NOT NULL DEFAULT FALSE,
compiledSize INT NOT NULL DEFAULT -1,
errorDescription VARCHAR(255),
INDEX (userId, id)
);


create table RobotPartType
(
id INT AUTO_INCREMENT PRIMARY KEY,
typeName VARCHAR(255) NOT NULL
);


create table RobotPart
(
id INT AUTO_INCREMENT PRIMARY KEY,
typeId INT NOT NULL REFERENCES RobotPartType (id) ON DELETE CASCADE,
tierId INT NULL REFERENCES Ore (id) ON DELETE SET NULL,
partName VARCHAR(255) NOT NULL,
orePriceId INT NOT NULL REFERENCES OrePrice (id),
oreCapacity INT NOT NULL DEFAULT 0,
miningCapacity INT NOT NULL DEFAULT 0,
batteryCapacity INT NOT NULL DEFAULT 0,
memoryCapacity INT NOT NULL DEFAULT 0,
cpuCapacity INT NOT NULL DEFAULT 0,
forwardCapacity INT NOT NULL DEFAULT 0,
backwardCapacity INT NOT NULL DEFAULT 0,
rotateCapacity INT NOT NULL DEFAULT 0,
rechargeTime INT NOT NULL DEFAULT 0,
scanTime INT NOT NULL DEFAULT 0,
scanDistance INT NOT NULL DEFAULT 0,
weight INT NOT NULL,
volume INT NOT NULL,
powerUsage INT NOT NULL
);


create table UserRobotPartAsset
(
userId INT NOT NULL REFERENCES User (id) ON DELETE CASCADE,
robotPartId INT NOT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
totalOwned INT NOT NULL DEFAULT 0,
PRIMARY KEY (userId, robotPartId)
);


create table Robot
(
id INT AUTO_INCREMENT PRIMARY KEY,
userId INT NOT NULL REFERENCES User (id) ON DELETE CASCADE,
robotName VARCHAR(255) NOT NULL,
sourceCode TEXT NOT NULL,
programSourceId INT NULL REFERENCES ProgramSource (id) ON DELETE SET NULL,
oreContainerId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
miningUnitId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
batteryId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
memoryModuleId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
cpuId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
engineId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
oreScannerId INT NULL REFERENCES RobotPart (id) ON DELETE SET NULL,
rechargeTime INT NOT NULL,
maxOre INT NOT NULL,
miningSpeed INT NOT NULL,
maxTurns INT NOT NULL,
memorySize INT NOT NULL DEFAULT 0,
cpuSpeed INT NOT NULL,
forwardSpeed DOUBLE NOT NULL,
backwardSpeed DOUBLE NOT NULL,
rotateSpeed INT NOT NULL,
robotSize DOUBLE NOT NULL,
scanTime INT NOT NULL DEFAULT 0,
scanDistance INT NOT NULL DEFAULT 0,
rechargeEndTime TIMESTAMP NOT NULL DEFAULT NOW(),
miningEndTime TIMESTAMP NULL,
totalMiningRuns INT NOT NULL DEFAULT 0,
INDEX (userId, id)
);


create table PendingRobotChanges
(
robotId INT PRIMARY KEY REFERENCES Robot (id) ON DELETE CASCADE,
submitTime TIMESTAMP NOT NULL DEFAULT NOW(),
sourceCode TEXT NOT NULL,
oreContainerId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
miningUnitId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
batteryId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
memoryModuleId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
cpuId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
engineId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oreScannerId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldOreContainerId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldMiningUnitId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldBatteryId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldMemoryModuleId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldCpuId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldEngineId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
oldOreScannerId INT NULL REFERENCES RobotPart (id) ON DELETE CASCADE,
rechargeTime INT NOT NULL,
maxOre INT NOT NULL,
miningSpeed INT NOT NULL,
maxTurns INT NOT NULL,
memorySize INT NOT NULL,
cpuSpeed INT NOT NULL,
forwardSpeed DOUBLE NOT NULL,
backwardSpeed DOUBLE NOT NULL,
rotateSpeed INT NOT NULL,
robotSize DOUBLE NOT NULL,
scanTime INT NOT NULL DEFAULT 0,
scanDistance INT NOT NULL DEFAULT 0,
changesCommitTime TIMESTAMP NULL
);


create table MiningArea
(
id INT AUTO_INCREMENT PRIMARY KEY,
areaName VARCHAR(255) NOT NULL,
orePriceId INT NOT NULL REFERENCES OrePrice (id),
sizeX INT NOT NULL,
sizeY INT NOT NULL,
maxMoves INT NOT NULL,
miningTime INT NOT NULL,
taxRate INT NOT NULL,
aiRobotId INT NOT NULL REFERENCES Robot (id)
);


create table MiningAreaOreSupply
(
id INT AUTO_INCREMENT PRIMARY KEY,
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
supply INT NOT NULL,
radius INT NOT NULL
);


create table MiningAreaLifetimeResult
(
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
totalAmount BIGINT NOT NULL,
totalContainerSize BIGINT NOT NULL,
PRIMARY KEY (miningAreaId, oreId)
);


create table UserMiningArea
(
userId INT NOT NULL REFERENCES User (id) ON DELETE CASCADE,
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
PRIMARY KEY (userId, miningAreaId)
);


create table RallyResult
(
id INT AUTO_INCREMENT PRIMARY KEY,
resultData MEDIUMTEXT NOT NULL
);


create table RobotMiningAreaScore
(
robotId INT NOT NULL REFERENCES Robot (id) ON DELETE CASCADE,
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
totalRuns INT NOT NULL DEFAULT 0,
score DOUBLE NOT NULL DEFAULT .0,
PRIMARY KEY (robotId, miningAreaId),
INDEX (miningAreaId, score)
);


create table MiningQueue
(
id INT AUTO_INCREMENT PRIMARY KEY,
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
robotId INT NOT NULL REFERENCES Robot (id) ON DELETE CASCADE,
rallyResultId INT NULL REFERENCES RallyResult (id) ON DELETE SET NULL,
playerNumber INT NULL,
score DOUBLE NULL,
creationTime TIMESTAMP NOT NULL DEFAULT NOW(),
miningEndTime TIMESTAMP NULL,
claimed BOOL NOT NULL DEFAULT FALSE,
executedSourceCode TEXT NULL,
INDEX (miningAreaId, rallyResultId)
);


create table MiningOreResult
(
miningQueueId INT NOT NULL REFERENCES MiningQueue (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
amount INT NOT NULL,
tax INT NULL,
PRIMARY KEY (miningQueueId, oreId)
);


create table RobotActionsDone
(
miningQueueId INT NOT NULL REFERENCES MiningQueue (id) ON DELETE CASCADE,
actionType INT NOT NULL,
amount INT NOT NULL,
PRIMARY KEY (miningQueueId, actionType)
);


create table RobotLifetimeResult
(
robotId INT NOT NULL REFERENCES Robot (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
amount INT NOT NULL,
tax INT NOT NULL,
PRIMARY KEY (robotId, oreId)
);


create table Achievement
(
id INT AUTO_INCREMENT PRIMARY KEY,
title VARCHAR(255) NOT NULL,
description TEXT NOT NULL
);

create table AchievementStep
(
achievementId INT NOT NULL REFERENCES Achievement (id) ON DELETE CASCADE,
step INT NOT NULL,
achievementPoints INT NOT NULL DEFAULT 10,
miningQueueReward INT NOT NULL DEFAULT 0,
robotReward INT NOT NULL DEFAULT 0,
miningAreaId INT NULL REFERENCES MiningArea (id) ON DELETE SET NULL,
oreId INT NULL REFERENCES Ore (id) ON DELETE SET NULL,
maxOreReward INT NOT NULL DEFAULT 0,
PRIMARY KEY (achievementId, step)
);


create table AchievementPredecessor
(
predecessorId INT NOT NULL REFERENCES Achievement (id) ON DELETE CASCADE,
predecessorStep INT NOT NULL,
successorId INT NOT NULL REFERENCES Achievement (id) ON DELETE CASCADE,
PRIMARY KEY (predecessorId, successorId)
);


create table AchievementStepMiningTotalRequirement
(
achievementId INT NOT NULL,
step INT NOT NULL,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
amount INT NOT NULL,
PRIMARY KEY (achievementId, step, oreId),
FOREIGN KEY (achievementId, step) REFERENCES AchievementStep (achievementId, step) ON DELETE CASCADE
);


create table AchievementStepMiningScoreRequirement
(
achievementId INT NOT NULL,
step INT NOT NULL,
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
minimumScore DOUBLE NOT NULL,
PRIMARY KEY (achievementId, step, miningAreaId),
FOREIGN KEY (achievementId, step) REFERENCES AchievementStep (achievementId, step) ON DELETE CASCADE
);


create table UserAchievement
(
userId INT NOT NULL REFERENCES User (id) ON DELETE CASCADE,
achievementId INT NOT NULL REFERENCES Achievement (id) ON DELETE CASCADE,
stepsClaimed INT NOT NULL DEFAULT 0,
PRIMARY KEY (userId, achievementId)
);


create view TopRobotsView
as
select Robot.id as robotId,
       Robot.robotName as robotName,
       User.username as username,
       Robot.totalMiningRuns as totalRuns,
       sum(RobotLifetimeResult.amount) as totalAmount,
       sum(RobotLifetimeResult.amount) / Robot.totalMiningRuns as orePerRun
from Robot
inner join User
on User.id = Robot.userId
left outer join RobotLifetimeResult
on RobotLifetimeResult.robotId = Robot.id
where Robot.totalMiningRuns > 0
group by Robot.id;


create table Pool
(
id INT AUTO_INCREMENT PRIMARY KEY,
miningAreaId INT NOT NULL REFERENCES MiningArea (id) ON DELETE CASCADE,
requiredRuns INT NOT NULL
);


create table PoolItem
(
id INT AUTO_INCREMENT PRIMARY KEY,
poolId INT NOT NULL REFERENCES Pool (id) ON DELETE CASCADE,
robotId INT NOT NULL REFERENCES Robot (id) ON DELETE CASCADE,
sourceCode TEXT NOT NULL,
totalScore DOUBLE NOT NULL DEFAULT 0,
runsDone INT NOT NULL DEFAULT 0,
INDEX (runsDone, totalScore, id)
);


create table PoolItemMiningTotals
(
poolItemId INT NOT NULL REFERENCES PoolItem (id) ON DELETE CASCADE,
oreId INT NOT NULL REFERENCES Ore (id) ON DELETE CASCADE,
totalMined BIGINT NOT NULL DEFAULT 0,
PRIMARY KEY (poolItemId, oreId)
);


create table SchemaMigration
(
version VARCHAR(64) PRIMARY KEY,
appliedAt TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);


grant SELECT,INSERT,UPDATE,DELETE,CREATE,ALTER,INDEX,LOCK TABLES on RoboMiner.* to robominer@localhost;
