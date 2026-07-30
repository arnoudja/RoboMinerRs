#!/usr/bin/env bash

echo "### Building the package ###"
rm -f ./target/aarch64-unknown-linux-gnu/debian/robominer_*_arm64.deb
resources/scripts/build-deb.sh

echo "### Copying to robopi ###"
rsync -a --delete ./target/aarch64-unknown-linux-gnu/debian/ robopi:/home/arnoud/deploy/

echo "### Installing the package ###"
ssh -t robopi "sudo apt-get install /home/arnoud/deploy/robominer_*_arm64.deb"

echo "### Done ###"
