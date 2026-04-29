use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use egui_glium::EguiGlium;
use winit::window::Window;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface, Texture2d, uniform};
use glium::uniforms::UniformBuffer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::window::CursorGrabMode;
use winit::event_loop::{ControlFlow};
use crate::audio::{AudioClip, AudioEngine};
use crate::shadow::ShadowMaps;
use crate::shadow::{directional_light_space_matrix, view_matrix, perspective_90_matrix, mat4_mul, CUBE_FACE_DIRS, face_viewport};
use crate::camera::{Camera, CameraSerializer};
use crate::collision_world::MouseState;
use crate::data::AppStateData;
use crate::event::EventModifiers;
use crate::geometry::BoneTransforms;
use crate::light::{Light, LightEmissionType};
use crate::logging::{EnigmaError, EnigmaMessage, EnigmaWarning};
use crate::material::Material;
use crate::object::{Object, ObjectInstance};
use crate::postprocessing::PostProcessingEffect;
use crate::texture::Texture;

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
pub mod ui;
pub mod resources;
pub mod data;
pub mod example_resources;
pub mod animation;
pub mod logging;
pub mod audio;
pub mod shadow;

pub fn init_default(app_state: &mut AppState) {
    app_state.set_renderscale(1);
    app_state.set_fps(60);
    app_state.set_max_buffers(3);

    if app_state.get_camera().is_none(){
        app_state.set_camera(Camera::default());
    }

    app_state.inject_event(
        event::EventCharacteristic::MousePress(event::MouseButton::Left),
        Arc::new(default_events::select_object),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::MousePress(event::MouseButton::Right),
        Arc::new(default_events::select_object_add),
        None,
    );

    //event functions for moving the camera
    // adding the camera move and rotation speed as a state data entry. this allows us to retrieve it in all camera related functions while having
    // a unique place to control it. See, that we need to pass the value in with explicit type declaration, this is so enigma can properly use it
    app_state.add_state_data("camera_move_speed", Box::new(10.0f32));
    app_state.add_state_data("camera_rotate_speed", Box::new(2.0f32));

    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::W),
        Arc::new(default_events::camera_fly_forward),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::A),
        Arc::new(default_events::camera_fly_left),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::S),
        Arc::new(default_events::camera_fly_backward),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::D),
        Arc::new(default_events::camera_fly_right),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Space),
        Arc::new(default_events::camera_up),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Space),
        Arc::new(default_events::camera_down),
        Some(EventModifiers::new(true, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::MouseDown(event::MouseButton::Right),
        Arc::new(default_events::camera_rotate),
        Some(EventModifiers::new(true, false, false)),
    );
}

#[derive(Serialize, Deserialize)]
pub struct AppStateSerializer {
    pub camera: Option<CameraSerializer>,
    pub light: Vec<light::LightSerializer>,
    pub ambient_light: Option<light::LightSerializer>,
    pub skybox: Option<object::ObjectSerializer>,
    pub materials: Vec<material::MaterialSerializer>,
    pub skybox_texture: Option<texture::TextureSerializer>,
    pub objects: Vec<object::ObjectSerializer>,
    pub object_selection: Vec<String>,
}

pub struct AppState {
    pub fps: u64,
    pub camera: Option<camera::Camera>,
    pub light: Vec<light::Light>,
    pub ambient_light: Option<light::Light>,
    pub skybox: Option<object::Object>,
    pub skybox_texture: Option<texture::Texture>,
    pub objects: Vec<object::Object>,
    pub materials: Vec<material::Material>,
    pub object_selection: Vec<Uuid>,
    pub event_injections: Vec<(event::EventCharacteristic, event::EventFunction, event::EventModifiers)>,
    pub update_injections: Vec<event::EventFunction>,
    pub gui_injections: Vec<ui::GUIDrawFunction>,
    pub post_processes: Vec<Box<dyn PostProcessingEffect>>,
    pub display: Option<glium::Display<WindowSurface>>,
    pub time: f32,
    pub delta_time: f32,
    pub render_scale: u32,
    pub max_buffers: usize,
    mouse_state: MouseState,
    last_event_time: Instant,
    last_frame_time: Instant,
    is_mouse_down: bool,
    pub state_data: Vec<AppStateData>,
    audio_engine: AudioEngine,
    audio_clips:  HashMap<String, AudioClip>,
    pub shadow_resolution: u32,
    pub shadow_distance: f32,
    pub modifiers: EventModifiers,
    pub held_keys: HashSet<event::VirtualKeyCode>,
    pub cursor_locked: bool,
}

