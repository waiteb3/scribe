// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

//! This shows how an application can write on stderr
//! instead of stdout, thus making it possible to
//! the command API instead of the "old style" direct
//! unbuffered API.
//!
//! This particular example is only suited to Unix
//! for now.
//!
//! cargo run --example stderr

mod data;

use std::error;
use std::fmt;
use std::io::{stdout, stderr, Write};

use termion;

// #[derive(Debug)]
// enum ErrorKind {
//     Crossterm(crossterm::ErrorKind),
//     Scribe(ScribeError),
//     Io(std::io::Error),
// }

// type Result<T> = std::result::Result<T, ErrorKind>;

// macro_rules! impl_from {
//     ($from:path, $to:expr) => {
//         impl From<$from> for ErrorKind {
//             fn from(e: $from) -> Self {
//                 $to(e)
//             }
//         }
//     };
// }

// impl_from!(crossterm::ErrorKind, ErrorKind::Crossterm);
// impl_from!(ScribeError, ErrorKind::Scribe);
// impl_from!(std::io::Error, ErrorKind::Io);

// #[derive(Debug, Clone)]
// struct ScribeError {
//     inner: String
// }

// impl fmt::Display for ScribeError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "invalid first item to double")
//     }
// }

// // This is important for other errors to wrap this one.
// impl error::Error for ScribeError {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         // Generic error, underlying cause isn't tracked.
//         None
//     }
// }

// const MATCHES: [&str; 8] = [
//     "echo test",
//     "echo fail",
//     "echo feed",
//     "echo find",
//     "echo free",
//     "echo asdf",
//     "echo zxcv",
//     "echo qwer",
// ];

fn run_app(writer: &mut Write) -> Option<String> {

}

// fn run_app(writer: &mut Write) -> Result<String>
// {
//     // queue!(
//     //     writer,
//     //     EnterAlternateScreen, // enter alternate screen
//     //     Hide,                  // hide the cursor
//     // )?;

//     terminal::enable_raw_mode().unwrap();

//     let user_char = prompt_search(writer);

//     terminal::disable_raw_mode().unwrap();

//     // execute!(writer, Show, LeaveAlternateScreen)?; // restore the cursor and leave the alternate screen

//     user_char
// }

// fn search(result: String) -> Result<Vec<String>> {
//     let matches = MATCHES.to_vec()
//         .iter()
//         .map(|s| s.to_string())
//         .filter(|s| s.contains(result.as_str()))
//         .collect();

//     Ok(matches)
// }

// fn prompt_search(writer: &mut Write) -> Result<String>
// {
//     let mut result = String::new();
//     loop {
//         let (x, y) = terminal::size()?;

//         queue!(writer, cursor::SavePosition, Print(result.clone())).unwrap();

//         let mut y = 1;
//         let matches = search(result.clone()).unwrap();
//         let first = matches.get(0).unwrap_or(&result).clone();
//         for matched in matches.iter() {
//             queue!(writer, cursor::MoveDown(1), cursor::MoveToColumn(0), Print(matched.to_string())).unwrap();
//             y += 1;
//         }
//         writer.flush().unwrap();

//         match event::read().unwrap() {
//             Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
//                 return Ok(first);
//             },
//             Event::Key(KeyEvent { code: KeyCode::Char(_), modifiers: KeyModifiers::CONTROL }) => {
//                 return Ok(first);
//             },
//             Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
//                 if result.len() > 0 {
//                     result.remove(result.len() - 1);
//                 }
//             },
//             Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
//                 result.push(c);
//             },
//             Event::Key(default) => {
//                 queue!(writer, cursor::RestorePosition, terminal::Clear(terminal::ClearType::FromCursorDown), Print(format!("{:?}", default))).unwrap();
//                 println!("{:?}", default);
//                 return Ok(first);
//             },
//             Event::Mouse(default) => {
//                 queue!(writer, cursor::RestorePosition, terminal::Clear(terminal::ClearType::FromCursorDown), Print(format!("{:?}", default))).unwrap();
//                 println!("{:?}", default);
//                 return Ok(first);
//             },
//             Event::Resize(x, y) => {
//                 queue!(writer, cursor::RestorePosition, terminal::Clear(terminal::ClearType::FromCursorDown), Print(format!("Resized: ({}, {})", x, y))).unwrap();
//                 println!("Resized: ({}, {})", x, y);
//                 return Ok(first);
//             },
//         };
//         writer.flush().unwrap();
//     }
// }

// cargo run --example stderr
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&("init".to_string())) {
        println!("{}", data::INIT_ZSH);
        return;
    }
    if !args.contains(&("search".to_string())) {
        return;
    }
    // let mut tty = std::fs::OpenOptions::new()
    //     .read(true)
    //     .write(true)
    //     .open("/dev/tty").unwrap();

    let response = run_app(&mut stderr()).unwrap();
    println!("{}", response);
}