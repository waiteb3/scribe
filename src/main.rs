// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

use termion;

use log::{info};
mod debug;
mod init;
mod search;

fn main() {
    debug::init().unwrap();

    // info!("Test");

    let args: Vec<String> = std::env::args().collect();
    let (_command, full_flags) = args.split_first().unwrap();
    let (subcommand, flags) = full_flags.split_first().unwrap();

    match subcommand.as_str() {
        "init" => {
            init::env_init().unwrap();
            return;
        }
        "record" => {
            return;
        }
        "search" if flags.len() == 0 => {
            panic!("Search requires at least 1 argument");
        }
        "search" => {
            let mut tty = termion::get_tty().unwrap();
            let mut reader = tty.try_clone().unwrap();
            let mut writer = tty.try_clone().unwrap();
        
            // TODO separate subcommand
            if flags.get(0).unwrap() == &String::from("--interactive") {
                let response = search::interactive(&mut tty, &mut reader, &mut writer);
                if let Some(response) = response {
                    println!("{}", response);
                }
            } else {
                let cursor = & mut search::Cursor{ count: 10, direction: search::Direction::Up };
                let matches = search::matches(cursor, flags.join(" ").to_string());
                for m in matches.iter() {
                    println!("{}", m);
                }
            }
        }
        _ => {
            panic!("Unknown subcommand {}", subcommand);
        }
    }
}