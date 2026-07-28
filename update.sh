#!/usr/bin/env bash

set -euo pipefail

echo "### Building the package ###"
rm -f ./target/debian/robominer_*_amd64.deb
resources/scripts/build-deb.sh

echo "### Installing the package ###"
sudo apt-get install --reinstall ./target/debian/robominer_*_amd64.deb

echo "### Done ###"
