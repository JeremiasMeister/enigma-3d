use egui_glium::EguiGlium;
use glium::{Display, Frame};
use glium::glutin::surface::WindowSurface;
use crate::EventLoop;

pub struct UI {
    egui_glium: Option<EguiGlium>,
    gui_draw_functions: Vec<Box<dyn FnMut(&egui::Context)>>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            egui_glium: None,
            gui_draw_functions: Vec::new(),
        }
    }

    pub fn init(&mut self, event_loop: &EventLoop) {
        self.egui_glium = Some(EguiGlium::new(&event_loop.display, &event_loop.window, &event_loop.event_loop));
    }

    pub fn inject_gui<F>(&mut self, function: F)
    where
        F: FnMut(&egui::Context) + 'static,
    {
        self.gui_draw_functions.push(Box::new(function));
    }

    pub fn draw_gui(&mut self, window: &winit::window::Window, display: &Display<WindowSurface>, frame: &mut Frame) {
        let egui_glium = self.egui_glium.as_mut().unwrap();
        egui_glium.run(window, | egui_ctx| {
            for draw_function in &mut self.gui_draw_functions {
                draw_function(egui_ctx);
            }
        });
        egui_glium.paint(display, frame);
    }
}