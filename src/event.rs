use std::sync::Arc;
use crate::AppState;

#[derive(Copy, Clone)]
pub enum EventCharacteristic {
    KeyPress(winit::event::VirtualKeyCode),
    MousePress(winit::event::MouseButton),
    MouseScroll(winit::event::MouseScrollDelta),
    //TODO: impl more events
}

pub type EventFunction = Arc<dyn Fn(&mut AppState)>;