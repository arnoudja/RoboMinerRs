#!/usr/bin/env bash

echo "### Buidling and deploying RoboMiner ###"
deploy/systemd/install-robominer.sh

echo "### Stopping RoboMiner services ###"
sudo systemctl stop robominer-engine robominer-web

echo "### Removing old databases ###"
echo "DROP DATABASE IF EXISTS RoboMiner;" | sudo mysql 
echo "DROP DATABASE IF EXISTS RoboMinerAccept;" | sudo mysql 

echo "### Create databases ###"
echo "CREATE DATABASE RoboMiner;" | sudo mysql
echo "CREATE DATABASE RoboMinerAccept;" | sudo mysql

echo "### Create tables ###"
sudo mysql RoboMiner < resources/database/createDatabase.sql
sudo mysql RoboMinerAccept < resources/database/createDatabase.sql

echo "### Adding game data ###"
sudo mysql RoboMiner < resources/database/gameData.sql
sudo mysql RoboMinerAccept < resources/database/gameData.sql

echo "### Starting RoboMiner services ###"
sudo systemctl start robominer-engine robominer-web

echo "### Done ###"
