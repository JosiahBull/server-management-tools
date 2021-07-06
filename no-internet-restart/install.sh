#!/usr/bin/env bash
# Author: Josiah Bull 2021
# Script to install the wifi-restart program conviently.

service_exists() {
    local n=$1
    if [[ $(systemctl list-units --all -t service --full --no-legend "$n.service" | cut -f1 -d' ') == $n.service ]]; then
        return 0
    else
        return 1
    fi
}

# Check we are sudo
if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

# cargo build --release

#Stop the service if it exists
if service_exists wifi-restart; then
    echo "Existing service found, stopping..."
    systemctl stop wifi-restart.service
fi


# Copy the binary to the correct location
cp ./target/release/no-internet-restart /bin/no-internet-restart

# Create the service script
cp ./wifi-restart.service /etc/systemd/system/wifi-restart.service

systemctl enable wifi-restart.service
systemctl start wifi-restart.service