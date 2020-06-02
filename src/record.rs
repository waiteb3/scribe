use std::convert::From;
use std::io::Write;

use super::init;

use rusqlite::{self, named_params};
use base64;

pub struct RecordError {
    pub cause: String
}

impl From<rusqlite::Error> for RecordError {
    fn from(err: rusqlite::Error) -> Self {
        RecordError {
            cause: format!("Underlying IndexSQL (sqlite3) error occured: {:?}", err),
        }
    }
}

impl From<std::io::Error> for RecordError {
    fn from(err: std::io::Error) -> Self {
        RecordError {
            cause: format!("Underlying IndexSQL error occured: {:?}", err),
        }
    }
}

impl From<std::time::SystemTimeError> for RecordError {
    fn from(err: std::time::SystemTimeError) -> Self {
        RecordError {
            cause: format!("SystemTime error: {}", err)
        }
    }
}

pub enum Precheck {
    Append,
    Skip,
    Unset,
}

pub fn precheck(cmd: String) -> Precheck {
    match cmd.get(0..1) {
        Some(c) => {
            if c.chars().any(|c| c.is_whitespace()) {
                Precheck::Skip
            } else if cmd.eq("unset HISTFILE") {
                Precheck::Unset
            } else {
                Precheck::Append
            }
        }
        None => {
            Precheck::Skip
        }
    }
}

pub fn append_history(deps: init::DataStores, cmd: String) -> Result<(), RecordError> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let encoded = base64::encode(cmd.as_bytes());

    deps.archive.try_clone()?.write(format!("{}:{}\n", now, encoded).as_bytes())?;
    deps.index.try_lock().unwrap().execute_named("INSERT INTO history(command, timestamp) VALUES (:command, :timestamp)", named_params!{
        ":command": cmd,
        ":timestamp": now as u32,
    })?;

    Ok(())
}