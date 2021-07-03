#!/usr/bin/env bash
# Small install script to install the management script more conviently.

# Check we are sudo https://stackoverflow.com/questions/18215973/how-to-check-if-running-as-root-in-a-bash-script
if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

#Stop the service if it exists
systemctl stop virtual-machine-manager.service

# Copy the binary to the correct location
cp ../target/release/vm-restart-script /bin/vm-restart-script

# Create the service script
cp ./virtual-machine-manager.service /etc/systemd/system/virtual-machine-manager.service

systemctl enable virtual-machine-manager.service
systemctl start virtual-machine-manager.service