use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface, Texture2d, uniform};
use glium::texture::{DepthCubemap, DepthTexture2d};
use itertools::enumerate;
use uuid::Uuid;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow};
use crate::collision_world::MousePosition;
use crate::light::LightType;
use crate::object::Object;
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
    app_state.set_fps(120);
    app_state.set_max_buffers(3);

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
    pub lights: Vec<light::Light>,
    pub ambient_light: Option<light::Light>,
    pub skybox: Option<object::Object>,
    pub skybox_texture: Option<texture::Texture>,
    pub objects: Vec<object::Object>,
    pub object_selection: Vec<Uuid>,
    pub event_injections: Vec<(event::EventCharacteristic, event::EventFunction)>,
    pub update_injections: Vec<event::EventFunction>,
    pub post_processes: Vec<Box<dyn PostProcessingEffect>>,
    pub display: Option<glium::Display<WindowSurface>>,
    pub time: f32,
    pub render_scale: u32,
    pub max_buffers: usize,
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
            skybox: None,
            skybox_texture: None,
            objects: Vec::new(),
            object_selection: Vec::new(),
            lights: Vec::new(),
            ambient_light: None,
            event_injections: Vec::new(),
            update_injections: Vec::new(),
            post_processes: Vec::new(),
            display: None,
            time: 0.0,
            render_scale: 1,
            max_buffers: 3,
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

    pub fn remove_object(&mut self, unique_id: Uuid) {
        let mut index = 0;
        for object in self.objects.iter() {
            if object.get_unique_id() == unique_id {
                break;
            }
            index += 1;
        }
        self.objects.remove(index);
    }

    pub fn get_shadow_casting_light_count(&self) -> usize {
        let mut count = 0;
        for light in self.lights.iter() {
            if light.cast_shadow {
                count += 1;
            }
        }
        count
    }

    pub fn remove_object_by_name(&mut self, name: &str) {
        let mut index = 0;
        for object in self.objects.iter() {
            if object.name == name {
                break;
            }
            index += 1;
        }
        self.objects.remove(index);
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
            LightType::Point => self.lights.push(light),
            LightType::Ambient => self.ambient_light = Some(light),
        }
    }

    pub fn remove_light(&mut self, index: usize, light_type: LightType) {
        match light_type {
            LightType::Point => {
                if index >= self.lights.len() {
                    panic!("Index out of bounds");
                }
                self.lights.remove(index);
            }
            LightType::Ambient => {
                self.ambient_light = None;
            }
        };
    }

    pub fn get_lights(&self) -> &Vec<light::Light> {
        &self.lights
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

    pub fn set_renderscale(&mut self, scale: u32) {
        self.render_scale = scale;
    }

    pub fn get_renderscale(&self) -> u32 {
        self.render_scale
    }

    pub fn set_max_buffers(&mut self, max_buffers: usize) {
        self.max_buffers = max_buffers;
    }

    pub fn get_max_buffers(&self) -> usize {
        self.max_buffers
    }

    pub fn inject_event(&mut self, characteristic: event::EventCharacteristic, function: event::EventFunction) {
        self.event_injections.push((characteristic, function));
    }
    pub fn inject_update_function(&mut self, function: event::EventFunction) {
        self.update_injections.push(function);
    }

    pub fn set_skybox(&mut self, skybox: object::Object) {
        self.skybox = Some(skybox);
    }

    pub fn get_skybox(&self) -> &Option<object::Object> {
        &self.skybox
    }

    pub fn get_skybox_mut(&mut self) -> &mut Option<object::Object> {
        &mut self.skybox
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

    pub fn spawn_skybox(&self) -> (crate::object::Object, texture::Texture) {
        let mut material = crate::material::Material::unlit(self.display.clone(), false);
        material.set_texture_from_file("res/textures/skybox.png", crate::material::TextureType::Albedo);

        // create a default object
        let mut object = Object::load_from_gltf("res/models/skybox.gltf");

        // set the material
        object.add_material(material);
        object.get_shapes_mut()[0].set_material_from_object_list(0);

        object.name = "Skybox".to_string();

        object.transform.set_scale([1.0, 1.0, 1.0]);

        // skybox texture
        let skybox_texture = texture::Texture::new(&self.display, "res/textures/skybox.png");

        (object, skybox_texture)
    }

    pub fn run(self, app_state: Arc<Mutex<AppState>>) {
        let mut temp_app_state = app_state.lock().unwrap();
        temp_app_state.display = Some(self.display.clone());

        //spawning skybox
        let (skybox, skybox_texture) = self.spawn_skybox();
        temp_app_state.set_skybox(skybox);


        // managing fps
        let mut next_frame_time = Instant::now();
        let nanos = 1_000_000_000 / temp_app_state.fps;
        let frame_duration = Duration::from_nanos(nanos); // 60 FPS (1,000,000,000 ns / 60)

        // set width and height
        let width = self.window.inner_size().width * temp_app_state.render_scale;
        let height = self.window.inner_size().height * temp_app_state.render_scale;

        let mut texture = Texture2d::empty(&self.display, width, height).expect("Failed to create texture");
        let mut depth_texture = DepthTexture2d::empty(&self.display, width, height).expect("Failed to create depth texture");

        let mut buffer_textures: Vec<Texture2d> = Vec::new();
        for _ in 0..temp_app_state.max_buffers {
            buffer_textures.push(Texture2d::empty(&self.display, width, height).expect("Failed to create texture"));
        }

        // shadow map creation
        let mut shadow_maps: Vec<DepthCubemap> = Vec::new();
        for l in temp_app_state.lights.iter() {
            if l.cast_shadow {
                let shadow_map = DepthCubemap::empty(&self.display, 1024).expect("Failed to create shadow map"); //TODO: expose shadow map resolution in app_state
                shadow_maps.push(shadow_map);
            }
        }
        let shadow_map_program = light::get_shadow_map_program(&self.display);

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
            let mut lights = app_state.lights.clone();
            let ambient_light = app_state.ambient_light.clone();
            let camera = app_state.camera.clone();
            let event_injections = app_state.event_injections.clone();
            let update_injections = app_state.update_injections.clone();


            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            next_frame_time = Instant::now() + frame_duration;

            // passing skybox
            let skybox_texture = &skybox_texture;

            // passing textures and creating fbo's
            let texture = &mut texture;
            let depth_texture = &mut depth_texture;
            let buffer_textures = &mut buffer_textures;
            let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&self.display, &*texture, &*depth_texture).expect("Failed to create framebuffer");

            // passing shadow map
            let shadow_maps = &mut shadow_maps;
            while shadow_maps.len() < app_state.get_shadow_casting_light_count() {
                shadow_maps.push(DepthCubemap::empty(&self.display, 1024).expect("Failed to create shadow map")); //TODO: expose shadow map resolution in app_state
            }


            // prepare rendering parameters
            let opaque_rendering_parameter = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            };
            let skybox_rendering_parameter = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: false,
                    ..Default::default()
                },
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            };
            let transparent_rendering_parameter = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            };


            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit; }
                    WindowEvent::Resized(new_size) => {
                        app_state.camera.as_mut().expect("failed to retrieve camera").set_aspect(new_size.width as f32, new_size.height as f32);
                        self.display.resize(new_size.into());
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
                    let render_target = &mut framebuffer;
                    render_target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                    // sort objects by distance to camera
                    app_state.objects.sort_by(|a, b| {
                        let distance_a = (camera.expect("failed to retrieve camera").transform.get_position() - a.transform.get_position()).len();
                        let distance_b = (camera.expect("failed to retrieve camera").transform.get_position() - b.transform.get_position()).len();
                        distance_b.partial_cmp(&distance_a).unwrap()
                    });

                    //render shadow maps
                    for (index, light) in enumerate(lights.iter_mut()){
                        if !light.cast_shadow {
                            continue;
                        }
                        let map = &mut shadow_maps[index];
                        let light_projection_matrix = light.calculate_projection_matrix_for_point_light(0.0, 100.0); //TODO: expose shadow far clip planes in app_state
                        for i in 0..6{
                            let face = texture::cube_layer_from_index(i);
                            let mut shadow_fbo = glium::framebuffer::SimpleFrameBuffer::depth_only(&self.display, map.main_level().image(face)).expect("Failed to create shadow framebuffer");
                            shadow_fbo.clear_depth(1.0);

                            let view_matrix = light.calculate_view_matrix_for_cubemap_face(i);
                            let light_view_projection_matrix = light_projection_matrix * view_matrix;
                            for object in app_state.objects.iter_mut(){
                                let model_matrix = object.transform.get_matrix();
                                let mvp: [[f32; 4]; 4] = (light_view_projection_matrix * model_matrix).into();
                                let uniforms = uniform! {
                                    depth_mvp: mvp,
                                };
                                for (buffer, indices) in object.get_vertex_buffers().iter().zip(object.get_index_buffers().iter()){
                                    shadow_fbo.draw(buffer, indices, &shadow_map_program, &uniforms, &opaque_rendering_parameter).expect("Failed to draw object");
                                }
                            }
                        }
                    }

                    // render objects opaque
                    for object in app_state.objects.iter_mut() {
                        let model_matrix = object.transform.get_matrix();
                        let closest_lights = object.get_closest_lights(&lights);
                        let mut closest_shadowmaps: Vec<&DepthCubemap> = Vec::new();
                        for index in closest_lights.0.iter(){
                            if closest_lights.1[*index].cast_shadow {
                                closest_shadowmaps.push(&shadow_maps[*index]);
                            }
                        }
                        let closest_lights = closest_lights.1;
                        for (buffer, (material, indices)) in object.get_vertex_buffers().iter().zip(object.get_materials().iter().zip(object.get_index_buffers().iter())) {
                            if material.render_transparent {
                                continue;
                            }
                            let uniforms = &material.get_uniforms(closest_lights.clone(), ambient_light, camera, Some(model_matrix.into()), skybox_texture, &mut closest_shadowmaps);
                            render_target.draw(buffer, indices, &material.program, uniforms, &opaque_rendering_parameter).expect("Failed to draw object");
                        }
                    }

                    // render skybox
                    match app_state.get_skybox_mut() {
                        Some(skybox) => {
                            let model_matrix = skybox.transform.get_matrix();
                            let closest_lights = skybox.get_closest_lights(&lights);
                            let mut closest_shadowmaps: Vec<&DepthCubemap> = Vec::new();
                            for index in closest_lights.0.iter(){
                                if closest_lights.1[*index].cast_shadow {
                                    closest_shadowmaps.push(&shadow_maps[*index]);
                                }
                            }
                            let closest_lights = closest_lights.1;
                            for (buffer, (material, indices)) in skybox.get_vertex_buffers().iter().zip(skybox.get_materials().iter().zip(skybox.get_index_buffers().iter())) {
                                let uniforms = &material.get_uniforms(closest_lights.clone(), ambient_light, camera, Some(model_matrix.into()), skybox_texture, &mut closest_shadowmaps);
                                render_target.draw(buffer, indices, &material.program, uniforms, &skybox_rendering_parameter).expect("Failed to draw object");
                            }
                        }
                        None => {}
                    }

                    // render objects transparent
                    for object in app_state.objects.iter_mut() {
                        let model_matrix = object.transform.get_matrix();
                        let closest_lights = object.get_closest_lights(&lights);
                        let mut closest_shadowmaps: Vec<&DepthCubemap> = Vec::new();
                        for index in closest_lights.0.iter(){
                            if closest_lights.1[*index].cast_shadow {
                                closest_shadowmaps.push(&shadow_maps[*index]);
                            }
                        }
                        let closest_lights = closest_lights.1;
                        for (buffer, (material, indices)) in object.get_vertex_buffers().iter().zip(object.get_materials().iter().zip(object.get_index_buffers().iter())) {
                            if !material.render_transparent {
                                continue;
                            }
                            let uniforms = &material.get_uniforms(closest_lights.clone(), ambient_light, camera, Some(model_matrix.into()), skybox_texture, &mut closest_shadowmaps);
                            render_target.draw(buffer, indices, &material.program, uniforms, &transparent_rendering_parameter).expect("Failed to draw object");
                        }
                    }

                    // execute post processing#
                    for process in app_state.get_post_processes() {
                        process.render(&app_state, &screen_vert_rect, &screen_indices_rect, &mut framebuffer, &texture, &depth_texture, &buffer_textures);
                    }

                    // drawing to screen
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
