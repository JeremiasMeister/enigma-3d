use std::sync::Arc;
use enigma_3d::object::Object;
use enigma_3d::camera::Camera;
use enigma_3d::{AppState, event, resources};
use rand::Rng;
use enigma_3d::event::EventModifiers;

fn rotate_left(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        object.transform.rotate([0.0, -5.0, 0.0]);
    }
}

fn rotate_right(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        object.transform.rotate([0.0, 5.0, 0.0]);
    }
}

fn rotate_up(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        object.transform.rotate([-5.0, 0.0, 0.0]);
    }
}

fn rotate_down(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        object.transform.rotate([5.0, 0.0, 0.0]);
    }
}

fn roll_left(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        object.transform.rotate([0.0, 0.0, 5.0]);
    }
}

fn roll_right(app_state: &mut AppState) {
    for object in app_state.get_selected_objects_mut() {
        object.transform.rotate([0.0, 0.0, -5.0]);
    }
}

fn hopping_objects(app_state: &mut AppState) {
    for object in app_state.objects.iter_mut() {
        let rand_scale = rand::thread_rng().gen_range(0.0..0.015);
        object.transform.move_dir_array([0.0, (app_state.time * 20.0).sin() * rand_scale, 0.0])
    }
}

fn spawn_object(app_state: &mut AppState) {
    match &app_state.display {
        Some(_) => {
            let material_name = if rand::random() {
                "transparent_mat"
            } else {
                "opaque_mat"
            };
            let material = app_state.get_material_by_name(material_name).expect("we explicitly added the material when starting the application");

            let mut object = Object::load_from_gltf_resource(resources::suzanne());
            object.name = format!("Suzanne_{}", rand::thread_rng().gen_range(0..1000));
            object.add_material(material.uuid);
            let random_x = rand::thread_rng().gen_range(-4.0..4.0);
            let random_z = rand::thread_rng().gen_range(-4.0..-1.0);

            object.transform.set_position([random_x, 0.0, random_z]);
            object.transform.set_scale([0.3, 0.3, 0.3]);

            app_state.add_object(object);
        }
        None => {
            println!("No display found, could not spawn object");
        }
    }
}

fn enigma_ui_function(ctx: &egui::Context, app_state: &mut AppState) {
    egui::Window::new("Enigma")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.label("Enigma 3D Renderer");
            ui.label("Press A, D, W, S, E, Q to rotate the selected object");
            ui.label("Press Space to spawn a new object");
            ui.label("Press Ctrl + S to save the current state");
            ui.label("Press Ctrl + O to load the saved state");
        });

    egui::Window::new("Scene")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.label("Scene Objects");
            for object in app_state.objects.iter() {
                if ui.button(object.name.clone()).clicked() {
                    let uuid = object.get_unique_id();
                    if !app_state.object_selection.contains(&uuid) {
                        app_state.object_selection.push(uuid);
                    } else {
                        app_state.object_selection.remove(app_state.object_selection.iter().position(|x| *x == uuid).unwrap());
                    }
                }
            }
        });
    egui::Window::new("Transform Edit")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            if app_state.get_selected_objects_mut().len() > 0 {
                ui.label("Transform Edit");
                ui.label("Position");
                ui.add(egui::Slider::new(&mut app_state.get_selected_objects_mut()[0].transform.position[0], -10.0..=10.0).text("X"));
                ui.add(egui::Slider::new(&mut app_state.get_selected_objects_mut()[0].transform.position[1], -10.0..=10.0).text("Y"));
                ui.add(egui::Slider::new(&mut app_state.get_selected_objects_mut()[0].transform.position[2], -10.0..=10.0).text("Z"));
                ui.label("Rotation");

                let mut rotation = app_state.get_selected_objects_mut()[0].transform.get_rotation();
                ui.add(egui::Slider::new(&mut rotation.x, -180.0..=180.0).text("X"));
                ui.add(egui::Slider::new(&mut rotation.y, -180.0..=180.0).text("Y"));
                ui.add(egui::Slider::new(&mut rotation.z, -180.0..=180.0).text("Z"));
                app_state.get_selected_objects_mut()[0].transform.set_rotation(rotation.into());

                ui.label("Scale");
                ui.add(egui::Slider::new(&mut app_state.get_selected_objects_mut()[0].transform.scale[0], 0.0..=10.0).text("X"));
                ui.add(egui::Slider::new(&mut app_state.get_selected_objects_mut()[0].transform.scale[1], 0.0..=10.0).text("Y"));
                ui.add(egui::Slider::new(&mut app_state.get_selected_objects_mut()[0].transform.scale[2], 0.0..=10.0).text("Z"));
            } else {
                ui.label("No object selected");
            }
        });
}

