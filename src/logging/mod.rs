pub mod format;

use std::env;
use colored::Colorize;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use once_cell::sync::Lazy;

const LOG_DIRECTORY: &str = "enigma_logs";
static LOG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| initialize_log_file());

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
    // Simply clone the lazy initialized path
    LOG_FILE_PATH.clone()
}

fn save_to_disk(log: Box<&dyn EnigmaLog>) -> std::io::Result<()> {
    let prefix = match log.log_type() {
        EnigmaLogType::Error => "ERROR >> ",
        EnigmaLogType::Warning => "WARNING >> ",
        EnigmaLogType::Message => "MESSAGE >> ",
        EnigmaLogType::Unknown => ">> ",
    };

    let joined_string = log.get()
        .iter()
        .map(|s| format!("{}{}", prefix, s))
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
    fn get(&self) -> &Vec<String>;
    fn log_type(&self) -> EnigmaLogType;
}

impl EnigmaLog for EnigmaError {
    fn get(&self) -> &Vec<String> {
        &self.errors
    }
    fn log_type(&self) -> EnigmaLogType {
        EnigmaLogType::Error
    }
}
impl EnigmaLog for EnigmaWarning {
    fn get(&self) -> &Vec<String> {
        &self.warnings
    }
    fn log_type(&self) -> EnigmaLogType {
        EnigmaLogType::Warning
    }
}
impl EnigmaLog for EnigmaMessage {
    fn get(&self) -> &Vec<String> {
        &self.messages
    }
    fn log_type(&self) -> EnigmaLogType {
        EnigmaLogType::Message
    }
}


#[derive(Debug)]
pub struct EnigmaError {
    errors: Vec<String>,
    disk: bool
}

#[derive(Debug)]
pub struct EnigmaWarning {
    warnings: Vec<String>,
    disk: bool
}

#[derive(Debug)]
pub struct EnigmaMessage {
    messages: Vec<String>,
    disk: bool
}

impl EnigmaError {
    pub fn new(error: Option<&str>, disk: bool) -> Self {
        Self {
            errors: match error {
                Some(e) => vec![e.to_string()],
                None => Vec::new()
            },
            disk
        }
    }

    pub fn extent(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    pub fn log(&self) {
        for error in self.errors.iter() {
            println!("{} {}","ERROR >>".red(), error.red());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }

    pub fn merge(&mut self, error: EnigmaError) {
        for e in error.errors {
            self.extent(e.as_str())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl EnigmaWarning {
    pub fn new(warning: Option<&str>, disk: bool) -> Self {
        Self {
            warnings: match warning {
                Some(w) => vec![w.to_string()],
                None => Vec::new()
            },
            disk
        }
    }

    pub fn extent(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    pub fn log(&self) {
        for warning in self.warnings.iter() {
            println!("{} {}","WARNING >>".yellow(), warning.yellow());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }

    pub fn merge(&mut self, error: EnigmaWarning) {
        for w in error.warnings {
            self.extent(w.as_str())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }
}

impl EnigmaMessage {
    pub fn new(message: Option<&str>, disk: bool) -> Self {
        Self {
            messages: match message {
                Some(m) => vec![m.to_string()],
                None => Vec::new()
            },
            disk
        }
    }

    pub fn extent(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    pub fn log(&self) {
        for message in self.messages.iter() {
            println!("{} {}","MESSAGE >>".bright_blue(), message.bright_blue());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }

    pub fn merge(&mut self, error: EnigmaMessage) {
        for m in error.messages {
            self.extent(m.as_str())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}