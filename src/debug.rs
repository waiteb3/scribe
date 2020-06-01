use std::io::Write;
use std::fs::{File, OpenOptions};
use log::{Metadata, Record, SetLoggerError};
use std::sync::Mutex;

struct Logger {
    stream: Mutex<File>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        writeln!(self.stream.lock().unwrap(),
            "[{}] {}",
            record.level(),
            record.args(),
        ).unwrap();
    }

    fn flush(&self) {
        self.stream.lock().unwrap().flush().unwrap();
     }
}

pub fn init(home: std::path::PathBuf) -> Result<(), SetLoggerError> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .truncate(false)
        .open(home.join(".scribe_next").join("log"))
        .unwrap();

    log::set_max_level(log::LevelFilter::Info);
    log::set_boxed_logger(Box::new(Logger{stream: Mutex::new(file)}))
}