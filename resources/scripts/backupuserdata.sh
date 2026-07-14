#!/bin/sh

DBPassword=$1
SQLFile=$2

mysqldump --compact --no-create-info --complete-insert --skip-tz-utc -u robominer -p$DBPassword RoboMiner User --where='id <> 1' > $SQLFile

mysqldump --compact --no-create-info --complete-insert --skip-tz-utc -u robominer -p$DBPassword RoboMiner UserOreAsset ProgramSource UserRobotPartAsset >> $SQLFile

mysqldump --compact --no-create-info --complete-insert --skip-tz-utc -u robominer -p$DBPassword RoboMiner Robot --where='userId <> 1' >> $SQLFile

mysqldump --compact --no-create-info --complete-insert --skip-tz-utc -u robominer -p$DBPassword RoboMiner PendingRobotChanges MiningAreaLifetimeResult UserMiningArea RallyResult RobotMiningAreaScore MiningQueue MiningOreResult RobotActionsDone RobotLifetimeResult UserAchievement >> $SQLFile
