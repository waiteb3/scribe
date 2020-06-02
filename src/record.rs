use std::convert::From;
use std::io::Write;
use std::fs::{File, OpenOptions};

use log::info;
use rusqlite::{self, named_params};
use base64;

pub struct RecordError {
    pub cause: String
}

impl From<rusqlite::Error> for RecordError {
    fn from(err: rusqlite::Error) -> Self {
        RecordError {
            cause: format!("Underlying IndexSQL error occured: {:?}", err),
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

pub struct DataStores<W: Write> {
    index: rusqlite::Connection,
    archive: W,
}

pub fn init(home: std::path::PathBuf) -> Result<DataStores<File>, RecordError> {
    let index = rusqlite::Connection::open(home.join("data").join("index.db"))?;

    let latest = home.join("history").join("LATEST");
    let exists = latest.exists();

    let mut archive = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .truncate(false)
        .open(latest)?;

    if !exists {
        archive.write(format!("version=1,encoder=base64\n---\n").as_bytes())?;
    }

    let exec = index.execute_named(include_str!("etc/schema.sql"), named_params![]);
    if let Err(rusqlite::Error::SqliteFailure(code, Some(msg))) = exec {
        if msg != "table history already exists" {
            return Err(rusqlite::Error::SqliteFailure(code, Some(msg)).into());
        }
    } else {
        exec.map(|_| ())?;
    }

    Ok(DataStores{
        index,
        archive,
    })
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

pub fn append_history<W: Write>(mut deps: DataStores<W>, cmd: String) -> Result<(), RecordError> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let encoded = base64::encode(cmd.as_bytes());

    deps.archive.write(format!("{}:{}\n", now, encoded).as_bytes())?;
    deps.index.execute_named("INSERT INTO history(command, timestamp) VALUES (:command, :timestamp)", named_params!{
        ":command": cmd,
        ":timestamp": now as u32,
    })?;

    Ok(())
}