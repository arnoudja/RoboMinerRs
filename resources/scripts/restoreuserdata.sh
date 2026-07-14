#!/bin/sh

DBPassword=$1
SQLFile=$2

mysql -u robominer -p$DBPassword RoboMiner < $SQLFile
