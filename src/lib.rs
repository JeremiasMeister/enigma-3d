use std::time::{Duration, Instant};
use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface};
use winit::event::Event;
use winit::event_loop::{ControlFlow};
use crate::light::LightType;

pub mod shader;
pub mod geometry;
pub mod debug_geo;
pub mod texture;
pub mod material;
pub mod object;
pub mod obj_loader;
pub mod light;
pub mod camera;

pub struct AppState {
    pub fps: u64,
    pub light: Option<light::Light>,
    pub ambient_light: Option<light::Light>,
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
            fps: 60,
            objects: Vec::new(),
            light: None,
            ambient_light: None,
        }
    }

    pub fn add_object(&mut self, object: object::Object) {
        self.objects.push(object);
    }

    pub fn get_objects(&self) -> &Vec<object::Object> {
        &self.objects
    }


    pub fn set_light(&mut self, light: light::Light, light_type: LightType) {
        match light_type {
            LightType::Point => self.light = Some(light),
            LightType::Ambient => self.ambient_light = Some(light),
        }
    }

    pub fn get_light(&self) -> &Option<light::Light> {
        &self.light
    }

    pub fn set_fps(&mut self, fps: u64) {
        self.fps = fps;
    }

    pub fn get_fps(&self) -> u64 {
        self.fps
    }

    pub fn get_objects_mut(&mut self) -> &mut Vec<object::Object> {
        &mut self.objects
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
        let mut next_frame_time = Instant::now();
        let nanos = 1_000_000_000 / app_state.fps;
        let frame_duration = Duration::from_nanos(nanos); // 60 FPS (1,000,000,000 ns / 60)
        self.event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            next_frame_time = Instant::now() + frame_duration;
            match event {
                Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => { *control_flow = ControlFlow::Exit; }
                Event::RedrawEventsCleared => {}
                Event::RedrawRequested(_) => {
                    let mut target = self.display.draw();
                    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        .. Default::default()
                    };
                    for object in app_state.objects.iter_mut() {
                        //TODO: remove hardcoded rotation and allow attaching update functions to the app_state to be more flexible
                        object.transform.set_rotation([0.0, object.transform.get_rotation()[1] + 0.05, 0.0]);
                        object.update();
                        for (buffer, (material, indices)) in object.get_vertex_buffers().iter().zip(object.get_materials().iter().zip(object.get_index_buffers().iter())) {
                            target.draw(buffer, indices, &material.program, &material.get_uniforms(app_state.light, app_state.ambient_light, Some(<[[f32; 4]; 4]>::from(object.transform.matrix))), &params).unwrap();
                        }
                    }
                    target.finish().unwrap();
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => (),
            }
        });
    }
}
