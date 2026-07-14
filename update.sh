#!/usr/bin/env bash

echo "### Buidling and deploying RoboMiner ###"
deploy/systemd/install-robominer.sh

echo "### Updating databases ###"
sudo mysql RoboMiner < resources/database/gameData.sql
sudo mysql RoboMinerAccept < resources/database/gameData.sql

echo "### Restarting RoboMiner services ###"
sudo systemctl restart robominer-engine robominer-web

echo "### Done ###"
