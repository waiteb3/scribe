use std::convert::From;

use libc::{c_ushort, ioctl, TIOCGWINSZ};
use std::io::{Read, Write};
use std::os::unix::io::{ AsRawFd };

use termion;
use termion::{clear, color, cursor, style};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use rusqlite::{self, named_params};

use super::init::DataStores;

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

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Older,
    Newer,
}

#[derive(Copy, Clone)]
pub struct Cursor {
    pub direction: Direction,
    pub oid: Option<u32>,
}

pub struct SearchError {
    pub cause: String,
}

impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        SearchError { cause: format!("IO Error encountered: {}", err) }
    }
}

impl From<rusqlite::Error> for SearchError {
    fn from(err: rusqlite::Error) -> Self {
        SearchError { cause: format!("SQL Error encountered: {}", err) }
    }
}

fn row_to_result(prev: Cursor, row: &rusqlite::Row) -> Result<(String, Cursor), rusqlite::Error> {
    let cmd = row.get::<_, String>(1)?;
    let cursor = Cursor{
        direction: prev.direction,
        oid: row.get(0).unwrap_or(prev.oid),
    };

    Ok((cmd, cursor))
}

pub fn find_next_match(deps: DataStores, query: String, cursor: Cursor) -> Result<(Option<String>, Cursor), SearchError> {
    if query.len() == 0 {
        return Ok((None, cursor));
    }

    let result = match cursor.direction {
        Direction::Older => {
            deps.index.try_lock().unwrap().query_row_named(
                r#"
                    SELECT oid, command
                    FROM history
                    WHERE command LIKE '%' || :query || '%'
                    AND oid <= :oid
                    ORDER BY oid DESC
                    LIMIT 1
                "#,
                named_params!{
                    ":query": query,
                    ":oid": cursor.oid.unwrap_or(std::u32::MAX),
                },
                 |row| row_to_result(cursor, row),
            )
        }
        Direction::Newer => {
            deps.index.try_lock().unwrap().query_row_named(
                r#"
                    SELECT oid, command
                    FROM history
                    WHERE command LIKE '%' || :query || '%'
                    AND oid >= :oid
                    ORDER BY oid ASC
                    LIMIT 1
                "#,
                named_params! {
                    ":query": query,
                    ":oid": cursor.oid.unwrap_or(0),
                },
                |row| row_to_result(cursor, row),
            )
        }
    };

    match result {
        Ok((cmd, next)) => {
            Ok((Some(cmd), next))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok((None, cursor))
        }
        Err(e) => {
            Err(e.into())
        }
    }
}

pub fn find_recent_matches(deps: DataStores, query: String) -> Result<Vec<String>, SearchError> {
    if query.len() == 0 {
        return Ok(vec![]);
    }

    let index = deps.index.try_lock().unwrap();
    let mut statement = index.prepare(r#"
        SELECT command
        FROM history
        WHERE command LIKE '%' || :query || '%'
        LIMIT 20
    "#)?;

    let rows = statement.query_map_named(
        named_params![
            ":query": query,
        ],
        |row| {
            Ok(row.get::<_, String>(0)?)
        },
    )?;

    let mut choices = vec![];
    for row in rows {
        choices.push(row?);
    }
    Ok(choices)
}

pub fn interactive(deps: DataStores, tty: &mut std::fs::File, reader: &mut dyn Read, writer: &mut dyn Write) -> Result<Option<String>, SearchError> {
    let mut size: TermSize;
    let mut init = tty.cursor_pos().map(|(x, y)| Position{x, y})?;

    let mut query = String::new();
    let mut current: Option<String> = None;

    let mut input = reader.keys();
    let mut running = true;
    let mut cursor = Cursor{ direction: Direction::Older, oid: None };

    let prompt_prefix = "(scribe): ";
    let search_prefix = "~ ";

    let max_width = 500;
    while running {
        write!(writer, "{}{}", cursor::Goto(init.x, init.y), clear::AfterCursor)?;
        write!(writer, "{}{}{}{}\n{}", color::Fg(color::Green), prompt_prefix, style::Reset, query, search_prefix)?;

        let (result, next) = find_next_match(deps.clone(), query.clone(), cursor)?;
        current = result;
        cursor = next;

        let rendered_text = current.clone().map(|cmd| {
            if cmd.len() >= max_width {
                format!("{}...", cmd.get(0..max_width).unwrap())
            } else {
                cmd
            }
        });

        if let Some(cmd_text) = rendered_text.clone() {
            write!(writer, "{}", cmd_text)?;
        } else {
            write!(writer, "{}{}{}", color::Fg(color::LightBlack), "<no match>", style::Reset)?;
        }

        write!(writer, "{}", cursor::Goto(init.x + (prompt_prefix.len() as u16) + (query.len() as u16), init.y))?;
        writer.flush()?;

        let next = input.next().ok_or(
            SearchError{ cause: format!("Error occured while waiting on input") }
        )?;

        match next? {
            // Key::Right if truncated && current.is_some() => {
            //     // TODO split out render & wait calls
            // }
            Key::Right | Key::Left |
            Key::Home | Key::End |
            Key::Esc | Key::Ctrl('d') |
            Key::Char('\n') => {
                running = false;
            }
            Key::Ctrl('u') => {
                current = None;
                running = false;
            }
            Key::Ctrl('r') | // SCRIBE-20: needs logic to handle highlighting & resuming from substring match
            Key::Up | Key::PageUp if cursor.oid.is_some() => {
                cursor.direction = Direction::Older;
                if let Some(oid) = cursor.oid {
                    cursor.oid = Some(if oid > 0 { oid - 1 } else { 0 });
                }
            }
            Key::Down | Key::PageDown if cursor.oid.is_some() => {
                cursor.direction = Direction::Newer;
                if let Some(oid) = cursor.oid {
                    cursor.oid = Some(if oid < std::u32::MAX { oid + 1 } else { std::u32::MAX });
                }
            }
            Key::Char(c) => {
                query.push(c);
            }
            // TODO ctrl+a ctrl+e
            Key::Ctrl('w') => {
                query = String::new()
            }
            Key::Backspace if query.len() > 0 => {
                query.remove(query.len() - 1);
            }
            e => {
                log::log!(log::Level::Debug, "input '{:?}' was ignored", e);
            }
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

        if let Some(cmd_text) = rendered_text.clone() {
            let mut combined = search_prefix.to_owned();
            combined.push_str(cmd_text.as_str());

            let mut rows: u16 = 0;
            for line in combined.split('\n') {
                rows += 1;
                rows += line.len() as u16 / size.ws_col;
            }
            if rows + init.y >= size.ws_row {
                init.y = size.ws_row - rows;
            }
        }
    }

    write!(writer, "{}{}", cursor::Goto(init.x, init.y), clear::AfterCursor)?;
    writer.flush()?;

    Ok(current.map(|s| s.clone()))
}
