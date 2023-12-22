use std::time::{Duration, Instant};
use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface};
use winit::event::Event;
use winit::event_loop::{ControlFlow};
use crate::camera::Camera;
use crate::light::LightType;

pub mod shader;
pub mod geometry;
pub mod debug_geo;
pub mod texture;
pub mod material;
pub mod object;
pub mod light;
pub mod camera;

pub struct AppState {
    pub fps: u64,
    pub camera: Option<camera::Camera>,
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
            camera: None,
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

    pub fn set_camera(&mut self, camera: camera::Camera) {
        self.camera = Some(camera);
    }

    pub fn get_camera(&self) -> &Option<camera::Camera> {
        &self.camera
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

    // This is just the render loop . an actual event loop still needs to be set up
    pub fn run(self, mut app_state: AppState) {
        let mut next_frame_time = Instant::now();
        let nanos = 1_000_000_000 / app_state.fps;
        let frame_duration = Duration::from_nanos(nanos); // 60 FPS (1,000,000,000 ns / 60)
        self.event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            next_frame_time = Instant::now() + frame_duration;
            match event {
                Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => { *control_flow = ControlFlow::Exit; }
                Event::RedrawRequested(_) => {
                    let mut target = self.display.draw();
                    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            ..Default::default()
                        },
                        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,

                        ..Default::default()
                    };
                    for object in app_state.objects.iter_mut() {
                        object.transform.set_rotation([object.transform.get_rotation()[0], object.transform.get_rotation()[1] + 1.0, object.transform.get_rotation()[2]]);
                        let model_matrix = object.transform.get_matrix();
                        for (buffer, (material, indices)) in object.get_vertex_buffers().iter().zip(object.get_materials().iter().zip(object.get_index_buffers().iter())) {
                            let uniforms = &material.get_uniforms(app_state.light, app_state.ambient_light, app_state.camera, Some(model_matrix));
                            target.draw(buffer, indices, &material.program, uniforms, &params).unwrap();
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
