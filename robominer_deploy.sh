#!/usr/bin/env bash

set -euo pipefail

echo "### Stopping RoboMiner services ###"
sudo systemctl stop robominer-engine robominer-web

echo "### Placing binaries ###"
sudo install -D -m 0755 "deploy/robominer-engine" "/opt/robominer/bin/robominer-engine"
sudo install -D -m 0755 "deploy/robominer-web" "/opt/robominer/bin/robominer-web"

echo "### Placing static contents ###"
sudo install -d -o robominer -g robominer -m 0755 "/opt/robominer/static/css"
sudo rsync -a --delete --chown=robominer:robominer --chmod=Du=rwx,Dg=rx,Do=rx,Fu=rw,Fg=r,Fo=r "deploy/static/css/" "/opt/robominer/static/css/"

sudo install -d -o robominer -g robominer -m 0755 "/opt/robominer/static/js"
sudo rsync -a --delete --chown=robominer:robominer --chmod=Du=rwx,Dg=rx,Do=rx,Fu=rw,Fg=r,Fo=r "deploy/static/js/" "/opt/robominer/static/js/"

echo "### Migrating database ###"
sudo /opt/robominer/bin/robominer-engine migrate

echo "### Updating databases ###"
sudo mysql RoboMiner < deploy/gameData.sql

echo "### Starting RoboMiner services ###"
sudo systemctl start robominer-engine robominer-web

echo "### Done ###"
