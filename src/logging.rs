use log::{self, SetLoggerError, LevelFilter, Record, Metadata};
use crate::serial_println;

struct SerialLogger;

static LOGGER: SerialLogger = SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            serial_println!("{}:{}: {} - {}",
                record.module_path().unwrap_or("_"),
                record.line().unwrap_or(0),
                record.level(),
                record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
}