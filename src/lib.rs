use std::io::{self, Write};

use env_logger::fmt::Formatter;
use log::Record;

#[cfg(feature = "json")]
use serde::Serialize;

#[cfg(feature = "simple")]
#[inline]
pub fn simple_formatter(formatter: &mut Formatter, record: &Record) -> io::Result<()> {
    writeln!(
        formatter,
        "{} [{}] {}",
        record.level(),
        record.module_path().unwrap_or_default(),
        record.args()
    )
}

#[cfg(feature = "json")]
#[inline]
pub fn json_formatter(formatter: &mut Formatter, record: &Record) -> io::Result<()> {
    let entry = LogEntry {
        level: record.level().as_str(),
        msg: format!(
            "[{}] {}",
            record.module_path().unwrap_or_default(),
            record.args()
        ),
    };
    writeln!(formatter, "{}", serde_json::to_string(&entry)?)
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct LogEntry {
    level: &'static str,
    msg: String,
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{channel, Sender};

    use env_logger::Target;

    use super::*;

    struct WriteAdapter {
        sender: Sender<u8>,
    }

    impl io::Write for WriteAdapter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for chr in buf {
                self.sender.send(*chr).unwrap();
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[cfg(feature = "simple")]
    #[test]
    fn simple_log_format() {
        let (rx, tx) = channel();

        let _ = env_logger::builder()
            .target(Target::Pipe(Box::new(WriteAdapter { sender: rx })))
            .filter_level(log::LevelFilter::Debug)
            .format(simple_formatter)
            .is_test(true)
            .try_init();

        log::debug!("some debug log");

        assert_eq!(
            "DEBUG [simple_log_formatter::tests] some debug log\n",
            String::from_utf8(tx.try_iter().collect()).unwrap()
        );
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_log_format() {
        let (rx, tx) = channel();

        let _ = env_logger::builder()
            .target(Target::Pipe(Box::new(WriteAdapter { sender: rx })))
            .filter_level(log::LevelFilter::Debug)
            .format(json_formatter)
            .is_test(true)
            .try_init();

        log::debug!("some debug log");

        assert_eq!(
            "{\"level\":\"DEBUG\",\"msg\":\"[simple_log_formatter::tests] some debug log\"}\n",
            String::from_utf8(tx.try_iter().collect()).unwrap()
        );
    }
}