pub struct EventLoop {
    pub event_loop: winit::event_loop::EventLoop<()>,
    pub window: Window,
    pub display: Display<WindowSurface>,
    pub modifiers: EventModifiers,
    gui_renderer: Option<EguiGlium>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            fps: 60,
            camera: None,
            skybox: None,
            skybox_texture: None,
            objects: Vec::new(),
            materials: Vec::new(),
            object_selection: Vec::new(),
            light: Vec::new(),
            ambient_light: None,
            event_injections: Vec::new(),
            update_injections: Vec::new(),
            post_processes: Vec::new(),
            display: None,
            time: 0.0,
            delta_time: 0.0,
            render_scale: 1,
            max_buffers: 3,
            mouse_state: MouseState::new(),
            gui_injections: Vec::new(),
            state_data: Vec::new(),
            last_event_time: Instant::now(),
            last_frame_time: Instant::now(),
            is_mouse_down: false,
            audio_engine: AudioEngine::new(),
            audio_clips: HashMap::new(),
            shadow_resolution: 1024,
            shadow_distance: 50.0,
            modifiers: EventModifiers::default(),
            held_keys: HashSet::new(),
            cursor_locked: false,
        }
    }

    pub fn add_audio(&mut self, clip: AudioClip){
        if self.audio_clips.contains_key(&clip.name){
            EnigmaError::new(Some("Cannot add audio clip, since it is already added"), true).log();
            return;
        }
        self.audio_clips.insert(clip.name.to_string(), clip);
    }

    pub fn play_audio_once(&mut self, name: &str){
        self.audio_engine.play_clip_once(name, &self.audio_clips);
    }

    pub fn play_audio_loop(&mut self, name: &str) {
        self.audio_engine.play_clip_loop(name, &self.audio_clips);
    }

    pub fn stop_audio(&mut self, name: &str) {
        self.audio_engine.stop_clip(name);
    }

    pub fn toggle_pause_audio(&mut self, name: &str) {
        self.audio_engine.toggle_pause_clip(name);
    }

    pub fn set_audio_volume(&mut self, name: &str, volume: f32){
        self.audio_engine.set_clip_volume(name, volume);
    }

    fn setup_skybox_instance(&self, display: &Display<WindowSurface>, sky_box_matrix: &Option<[[f32; 4]; 4]>) -> Option<(Uuid, object::ObjectInstance)> {
        match &self.skybox {
            Some(skybox) => {
                let mut instance = ObjectInstance::new(display);
                let model_matrix = sky_box_matrix.unwrap_or_else(|| {
                    [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
                });
                instance.set_vertex_buffers(skybox.get_vertex_buffers(display));
                instance.set_index_buffers(skybox.get_index_buffers(display));
                instance.instance_matrices.push(model_matrix);
                let data = instance.instance_matrices
                    .iter()
                    .map(|i| geometry::InstanceAttribute {
                        model_matrix: *i,
                    })
                    .collect::<Vec<_>>();
                instance.instance_attributes = glium::vertex::VertexBuffer::dynamic(display, &data).unwrap();
                Some((skybox.get_unique_id(), instance))
            }
            None => None
        }
    }

    fn setup_instances(&mut self, display: &Display<WindowSurface>, model_matrices: &HashMap<Uuid, [[f32; 4]; 4]>) -> HashMap<Uuid, object::ObjectInstance> {
        let mut instances = HashMap::new();
        // sort objects for transparent rendering
        let cam_pos = self.camera.as_ref().expect("failed to retrieve camera").transform.get_position();
        self.objects.sort_by(|a, b| {
            let distance_a = (cam_pos - a.transform.get_position()).len();
            let distance_b = (cam_pos - b.transform.get_position()).len();
            distance_b.partial_cmp(&distance_a).unwrap()
        });

        // iterating over the objects, making instances
        for object in self.objects.iter() {
            let instance_id = object.get_instance_id();
            let model_matrix = model_matrices.get(&object.get_unique_id()).unwrap_or_else(|| {
                &[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
            });
            if !instances.contains_key(&instance_id) {
                let mut object_instance = ObjectInstance::new(display);
                object_instance.set_vertex_buffers(object.get_vertex_buffers(display));
                object_instance.set_index_buffers(object.get_index_buffers(display));
                instances.insert(instance_id, object_instance);
            }
            instances.get_mut(&instance_id).expect("No instance of this uuid found. which is weird, because we just added it above").add_instance(*model_matrix);


            //updating instance attributes
            match instances.get_mut(&instance_id) {
                Some(instance) => {
                    let data = instance.instance_matrices
                        .iter()
                        .map(|i| geometry::InstanceAttribute {
                            model_matrix: *i,
                        })
                        .collect::<Vec<_>>();
                    instance.instance_attributes = glium::vertex::VertexBuffer::dynamic(display, &data).unwrap();
                }
                None => panic!("Something went wrong, when adding the instance")
            }
        }
        instances
    }

    pub fn to_serializer(&self) -> AppStateSerializer {
        EnigmaMessage::new(Some("An AppState Serializer does not completely serialize the AppState but only scene objects like Objects, Camera, Lights. It does NOT serialize any injections like code in form of functions or GUI!"), true).log();
        let camera = match &self.camera {
            Some(camera) => Some(camera.to_serializer()),
            None => None,
        };
        let light = self.light.iter().map(|l| l.to_serializer()).collect();
        let ambient_light = match &self.ambient_light {
            Some(light) => Some(light.to_serializer()),
            None => None,
        };
        let skybox = match &self.skybox {
            Some(skybox) => Some(skybox.to_serializer()),
            None => None,
        };
        let skybox_texture = match &self.skybox_texture {
            Some(texture) => Some(texture.to_serializer()),
            None => None,
        };
        let objects = self.objects.iter().map(|o| o.to_serializer()).collect();
        let materials = self.materials.iter().map(|o| o.to_serializer()).collect();
        let object_selection = self.object_selection.iter().map(|o| o.to_string()).collect();
        AppStateSerializer {
            camera,
            light,
            ambient_light,
            skybox,
            skybox_texture,
            objects,
            materials,
            object_selection,
        }
    }

    pub fn inject_serializer(&mut self, serializer: AppStateSerializer, display: Display<WindowSurface>, additive: bool) {
        self.camera = match serializer.camera {
            Some(camera) => Some(Camera::from_serializer(camera)),
            None => None,
        };
        match serializer.ambient_light {
            Some(light) => {
                self.add_light(Light::from_serializer(light), LightEmissionType::Ambient);
            }
            None => {}
        };
        self.skybox = match serializer.skybox {
            Some(skybox) => Some(Object::from_serializer(skybox)),
            None => None,
        };
        self.skybox_texture = match serializer.skybox_texture {
            Some(texture) => Some(Texture::from_serializer(texture, &display)),
            None => None,
        };

        if !additive {
            self.light.clear();
            self.objects.clear();
            self.object_selection.clear();
        }
        for l in serializer.light {
            self.add_light(Light::from_serializer(l), LightEmissionType::Source);
        }
        for o in serializer.objects {
            self.add_object(Object::from_serializer(o));
        }
        for m in serializer.materials {
            self.add_material(Material::from_serializer(m, &display));
        }
        for o in serializer.object_selection {
            self.object_selection.push(Uuid::parse_str(&o).unwrap());
        }
    }

    pub fn add_state_data(&mut self, name: &str, data: Box<dyn Any>) {
        self.state_data.push(AppStateData::new(name, data));
    }

    pub fn add_material(&mut self, material: Material) {
        self.materials.push(material);
    }

    pub fn get_material(&self, uuid: &Uuid) -> Option<&Material> {
        for material in &self.materials {
            if &material.uuid == uuid {
                return Some(&material);
            }
        }
        None
    }

    pub fn get_material_by_name(&self, name: &str) -> Option<&Material> {
        for material in &self.materials {
            if &material.name == name {
                return Some(&material);
            }
        }
        None
    }

    pub fn get_state_data_value<T: 'static>(&self, name: &str) -> Option<&T> {
        for data in self.state_data.iter() {
            if data.get_name() == name {
                // Attempt to downcast to the requested type T
                if let Some(value) = data.get_value().downcast_ref::<T>() {
                    return Some(value);
                }
            }
        }
        None
    }

    pub fn get_state_data_value_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        for data in self.state_data.iter_mut() {
            if data.get_name() == name {
                // Attempt to downcast to the requested type T
                if let Some(value) = data.get_value_mut().downcast_mut::<T>() {
                    return Some(value);
                }
            }
        }
        None
    }

    pub fn set_state_data_value(&mut self, name: &str, value: Box<dyn Any>) {
        for data in &mut self.state_data {
            if data.get_name() == name {
                data.set_value(value);
                return;
            }
        }
        // If no existing data is found with the name, add as new state data
        self.add_state_data(name, value);
    }

    pub fn inject_gui(&mut self, function: ui::GUIDrawFunction) {
        self.gui_injections.push(function);
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

    pub fn get_mouse_state(&self) -> &MouseState {
        &self.mouse_state
    }

    pub fn get_mouse_state_mut(&mut self) -> &mut MouseState {
        &mut self.mouse_state
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

    pub fn get_object_by_uuid(&self, uuid: &Uuid) -> Option<&object::Object> {
        for object in self.objects.iter() {
            if &object.get_unique_id() == uuid {
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

    pub fn add_light(&mut self, light: light::Light, light_type: LightEmissionType) {
        match light_type {
            LightEmissionType::Source => self.light.push(light),
            LightEmissionType::Ambient => self.ambient_light = Some(light),
        }
    }

    pub fn remove_light(&mut self, index: usize, light_type: LightEmissionType) {
        match light_type {
            LightEmissionType::Source => {
                if index >= self.light.len() {
                    panic!("Index out of bounds");
                }
                self.light.remove(index);
            }
            LightEmissionType::Ambient => {
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

    pub fn get_camera_mut(&mut self) -> &mut Option<camera::Camera> {
        &mut self.camera
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

    pub fn inject_event(&mut self, characteristic: event::EventCharacteristic, function: event::EventFunction, modifiers: Option<event::EventModifiers>) {
        match modifiers {
            Some(modifiers) => self.event_injections.push((characteristic, function, modifiers)),
            None => self.event_injections.push((characteristic, function, event::EventModifiers::default())),
        }
    }
    pub fn inject_update_function(&mut self, function: event::EventFunction) {
        self.update_injections.push(function);
    }

    pub fn set_skybox(&mut self, skybox: object::Object) {
        self.skybox = Some(skybox);
    }

    pub fn set_skybox_from_texture(&mut self, texture: Texture, event_loop: &EventLoop){
        let mut material = crate::material::Material::unlit(event_loop.get_display_clone(), false);
        material.set_name("INTERNAL::SkyBox");

        material.set_texture(texture, crate::material::TextureType::Albedo);
        // create a default object
        let mut object = Object::load_from_gltf_resource(resources::skybox(), None);
        // set the material
        object.add_material(material.uuid);
        object.get_shapes_mut()[0].set_material_from_object_list(0);
        object.name = "Skybox".to_string();
        object.transform.set_scale([1.0, 1.0, 1.0]);
        self.add_material(material);
        self.set_skybox(object);
    }

    pub fn get_skybox(&self) -> &Option<object::Object> {
        &self.skybox
    }

    pub fn get_skybox_mut(&mut self) -> &mut Option<object::Object> {
        &mut self.skybox
    }

    pub fn set_shadow_resolution(&mut self, resolution: crate::light::ShadowResolution) {
        self.shadow_resolution = resolution.value();
    }

    pub fn get_shadow_resolution(&self) -> u32 {
        self.shadow_resolution
    }

    pub fn set_shadow_distance(&mut self, distance: f32) {
        self.shadow_distance = distance;
    }

    pub fn get_shadow_distance(&self) -> f32 {
        self.shadow_distance
    }
}

impl EventLoop {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title(title)
            .with_inner_size(width, height)
            .build(&event_loop);
        EventLoop {
            event_loop,
            window,
            display,
            modifiers: EventModifiers::default(),
            gui_renderer: None,
        }
    }
    pub fn get_display_clone(&self) -> Display<WindowSurface> {
        self.display.clone()
    }

    pub fn get_display_reference(&self) -> &Display<WindowSurface> {
        &self.display
    }

    pub fn spawn_skybox(&mut self, app_state: &mut AppState) -> (crate::object::Object, texture::Texture) {
        if let Some(current_skybox_object) = app_state.get_skybox().clone() {
            // If we have an existing skybox, try to get its texture
            if let Some(texture_uuid) = current_skybox_object.get_materials().first() {
                if let Some(material) = app_state.get_material(texture_uuid) {
                    if let Some(texture) = &material.albedo {
                        // Successfully found texture, clone it and return with the existing object
                        return (
                            current_skybox_object,
                            texture.get_texture_clone(self.get_display_reference())
                        );
                    }
                }
            }

            // If we reached here, we couldn't get the texture from the existing skybox
            let mut logger = EnigmaWarning::new(None, true);
            logger.extent("Failed to get texture from existing skybox. Creating default skybox...");
            logger.log();
        }

        let mut material = crate::material::Material::unlit(self.display.clone(), false);
        material.set_name("INTERNAL::SkyBox");

        material.set_texture_from_resource(resources::skybox_texture(), crate::material::TextureType::Albedo);

        // create a default object
        let mut object = Object::load_from_gltf_resource(resources::skybox(), None);

        // set the material
        object.add_material(material.uuid);
        object.get_shapes_mut()[0].set_material_from_object_list(0);

        object.name = "Skybox".to_string();

        object.transform.set_scale([1.0, 1.0, 1.0]);

        app_state.add_material(material);
        // skybox texture
        let skybox_texture = texture::Texture::from_resource(&self.display, resources::skybox_texture());
        (object, skybox_texture)
    }

    pub fn set_icon_from_path(&self, path: &str) {
        let image = image::open(path).expect("failed to load icon").to_rgba8();
        let image_dimensions = image.dimensions();
        let data = image.into_raw();
        let icon = winit::window::Icon::from_rgba(data, image_dimensions.0, image_dimensions.1).expect("failed to load icon");
        self.window.set_window_icon(Some(icon));
    }

    pub fn set_icon_from_resource(&self, data: &[u8]) {
        let image = image::load_from_memory(data).expect("failed to load icon").to_rgba8();
        let image_dimensions = image.dimensions();
        let data = image.into_raw();
        let icon = winit::window::Icon::from_rgba(data, image_dimensions.0, image_dimensions.1).expect("failed to load icon");
        self.window.set_window_icon(Some(icon));
    }

    // This is just the render loop . an actual event loop still needs to be set up
    pub fn run(mut self, app_state: Arc<Mutex<AppState>>) {
        let mut temp_app_state = app_state.lock().unwrap();
        temp_app_state.display = Some(self.display.clone());

        //spawning skybox
        let (skybox, skybox_texture) = self.spawn_skybox(&mut temp_app_state);
        temp_app_state.set_skybox(skybox);


        // managing fps
        let mut next_frame_time = Instant::now();
        let nanos = 1_000_000_000 / temp_app_state.fps;
        let frame_duration = Duration::from_nanos(nanos); // 60 FPS (1,000,000,000 ns / 60)

        let mut texture = Texture2d::empty(&self.display, self.window.inner_size().width * temp_app_state.render_scale, self.window.inner_size().height * temp_app_state.render_scale).expect("Failed to create texture");
        let mut depth_texture = glium::texture::DepthTexture2d::empty(&self.display, self.window.inner_size().width * temp_app_state.render_scale, self.window.inner_size().height * temp_app_state.render_scale).expect("Failed to create depth texture");

        let mut buffer_textures: Vec<Texture2d> = Vec::new();
        for _ in 0..temp_app_state.max_buffers {
            buffer_textures.push(Texture2d::empty(&self.display, self.window.inner_size().width * temp_app_state.render_scale, self.window.inner_size().height * temp_app_state.render_scale).expect("Failed to create texture"));
        }

        let mut shadow_maps = ShadowMaps::new(&self.display, temp_app_state.shadow_resolution);

        let shadow_dir_program = glium::Program::from_source(
            &self.display,
            resources::shadow_depth_vert_shader(),
            resources::shadow_depth_dir_frag_shader(),
            None,
        ).expect("Failed to compile directional shadow shader");

        let shadow_point_program = glium::Program::from_source(
            &self.display,
            resources::shadow_depth_vert_shader(),
            resources::shadow_depth_point_frag_shader(),
            None,
        ).expect("Failed to compile point shadow shader");

        //dropping modified appstate
        drop(temp_app_state);

        // prepare post processing
        let screen_vert_rect = postprocessing::get_screen_vert_rect(&self.display);
        let screen_indices_rect = postprocessing::get_screen_indices_rect(&self.display);
        let screen_program = postprocessing::get_screen_program(&self.display);

        //initializing GUI
        match self.gui_renderer {
            Some(_) => {}
            None => {
                let egui_glium = EguiGlium::new(&self.display, &self.window, &self.event_loop);
                self.gui_renderer = Some(egui_glium);
            }
        }
        // run loop
        self.event_loop.run(move |event, _window_target, control_flow| {
            // unpacking appstate
            let mut app_state = app_state.lock().unwrap();
            let light = app_state.light.clone();
            let ambient_light = app_state.ambient_light.clone();
            let camera = app_state.camera.clone();
            let event_injections = app_state.event_injections.clone();
            let update_injections = app_state.update_injections.clone();
            let gui_injections = app_state.gui_injections.clone();

            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            next_frame_time = Instant::now() + frame_duration;

            // passing framebuffer
            let texture = &mut texture;
            let depth_texture = &mut depth_texture;
            let buffer_textures = &mut buffer_textures;
            let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&self.display, &*texture, &*depth_texture).expect("Failed to create framebuffer");

            // passing skybox
            let skybox_texture = &skybox_texture;

            match event {
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    app_state.get_mouse_state_mut().add_raw_delta(delta.0, delta.1);
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit; }
                    WindowEvent::Resized(new_size) => {
                        let response = self.gui_renderer.as_mut().expect("Failed to retrieve gui renderer").on_event(&event);
                        if !response.consumed {
                            app_state.camera.as_mut().expect("failed to retrieve camera").set_aspect(new_size.width as f32, new_size.height as f32);
                            self.display.resize(new_size.into());
                            if let Some(app_state_display) = app_state.display.as_mut() {
                                app_state_display.resize(new_size.into());
                            }
                        }
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        self.modifiers.ctrl = modifiers.ctrl();
                        self.modifiers.shift = modifiers.shift();
                        self.modifiers.alt = modifiers.alt();
                        app_state.modifiers = self.modifiers;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let response = self.gui_renderer.as_mut().expect("Failed to retrieve gui renderer").on_event(&event);
                        if !response.consumed {
                            app_state.get_mouse_state_mut().update_position((position.x, position.y));
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let mut response = self.gui_renderer.as_mut().expect("Failed to retrieve gui renderer").on_event(&event);
                        if !response.consumed {
                            for (characteristic, function, modifiers) in &event_injections {
                                if let event::EventCharacteristic::MouseDown(mouse_button) = characteristic {
                                    if button == *mouse_button && modifiers == &self.modifiers {
                                        if state == winit::event::ElementState::Pressed {
                                            app_state.is_mouse_down = true;
                                            app_state.last_event_time = Instant::now();
                                            function(&mut app_state);
                                        } else {
                                            app_state.is_mouse_down = false;
                                        }
                                    }
                                } else if let event::EventCharacteristic::MousePress(_) = characteristic {
                                    if state == winit::event::ElementState::Pressed && modifiers == &self.modifiers {
                                        function(&mut app_state);
                                        response.consumed = true;
                                    }
                                }
                            }
                        }
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        let response = self.gui_renderer.as_mut().expect("Failed to retrieve gui renderer").on_event(&event);
                        if let Some(key_code) = input.virtual_keycode {
                            if input.state == winit::event::ElementState::Pressed {
                                app_state.held_keys.insert(key_code);
                            } else {
                                app_state.held_keys.remove(&key_code);
                            }
                        }
                        if !response.consumed {
                            for (characteristic, function, modifiers) in event_injections {
                                if let event::EventCharacteristic::KeyPress(key_code) = characteristic {
                                    if input.state == winit::event::ElementState::Pressed && input.virtual_keycode == Some(key_code) && modifiers == self.modifiers {
                                        function(&mut app_state);
                                    }
                                }
                            };
                        }
                    }
                    _ => {
                        _ = self.gui_renderer.as_mut().expect("Failed to retrieve gui renderer").on_event(&event);
                    }
                }
                Event::RedrawRequested(_) => {
                    let current_time = Instant::now();
                    app_state.delta_time = (current_time - app_state.last_frame_time).as_secs_f32();
                    app_state.last_frame_time = current_time;
                    app_state.time += app_state.delta_time;
                    // updating materials
                    for material in app_state.materials.iter_mut() {
                        material.update();
                    }
                    // updating objects
                    let deltatime = app_state.delta_time;
                    for object in app_state.objects.iter_mut() {
                        object.update(deltatime);
                    }

                    let render_target = &mut framebuffer;
                    render_target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                    let model_matrices: std::collections::HashMap<Uuid, [[f32; 4]; 4]> = app_state.objects.iter_mut().map(|x| (x.get_unique_id(), x.transform.get_matrix())).collect();
                    let bone_uniform_buffers: std::collections::HashMap<Uuid, UniformBuffer<BoneTransforms>> = app_state.objects.iter_mut().map(|x| (x.get_unique_id(), x.get_bone_transform_buffer(&self.display))).collect();
                    let object_instances = app_state.setup_instances(&self.display, &model_matrices);

                    // --- Shadow pass ---
                    if shadow_maps.resolution != app_state.shadow_resolution {
                        shadow_maps = ShadowMaps::new(&self.display, app_state.shadow_resolution);
                    }
                    shadow_maps.clear();

                    let shadow_draw_params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            ..Default::default()
                        },
                        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                        ..Default::default()
                    };

                    let cam_pos: [f32; 3] = match camera {
                        Some(ref c) => { let p = c.transform.get_position(); [p.x, p.y, p.z] }
                        None => [0.0, 0.0, 0.0],
                    };

                    for (light_index, light_item) in light.iter().enumerate().take(4) {
                        if !light_item.cast_shadow { continue; }

                        if light_item.is_directional() {
                            // --- Directional shadow map ---
                            let half = app_state.shadow_distance;
                            let lsm = directional_light_space_matrix(light_item.direction, cam_pos, half);
                            shadow_maps.light_space_matrices[light_index] = lsm;

                            let shadow_tex = glium::texture::Texture2d::empty_with_format(
                                &self.display,
                                glium::texture::UncompressedFloatFormat::F32,
                                glium::texture::MipmapsOption::NoMipmap,
                                shadow_maps.resolution,
                                shadow_maps.resolution,
                            ).expect("Failed to create directional shadow texture");

                            {
                                let mut fb = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                                    &self.display,
                                    &shadow_tex,
                                    &shadow_maps.dir_depth_rb,
                                ).expect("Failed to create directional shadow framebuffer");
                                fb.clear_color_and_depth((1.0, 0.0, 0.0, 1.0), 1.0);

                                for (instance_id, object_instance) in object_instances.iter() {
                                    if let Some(object) = app_state.get_object_by_uuid(instance_id) {
                                        if object.get_materials().is_empty() { continue; }
                                        let has_skeleton = object.get_skeleton().is_some();
                                        let bone_transform = bone_uniform_buffers.get(&object.get_unique_id())
                                            .expect("Missing bone transforms in shadow pass");
                                        for ((buffer, _mat_index), indices) in object_instance.vertex_buffers.iter()
                                            .zip(object_instance.index_buffers.iter())
                                        {
                                            let uniforms = glium::uniform! {
                                                light_space_matrix: lsm,
                                                has_skeleton: has_skeleton,
                                                BoneTransforms: bone_transform,
                                            };
                                            fb.draw(
                                                (buffer, object_instance.instance_attributes.per_instance().unwrap()),
                                                indices,
                                                &shadow_dir_program,
                                                &uniforms,
                                                &shadow_draw_params,
                                            ).expect("Failed to draw shadow pass");
                                        }
                                    }
                                }
                            }
                            shadow_maps.directional_maps[light_index] = Some(shadow_tex);

                        } else {
                            // --- Point light shadow map (atlas) ---
                            let far_plane = 100.0f32;
                            shadow_maps.point_far_planes[light_index] = far_plane;
                            let res = shadow_maps.resolution;

                            let atlas_tex = glium::texture::Texture2d::empty_with_format(
                                &self.display,
                                glium::texture::UncompressedFloatFormat::F32,
                                glium::texture::MipmapsOption::NoMipmap,
                                res * 2,
                                res * 3,
                            ).expect("Failed to create point shadow atlas texture");

                            // Clear entire atlas to 1.0 (no shadow) before rendering faces
                            {
                                let mut clear_fb = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                                    &self.display,
                                    &atlas_tex,
                                    &shadow_maps.point_depth_rb,
                                ).expect("Failed to create atlas clear framebuffer");
                                clear_fb.clear_color_and_depth((1.0, 0.0, 0.0, 1.0), 1.0);
                            }

                            let near = 0.1f32;
                            let proj = perspective_90_matrix(near, far_plane);
                            let lp = light_item.position;

                            for face in 0..6usize {
                                let (dir, up) = CUBE_FACE_DIRS[face];
                                let view = view_matrix(&lp, &dir, &up);
                                let lsm = mat4_mul(proj, view);
                                let viewport = face_viewport(face, res);

                                let face_draw_params = glium::DrawParameters {
                                    depth: glium::Depth {
                                        test: glium::draw_parameters::DepthTest::IfLess,
                                        write: true,
                                        ..Default::default()
                                    },
                                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                                    viewport: Some(viewport),
                                    ..Default::default()
                                };

                                let mut fb = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                                    &self.display,
                                    &atlas_tex,
                                    &shadow_maps.point_depth_rb,
                                ).expect("Failed to create point shadow framebuffer");

                                for (instance_id, object_instance) in object_instances.iter() {
                                    if let Some(object) = app_state.get_object_by_uuid(instance_id) {
                                        if object.get_materials().is_empty() { continue; }
                                        let has_skeleton = object.get_skeleton().is_some();
                                        let bone_transform = bone_uniform_buffers.get(&object.get_unique_id())
                                            .expect("Missing bone transforms in shadow pass");
                                        for ((buffer, _mat_index), indices) in object_instance.vertex_buffers.iter()
                                            .zip(object_instance.index_buffers.iter())
                                        {
                                            let uniforms = glium::uniform! {
                                                light_space_matrix: lsm,
                                                has_skeleton: has_skeleton,
                                                BoneTransforms: bone_transform,
                                                light_pos: lp,
                                                far_plane: far_plane,
                                            };
                                            fb.draw(
                                                (buffer, object_instance.instance_attributes.per_instance().unwrap()),
                                                indices,
                                                &shadow_point_program,
                                                &uniforms,
                                                &face_draw_params,
                                            ).expect("Failed to draw shadow pass");
                                        }
                                    }
                                }
                            }
                            shadow_maps.point_maps[light_index] = Some(atlas_tex);
                        }
                    }
                    // --- End shadow pass ---

                    // render objects opaque
                    let opaque_rendering_parameter = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            ..Default::default()
                        },
                        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                        ..Default::default()
                    };

                    for (instance_id, object_instance) in object_instances.iter() {
                        let object_option = app_state.get_object_by_uuid(&instance_id);
                        match object_option {
                            Some(object) => {
                                let closest_lights = object.get_closest_lights(&light);
                                let has_skeleton = object.get_skeleton().is_some();
                                let bone_transform = bone_uniform_buffers.get(&object.get_unique_id()).expect("Missing Bone Transform Uniforms for Object");
                                for ((buffer, mat_index), indices) in object_instance.vertex_buffers.iter().zip(object_instance.index_buffers.iter()) {
                                    let mat_uuid: &Uuid = &object.get_materials()[*mat_index];
                                    match app_state.get_material(mat_uuid) {
                                        Some(material) => {
                                            if material.render_transparent {
                                                continue;
                                            }
                                            let uniforms = &material.get_uniforms(&closest_lights, ambient_light.as_ref(), camera.as_ref(), &bone_transform, has_skeleton, skybox_texture, &shadow_maps);
                                            render_target.draw((buffer, object_instance.instance_attributes.per_instance().expect("Error, unwrapping per instance in opaque draw")), indices, &material.program, uniforms, &opaque_rendering_parameter).expect("Failed to draw object");
                                        }
                                        None => ()
                                    }
                                }
                            }
                            None => EnigmaError::new(Some(smart_format!("Error, instancing the Object Instance with the instance id {}, because no Object with that Id could be found", instance_id).as_str()), true).log()
                        }
                    }

                    // render skybox
                    let skybox_rendering_parameter = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: false,
                            ..Default::default()
                        },
                        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                        ..Default::default()
                    };

                    //First get the matrix outside of the closure
                    let skybox_model_matrix = match app_state.get_skybox_mut() {
                        Some(obj) => Some(obj.transform.get_matrix().clone()),
                        None => None
                    };
                    let skybox_instance = app_state.setup_skybox_instance(&self.display, &skybox_model_matrix);

                    match skybox_instance {
                        Some((skybox_id, instance)) => {
                            let object_option = app_state.get_skybox();
                            match object_option {
                                Some(skybox) => {
                                    let closest_lights = skybox.get_closest_lights(&light);
                                    let skybox_bone_buffer = skybox.get_bone_transform_buffer(&self.display);
                                    for ((buffer, mat_index), indices) in instance.vertex_buffers.iter().zip(instance.index_buffers.iter()) {
                                        let mat_uuid: &Uuid = &skybox.get_materials()[*mat_index];
                                        match app_state.get_material(mat_uuid) {
                                            Some(material) => {
                                                let uniforms = &material.get_uniforms(&closest_lights, ambient_light.as_ref(), camera.as_ref(), &skybox_bone_buffer, false, skybox_texture, &shadow_maps);
                                                render_target.draw((buffer, instance.instance_attributes.per_instance().expect("Error, unwrapping per instance in skybox draw")), indices, &material.program, uniforms, &skybox_rendering_parameter).expect("Failed to draw object");
                                            }
                                            None => ()
                                        }
                                    }
                                }
                                None => EnigmaError::new(Some(smart_format!("Error, instancing the Skybox Instance with the instance id {}, because no Object with that Id could be found", skybox_id).as_str()), true).log()
                            }
                        }
                        None => {}
                    }

                    // render objects transparent
                    let transparent_rendering_parameter = glium::DrawParameters {
                        blend: glium::Blend::alpha_blending(),
                        ..opaque_rendering_parameter
                    };
                    for (instance_id, object_instance) in object_instances.iter() {
                        let object_option = app_state.get_object_by_uuid(&instance_id);
                        match object_option {
                            Some(object) => {
                                let closest_lights = object.get_closest_lights(&light);
                                let has_skeleton = object.get_skeleton().is_some();
                                let bone_transform = bone_uniform_buffers.get(&object.get_unique_id()).expect("Missing Bone Transform Uniforms for Object");
                                for ((buffer, mat_index), indices) in object_instance.vertex_buffers.iter().zip(object_instance.index_buffers.iter()) {
                                    let mat_uuid: &Uuid = &object.get_materials()[*mat_index];
                                    match app_state.get_material(mat_uuid) {
                                        Some(material) => {
                                            if !material.render_transparent {
                                                continue;
                                            }
                                            let uniforms = &material.get_uniforms(&closest_lights, ambient_light.as_ref(), camera.as_ref(), &bone_transform, has_skeleton, skybox_texture, &shadow_maps);
                                            render_target.draw((buffer, object_instance.instance_attributes.per_instance().expect("Error, unwrapping per instance in transparent draw")), indices, &material.program, uniforms, &transparent_rendering_parameter).expect("Failed to draw object");
                                        }
                                        None => ()
                                    }
                                }
                            }
                            None => EnigmaError::new(Some(smart_format!("Error, instancing the Transparent Object Instance with the instance id {}, because no Object with that Id could be found", instance_id).as_str()), true).log()
                        }
                    }

                    // execute post processing
                    // Each effect reads from a ping-pong buffer (not from `texture` directly),
                    // because `framebuffer` is backed by `texture` — sampling from a texture
                    // that is simultaneously attached as a render target is undefined in OpenGL.
                    let pp_src_idx = buffer_textures.len() - 1;
                    for process in app_state.get_post_processes() {
                        {
                            let mut pp_fb = glium::framebuffer::SimpleFrameBuffer::new(&self.display, &buffer_textures[pp_src_idx]).expect("Failed to create post-process ping-pong framebuffer");
                            let copy_uniforms = uniform! { scene: &*texture };
                            pp_fb.draw(&screen_vert_rect, &screen_indices_rect, &screen_program, &copy_uniforms, &Default::default()).expect("Failed to copy to ping-pong buffer");
                        }
                        process.render(&app_state, &screen_vert_rect, &screen_indices_rect, &mut framebuffer, &buffer_textures[pp_src_idx], &depth_texture, &buffer_textures);
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

                    // drawing GUI
                    let gui_renderer = self.gui_renderer.as_mut().expect("Failed to retrieve gui renderer");
                    gui_renderer.run(&self.window, |egui_context| {
                        for function in gui_injections.iter() {
                            function(egui_context, &mut app_state);
                        }
                    });
                    gui_renderer.paint(&self.display, &mut screen_target);
                    screen_target.finish().expect("Failed to swap buffers");
                }
                Event::MainEventsCleared => {

                    // executing mouse down events
                    if app_state.is_mouse_down && app_state.last_event_time.elapsed() >= Duration::from_millis(100) {
                        for (characteristic, function, modifiers) in &event_injections {
                            if let event::EventCharacteristic::MouseDown(_) = characteristic {
                                if modifiers == &self.modifiers {
                                    function(&mut app_state);
                                    app_state.last_event_time = Instant::now();
                                }
                            }
                        }
                    }

                    // executing key down events (held keys, fired every frame)
                    let held: Vec<event::VirtualKeyCode> = app_state.held_keys.iter().copied().collect();
                    for key in &held {
                        for (characteristic, function, modifiers) in &event_injections {
                            if let event::EventCharacteristic::KeyDown(key_code) = characteristic {
                                if key_code == key && modifiers == &self.modifiers {
                                    function(&mut app_state);
                                }
                            }
                        }
                    }

                    // executing update functions
                    for function in update_injections {
                        function(&mut app_state);
                    }

                    // sync cursor lock state
                    if app_state.cursor_locked {
                        if self.window.set_cursor_grab(CursorGrabMode::Locked).is_err() {
                            let _ = self.window.set_cursor_grab(CursorGrabMode::Confined);
                        }
                        self.window.set_cursor_visible(false);
                    } else {
                        let _ = self.window.set_cursor_grab(CursorGrabMode::None);
                        self.window.set_cursor_visible(true);
                    }

                    // reset raw delta — consumed by update functions above
                    app_state.get_mouse_state_mut().reset_delta();

                    self.window.request_redraw();
                }
                _ => (),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::light::ShadowResolution;

    #[test]
    fn appstate_shadow_defaults() {
        let s = AppState::new();
        assert_eq!(s.get_shadow_resolution(), 1024);
        assert_eq!(s.get_shadow_distance(), 50.0);
    }

    #[test]
    fn appstate_shadow_setters() {
        let mut s = AppState::new();
        s.set_shadow_resolution(ShadowResolution::High);
        assert_eq!(s.get_shadow_resolution(), 2048);
        s.set_shadow_distance(75.0);
        assert_eq!(s.get_shadow_distance(), 75.0);
    }
}
