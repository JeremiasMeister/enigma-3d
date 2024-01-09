
pub enum EventCharacteristic {
    KeyPress(winit::event::VirtualKeyCode),
    //TODO: impl more event characteristics
}
pub type EventFunction = Box<dyn Fn()>;