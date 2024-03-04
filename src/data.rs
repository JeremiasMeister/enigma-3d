use std::any::Any;

pub struct AppStateData {
    name: String,
    value: Box<dyn Any>,
}
impl AppStateData {
    pub fn new(name: &str, value: Box<dyn Any>) -> Self {
        AppStateData {
            name: name.to_string(),
            value,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &dyn Any {
        self.value.as_ref()
    }

    // Since the value is already a Box<dyn Any>, setting a new value should accept a Box<dyn Any>
    pub fn set_value(&mut self, value: Box<dyn Any>) {
        self.value = value;
    }
}