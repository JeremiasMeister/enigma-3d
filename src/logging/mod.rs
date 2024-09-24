use colored::Colorize;

pub struct EnigmaError {
    errors: Vec<String>
}

pub struct EnigmaWarning {
    warnings: Vec<String>
}

pub struct EnigmaMessage {
    messages: Vec<String>
}

impl EnigmaError {
    pub fn new(error: &str) -> Self {
        Self {
            errors: vec![error.to_string()]
        }
    }

    pub fn extent(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    pub fn log(&self) {
        for error in self.errors.iter() {
            println!("Error: {}", error.red());
        }
    }
}

impl EnigmaWarning {
    pub fn new(warning: &str) -> Self {
        Self {
            warnings: vec![warning.to_string()]
        }
    }

    pub fn extent(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    pub fn log(&self) {
        for warning in self.warnings.iter() {
            println!("Warning: {}", warning.yellow());
        }
    }
}

impl EnigmaMessage {
    pub fn new(message: &str) -> Self {
        Self {
            messages: vec![message.to_string()]
        }
    }

    pub fn extent(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    pub fn log(&self) {
        for message in self.messages.iter() {
            println!("{}", message.bright_blue());
        }
    }
}