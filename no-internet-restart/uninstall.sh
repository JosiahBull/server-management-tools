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
rm /opt/secure-user/no-internet-restart
rm /etc/systemd/system/wifi-restart.service

#Remove User and Group
#N.b. we expect to write other applications that may re-use this user, so lets ask if the user really wants to remove him.
read -p "Do you want to remove the secure-user? Y/N" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    userdel secure-user -r
    groupdel secure-user
else 
    yes | rm /opt/secure-user/no-internet-restart*
fi

#Reset systemctl on advice of https://superuser.com/questions/513159/how-to-remove-systemd-services
systemctl daemon-reload
systemctl reset-failed