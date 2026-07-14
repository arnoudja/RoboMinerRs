#!/usr/bin/env bash

echo "### Buidling RoboMiner ###"
resources/scripts/build-release.sh

echo "### Copying to robopi ###"
scp -r target/aarch64-unknown-linux-gnu/release/robominer-engine target/aarch64-unknown-linux-gnu/release/robominer-web robominer-web/static resources/database/*.sql robopi:/home/arnoud/deploy/

echo "### Running the deploy ###"
ssh -t robopi /home/arnoud/robominer_deploy.sh

echo "### Done ###"
