use colored::Colorize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const LOGLOCATION: &str = "enigma_logs/enigma.log";

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
    let log_path = Path::new(LOGLOCATION);
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

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


pub struct EnigmaError {
    errors: Vec<String>,
    disk: bool
}

pub struct EnigmaWarning {
    warnings: Vec<String>,
    disk: bool
}

pub struct EnigmaMessage {
    messages: Vec<String>,
    disk: bool
}

impl EnigmaError {
    pub fn new(error: &str, disk: bool) -> Self {
        Self {
            errors: vec![error.to_string()],
            disk
        }
    }

    pub fn extent(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    pub fn log(&self) {
        for error in self.errors.iter() {
            println!("Error: {}", error.red());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }
}

impl EnigmaWarning {
    pub fn new(warning: &str, disk: bool) -> Self {
        Self {
            warnings: vec![warning.to_string()],
            disk
        }
    }

    pub fn extent(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    pub fn log(&self) {
        for warning in self.warnings.iter() {
            println!("Warning: {}", warning.yellow());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }
}

impl EnigmaMessage {
    pub fn new(message: &str, disk: bool) -> Self {
        Self {
            messages: vec![message.to_string()],
            disk
        }
    }

    pub fn extent(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    pub fn log(&self) {
        for message in self.messages.iter() {
            println!("Message: {}", message.bright_blue());
        }
        if self.disk {
            save_to_disk(Box::new(self)).expect("failed to write log")
        }
    }
}