use std::io::Write;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

static STDOUT: Lazy<Mutex<Option<StandardStream>>> = Lazy::new(|| Mutex::new(None));

pub struct LogSystem;

pub enum LogType {
    Log,
    Subprocess,
    Success,
    Warn,
    Err,
}

impl LogSystem {
    pub fn log(text: String) {
        LogSystem::writeln(LogType::Log, text)
    }

    pub fn subprocess(text: String) {
        LogSystem::writeln(LogType::Subprocess, text)
    }

    pub fn success(text: String) {
        LogSystem::writeln(LogType::Success, text)
    }

    pub fn warn(text: String) {
        LogSystem::writeln(LogType::Warn, text)
    }

    pub fn err(text: String) {
        LogSystem::writeln(LogType::Err, text)
    }

    pub fn writeln(log_type: LogType, text: String) {
        let mut stdout = STDOUT.lock().unwrap();

        if stdout.is_none() {
            *stdout = Some(StandardStream::stdout(ColorChoice::Always));
        }

        assert!(stdout.is_some());

        let stream = &mut stdout.as_mut().unwrap();
        let log_color = log_type.color();
        if log_color.is_some() {
            stream
                .set_color(ColorSpec::new().set_fg(Some(log_color.unwrap())))
                .unwrap();
        }

        writeln!(*stream, "{}", text).unwrap();

        stream.reset().unwrap();
    }
}

impl LogType {
    pub fn color(&self) -> Option<Color> {
        match self {
            LogType::Log => None,
            LogType::Subprocess => Some(Color::Rgb(100, 100, 100)),
            LogType::Success => Some(Color::Green),
            LogType::Warn => Some(Color::Yellow),
            LogType::Err => Some(Color::Red),
        }
    }
}
