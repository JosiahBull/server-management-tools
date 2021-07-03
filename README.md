# Summary
This repo contains smaller rust cli projects that I use to remotely manage servers.
# No Internet Restart
## Description
A small rust CLI tool which automatically restarts the system if it doesn't have a connection to the internet in a configurable amount of time.

## Installation
```bash
git clone https://github.com/JosiahBull/server-management-tools.git
cd server-management-tools/no-internet-restart

#If you don't want the default config edit the consts
nano ./src/main.rs

#build
sudo cargo run --release

chmod +x install.sh
sudo ./install.sh

```

# Virtual Machine Manager
## Description
A small rust CLI tool to allow the automatic restarting of virtual machines if they become unproductive.

Sends a ping to f2pool, and collects the API data. If the mining rig has not sent a share in x minutes (configurable), then it automatically attempts to restart the virtual machine (at first gracefully, then forcefully).

## Installation
```bash
git clone https://github.com/JosiahBull/server-management-tools.git
cd server-management-tools/virtual-machine-management
sudo cargo run --release

#Navigate to the given uri and edit the config before the following step!
#e.g.:
nano ~/.config/virtual-machine-management/virtual-machine-management.toml

chmod +x install.sh
sudo ./install.sh
```

Note that this may spit out an error saying that it failed to stop a service, this will not prevent a successful installation.

IMPORTANT NOTE: This program has little-to-no validation of configuration. Make sure you get it right!



# Licensing
This project is licensed under MIT.
