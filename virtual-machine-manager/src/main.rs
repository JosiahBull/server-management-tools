// Author: Josiah Bull, Copyright 2021
//! A CLI applet to monitor the status of a VM, and restart them in the event of a crash.

mod structs;

use chrono::prelude::*;
use std::{thread, time};
use std::process::Command;
use structs::*;

fn virtual_machine_online(vm: &str) -> Result<bool, MachineError> {
    let online_status = match Command::new("bash").args(["-c", &format!("virsh list --all | grep \"{}\" | awk '{{print $3}}'", vm)]).output() {
        Ok(f) => f.stdout,
        Err(e) => return Err(MachineError::CommandError(e.to_string())),
    };
    Ok(online_status == "running\n".as_bytes())
}

async fn get_mining_status(address: String, client: &reqwest::Client, cfg: &Config) -> Result<bool, ServerError> {
    println!("Collecting status of machine!");
    let data: APIResponse = client.get(address)
    .send()
    .await?
    .json()
    .await?;
    
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
        if name == cfg.mining_rig_name {
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
            if time_diff > chrono::Duration::minutes(cfg.offline_threshold_minutes as i64) {
                println!("Machine is offline!");
                return Ok(false);
            }
            return Ok(true);
        }
    }
    Ok(false) //Failed to find the mining server
}

fn restart_virtual_machine(cfg: &Config) -> Result<(), MachineError> {
    //Shutdown VM
    if virtual_machine_online(&cfg.virtual_machine_name)? {
        println!("Shutting down vm");
        match Command::new("bash").args(["-c".to_owned(), format!("virsh shutdown {}", cfg.virtual_machine_name)]).output() {
            Ok(_) => (),
            Err(e) => return Err(MachineError::CommandError(e.to_string())),
        };
        //Wait a given time to ensure proper shutdown.
        thread::sleep(time::Duration::from_secs(cfg.shutdown_time_seconds));

        //If not shutdown after this time, force a shutdown as the VM is not responding
        if virtual_machine_online(&cfg.virtual_machine_name)? {
            eprintln!("Force shutting down VM");
            match Command::new("bash").args(["-c".to_owned(), format!("virsh destroy {}", cfg.virtual_machine_name)]).output() {
                Ok(_) => (),
                Err(e) => return Err(MachineError::CommandError(e.to_string())),
            };
            thread::sleep(time::Duration::from_secs(5));
        }
    }
    //Restart VM
    println!("Starting virtual machine!");
    match Command::new("bash").args(["-c".to_owned(), format!("virsh start {}", cfg.virtual_machine_name)]).output() {
        Ok(f) => {
            let stdout: String = String::from_utf8(f.stdout).unwrap_or("FAILED TO PARSE OUTPUT".into());
            let stderr: String = String::from_utf8(f.stderr).unwrap_or("FAILED TO PARSE OUTPUT".into());

            if stdout.len() > 3 {
                println!("Machine started with output: {}", stdout);
            }
            if stderr.len() > 3 {
                println!("Machine produced Errors: {}", stderr);
            }
        },
        Err(e) => return Err(MachineError::CommandError(e.to_string())),
    };

    thread::sleep(time::Duration::from_secs(cfg.virtual_machine_restart_time_seconds));

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start_time = Local::now();
    println!("VM Monitoring script started on {}.", start_time.format("%a %b %e %T %Y"));
    println!("Loading config file...");
    let cfg: Config = match confy::load_path("/opt/secure-user/virtual-machine-mangement.conf") {
        Ok(f) => f,
        Err(e) => panic!("Failed to load config file! Edit the config file at {}. \nError: {}", "/opt/secure-user/virtual-machine-mangement.conf", e),
    };
    if cfg.mining_rig_name == "" {
        panic!("Failed to load config correctly! Please edit the config file at: {}", "/opt/secure-user/virtual-machine-mangement.conf");
    }
    println!("Loaded config!");

    let client = reqwest::Client::new();

    loop {
        match get_mining_status(format!("https://api.f2pool.com/eth/{}", &cfg.mining_account_name), &client, &cfg).await {
            Ok(f) => {
                if !f {
                    println!("Attempting to (re)start machine!");
                    match restart_virtual_machine(&cfg) {
                        Ok(_) => (),
                        Err(e) => eprintln!("An error occured attempting to start the virtual machine! Error: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("An error occured attempting to collect the status! Error: {}", e),
        };

        thread::sleep(time::Duration::from_secs(cfg.check_time_seconds));
    }
}
