use libc::{c_ushort, ioctl, TIOCGWINSZ};
use std::io::{Read, Write};
use std::os::unix::io::{ AsRawFd };

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

pub enum Direction {
    Up,
    Down,
}

pub struct Cursor {
    pub count: usize,
    pub direction: Direction,
    // timestamp
}

pub fn matches(cursor: & mut Cursor, query: String) -> Vec<String> {
    if query.len() == 0 {
        return vec![];
    }

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

    if cursor.count == 1 {
        let choice = choices.iter().find(|c| c.contains(&query));
        if let Some(choice) = choice {
            return vec![choice.clone()];
        } else {
            return vec![];
        }
    }

    
    choices.iter().filter(|c| c.contains(&query)).map(|c| c.clone()).collect()
}

pub fn interactive(tty: &mut std::fs::File, reader: &mut dyn Read, writer: &mut dyn Write) -> Option<String> {
    let mut size: TermSize;
    let mut init = tty.cursor_pos().map(|(x, y)| Position{x, y}).unwrap();

    let mut query = String::new();
    let mut current: Option<String> = None;

    let mut input = reader.keys();
    let mut running = true;
    let mut cursor = Cursor{ count: 1, direction: Direction::Up };

    let search_prefix = "~ ";
    while running {
        write!(writer, "{}{}", cursor::Goto(init.x, init.y), clear::AfterCursor).unwrap();
        write!(writer, "{}{}{}{}\n{}", color::Fg(color::Green), "] ", style::Reset, query, search_prefix).unwrap();

        let matches = matches(&mut cursor, query.clone());
        current = matches.get(0).map(|c| c.clone());

        if let Some(cmd_text) = current.take() {
            write!(writer, "{}", cmd_text).unwrap();
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
            Key::Up | Key::PageUp => {
                cursor.direction = Direction::Up;
            }
            Key::Down | Key::PageDown => {
                cursor.direction = Direction::Down;
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

        if let Some(cmd_text) = current.take() {
            let rows = (cmd_text.len() + search_prefix.len()) as u16 / size.ws_col;
            if rows + init.y >= size.ws_row {
                init.y = size.ws_row - 1 - rows;
            }
        }
    }

    write!(writer, "{}{}", cursor::Goto(init.x, init.y), clear::AfterCursor).unwrap();
    writer.flush().unwrap();

    current.map(|s| s.clone())
}
