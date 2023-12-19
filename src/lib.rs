use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, IndexBuffer, Surface, VertexBuffer};
use winit::event::Event;
use winit::event_loop::{ControlFlow};

pub mod shader;
pub mod geometry;
pub mod debug_geo;
pub mod texture;
pub mod material;
pub mod object;

pub struct AppState {
    pub _vertex_buffers: Vec<VertexBuffer<geometry::Vertex>>,
    pub _index_buffers: Vec<IndexBuffer<u32>>,
    pub _materials: Vec<material::Material>,
}

pub struct EventLoop {
    pub event_loop: winit::event_loop::EventLoop<()>,
    pub window: Window,
    pub display: Display<WindowSurface>,
}


impl AppState {
    pub fn new() -> Self {
        AppState {
            _vertex_buffers: Vec::new(),
            _index_buffers: Vec::new(),
            _materials: Vec::new(),
        }
    }

    pub fn extend_vertex_buffers(&mut self, vertex_buffers: Vec<VertexBuffer<geometry::Vertex>>) {
        self._vertex_buffers.extend(vertex_buffers);
    }

    pub fn extend_index_buffers(&mut self, index_buffers: Vec<IndexBuffer<u32>>) {
        self._index_buffers.extend(index_buffers);
    }

    pub fn extend_materials(&mut self, materials: Vec<material::Material>) {
        self._materials.extend(materials);
    }

    pub fn clear_vertex_buffers(&mut self) {
        self._vertex_buffers.clear();
    }

    pub fn clear_index_buffers(&mut self) {
        self._index_buffers.clear();
    }

    pub fn clear_materials(&mut self) {
        self._materials.clear();
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
    pub fn run(self, mut app_state: AppState) {
        self.event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::RedrawEventsCleared => {
                    // Request a redraw here if necessary
                }
                Event::RedrawRequested(_) => {
                    let mut target = self.display.draw();
                    target.clear_color(0.0, 0.0, 0.0, 1.0);

                    for (buffer, (mut material, indices)) in app_state._vertex_buffers.iter().zip(app_state._materials.iter_mut().zip(app_state._index_buffers.iter())) {
                        material.update();
                        target.draw(buffer, indices, &material.program, &material.get_uniforms(), &Default::default()).unwrap();
                    }

                    target.finish().unwrap();
                }
                _ => (),
            }
        });
    }
}
