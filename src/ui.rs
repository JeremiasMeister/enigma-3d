use std::sync::Arc;
use crate::AppState;
pub type GUIDrawFunction = Arc<dyn Fn(&egui::Context, &mut AppState)>;

// full egui version re-export
pub use egui::*;
