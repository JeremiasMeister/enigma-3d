use std::sync::Arc;
use crate::AppState;

// winit event re-export
use winit::event::*;

#[derive(Copy, Clone)]
pub enum EventCharacteristic {
    KeyPress(VirtualKeyCode),
    MousePress(MouseButton),
    MouseDown(MouseButton),
    MouseScroll(MouseScrollDelta),
    //TODO: impl more events
}

#[derive(Copy, Clone)]
pub struct EventModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl EventModifiers {
    pub fn new(ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            ctrl,
            shift,
            alt,
        }
    }
    pub fn default() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }
}

impl PartialEq for EventModifiers {
    fn eq(&self, other: &Self) -> bool {
        self.ctrl == other.ctrl && self.shift == other.shift && self.alt == other.alt
    }
}

pub type EventFunction = Arc<dyn Fn(&mut AppState)>;