// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

use libc::{c_ushort, ioctl, TIOCGWINSZ};
use std::io::{Read, Write};

use termion;
use termion::{clear, color, cursor, style};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;

#[repr(C)]
struct TermSize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    _ws_xpixel: c_ushort,
    _ws_ypixel: c_ushort,
}

struct Position {
    x: u16,
    y: u16,
}

use std::os::unix::io::{ AsRawFd };

fn run_app(tty: &mut std::fs::File, reader: &mut dyn Read, writer: &mut dyn Write) -> Option<String> {
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

    let mut size: TermSize;
    let mut init = tty.cursor_pos().map(|(x, y)| Position{x, y}).unwrap();

    let mut query = String::new();
    let mut current: Option<&String> = None;

    let mut cur: usize = 0;

    let mut input = reader.keys();
    let mut running = true;

    let search_prefix = "~ ";
    while running {
        write!(writer, "{}{}", cursor::Goto(init.x, init.y), clear::AfterCursor).unwrap();
        write!(writer, "{}{}{}{}\n{}", color::Fg(color::Green), "] ", style::Reset, query, search_prefix).unwrap();

        current = choices.iter().find(|c| query.len() > 0 && c.contains(&query));

        if let Some(m) = current {
            write!(writer, "{}", m).unwrap();
        } else {
            write!(writer, "{}{}{}", color::Fg(color::LightBlack), "<no match>", style::Reset).unwrap();
        }

        writer.flush().unwrap();

        let next = input.next().unwrap().unwrap();

        match next {
            Key::Right | Key::Left |
            Key::Home | Key::End |
            Key::Esc | Key::Char('\n') => {
                running = false;
            }
            Key::Up | Key::PageUp if cur != 0 => {
                cur -= 1;
            }
            Key::Down | Key::PageDown if cur != choices.len() - 1 => {
                cur += 1;
            }
            Key::Char(c) => {
                query.push(c);
            }
            Key::Backspace if query.len() > 0 => {
                query.remove(query.len() - 1);
            }
            _ => { /* TODO */ }
        };

        // recalculate restore position if the window dimensions changed due to scrolling
        unsafe {
            size = std::mem::zeroed();
            let result = ioctl(tty.as_raw_fd(), TIOCGWINSZ.into(), &mut size as *mut _);
            if result < -1 {
                panic!(std::io::Error::last_os_error());
            }
        }

        if init.y == size.ws_row {
            init.y = size.ws_row - 1;
        }

        if let Some(string) = current {
            let rows = (string.len() + search_prefix.len()) as u16 / size.ws_col;
            if rows + init.y >= size.ws_row {
                init.y = size.ws_row - 1 - rows;
            }
        }
    }

    write!(writer, "{}{}", cursor::Goto(init.x, init.y), clear::AfterCursor).unwrap();
    writer.flush().unwrap();

    current.map(|s| s.clone())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&("init".to_string())) {
        let shell = std::env::var("SHELL").unwrap().as_str().to_lowercase();
        if shell.contains("zsh") {
            println!("{}", include_str!("etc/init.zsh"));
        } else if shell.contains("fish") {
            println!("{}", include_str!("etc/init.fish"));
        } else if shell.contains("bash") {
            println!("{}", include_str!("etc/init.bash"));
        } else {
            panic!("Unknown shell '{}', did not match supported list", shell);
        }
        return;
    }
    if !args.contains(&("search".to_string())) {
        return;
    }

    let mut tty = termion::get_tty().unwrap();
    let mut reader = tty.try_clone().unwrap();
    let mut writer = tty.try_clone().unwrap();

    let response = run_app(&mut tty, &mut reader, &mut writer);
    if let Some(response) = response {
        println!("{}", response);
    }
}