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

#Check the release file has been built, and exit if it hasn't.
if [ ! -f ./target/release/vm-restart-script ]; then
    echo "Cannot find release, don't forget to cargo build --release before running this script!"
    exit
fi

mkdir -p /opt/secure-user

#Create group secure-user if not exist
[ $(getent group secure-user) ] || groupadd secure-user

#Create user if not exist
id -u secure-user &>/dev/null || useradd --system --shell /usr/sbin/nologin --home /opt/secure-user -g secure-user secure-user

#Set permissions of home folder
chown secure-user:secure-user /opt/secure-user

#Stop the service if it exists
if service_exists virtual-machine-manager; then
    echo "Existing service found, stopping..."
    systemctl stop virtual-machine-manager.service
fi;
echo "Installing..."

# Copy the binary to the correct location
cp ./target/release/vm-restart-script /opt/secure-user/vm-restart-script

# Create the service script
cp ./virtual-machine-manager.service /etc/systemd/system/virtual-machine-manager.service

systemctl enable virtual-machine-manager.service
systemctl start virtual-machine-manager.service