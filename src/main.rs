use std::sync::Arc;
use enigma::object::Object;
use enigma::camera::Camera;
use enigma::{AppState, event};
use rand::Rng;
use enigma::postprocessing::bloom::Bloom;
use enigma::postprocessing::edge::Edge;

fn rotate_left(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        if object.get_name().contains("Ground") {
            continue;
        }
        object.transform.rotate([0.0, -5.0, 0.0]);
    }
}

fn rotate_right(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        if object.get_name().contains("Ground") {
            continue;
        }
        object.transform.rotate([0.0, 5.0, 0.0]);
    }
}

fn rotate_up(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        if object.get_name().contains("Ground") {
            continue;
        }
        object.transform.rotate([-5.0, 0.0, 0.0]);
    }
}

fn rotate_down(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        if object.get_name().contains("Ground") {
            continue;
        }
        object.transform.rotate([5.0, 0.0, 0.0]);
    }
}

fn roll_left(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        if object.get_name().contains("Ground") {
            continue;
        }
        object.transform.rotate([0.0, 0.0, 5.0]);
    }
}

fn roll_right(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        if object.get_name().contains("Ground") {
            continue;
        }
        object.transform.rotate([0.0, 0.0, -5.0]);
    }
}

fn hopping_objects(app_state: &mut AppState) {
    for object in app_state.objects.iter_mut() {
        if object.name.contains("Ground") {
            continue;
        }
        let rand_scale = rand::thread_rng().gen_range(0.0..0.015);
        object.transform.move_dir([0.0, (app_state.time * 20.0).sin() * rand_scale, 0.0])
    }
}

fn move_light(app_state: &mut AppState) {
    let light = &mut app_state.lights[0];
    let time = app_state.time;
    let new_pos = [(time * 20.0).sin() * 2.0, 1.0, -1.0];
    light.position = new_pos.clone();
}

fn remove_object(app_state: &mut AppState) {
    if app_state.time % 0.1 < 0.001 {
        if app_state.objects.len() < 3 {
            return;
        }
        let id = app_state.objects[2].get_unique_id();
        app_state.remove_object(id);
    }
}

fn spawn_object(app_state: &mut AppState) {
    match &app_state.display {
        Some(d) => {
            let random_bool = rand::thread_rng().gen_bool(0.5);
            let mut material = enigma::material::Material::lit_pbr(d.clone(), random_bool);
            material.set_transparency_strength(0.2);
            material.set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);

            let mut object = Object::load_from_gltf("res/models/suzanne.gltf");
            object.name = format!("Suzanne_{}", rand::thread_rng().gen_range(0..1000));
            object.add_material(material);
            let random_x = rand::thread_rng().gen_range(-4.0..4.0);
            let random_y = rand::thread_rng().gen_range(-4.0..4.0);
            let random_z = rand::thread_rng().gen_range(-4.0..-1.0);

            object.transform.set_position([random_x, random_y, random_z]);
            object.transform.set_scale([0.3, 0.3, 0.3]);

            app_state.add_object(object);
        }
        None => {
            println!("No display found, could not spawn object");
        }
    }
}

fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma::EventLoop::new("Enigma 3D Renderer Window");
    let mut app_state = enigma::AppState::new();

    // some default event setups like selection
    enigma::init_default(&mut app_state);

    let mut material = enigma::material::Material::lit_pbr(event_loop.display.clone(), false);
    material.set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);

    // create a default object
    let mut object = Object::load_from_gltf("res/models/suzanne.gltf");

    // set the material
    object.add_material(material);
    object.get_shapes_mut()[0].set_material_from_object_list(0);

    object.name = "Suzanne".to_string();
    object.transform.set_position([0.0, 0.0, -2.0]);

    // adding all the objects
    app_state.add_object(object);

    // add ground
    let mut ground_object = Object::primitive_plane(20,10);
    ground_object.set_name("Ground".to_string());
    ground_object.transform.set_position([-10.0, -0.5, -5.0]);
    let mut ground_material = enigma::material::Material::lit_pbr(event_loop.display.clone(), false);
    ground_material.set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);
    ground_material.set_color([1.0, 1.0, 1.0]);
    ground_material.set_roughness_strength(1.0);
    ground_object.add_material(ground_material);
    ground_object.get_shapes_mut()[0].set_material_from_object_list(0);
    app_state.add_object(ground_object);

    // add lighting
    let light1 = enigma::light::Light::new([0.0, 3.0, 0.0], [1.0, 1.0, 1.0], 30.0, true);
    let light2 = enigma::light::Light::new([5.0, 1.0, 1.0], [1.0, 0.0, 0.0], 10.0, false);
    let light3 = enigma::light::Light::new([0.0, 1.0, 5.0], [0.0, 0.0, 1.0], 10.0, false);
    let ambient_light = enigma::light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.10, false);

    app_state.add_light(light1, enigma::light::LightType::Point);
    //app_state.add_light(light2, enigma::light::LightType::Point);
    //app_state.add_light(light3, enigma::light::LightType::Point);
    //app_state.add_light(ambient_light, enigma::light::LightType::Ambient);

    // add a camera
    let camera = Camera::new(Some([0.0, 1.0, 1.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // add events
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::A),
        Arc::new(rotate_left),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::D),
        Arc::new(rotate_right),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::W),
        Arc::new(rotate_up),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::S),
        Arc::new(rotate_down),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::E),
        Arc::new(roll_right),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Q),
        Arc::new(roll_left),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Space),
        Arc::new(spawn_object),
    );

    // add update
    //app_state.inject_update_function(Arc::new(hopping_objects));
    //app_state.inject_update_function(Arc::new(remove_object));
    app_state.inject_update_function(Arc::new(move_light));

    // add post processing
    //app_state.add_post_process(Box::new(GrayScale::new(&event_loop.display.clone())));
    //app_state.add_post_process(Box::new(Bloom::new(&event_loop.display.clone(), 0.9, 15)));
    //app_state.add_post_process(Box::new(Edge::new(&event_loop.display.clone(), 0.8, [1.0, 0.0, 0.0])));

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}
