#!/usr/bin/env bash
# Author: Josiah Bull 2021
# Script to uninstall the virtual machine management script more conviently.

# Check we are sudo https://stackoverflow.com/questions/18215973/how-to-check-if-running-as-root-in-a-bash-script
if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

#Stop the service
systemctl stop wifi-restart.service
systemctl disable wifi-restart.service

# Remove files
rm  /bin/no-internet-restart
rm /etc/systemd/system/wifi-restart.service

#Reset systemctl on advice of https://superuser.com/questions/513159/how-to-remove-systemd-services
systemctl daemon-reload
systemctl reset-failed