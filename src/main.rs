// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

mod data;

use libc::{c_ushort, ioctl, STDOUT_FILENO, TIOCGWINSZ};
use std::io::{Read, Write/*, stdin, stdout, stderr, BufReader, BufWriter*/};

use termion;
use termion::{clear, color, cursor, style};
// use termion::raw::IntoRawMode;
use termion::cursor::{DetectCursorPos};
use termion::event::Key;
use termion::input::TermRead;

fn run_app(init: (u16, u16), reader: &mut dyn Read, writer: &mut dyn Write) -> Option<String> {
    let choices: [String; 8] = [
        String::from("echo test"),
        String::from("echo fail"),
        String::from("echo feed"),
        String::from("echo find"),
        String::from("echo free"),
        String::from("echo asdf"),
        String::from("echo zxcv"),
        String::from("echo really loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong"),
    ];

    let mut query = String::new();
    let mut current: Option<&String> = None;

    let mut cur: usize = 0;

    let mut input = reader.keys();
    let mut running = true;

    // std::thread::sleep_ms(1000);
    // write!(writer, "{}{}{}", cursor::Restore, cursor::Save, "I").unwrap();
    // writer.flush().unwrap();
    // std::thread::sleep_ms(1000);
    // write!(writer, "{}", clear::CurrentLine).unwrap();
    // writer.flush().unwrap();
    // std::thread::sleep_ms(1000);

    while running {
        write!(writer, "{}{}", cursor::Goto(init.0, init.1), clear::AfterCursor).unwrap();
        write!(writer, "{}{}{}{}\n~ ", color::Fg(color::Green), "] ", style::Reset, query).unwrap();

        current = choices.iter().find(|c| query.len() > 0 && c.contains(&query));

        if let Some(m) = current {
            write!(writer, "{}", m).unwrap();
        } else {
            write!(writer, "{}{}{}", color::Fg(color::LightBlack), "<no match>", style::Reset).unwrap();
        }

        writer.flush().unwrap();

        let next = input.next().unwrap().unwrap();

        match next {
            Key::Char('\n') => {
                running = false;
            }
            Key::Up if cur != 0 => {
                cur -= 1;
            }
            Key::Down if cur != choices.len() - 1 => {
                cur += 1;
            }
            Key::Char(c) => {
                query.push(c);
            }
            Key::Backspace => {
                if query.len() > 0 {
                    query.remove(query.len() - 1);
                }
            }
            _ => { }
        };

        write!(writer, "{}{}{}", clear::CurrentLine, cursor::Up(1), cursor::Left(0)).unwrap();
    }

    write!(writer, "{}{}", cursor::Restore, clear::AfterCursor).unwrap();

    current.map(|s| s.clone())
}

#[repr(C)]
struct TermSize {
    row: c_ushort,
    col: c_ushort,
    x: c_ushort,
    y: c_ushort,
}

use std::os::unix::io::{ AsRawFd };

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&("init".to_string())) {
        println!("{}", data::INIT_ZSH);
        return;
    }
    if !args.contains(&("search".to_string())) {
        return;
    }

    let mut tty = termion::get_tty().unwrap();
    let mut stdin = tty.try_clone().unwrap();

    let mut size: TermSize;
    unsafe {
        size = std::mem::zeroed();
        let result = ioctl(tty.as_raw_fd(), TIOCGWINSZ.into(), &mut size as *mut _);
        if result < -1 {
            panic!(std::io::Error::last_os_error());
        }
    }

    let initial_position = tty.cursor_pos().unwrap();
    let response = run_app(initial_position, &mut stdin, &mut tty);
    if let Some(response) = response {
        println!("{}", response);
    } else {
        println!("{} {} {} {}", size.col, size.row, size.x, size.y);
    }
}