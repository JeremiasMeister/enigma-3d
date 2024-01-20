use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface, uniform};
use uuid::Uuid;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow};
use crate::collision_world::MousePosition;
use crate::light::LightType;
use crate::postprocessing::PostProcessingEffect;

pub mod shader;
pub mod geometry;
pub mod debug_geo;
pub mod texture;
pub mod material;
pub mod object;
pub mod light;
pub mod camera;
pub mod event;
pub mod collision_world;
pub mod default_events;
pub mod postprocessing;


pub fn init_default(app_state: &mut AppState) {
    app_state.set_renderscale(2);
    app_state.inject_event(
        event::EventCharacteristic::MousePress(winit::event::MouseButton::Left),
        Arc::new(default_events::select_object),
    );
    app_state.inject_event(
        event::EventCharacteristic::MousePress(winit::event::MouseButton::Right),
        Arc::new(default_events::select_object_add),
    );
}

pub struct AppState {
    pub fps: u64,
    pub camera: Option<camera::Camera>,
    pub light: Vec<light::Light>,
    pub ambient_light: Option<light::Light>,
    pub objects: Vec<object::Object>,
    pub object_selection: Vec<Uuid>,
    pub event_injections: Vec<(event::EventCharacteristic, event::EventFunction)>,
    pub update_injections: Vec<event::EventFunction>,
    pub post_processes: Vec<Box<dyn PostProcessingEffect>>,
    pub display: Option<glium::Display<WindowSurface>>,
    pub time: f32,
    pub renderscale: u32,
    mouse_position: MousePosition,
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
            object_selection: Vec::new(),
            light: Vec::new(),
            ambient_light: None,
            event_injections: Vec::new(),
            update_injections: Vec::new(),
            post_processes: Vec::new(),
            display: None,
            time: 0.0,
            renderscale: 1,
            mouse_position: MousePosition::new(),
        }
    }

    pub fn add_post_process(&mut self, post_process: Box<dyn PostProcessingEffect>) {
        self.post_processes.push(post_process);
    }

    pub fn get_post_processes(&self) -> &Vec<Box<dyn PostProcessingEffect>> {
        &self.post_processes
    }

    pub fn get_post_processes_mut(&mut self) -> &mut Vec<Box<dyn PostProcessingEffect>> {
        &mut self.post_processes
    }

    pub fn get_mouse_position(&self) -> &MousePosition {
        &self.mouse_position
    }

    pub fn get_mouse_position_mut(&mut self) -> &mut MousePosition {
        &mut self.mouse_position
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

    pub fn get_object_by_uuid(&self, uuid: Uuid) -> Option<&object::Object> {
        for object in self.objects.iter() {
            if object.get_unique_id() == uuid {
                return Some(object);
            }
        }
        None
    }

    pub fn get_object_by_uuid_mut(&mut self, uuid: Uuid) -> Option<&mut object::Object> {
        for object in self.objects.iter_mut() {
            if object.get_unique_id() == uuid {
                return Some(object);
            }
        }
        None
    }

    pub fn get_selected_objects_mut(&mut self) -> Vec<&mut object::Object> {
        let mut selected = Vec::new();
        for object in self.objects.iter_mut() {
            if self.object_selection.contains(&object.get_unique_id()) {
                selected.push(object);
            }
        }
        selected
    }

    pub fn add_light(&mut self, light: light::Light, light_type: LightType) {
        match light_type {
            LightType::Point => self.light.push(light),
            LightType::Ambient => self.ambient_light = Some(light),
        }
    }

    pub fn remove_light(&mut self, index: usize, light_type: LightType) {
        match light_type {
            LightType::Point => {
                if index >= self.light.len() {
                    panic!("Index out of bounds");
                }
                self.light.remove(index);
            }
            LightType::Ambient => {
                self.ambient_light = None;
            }
        };
    }

    pub fn get_lights(&self) -> &Vec<light::Light> {
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

    pub fn set_renderscale(&mut self, scale: u32){
        self.renderscale = scale;
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
    pub fn run(self, app_state: Arc<Mutex<AppState>>) {
        let mut temp_app_state = app_state.lock().unwrap();
        temp_app_state.display = Some(self.display.clone());

        // managing fps
        let mut next_frame_time = Instant::now();
        let nanos = 1_000_000_000 / temp_app_state.fps;
        let frame_duration = Duration::from_nanos(nanos); // 60 FPS (1,000,000,000 ns / 60)

        let mut texture = glium::texture::Texture2d::empty(&self.display, self.window.inner_size().width * temp_app_state.renderscale, self.window.inner_size().height * temp_app_state.renderscale).expect("Failed to create texture");
        let depth_texture = glium::texture::DepthTexture2d::empty(&self.display, self.window.inner_size().width * temp_app_state.renderscale, self.window.inner_size().height * temp_app_state.renderscale).expect("Failed to create depth texture");


        //dropping modified appstate
        drop(temp_app_state);

        // prepare post processing
        let screen_vert_rect = postprocessing::get_screen_vert_rect(&self.display);
        let screen_indices_rect = postprocessing::get_screen_indices_rect(&self.display);
        let screen_program = postprocessing::get_screen_program(&self.display);



        // run loop
        self.event_loop.run(move |event, _window_target, control_flow| {
            // unpacking appstate
            let mut app_state = app_state.lock().unwrap();
            let light = app_state.light.clone();
            let ambient_light = app_state.ambient_light.clone();
            let camera = app_state.camera.clone();
            let event_injections = app_state.event_injections.clone();
            let update_injections = app_state.update_injections.clone();


            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            next_frame_time = Instant::now() + frame_duration;

            // passing framebuffer
            let texture = &mut texture;
            let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&self.display, &*texture, &depth_texture).expect("Failed to create framebuffer");


            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit; }
                    WindowEvent::Resized(new_size) => {
                        app_state.camera.as_mut().unwrap().set_aspect(new_size.width as f32, new_size.height as f32);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        app_state.get_mouse_position_mut().set_screen_position((position.x, position.y));
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        for (characteristic, function) in event_injections {
                            if let event::EventCharacteristic::MousePress(mouse_button) = characteristic {
                                if state == winit::event::ElementState::Pressed && button == mouse_button {
                                    function(&mut app_state);
                                }
                            }
                        };
                    }
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
                    let render_target = &mut framebuffer; //self.display.draw();
                    render_target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
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
                        let closest_lights = object.get_closest_lights(&light);
                        for (buffer, (material, indices)) in object.get_vertex_buffers().iter().zip(object.get_materials().iter().zip(object.get_index_buffers().iter())) {
                            let uniforms = &material.get_uniforms(closest_lights.clone(), ambient_light, camera, Some(model_matrix));
                            render_target.draw(buffer, indices, &material.program, uniforms, &params).expect("Failed to draw object");
                        }
                    }
                    // execute post processing#
                    for process in app_state.get_post_processes() {
                        process.render(&screen_vert_rect, &screen_indices_rect, &mut framebuffer, &texture);
                    }
                    let mut screen_target = self.display.draw();
                    let screen_uniforms = uniform! {
                        scene: &*texture,
                    };
                    screen_target.draw(
                        &screen_vert_rect,
                        &screen_indices_rect,
                        &screen_program,
                        &screen_uniforms,
                        &Default::default(),
                    ).expect("Failed to draw screen");
                    screen_target.finish().expect("Failed to swap buffers");
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
