use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface};
use winit::event::{Event, WindowEvent};
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
pub mod event;


pub struct AppState {
    pub fps: u64,
    pub camera: Option<camera::Camera>,
    pub light: Option<light::Light>,
    pub ambient_light: Option<light::Light>,
    pub objects: Vec<object::Object>,
    pub event_injections: Vec<(event::EventCharacteristic, event::EventFunction)>,
    pub update_injections: Vec<event::EventFunction>,
    pub display: Option<glium::Display<WindowSurface>>,
    pub time: f32,
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
            event_injections: Vec::new(),
            update_injections: Vec::new(),
            display: None,
            time: 0.0,
        }
    }

    pub fn convert_to_arc_mutex(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    pub fn add_object(&mut self, object: object::Object) {
        self.objects.push(object);
    }

    pub fn get_objects(&self) -> &Vec<object::Object> {
        &self.objects
    }

    pub fn get_object(&self, name: &str) -> Option<&object::Object> {
        for object in self.objects.iter() {
            if object.name == name {
                return Some(object);
            }
        }
        None
    }

    pub fn get_object_mut(&mut self, name: &str) -> Option<&mut object::Object> {
        for object in self.objects.iter_mut() {
            if object.name == name {
                return Some(object);
            }
        }
        None
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

    pub fn inject_event(&mut self, characteristic: event::EventCharacteristic, function: event::EventFunction) {
        self.event_injections.push((characteristic, function));
    }
    pub fn inject_update_function(&mut self, function: event::EventFunction) {
        self.update_injections.push(function);
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
    pub fn run(self, mut app_state: Arc<Mutex<AppState>>) {

        let display = self.display.clone();
        let mut temp_app_state = app_state.lock().unwrap();
        temp_app_state.display = Some(display);

        // managing fps
        let mut next_frame_time = Instant::now();
        let nanos = 1_000_000_000 / temp_app_state.fps; //TODO: not ideal to already unpack here once
        let frame_duration = Duration::from_nanos(nanos); // 60 FPS (1,000,000,000 ns / 60)
        drop(temp_app_state);

        // run loop
        self.event_loop.run(move |event, _window_target, control_flow| {
            // unpacking appstate
            let mut app_state = app_state.lock().unwrap();
            let light = app_state.light.clone();
            let ambient_light = app_state.ambient_light.clone();
            let camera = app_state.camera.clone();
            let mut event_injections = app_state.event_injections.clone();
            let mut update_injections = app_state.update_injections.clone();

            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            next_frame_time = Instant::now() + frame_duration;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit; }
                    WindowEvent::KeyboardInput { input, .. } => {
                        for (characteristic, function) in event_injections {
                            if let event::EventCharacteristic::KeyPress(key_code) = characteristic {
                                if input.state == winit::event::ElementState::Pressed && input.virtual_keycode == Some(key_code) {
                                    function(&mut app_state);
                                }
                            }
                        };
                    }
                    _ => ()
                }
                Event::RedrawRequested(_) => {
                    app_state.time += 0.001;
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
                        let model_matrix = object.transform.get_matrix();
                        for (buffer, (material, indices)) in object.get_vertex_buffers().iter().zip(object.get_materials().iter().zip(object.get_index_buffers().iter())) {
                            let uniforms = &material.get_uniforms(light, ambient_light, camera, Some(model_matrix));
                            target.draw(buffer, indices, &material.program, uniforms, &params).unwrap();
                        }
                    }
                    target.finish().unwrap();
                }
                Event::MainEventsCleared => {
                    // executing update functions
                    for function in update_injections {
                        function(&mut app_state);
                    }
                    self.window.request_redraw();
                }
                _ => (),
            }
        });
    }
}
