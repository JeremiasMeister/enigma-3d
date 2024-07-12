use std::sync::Arc;
use egui::ScrollArea;
use enigma_3d::object::Object;
use enigma_3d::camera::Camera;
use enigma_3d::{AppState, event, EventLoop, resources};
use enigma_3d::event::EventModifiers;

fn camera_fly_forward(app_state: &mut AppState) {
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.forward() * -0.15;
        camera.transform.move_dir_vector(direction);
    }
}

fn camera_fly_backward(app_state: &mut AppState) {
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.forward() * 0.15;  // Note: positive here
        camera.transform.move_dir_vector(direction);
    }
}

fn camera_fly_left(app_state: &mut AppState) {
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.left() * 0.15;
        camera.transform.move_dir_vector(direction);
    }
}

fn camera_fly_right(app_state: &mut AppState) {
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.left() * -0.15;  // Note: negative here
        camera.transform.move_dir_vector(direction);
    }
}

fn camera_rotate(app_state: &mut AppState) {
    let mouse_delta = app_state.get_mouse_state().get_delta();
    if let Some(camera) = app_state.get_camera_mut() {
        // Convert delta to radians and apply a sensitivity factor
        let sensitivity = 0.01; // Adjust this value to change rotation speed
        let (delta_yaw, delta_pitch) = (
            mouse_delta.0 as f32 * sensitivity,
            mouse_delta.1 as f32 * sensitivity
        );

        // Update camera rotation
        let mut rotation = camera.transform.rotation;

        // Yaw rotation (around Y-axis)
        rotation.y -= delta_yaw;

        // Pitch rotation (around X-axis)
        rotation.x -= delta_pitch;

        // Clamp pitch to prevent camera flipping
        rotation.x = rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        // Apply the new rotation
        camera.transform.rotation = rotation;
    }
}


