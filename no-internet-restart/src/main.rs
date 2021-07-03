//Author: Josiah Bull, Copyright 2021
//! A CLI applet to monitor the status of the internet connection on the current system, and reboot it if/when needed.

use std::{thread, time};
use std::process::Command;

const PING_ADDRESS: &str = "https://www.google.com/";
const CHECK_TIME_MINUTES: u16 = 15;
const MAX_COUNT_OVERFLOW: u8 = 1;

async fn heartbeat(address: &str) -> Result<(), reqwest::Error> {
    reqwest::get(address).await?;
    Ok(())
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
    let mut count: u8 = 0;
    loop {
        //Placed at top to prevent a boot loop (i.e. you will always have at least CHECK_TIME_MINUTES to shutdown this service)
        thread::sleep(time::Duration::from_secs(CHECK_TIME_MINUTES as u64 * 60));

        if let Err(e) = heartbeat(PING_ADDRESS).await {
            eprintln!("Failed to acquire a heartbeat! Count: {} Error: {}", count, e);
            count = count + 1;
        } else {
            count = 0;
        }
        if count >= MAX_COUNT_OVERFLOW {
            if let Err(e) = restart() {
                eprintln!("Failed to restart the server! Error: {}", e);
            };
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
