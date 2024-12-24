use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};
use log::{Level, LevelFilter, Metadata, Record};

pub struct UartLogger;
static PAGING_INITIALISED: AtomicBool = AtomicBool::new(false);
static LOGGER: UartLogger = UartLogger;

impl UartLogger {
    pub fn write(&mut self, c: u8) {
        let mut addr = 0x1000_0000usize;
        if PAGING_INITIALISED.load(Ordering::Relaxed) {
            // Mapped at 511GiB offset.
            addr += 0xffffffffc0000000;
        }
        unsafe { (addr as *mut u8).write_volatile(c) };
    }
}

impl fmt::Write for UartLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\x1b' || byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                self.write(byte);
            } else {
                self.write(b'?');
            }
        }

        Ok(())
    }
}

impl log::Log for UartLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let (color_prefix, color_suffix) = match record.metadata().level() {
                Level::Trace => ("\x1b[36m", "\x1b[0m"),
                Level::Debug => ("\x1b[34m", "\x1b[0m"),
                Level::Info => ("\x1b[32m", "\x1b[0m"),
                Level::Warn => ("\x1b[33m", ""),
                Level::Error => ("\x1b[31m", ""),
            };
            let line_suffix = "\x1b[0m";
            writeln!(
                UartLogger,
                "[{}{:5}{}] {}{}",
                color_prefix,
                record.level(),
                color_suffix,
                record.args(),
                line_suffix
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
        .unwrap()
}

pub fn paging_initialised() {
    PAGING_INITIALISED.store(true, Ordering::Relaxed);
}