fn enigma_ui_function(ctx: &egui::Context, app_state: &mut AppState) {
    egui::Window::new("Enigma - Chessboard Example")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.label("Enigma 3D Renderer - Chessboard");
            ui.label("This Example tries so showcase mass loading of assets");
            ui.label("and effective use of cached textures");
        });

    egui::Window::new("Scene")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.label("Scene Objects");

            ScrollArea::vertical().show(ui, |ui| {
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
        });
    egui::Window::new("Transform Edit")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            if app_state.get_selected_objects_mut().len() > 0 {
                ui.label("Selected Object: ");
                ui.label(app_state.get_selected_objects_mut()[0].get_name());
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

fn initialize_board(app_state: &mut AppState, event_loop: &EventLoop){
    // setup materials
    let mut board_material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    board_material.name = Some("mat_board".to_string());
    board_material.set_texture_from_resource(resources::chess_board_albedo(), enigma_3d::material::TextureType::Albedo);
    board_material.set_texture_from_resource(resources::chess_board_normal(), enigma_3d::material::TextureType::Normal);
    board_material.set_texture_from_resource(resources::chess_board_metallic(), enigma_3d::material::TextureType::Metallic);
    board_material.set_texture_from_resource(resources::chess_board_roughness(), enigma_3d::material::TextureType::Roughness);

    let mut figures_white_material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    figures_white_material.name = Some("mat_figures_white".to_string());
    figures_white_material.set_texture_from_resource(resources::chess_figures_white_albedo(), enigma_3d::material::TextureType::Albedo);
    figures_white_material.set_texture_from_resource(resources::chess_figures_normal(), enigma_3d::material::TextureType::Normal);
    figures_white_material.set_texture_from_resource(resources::chess_figures_metallic(), enigma_3d::material::TextureType::Metallic);
    figures_white_material.set_texture_from_resource(resources::chess_figures_white_roughness(), enigma_3d::material::TextureType::Roughness);


    let mut figures_black_material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    figures_black_material.name = Some("mat_figures_black".to_string());
    figures_black_material.set_texture_from_resource(resources::chess_figures_black_albedo(), enigma_3d::material::TextureType::Albedo);
    figures_black_material.set_texture_from_resource(resources::chess_figures_normal(), enigma_3d::material::TextureType::Normal);
    figures_black_material.set_texture_from_resource(resources::chess_figures_metallic(), enigma_3d::material::TextureType::Metallic);
    figures_black_material.set_texture_from_resource(resources::chess_figures_black_roughness(), enigma_3d::material::TextureType::Roughness);

    // setup models
    let mut obj_board = Object::load_from_gltf_resource(resources::chess_board_gltf());
    obj_board.set_name("obj_board".to_string());
    obj_board.add_material(board_material);
    obj_board.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_board.transform.set_position([0.0, -1.5, -6.0]);

    let mut obj_bishop_white_1 = Object::load_from_gltf_resource(resources::chess_bishop_gltf());
    obj_bishop_white_1.set_name("obj_bishop_white_1".to_string());
    obj_bishop_white_1.add_material(figures_white_material.clone());
    obj_bishop_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_bishop_white_1.transform.set_position([-1.45, -1.15, -2.4]);
    obj_bishop_white_1.transform.set_rotation([0.0,-45.0, 0.0]);
    let mut obj_bishop_white_2 = obj_bishop_white_1.clone();
    obj_bishop_white_2.set_name("obj_bishop_white_2".to_string());
    obj_bishop_white_2.transform.set_position([1.45, -1.15, -2.4]);
    obj_bishop_white_2.transform.set_rotation([0.0,45.0, 0.0]);
    let mut obj_king_white = Object::load_from_gltf_resource(resources::chess_king_gltf());
    obj_king_white.set_name("obj_king_white".to_string());
    obj_king_white.add_material(figures_white_material.clone());
    obj_king_white.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_king_white.transform.set_position([0.5, -1.15, -2.4]);
    let mut obj_queen_white = Object::load_from_gltf_resource(resources::chess_queen_gltf());
    obj_queen_white.set_name("obj_queen_white".to_string());
    obj_queen_white.add_material(figures_white_material.clone());
    obj_queen_white.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_queen_white.transform.set_position([-0.5, -1.15, -2.4]);
    let mut obj_knight_white_1 = Object::load_from_gltf_resource(resources::chess_knight_gltf());
    obj_knight_white_1.set_name("obj_knight_white_1".to_string());
    obj_knight_white_1.add_material(figures_white_material.clone());
    obj_knight_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_knight_white_1.transform.set_position([2.5, -1.15, -2.4]);
    obj_knight_white_1.transform.set_rotation([0.0,-90.0, 0.0]);
    let mut obj_knight_white_2 = obj_knight_white_1.clone();
    obj_knight_white_2.set_name("obj_knight_white_2".to_string());
    obj_knight_white_2.transform.set_position([-2.5, -1.15, -2.4]);
    let mut obj_rook_white_1 = Object::load_from_gltf_resource(resources::chess_rook_gltf());
    obj_rook_white_1.set_name("obj_rook_white_1".to_string());
    obj_rook_white_1.add_material(figures_white_material.clone());
    obj_rook_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_rook_white_1.transform.set_position([3.5, -1.15, -2.4]);
    let mut obj_rook_white_2 = obj_rook_white_1.clone();
    obj_rook_white_2.set_name("obj_rook_white_2".to_string());
    obj_rook_white_2.transform.set_position([-3.5, -1.15, -2.4]);
    let mut obj_pawn_white_1 = Object::load_from_gltf_resource(resources::chess_pawn_gltf());
    obj_pawn_white_1.set_name("obj_pawn_white_1".to_string());
    obj_pawn_white_1.add_material(figures_white_material.clone());
    obj_pawn_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_pawn_white_1.transform.set_position([-3.5, -1.15, -3.5]);
    let mut obj_pawn_white_2 = obj_pawn_white_1.clone();
    obj_pawn_white_2.transform.set_position([-2.5, -1.15, -3.5]);
    let mut obj_pawn_white_3 = obj_pawn_white_1.clone();
    obj_pawn_white_3.transform.set_position([-1.5, -1.15, -3.5]);
    let mut obj_pawn_white_4 = obj_pawn_white_1.clone();
    obj_pawn_white_4.transform.set_position([-0.5, -1.15, -3.5]);
    let mut obj_pawn_white_5 = obj_pawn_white_1.clone();
    obj_pawn_white_5.transform.set_position([0.5, -1.15, -3.5]);
    let mut obj_pawn_white_6 = obj_pawn_white_1.clone();
    obj_pawn_white_6.transform.set_position([1.5, -1.15, -3.5]);
    let mut obj_pawn_white_7 = obj_pawn_white_1.clone();
    obj_pawn_white_7.transform.set_position([2.5, -1.15, -3.5]);
    let mut obj_pawn_white_8 = obj_pawn_white_1.clone();
    obj_pawn_white_8.transform.set_position([3.5, -1.15, -3.5]);

    let mut obj_bishop_black_1 = Object::load_from_gltf_resource(resources::chess_bishop_gltf());
    obj_bishop_black_1.set_name("obj_bishop_black_1".to_string());
    obj_bishop_black_1.add_material(figures_black_material.clone());
    obj_bishop_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_bishop_black_1.transform.set_position([-1.45, -1.15, -9.4]);
    obj_bishop_black_1.transform.set_rotation([0.0,-45.0, 0.0]);
    let mut obj_bishop_black_2 = obj_bishop_black_1.clone();
    obj_bishop_black_2.set_name("obj_bishop_black_2".to_string());
    obj_bishop_black_2.transform.set_position([1.45, -1.15, -9.4]);
    obj_bishop_black_2.transform.set_rotation([0.0,45.0, 0.0]);
    let mut obj_king_black = Object::load_from_gltf_resource(resources::chess_king_gltf());
    obj_king_black.set_name("obj_king_black".to_string());
    obj_king_black.add_material(figures_black_material.clone());
    obj_king_black.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_king_black.transform.set_position([-0.5, -1.15, -9.4]);
    let mut obj_queen_black = Object::load_from_gltf_resource(resources::chess_queen_gltf());
    obj_queen_black.set_name("obj_queen_black".to_string());
    obj_queen_black.add_material(figures_black_material.clone());
    obj_queen_black.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_queen_black.transform.set_position([0.5, -1.15, -9.4]);
    let mut obj_knight_black_1 = Object::load_from_gltf_resource(resources::chess_knight_gltf());
    obj_knight_black_1.set_name("obj_knight_black_1".to_string());
    obj_knight_black_1.add_material(figures_black_material.clone());
    obj_knight_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_knight_black_1.transform.set_position([2.5, -1.15, -9.4]);
    obj_knight_black_1.transform.set_rotation([0.0, 90.0, 0.0]);
    let mut obj_knight_black_2 = obj_knight_black_1.clone();
    obj_knight_black_2.set_name("obj_knight_black_2".to_string());
    obj_knight_black_2.transform.set_position([-2.5, -1.15, -9.4]);
    let mut obj_rook_black_1 = Object::load_from_gltf_resource(resources::chess_rook_gltf());
    obj_rook_black_1.set_name("obj_rook_black_1".to_string());
    obj_rook_black_1.add_material(figures_black_material.clone());
    obj_rook_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_rook_black_1.transform.set_position([3.5, -1.15, -9.4]);
    let mut obj_rook_black_2 = obj_rook_black_1.clone();
    obj_rook_black_2.set_name("obj_rook_black_2".to_string());
    obj_rook_black_2.transform.set_position([-3.5, -1.15, -9.4]);
    let mut obj_pawn_black_1 = Object::load_from_gltf_resource(resources::chess_pawn_gltf());
    obj_pawn_black_1.set_name("obj_pawn_black_1".to_string());
    obj_pawn_black_1.add_material(figures_black_material.clone());
    obj_pawn_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_pawn_black_1.transform.set_position([-3.5, -1.15, -8.5]);
    let mut obj_pawn_black_2 = obj_pawn_black_1.clone();
    obj_pawn_black_2.transform.set_position([-2.5, -1.15, -8.5]);
    let mut obj_pawn_black_3 = obj_pawn_black_1.clone();
    obj_pawn_black_3.transform.set_position([-1.5, -1.15, -8.5]);
    let mut obj_pawn_black_4 = obj_pawn_black_1.clone();
    obj_pawn_black_4.transform.set_position([-0.5, -1.15, -8.5]);
    let mut obj_pawn_black_5 = obj_pawn_black_1.clone();
    obj_pawn_black_5.transform.set_position([0.5, -1.15, -8.5]);
    let mut obj_pawn_black_6 = obj_pawn_black_1.clone();
    obj_pawn_black_6.transform.set_position([1.5, -1.15, -8.5]);
    let mut obj_pawn_black_7 = obj_pawn_black_1.clone();
    obj_pawn_black_7.transform.set_position([2.5, -1.15, -8.5]);
    let mut obj_pawn_black_8 = obj_pawn_black_1.clone();
    obj_pawn_black_8.transform.set_position([3.5, -1.15, -8.5]);

    // adding to app_state
    app_state.add_object(obj_board);
    app_state.add_object(obj_king_white);
    app_state.add_object(obj_queen_white);
    app_state.add_object(obj_bishop_white_1);
    app_state.add_object(obj_bishop_white_2);
    app_state.add_object(obj_knight_white_1);
    app_state.add_object(obj_knight_white_2);
    app_state.add_object(obj_rook_white_1);
    app_state.add_object(obj_rook_white_2);
    app_state.add_object(obj_pawn_white_1);
    app_state.add_object(obj_pawn_white_2);
    app_state.add_object(obj_pawn_white_3);
    app_state.add_object(obj_pawn_white_4);
    app_state.add_object(obj_pawn_white_5);
    app_state.add_object(obj_pawn_white_6);
    app_state.add_object(obj_pawn_white_7);
    app_state.add_object(obj_pawn_white_8);
    app_state.add_object(obj_king_black);
    app_state.add_object(obj_queen_black);
    app_state.add_object(obj_bishop_black_1);
    app_state.add_object(obj_bishop_black_2);
    app_state.add_object(obj_knight_black_1);
    app_state.add_object(obj_knight_black_2);
    app_state.add_object(obj_rook_black_1);
    app_state.add_object(obj_rook_black_2);
    app_state.add_object(obj_pawn_black_1);
    app_state.add_object(obj_pawn_black_2);
    app_state.add_object(obj_pawn_black_3);
    app_state.add_object(obj_pawn_black_4);
    app_state.add_object(obj_pawn_black_5);
    app_state.add_object(obj_pawn_black_6);
    app_state.add_object(obj_pawn_black_7);
    app_state.add_object(obj_pawn_black_8);
}

fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma_3d::EventLoop::new("Enigma 3D Renderer - Chessboard", 1080, 720);
    let mut app_state = enigma_3d::AppState::new();
    // set the icon from the resources
    event_loop.set_icon_from_resource(resources::icon());
    // some default event setups like e.g. selection
    enigma_3d::init_default(&mut app_state);
    //initialize board
    initialize_board(&mut app_state, &event_loop);
    // create a bunch of lights
    let light1 = enigma_3d::light::Light::new([1.0, 1.0, 5.0], [0.0, 1.0, 0.0], 100.0, None, false);
    let light2 = enigma_3d::light::Light::new([5.0, 1.0, 1.0], [1.0, 0.0, 0.0], 100.0, None, false);
    let light3 = enigma_3d::light::Light::new([-5.0, 1.0, 1.0], [0.0, 0.0, 1.0], 100.0, None, false);

    let light4 = enigma_3d::light::Light::new([1.0, 2.0, -8.0], [0.0, 1.0, 0.0], 100.0, None, false);
    let light5 = enigma_3d::light::Light::new([5.0, 2.0, -8.0], [1.0, 0.0, 0.0], 100.0, None, false);
    let light6 = enigma_3d::light::Light::new([-5.0, 2.0, -8.0], [0.0, 0.0, 1.0], 100.0, None, false);

    let ambient_light = enigma_3d::light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.1, None, false);

    // add the lights to the app state
    app_state.add_light(light1, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light2, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light3, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light4, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light5, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light6, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(ambient_light, enigma_3d::light::LightEmissionType::Ambient); // only one ambient light is supported atm

    // create and add a camera to the app state
    let camera = Camera::new(Some([0.0, 1.0, 1.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    //event functions for moving the camera
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::W),
        Arc::new(camera_fly_forward),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::A),
        Arc::new(camera_fly_left),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::S),
        Arc::new(camera_fly_backward),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::D),
        Arc::new(camera_fly_right),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::MouseDown(winit::event::MouseButton::Right),
        Arc::new(camera_rotate),
        Some(EventModifiers::new(true, false, false)),
    );


    // add post processing effects
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::bloom::Bloom::new(&event_loop.display.clone(), 0.95, 5)));

    //add one ui function to the app state. multiple ui functions can be added modularly
    app_state.inject_gui(Arc::new(enigma_ui_function));

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}
