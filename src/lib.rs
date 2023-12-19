use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, IndexBuffer, Surface, VertexBuffer};
use itertools::Itertools;
use winit::event::Event;
use winit::event_loop::{ControlFlow};
use crate::material::Material;

pub mod shader;
pub mod geometry;
pub mod debug_geo;
pub mod texture;
pub mod material;
pub mod object;

pub struct AppState {
    pub objects: Vec<object::Object>,
}

pub struct EventLoop {
    pub event_loop: winit::event_loop::EventLoop<()>,
    pub window: Window,
    pub display: Display<WindowSurface>,
}


impl AppState {
    pub fn new() -> Self {
        AppState {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: object::Object) {
        self.objects.push(object);
    }
}

impl EventLoop {
    pub fn new(title: &str) -> Self {
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title(title)
            .build(&event_loop);
        EventLoop {
            event_loop,
            window,
            display,
        }
    }
    pub fn get_display_clone(&self) -> Display<WindowSurface> {
        self.display.clone()
    }

    pub fn get_display_reference(&self) -> &Display<WindowSurface> {
        &self.display
    }

    pub fn run(self, mut app_state: AppState) {
        self.event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {*control_flow = ControlFlow::Exit;}
                Event::RedrawEventsCleared => {
                    // Request a redraw here if necessary
                }
                Event::RedrawRequested(_) => {
                    let mut target = self.display.draw();
                    target.clear_color(0.0, 0.0, 0.0, 1.0);
                    for object in app_state.objects.iter_mut() {
                        object.update();
                        for (buffer, (mut material, indices)) in object.get_vertex_buffers().iter().zip(object.materials.iter().zip(object.get_index_buffers().iter())) {
                            target.draw(buffer, indices, &material.program, &material.get_uniforms(), &Default::default()).unwrap();
                        }
                    }

                    target.finish().unwrap();
                }
                _ => (),
            }
        });
    }
}
