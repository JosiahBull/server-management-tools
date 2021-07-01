// Author: Josiah Bull, Copyright 2021
//! A CLI applet to monitor the status of a VM, and restart them in the event of a crash.

mod structs;

use chrono::prelude::*;
use std::{thread, time};
use std::process::Command;
use structs::*;

///How often to ping F2Pool for latest mining time
const CHECK_TIME_SECONDS: u64 = 120;
///How long a machine has to be offline for before we restart it
const OFFLINE_THRESHOLD_MINUTES: i64 = 20;
///The name of the account you are using to mine on F2Pool
const MINING_ACCOUNT_NAME: &str = "";
///Name of mining machine to check on F2Pool
const MINING_RIG_NAME: &str = "";
///Name of virtual machine to restart
const VIRTUAL_MACHINE_NAME: &str = "";
///How long to wait for a virtual machine to shutdown gracefully
const SHUTDOWN_TIME_SECONDS: u64 = 20;
///How long to wait for a virtual machine to start correctly
const VIRTUAL_MACHINE_RESTART_TIME_SECONDS: u64 = 180;

fn virtual_machine_online(vm: &str) -> Result<bool, MachineError> {
    let online_status = match Command::new("bash").args(["-c", &format!("virsh list --all | grep \"{}\" | awk '{{print $3}}'", vm)]).output() {
        Ok(f) => f.stdout,
        Err(e) => return Err(MachineError::CommandError(e.to_string())),
    };
    Ok(online_status == "running\n".as_bytes())
}

async fn get_mining_status(address: String, client: &reqwest::Client, vm: &str) -> Result<bool, ServerError> {
    let body = client.get(address)
    .send()
    .await?
    .text()
    .await?;
    
    let data: APIResponse = serde_json::from_str(&body)?;
    if data.workers.is_empty() {
        return Err(ServerError::ParseError("No workers exist for this client!".into()));
    }
    //Loop over the workers for our worker
    for worker in data.workers.iter() {
        if !worker[0].is_string() {
            return Err(ServerError::ParseError("Worker name should be a string!".into()));
        }
        let name: String = match worker[0].as_str() {
            Some(f) => f.to_owned(),
            None => return Err(ServerError::ParseError("Failed to parse worker name as string.".into())),
        };
        if name == vm {
            if !worker[6].is_string() {
                return Err(ServerError::ParseError("Worker last submitted share should be a string!".into()));
            }
            let datetime: String = match worker[6].as_str() {
                Some(f) => f.to_owned(),
                None => return Err(ServerError::ParseError("Failed to parse worker last submitted share as string.".into())),
            };
            let datetime = match DateTime::parse_from_rfc3339(&datetime) {
                Ok(f) => f,
                Err(e) => return Err(ServerError::ParseError(format!("Failed to parse worker last submitted share as datetime. {}", e))),
            };
            let local: DateTime<Utc> = Utc::now();
            let time_diff = datetime.signed_duration_since(local) * -1;
            if time_diff < chrono::Duration::minutes(0) {
                return Err(ServerError::ParseError("Somehow we submitted the most recent share in the future?".into()));
            }
            if time_diff > chrono::Duration::minutes(OFFLINE_THRESHOLD_MINUTES) {
                return Ok(false);
            }
            return Ok(true);
        }
    }

    Ok(false) //Failed to find the mining server
}

fn restart_virtual_machine(vm: &str) -> Result<(), MachineError> {
    //Shutdown VM
    if virtual_machine_online(vm)? {
        println!("Shutting down vm");
        match Command::new("bash").args(["-c", "virsh shutdown test-vm"]).output() {
            Ok(_) => (),
            Err(e) => return Err(MachineError::CommandError(e.to_string())),
        };
        //Wait a given time to ensure proper shutdown.
        thread::sleep(time::Duration::from_secs(SHUTDOWN_TIME_SECONDS));

        //If not shutdown after this time, force a shutdown as the VM is not responding
        if virtual_machine_online(vm)? {
            eprintln!("Force shutting down VM");
            match Command::new("bash").args(["-c", "virsh destroy test-vm"]).output() {
                Ok(_) => (),
                Err(e) => return Err(MachineError::CommandError(e.to_string())),
            };
            thread::sleep(time::Duration::from_secs(5));
        }
    }
    //Restart VM
    match Command::new("bash").args(["-c", "virsh start test-vm"]).output() {
        Ok(_) => (),
        Err(e) => return Err(MachineError::CommandError(e.to_string())),
    };

    thread::sleep(time::Duration::from_secs(VIRTUAL_MACHINE_RESTART_TIME_SECONDS));

    Ok(())
}

#[tokio::main]
async fn main() {
    let start_time = Local::now();
    println!("VM Monitoring script started on {}.", start_time.format("%a %b %e %T %Y"));

    let client = reqwest::Client::new();

    loop {
        match get_mining_status(format!("https://api.f2pool.com/eth/{}", MINING_ACCOUNT_NAME), &client, MINING_RIG_NAME).await {
            Ok(f) => {
                if !f {
                    match restart_virtual_machine(VIRTUAL_MACHINE_NAME) {
                        Ok(_) => (),
                        Err(e) => eprintln!("An error occured attempting to start the virtual machine! Error: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("An error occured attempting to collect the status! Error: {}", e),
        };

        thread::sleep(time::Duration::from_secs(CHECK_TIME_SECONDS));
    }
}
