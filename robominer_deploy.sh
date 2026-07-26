#!/usr/bin/env bash

set -euo pipefail

echo "### Stopping RoboMiner services ###"
sudo systemctl stop robominer-engine robominer-web

echo "### Placing binaries ###"
sudo install -D -m 0755 "deploy/robominer-engine" "/opt/robominer/bin/robominer-engine"
sudo install -D -m 0755 "deploy/robominer-web" "/opt/robominer/bin/robominer-web"

sudo install -D -m 0644 deploy/systemd/*.service "/etc/systemd/system/"
sudo install -D -m 0755 "deploy/systemd/wait-web-health.sh" "/opt/robominer/bin/robominer-wait-web-health"

sudo systemctl daemon-reload

echo "### Placing static contents ###"
sudo install -d -o robominer -g robominer -m 0755 "/opt/robominer/static/css"
sudo rsync -a --delete --chown=robominer:robominer --chmod=Du=rwx,Dg=rx,Do=rx,Fu=rw,Fg=r,Fo=r "deploy/static/css/" "/opt/robominer/static/css/"

sudo install -d -o robominer -g robominer -m 0755 "/opt/robominer/static/js"
sudo rsync -a --delete --chown=robominer:robominer --chmod=Du=rwx,Dg=rx,Do=rx,Fu=rw,Fg=r,Fo=r "deploy/static/js/" "/opt/robominer/static/js/"

echo "### Migrating database ###"
sudo /opt/robominer/bin/robominer-engine migrate apply
sudo /opt/robominer/bin/robominer-engine migrate status --check

echo "### Updating databases ###"
sudo mysql RoboMiner < deploy/gameData.sql

echo "### Starting RoboMiner services ###"
sudo systemctl start robominer-engine robominer-web

echo "### Done ###"
