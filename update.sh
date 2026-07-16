#!/usr/bin/env bash

set -euo pipefail

echo "### Stopping RoboMiner services ###"
sudo systemctl stop robominer-engine robominer-web

echo "### Buidling and deploying RoboMiner ###"
deploy/systemd/install-robominer.sh

echo "### Migrating database ###"
sudo /opt/robominer/bin/robominer-engine migrate

echo "### Updating databases ###"
sudo mysql RoboMiner < resources/database/gameData.sql
sudo mysql RoboMinerAccept < resources/database/gameData.sql

echo "### Restarting RoboMiner services ###"
sudo systemctl restart robominer-engine robominer-web

echo "### Done ###"
