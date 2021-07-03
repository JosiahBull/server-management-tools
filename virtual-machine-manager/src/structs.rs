//! Error and API response structs
use serde::{Serialize, Deserialize};

//Config
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    ///How often to ping F2Pool for latest mining time
    pub check_time_seconds: u64,
    ///How long a machine has to be offline for before we restart it
    pub offline_threshold_minutes: u64,
    ///The name of the account you are using to mine on F2Pool
    pub mining_account_name: String,
    ///Name of mining machine to check on F2Pool
    pub mining_rig_name: String,
    ///Name of virtual machine to restart
    pub virtual_machine_name: String,
    ///How long to wait for a virtual machine to shutdown gracefully
    pub shutdown_time_seconds: u64,
    ///How long to wait for a virtual machine to start correctly
    pub virtual_machine_restart_time_seconds: u64,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            check_time_seconds: 120,
            offline_threshold_minutes: 20,
            mining_account_name: "".into(),
            mining_rig_name: "".into(),
            virtual_machine_name: "".into(),
            shutdown_time_seconds: 20,
            virtual_machine_restart_time_seconds: 180,
        }
    }
}


//Error Structs
#[derive(Debug)]
pub enum ServerError {
    HttpError(reqwest::Error),
    ParseError(String),
}

impl From<reqwest::Error> for ServerError {
    fn from(error: reqwest::Error) -> Self {
        ServerError::HttpError(error)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum MachineError {
    CommandError(String),
}

impl std::fmt::Display for MachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//HTTP Reponse Structs

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    pub hashrate: f64,
    pub workers: Vec<Vec<serde_json::value::Value>>,
}