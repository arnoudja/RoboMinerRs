#!/usr/bin/env bash

set -euo pipefail

echo "### Stopping RoboMiner services ###"
sudo systemctl stop robominer-engine robominer-web

echo "### Placing binaries ###"
sudo install -D -m 0755 "deploy/robominer-engine" "/opt/robominer/bin/robominer-engine"
sudo install -D -m 0755 "deploy/robominer-web" "/opt/robominer/bin/robominer-web"

echo "### Placing static contents ###"
sudo install -m 0644 "deploy/static/css/robominer.css" "/opt/robominer/static/css/robominer.css"

echo "### Migrating database ###"
sudo /opt/robominer/bin/robominer-engine migrate

echo "### Updating databases ###"
sudo mysql RoboMiner < deploy/gameData.sql

echo "### Starting RoboMiner services ###"
sudo systemctl start robominer-engine robominer-web

echo "### Done ###"
