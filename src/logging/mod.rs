pub mod format;

use std::env;
use colored::Colorize;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Once;

#[derive(PartialEq, PartialOrd)]
enum LogLevel {
    None,
    Error,
    Warning,
    Message,
}

const LOG_DIRECTORY: &str = "enigma_logs";

static INIT: Once = Once::new();
static mut LOG_FILE_PATH: Option<PathBuf> = None;

fn get_logging_level() -> LogLevel {
    match env::var("LOG_LEVEL") {
        Ok(var) => {
            match var.to_lowercase().as_str() {
                "message" => return LogLevel::Message,
                "warning" => return LogLevel::Warning,
                "error" => return LogLevel::Error,
                "none" => return LogLevel::None,
                _ => return LogLevel::Message
            }
        },
        Err(_) => return LogLevel::Message
    }
}

fn get_save_log_files() -> bool {
    match env::var("SAVE_LOG_FILES") {
        Ok(var) => {
            match var.to_lowercase().as_str() {
                "1" => return true,
                "true" => return true,
                "0" => return false,
                "false" => return false,
                _ => return true
            }
        },
        Err(_) => return true
    }
}

fn initialize_log_file() -> PathBuf {
    let exec_name = env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown".to_string());

    let datetime = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("{}_{}.log", exec_name, datetime);
    PathBuf::from(LOG_DIRECTORY).join(filename)
}

fn get_log_filepath() -> PathBuf {
    unsafe {
        INIT.call_once(|| {
            let path = initialize_log_file();
            LOG_FILE_PATH = Some(path);
        });
        LOG_FILE_PATH.clone().unwrap()
    }
}

fn save_to_disk(log: Box<&dyn EnigmaLog>) -> std::io::Result<()> {
    if !get_save_log_files() {
        return Ok(());
    }

    let prefix = match log.log_type() {
        EnigmaLogType::Error => "ERROR >> ",
        EnigmaLogType::Warning => "WARNING >> ",
        EnigmaLogType::Message => "MESSAGE >> ",
        EnigmaLogType::Unknown => ">> ",
    };

    let joined_string = log.get()
        .iter()
        .map(|((t,s))| format!("{} {}{}", t, prefix, s))
        .collect::<Vec<String>>()
        .join("\n");

    // Ensure the directory exists
    fs::create_dir_all(LOG_DIRECTORY)?;

    let log_path = get_log_filepath();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    writeln!(file, "{}", joined_string)?;

    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum EnigmaLogType {
    Error,
    Warning,
    Message,
    Unknown,
}

pub trait EnigmaLog {
    fn get(&self) -> &Vec<(String, String)>;
    fn log_type(&self) -> EnigmaLogType;
}

impl EnigmaLog for EnigmaError {
    fn get(&self) -> &Vec<(String, String)> {
        &self.errors
    }
    fn log_type(&self) -> EnigmaLogType {
        EnigmaLogType::Error
    }
}
impl EnigmaLog for EnigmaWarning {
    fn get(&self) -> &Vec<(String, String)> {
        &self.warnings
    }
    fn log_type(&self) -> EnigmaLogType {
        EnigmaLogType::Warning
    }
}
impl EnigmaLog for EnigmaMessage {
    fn get(&self) -> &Vec<(String, String)> {
        &self.messages
    }
    fn log_type(&self) -> EnigmaLogType {
        EnigmaLogType::Message
    }
}


#[derive(Debug)]
pub struct EnigmaError {
    errors: Vec<(String, String)>,
    disk: bool
}

#[derive(Debug)]
pub struct EnigmaWarning {
    warnings: Vec<(String, String)>,
    disk: bool
}

#[derive(Debug)]
pub struct EnigmaMessage {
    messages: Vec<(String, String)>,
    disk: bool
}

impl EnigmaError {
    pub fn new(error: Option<&str>, disk: bool) -> Self {
        let time = Local::now().format("%H-%M-%S").to_string();
        Self {
            errors: match error {
                Some(e) => vec![(time, e.to_string())],
                None => Vec::new()
            },
            disk
        }
    }

    pub fn extent(&mut self, error: &str) {
        let time = Local::now().format("%H-%M-%S").to_string();
        self.errors.push((time, error.to_string()));
    }

    pub fn log(&self) {
        if get_logging_level() < LogLevel::Error {
            return;
        }
        for (time, error) in self.errors.iter() {
            println!("{} {} {}", time.red(), "ERROR >>".red(), error.red());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }

    pub fn merge(&mut self, error: EnigmaError) {
        for (_, e) in error.errors {
            self.extent(e.as_str())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl EnigmaWarning {
    pub fn new(warning: Option<&str>, disk: bool) -> Self {
        let time = Local::now().format("%H-%M-%S").to_string();
        Self {
            warnings: match warning {
                Some(w) => vec![(time, w.to_string())],
                None => Vec::new()
            },
            disk
        }
    }

    pub fn extent(&mut self, warning: &str) {
        let time = Local::now().format("%H-%M-%S").to_string();
        self.warnings.push((time, warning.to_string()));
    }

    pub fn log(&self) {
        if get_logging_level() < LogLevel::Warning {
            return;
        }
        for (time, warning) in self.warnings.iter() {
            println!("{} {} {}",time.yellow(), "WARNING >>".yellow(), warning.yellow());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }

    pub fn merge(&mut self, error: EnigmaWarning) {
        for (_, w) in error.warnings {
            self.extent(w.as_str())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }
}

impl EnigmaMessage {
    pub fn new(message: Option<&str>, disk: bool) -> Self {
        let time = Local::now().format("%H-%M-%S").to_string();
        Self {
            messages: match message {
                Some(m) => vec![(time, m.to_string())],
                None => Vec::new()
            },
            disk
        }
    }

    pub fn extent(&mut self, message: &str) {
        let time = Local::now().format("%H-%M-%S").to_string();
        self.messages.push((time, message.to_string()));
    }

    pub fn log(&self) {
        if get_logging_level() < LogLevel::Message {
            return;
        }
        for (time, message) in self.messages.iter() {
            println!("{} {} {}", time.bright_blue(),"MESSAGE >>".bright_blue(), message.bright_blue());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }

    pub fn merge(&mut self, error: EnigmaMessage) {
        for (_, m) in error.messages {
            self.extent(m.as_str())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}