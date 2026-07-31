#!/usr/bin/env bash

set -euo pipefail

echo "### Updating the RoboMiner database ###"
sudo mysql RoboMiner < resources/database/gameData.sql

echo "### Updating the RoboMinerAccept database ###"
sudo mysql RoboMinerAccept < resources/database/gameData.sql

echo "### Done ###"
