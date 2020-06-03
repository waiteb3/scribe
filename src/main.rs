// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

use std::convert::From;

use termion;

mod init;
mod debug;
mod search;
mod record;

#[derive(Debug)]
struct ScribeError {
    text: String,
}

impl std::fmt::Display for ScribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "scribe encountered a fatal error: {}", self.text)
    }
}

impl std::error::Error for ScribeError {}

impl From<std::io::Error> for ScribeError {
    fn from(err: std::io::Error) -> Self {
        ScribeError{ text: format!("IO Error Occured: {}", err) }
    }
}

impl From<log::SetLoggerError> for ScribeError {
    fn from(_: log::SetLoggerError) -> Self {
        ScribeError{ text: format!("Unable to configure debug log file") }
    }
}

impl From<init::InitError> for ScribeError {
    fn from(err: init::InitError) -> Self {
        ScribeError{ text: format!("Failure occured during 'init' command: {}", err.cause) }
    }
}

impl From<record::RecordError> for ScribeError {
    fn from(err: record::RecordError) -> Self {
        ScribeError{ text: format!("Failure occured during 'record' command: {}", err.cause) }
    }
}

impl From<search::SearchError> for ScribeError {
    fn from(err: search::SearchError) -> Self {
        ScribeError{ text: format!("Failure occured during 'init' command: {}", err.cause) }
    }
}

fn init() -> Result<bool, ScribeError> {
    let mut new = false;
    for dir in init::dirs() {
        let path = init::scribe_dir()?.join(dir);
        if !std::path::Path::exists(&path) {
            new = true;
            std::fs::create_dir_all(path)?;
        }
    }

    Ok(new)
}

fn main() -> Result<(), ScribeError> {
    let fresh = init()?;
    debug::init(init::scribe_dir()?)?;

    let args: Vec<String> = std::env::args().collect();
    let (_command, full_flags) = args.split_first().ok_or(ScribeError{
        text: format!("Unknown error: Unable to parse program arguments from '{:?}", args),
    })?;

    if full_flags.len() < 1 {
        println!("HELP TEXT TODO");
        return Ok(())
    }

    let (subcommand, flags) = full_flags.split_first().ok_or(ScribeError{
        text: format!("Unknown error: Unable to parse program subcommand and arguments from '{:?}", full_flags),
    })?;

    match subcommand.as_str() {
        // TODO split init into two cmds
        "init" | "bind" => {
            if fresh {
                let deps = init::deps(init::scribe_dir()?)?;
                init::import_history(deps)?;
            }
            Ok(init::env_init()?)
        }
        "record" => {
            let deps = init::deps(init::scribe_dir()?)?;

            let cmd = flags.join(" ");
            match record::precheck(cmd.clone()) {
                record::Precheck::Append => {
                    Ok(record::append_history(deps, cmd)?)
                }
                record::Precheck::Skip => {
                    Ok(())
                }
                record::Precheck::Unset => {
                    Ok(println!("release-hooks"))
                }
            }
        }
        "search" if flags.len() == 0 => {
            Err(ScribeError{ text: format!("Search requires at least 1 argument") })
        }
        "search" => {
            let deps = init::deps(init::scribe_dir()?)?;

            let mut tty = termion::get_tty()?;
            let mut reader = tty.try_clone()?;
            let mut writer = tty.try_clone()?;

            // TODO separate subcommand
            if flags.get(0).unwrap() == &String::from("--interactive") {
                let response = search::interactive(deps, &mut tty, &mut reader, &mut writer)?;
                if let Some(response) = response {
                    println!("{}", response);
                }
            } else {
                let matches = search::find_recent_matches(deps, flags.join(" ").to_string())?;
                for m in matches.iter() {
                    println!("{}", m);
                }
            }
            Ok(())
        }
        _ => {
            Err(ScribeError{ text: format!("Unknown subcommand {}", subcommand) })
        }
    }
}