use std::convert::From;
use std::error::Error;
use std::env::VarError;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{prelude::*, BufReader};

use rusqlite::named_params;

use super::record;

#[derive(Debug)]
pub struct InitError {
    pub cause: String,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Error for InitError {}

impl From<VarError> for InitError {
    fn from(err: VarError) -> Self {
        match err {
            VarError::NotPresent => {
                InitError{ cause: "SHELL env-var was not defined".to_owned() }
            }
            VarError::NotUnicode(text) => {
                InitError{ cause: format!("SHELL env-var encountered a unicode error: {:?}", text) }
            }
        }
    }
}

impl From<std::io::Error> for InitError {
    fn from(err: std::io::Error) -> Self {
        InitError{ cause: format!("IO Error: {}", err) }
    }
}

impl From<rusqlite::Error> for InitError {
    fn from(err: rusqlite::Error) -> Self {
        InitError {
            cause: format!("Underlying IndexSQL (sqlite3) error occured: {:?}", err),
        }
    }
}

enum Shell {
    ZSH,
    FISH,
    BASH,
}

fn current_shell() -> Result<Shell, InitError> {
    let shell = std::env::var("SHELL")?.as_str().to_lowercase();
    if shell.contains("zsh") {
        Ok(Shell::ZSH)
    } else if shell.contains("fish") {
        Ok(Shell::FISH)
    } else if shell.contains("bash") {
        Ok(Shell::BASH)
    } else {
        Err(InitError{
            cause: format!("'{}' is not a supported shell", shell),
        })
    }
}

pub fn scribe_dir() -> Result<std::path::PathBuf, InitError> {
    let home = dirs::home_dir().ok_or(InitError{ cause: String::from("Unable to detect home dir, most likely $HOME is not set") })?;
    if let Ok(dir) = std::env::var("SCRIBE_DIR") {
        Ok(dir.into())
    } else {
        Ok(home.join(".scribe"))
    }
}

pub fn dirs() -> Vec<String> {
    vec![
        String::from("data"),
        String::from("log"),
        String::from("history"),
    ]
}

pub fn env_init() -> Result<(), InitError> {
    match current_shell()? {
        Shell::ZSH => {
            println!("{}", include_str!("etc/init.zsh"));
        }
        Shell::FISH => {
            println!("{}", include_str!("etc/init.fish"));
        }
        Shell::BASH => {
            println!("{}", include_str!("etc/init.bash"));
        }
    }
    Ok(())
}

pub struct DataStores {
    pub index: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    pub archive: File,
}

impl std::clone::Clone for DataStores {
    fn clone(&self) -> Self {
        DataStores{
            index: self.index.clone(),
            archive: self.archive.try_clone().unwrap(),
        }
    }
}

pub fn deps(home: std::path::PathBuf) -> Result<DataStores, InitError> {
    let index = rusqlite::Connection::open(home.join("data").join("index.db"))?;
    index.execute_named("PRAGMA case_sensitive_like=ON", named_params! {})?;

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
        index: std::sync::Arc::new(std::sync::Mutex::new(index)),
        archive,
    })
}

fn import_zsh_history(deps: DataStores) -> Result<(), InitError> {
    let home = dirs::home_dir().ok_or(InitError{
        cause: format!("$HOME was not set while trying to import zsh history into the index")
    })?;
    let histfile = home.join(".zsh_history");

    if !histfile.exists() {
        log::warn!("ZSH_history file was not found on import attemp");
        return Ok(());
    }

    let history = std::fs::File::open(histfile)?;
    let reader = BufReader::new(history);
    for line in reader.lines() {
        record::append_history(deps.clone(), line?).or_else(|e| Err(InitError{
            cause: format!("Unable to complete import from ZSH history: {}", e.cause)
        }))?;
    }
    Ok(())
}

pub fn import_history(deps: DataStores) -> Result<(), InitError>  {
    match current_shell()? {
        Shell::ZSH => {
            import_zsh_history(deps)
        }
        Shell::FISH => {
            Ok(())
        }
        Shell::BASH => {
            Ok(())
        }
    }
}