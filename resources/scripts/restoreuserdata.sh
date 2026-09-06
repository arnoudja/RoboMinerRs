#!/bin/sh
# DEPRECATED: prefer resources/scripts/backup-database.sh and
# resources/scripts/restore-database.sh (full dumps; password via MYSQL_PWD / URL).
# This script still accepts a DB password on argv (visible in process lists).
#

DBPassword=$1
SQLFile=$2

mysql -u robominer -p$DBPassword RoboMiner < $SQLFile