pub fn print_data(app_state: &mut AppState) {
    if app_state.time % 2.0 < 0.01 {
        let intdata = app_state.get_state_data_value::<i32>("intdata");
        let stringdata = app_state.get_state_data_value::<String>("stringdata");
        let booldata = app_state.get_state_data_value::<bool>("booldata");

        println!("Data: ");
        if let Some(intdata) = intdata {
            println!("intdata: {}", intdata);
        }
        if let Some(stringdata) = stringdata {
            println!("stringdata: {}", stringdata);
        }
        if let Some(booldata) = booldata {
            println!("booldata: {}", booldata);
        }
    }
}

fn save_app_state(app_state: &mut AppState) {
    let serialize_app_state = app_state.to_serializer();
    let serialized = serde_json::to_string_pretty(&serialize_app_state).unwrap();
    std::fs::write("../app_state.json", serialized).unwrap();
}

fn load_app_state(app_state: &mut AppState) {
    let serialized = std::fs::read_to_string("../app_state.json").unwrap();
    match serde_json::from_str(&serialized) {
        Ok(deserialized) => {
            let display = app_state.display.clone().unwrap();
            app_state.inject_serializer(deserialized, display, false);
        }
        Err(e) => {
            println!("Could not load app state: {}", e);
        }
    }
}


fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma_3d::EventLoop::new("Enigma 3D Renderer Window", 1080, 720);
    let mut app_state = enigma_3d::AppState::new();

    // set the icon from the resources
    event_loop.set_icon_from_resource(resources::icon());

    // some default event setups like e.g. selection
    enigma_3d::init_default(&mut app_state);

    // create a material and assign the UV checker texture from resources
    let mut material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    material.set_texture_from_resource(resources::uv_checker(), enigma_3d::material::TextureType::Albedo);
    material.set_name("opaque_mat");

    let mut transparent_material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), true);
    transparent_material.set_transparency_strength(0.2);
    transparent_material.set_texture_from_resource(resources::uv_checker(), enigma_3d::material::TextureType::Albedo);
    transparent_material.set_name("transparent_mat");


    // create an object, and load the Suzanne model from resources
    let mut object = Object::load_from_gltf_resource(resources::suzanne());

    // set the material to the suzan object to the first shape (submesh) slot
    object.add_material(material.uuid);
    object.get_shapes_mut()[0].set_material_from_object_list(0);

    // set the name and position of the object
    object.name = "Suzanne".to_string();
    object.transform.set_position([0.0, 0.0, -2.0]);

    // adding the object to the app state
    app_state.add_object(object);

    //also add materials to appstate
    app_state.add_material(material);
    app_state.add_material(transparent_material);

    // create a bunch of lights
    let light1 = enigma_3d::light::Light::new([1.0, 1.0, 5.0], [0.0, 1.0, 0.0], 100.0, Some([1.0,0.0,0.0]), false);
    let light2 = enigma_3d::light::Light::new([5.0, 1.0, 1.0], [1.0, 0.0, 0.0], 100.0, None, false);
    let light3 = enigma_3d::light::Light::new([-5.0, 1.0, 1.0], [0.0, 0.0, 1.0], 100.0, None, false);
    let ambient_light = enigma_3d::light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.1, None, false);

    // add the lights to the app state
    app_state.add_light(light1, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light2, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light3, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(ambient_light, enigma_3d::light::LightEmissionType::Ambient); // only one ambient light is supported atm

    // create and add a camera to the app state
    let camera = Camera::new(Some([0.0, 1.0, 1.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // add events
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::A),
        Arc::new(rotate_left),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::D),
        Arc::new(rotate_right),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::W),
        Arc::new(rotate_up),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::S),
        Arc::new(rotate_down),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::E),
        Arc::new(roll_right),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Q),
        Arc::new(roll_left),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Space),
        Arc::new(spawn_object),
        None,
    );

    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::S),
        Arc::new(save_app_state),
        Some(EventModifiers::new(true, false, false)),
    );

    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::O),
        Arc::new(load_app_state),
        Some(EventModifiers::new(true, false, false)),
    );

    // add update functions
    app_state.inject_update_function(Arc::new(hopping_objects));
    app_state.inject_update_function(Arc::new(print_data));

    // add post processing effects
    //app_state.add_post_process(Box::new(enigma::postprocessing::grayscale::GrayScale::new(&event_loop.display.clone())));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::bloom::Bloom::new(&event_loop.display.clone(), 0.9, 15)));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::edge::Edge::new(&event_loop.display.clone(), 0.8, [1.0, 0.0, 0.0])));

    //add one ui function to the app state. multiple ui functions can be added modularly
    app_state.inject_gui(Arc::new(enigma_ui_function));


    // add some arbitrary state data. This can be used to store any kind of data in the app state
    // game globals, or other data that needs to be shared between different parts of the application
    app_state.add_state_data( "intdata", Box::new(10i32));
    app_state.add_state_data( "stringdata", Box::new("Hello World".to_string() as String));
    app_state.add_state_data( "booldata", Box::new(true as bool));

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}
