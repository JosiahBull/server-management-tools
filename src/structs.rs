//! Error and API response structs
use serde::{Serialize, Deserialize};

//Error Structs
#[derive(Debug)]
pub enum ServerError {
    HttpError(reqwest::Error),
    SerdeParseError(serde_json::Error),
    ParseError(String),
}

impl From<reqwest::Error> for ServerError {
    fn from(error: reqwest::Error) -> Self {
        ServerError::HttpError(error)
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(error: serde_json::Error) -> Self {
        ServerError::SerdeParseError(error)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", &*self))
    }
}

#[derive(Debug)]
pub enum MachineError {
    CommandError(String),
}

impl std::fmt::Display for MachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", &*self))
    }
}

//HTTP Reponse Structs

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    pub hashrate: f64,
    pub workers: Vec<Vec<serde_json::value::Value>>,
}