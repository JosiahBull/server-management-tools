#!/usr/bin/env bash
# Author: Josiah Bull 2021
# Script to install the virtual machine management script more conviently.

service_exists() {
    local n=$1
    if [[ $(systemctl list-units --all -t service --full --no-legend "$n.service" | cut -f1 -d' ') == $n.service ]]; then
        return 0
    else
        return 1
    fi
}

# Check we are sudo https://stackoverflow.com/questions/18215973/how-to-check-if-running-as-root-in-a-bash-script
if [ "$EUID" -ne 0 ]
    then echo "Please run as root"
    exit
fi

# build
cargo build --release

#Stop the service if it exists
if service_exists virtual-machine-manager; then
    echo "Existing service found, stopping..."
    systemctl stop virtual-machine-manager.service
fi;
echo "Installing..."

# Copy the binary to the correct location
cp ./target/release/vm-restart-script /bin/vm-restart-script

# Create the service script
cp ./virtual-machine-manager.service /etc/systemd/system/virtual-machine-manager.service

systemctl enable virtual-machine-manager.service
systemctl start virtual-machine-manager.service