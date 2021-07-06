//Author: Josiah Bull, Copyright 2021
//! A CLI applet to monitor the status of the internet connection on the current system, and reboot it if/when needed.

use std::{thread, time, path, fs, process::Command, io::prelude::*};
use dirs;

const PING_ADDRESS: &str = "https://www.google.com/";
const CHECK_TIME_MINUTES: u16 = 15;
const MAX_COUNT_OVERFLOW: u8 = 1;
const PROGRAM_NAME: &str = "no-internet-restart";

async fn heartbeat(address: &str) -> Result<(), reqwest::Error> {
    reqwest::get(address).await?;
    Ok(())
}

#[cfg(test)]
#[doc(hidden)]
mod heartbeat_tests {
    use crate::{heartbeat, PING_ADDRESS};
    #[tokio::test]
    async fn test_heartbeat_success() {
        heartbeat("https://google.com/").await.unwrap();
    }

    #[tokio::test]
    async fn test_heartbeat_success_default() {
        heartbeat(PING_ADDRESS).await.unwrap();
    }

    #[tokio::test]
    async fn test_heartbeat_failure() {
        heartbeat("https://doesntexist.com/aeflkj/eafe").await.unwrap_err();
    }
}

///Check to see if a boot record exists, this means we have rebooted at least once before. If the connection has not been resolved, then there must be a problem!
fn check_boot_record(path: &path::PathBuf) -> bool {
    path.exists()
}

fn create_boot_record(path: &path::PathBuf) -> Result<(), std::io::Error> {
    fs::File::create(path)?;
    Ok(())
}

fn remove_boot_record(path: &path::PathBuf) -> Result<(), std::io::Error> {
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
#[doc(hidden)]
mod boot_record_tests {
    use crate::*;
    use std::path::PathBuf;
    #[tokio::test]
    async fn create_boot_record_test() {
        let path = PathBuf::from("create_boot_record.test");
        create_boot_record(&path).unwrap();
        assert!(path.exists());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn remove_boot_record_test() {
        let path = PathBuf::from("remove_boot_record.test");
        create_boot_record(&path).unwrap();
        assert!(path.exists());
        remove_boot_record(&path).unwrap();
        assert!(!path.exists());
    }
    #[test]
    fn check_boot_record_test() {
        let path = PathBuf::from("check_boot_record.test");
        create_boot_record(&path).unwrap();
        assert_eq!(path.exists(), check_boot_record(&path));
        remove_boot_record(&path).unwrap();
        assert_eq!(path.exists(), check_boot_record(&path));
    }
}

fn restart() -> Result<(), String> {
    //Restart machine
    match Command::new("bash").args(["-c", "shutdown -r +1 'No internet.'"]).output() {
        Ok(_) => (),
        Err(e) => return Err(format!("{}", e)),
    };
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut config_path: path::PathBuf = dirs::config_dir().expect("Failed to find config directory!");
    config_path.push(PROGRAM_NAME);
    config_path.push("boot_record");
    config_path.set_extension(".conf");

    let mut count: u8 = 0;
    loop {
        //Placed at top to prevent a boot loop (i.e. you will always have at least CHECK_TIME_MINUTES to shutdown this service)
        thread::sleep(time::Duration::from_secs(CHECK_TIME_MINUTES as u64 * 60));

        if let Err(e) = heartbeat(PING_ADDRESS).await {
            eprintln!("Failed to acquire a heartbeat! Count: {} Error: {}", count, e);
            count = count + 1;
        } else {
            count = 0;
            if check_boot_record(&config_path) {
                if let Err(e) = remove_boot_record(&config_path) {
                    eprintln!("Failed to remove the boot record! Error: {}", e);
                }
            }
        }
        if count >= MAX_COUNT_OVERFLOW {
            if !check_boot_record(&config_path) {   
                if let Err(e) = create_boot_record(&config_path) {
                    eprintln!("Failed to create restart record! Error: {}", e);
                }
                if let Err(e) = restart() {
                    eprintln!("Failed to restart the server! Error: {}", e);
                }
            }
        }
    }
}

#[test]
fn test_consts() {
    if CHECK_TIME_MINUTES < 5 {
        panic!("Check time has been set too low!");
    }
    if CHECK_TIME_MINUTES > 1000 {
        panic!("Check time has been set too high!");
    }
    if MAX_COUNT_OVERFLOW > 10 {
        panic!("Max overflow count is too high!");
    }
    if MAX_COUNT_OVERFLOW < 1 {
        panic!("Max count overflow is set too low!");
    }
}
