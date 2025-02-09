use std::sync::Arc;
use egui::ScrollArea;
use enigma_3d::object::Object;
use enigma_3d::camera::Camera;
use enigma_3d::{AppState, EventLoop, resources, example_resources, shader, material, object, texture};
use enigma_3d::material::Material;

fn enigma_ui_function(ctx: &egui::Context, app_state: &mut AppState) {
    egui::Window::new("Enigma - Chessboard Example")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.label("Enigma 3D Renderer - Chessboard");
            ui.label("This Example tries so showcase mass loading of assets");
            ui.label("and effective use of cached textures");
        });

    egui::Window::new("Camera")
        .default_width(200.0)
        .default_height(200.0)
        .show(ctx, |ui| {
            let pos = app_state.camera.unwrap().transform.get_position();
            let rot = app_state.camera.unwrap().transform.get_rotation();
            let position = format!("Position: {}, {}, {}", pos.x, pos.y, pos.z);
            let rotation = format!("Rotation: {}, {}, {}", rot.x, rot.y, rot.z);
            ui.label(position);
            ui.label(rotation);
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

fn initialize_board(app_state: &mut AppState, event_loop: &EventLoop) {
    // setup materials
    let mut board_material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    board_material.set_name("mat_board");
    board_material.set_texture_from_resource(example_resources::chess_board_albedo(), enigma_3d::material::TextureType::Albedo);
    board_material.set_texture_from_resource(example_resources::chess_board_normal(), enigma_3d::material::TextureType::Normal);
    board_material.set_texture_from_resource(example_resources::chess_board_metallic(), enigma_3d::material::TextureType::Metallic);
    board_material.set_texture_from_resource(example_resources::chess_board_roughness(), enigma_3d::material::TextureType::Roughness);

    let mut figures_white_material = Material::lit_pbr(event_loop.get_display_clone(), false);
    figures_white_material.set_name("mat_figures_white");
    figures_white_material.set_texture_from_resource(example_resources::chess_figures_white_albedo(), enigma_3d::material::TextureType::Albedo);
    figures_white_material.set_texture_from_resource(example_resources::chess_figures_normal(), enigma_3d::material::TextureType::Normal);
    figures_white_material.set_texture_from_resource(example_resources::chess_figures_metallic(), enigma_3d::material::TextureType::Metallic);
    figures_white_material.set_texture_from_resource(example_resources::chess_figures_white_roughness(), enigma_3d::material::TextureType::Roughness);


    let mut figures_black_material = Material::lit_pbr(event_loop.get_display_clone(), false);
    figures_black_material.set_name("mat_figures_black");
    figures_black_material.set_texture_from_resource(example_resources::chess_figures_black_albedo(), enigma_3d::material::TextureType::Albedo);
    figures_black_material.set_texture_from_resource(example_resources::chess_figures_normal(), enigma_3d::material::TextureType::Normal);
    figures_black_material.set_texture_from_resource(example_resources::chess_figures_metallic(), enigma_3d::material::TextureType::Metallic);
    figures_black_material.set_texture_from_resource(example_resources::chess_figures_black_roughness(), enigma_3d::material::TextureType::Roughness);

    // setup models
    let mut obj_board = Object::load_from_gltf_resource(example_resources::chess_board_gltf());
    obj_board.set_name("obj_board".to_string());
    obj_board.add_material(board_material.uuid);
    obj_board.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_board.transform.set_position([0.0, -1.5, -6.0]);

    let mut obj_bishop_white_1 = Object::load_from_gltf_resource(example_resources::chess_bishop_gltf());
    obj_bishop_white_1.set_name("obj_bishop_white_1".to_string());
    obj_bishop_white_1.add_material(figures_white_material.uuid);
    obj_bishop_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_bishop_white_1.transform.set_position([-1.45, -1.15, -2.4]);
    obj_bishop_white_1.transform.set_rotation([0.0, -45.0, 0.0]);
    let mut obj_bishop_white_2 = obj_bishop_white_1.clone();
    obj_bishop_white_2.set_name("obj_bishop_white_2".to_string());
    obj_bishop_white_2.transform.set_position([1.45, -1.15, -2.4]);
    obj_bishop_white_2.transform.set_rotation([0.0, 45.0, 0.0]);
    let mut obj_king_white = Object::load_from_gltf_resource(example_resources::chess_king_gltf());
    obj_king_white.set_name("obj_king_white".to_string());
    obj_king_white.add_material(figures_white_material.uuid);
    obj_king_white.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_king_white.transform.set_position([0.5, -1.15, -2.4]);
    let mut obj_queen_white = Object::load_from_gltf_resource(example_resources::chess_queen_gltf());
    obj_queen_white.set_name("obj_queen_white".to_string());
    obj_queen_white.add_material(figures_white_material.uuid);
    obj_queen_white.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_queen_white.transform.set_position([-0.5, -1.15, -2.4]);
    let mut obj_knight_white_1 = Object::load_from_gltf_resource(example_resources::chess_knight_gltf());
    obj_knight_white_1.set_name("obj_knight_white_1".to_string());
    obj_knight_white_1.add_material(figures_white_material.uuid);
    obj_knight_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_knight_white_1.transform.set_position([2.5, -1.15, -2.4]);
    obj_knight_white_1.transform.set_rotation([0.0, -90.0, 0.0]);
    let mut obj_knight_white_2 = obj_knight_white_1.clone();
    obj_knight_white_2.set_name("obj_knight_white_2".to_string());
    obj_knight_white_2.transform.set_position([-2.5, -1.15, -2.4]);
    let mut obj_rook_white_1 = Object::load_from_gltf_resource(example_resources::chess_rook_gltf());
    obj_rook_white_1.set_name("obj_rook_white_1".to_string());
    obj_rook_white_1.add_material(figures_white_material.uuid);
    obj_rook_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_rook_white_1.transform.set_position([3.5, -1.15, -2.4]);
    let mut obj_rook_white_2 = obj_rook_white_1.clone();
    obj_rook_white_2.set_name("obj_rook_white_2".to_string());
    obj_rook_white_2.transform.set_position([-3.5, -1.15, -2.4]);
    let mut obj_pawn_white_1 = Object::load_from_gltf_resource(example_resources::chess_pawn_gltf());
    obj_pawn_white_1.set_name("obj_pawn_white_1".to_string());
    obj_pawn_white_1.add_material(figures_white_material.uuid);
    obj_pawn_white_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_pawn_white_1.transform.set_position([-3.5, -1.15, -3.5]);
    let mut obj_pawn_white_2 = obj_pawn_white_1.clone();
    obj_pawn_white_2.transform.set_position([-2.5, -1.15, -3.5]);
    obj_pawn_white_2.set_name("obj_pawn_white_2".to_string());
    let mut obj_pawn_white_3 = obj_pawn_white_1.clone();
    obj_pawn_white_3.transform.set_position([-1.5, -1.15, -3.5]);
    obj_pawn_white_3.set_name("obj_pawn_white_3".to_string());
    let mut obj_pawn_white_4 = obj_pawn_white_1.clone();
    obj_pawn_white_4.transform.set_position([-0.5, -1.15, -3.5]);
    obj_pawn_white_4.set_name("obj_pawn_white_4".to_string());
    let mut obj_pawn_white_5 = obj_pawn_white_1.clone();
    obj_pawn_white_5.transform.set_position([0.5, -1.15, -3.5]);
    obj_pawn_white_5.set_name("obj_pawn_white_5".to_string());
    let mut obj_pawn_white_6 = obj_pawn_white_1.clone();
    obj_pawn_white_6.transform.set_position([1.5, -1.15, -3.5]);
    obj_pawn_white_6.set_name("obj_pawn_white_6".to_string());
    let mut obj_pawn_white_7 = obj_pawn_white_1.clone();
    obj_pawn_white_7.transform.set_position([2.5, -1.15, -3.5]);
    obj_pawn_white_7.set_name("obj_pawn_white_7".to_string());
    let mut obj_pawn_white_8 = obj_pawn_white_1.clone();
    obj_pawn_white_8.transform.set_position([3.5, -1.15, -3.5]);
    obj_pawn_white_8.set_name("obj_pawn_white_8".to_string());

    let mut obj_bishop_black_1 = Object::load_from_gltf_resource(example_resources::chess_bishop_gltf());
    obj_bishop_black_1.set_name("obj_bishop_black_1".to_string());
    obj_bishop_black_1.add_material(figures_black_material.uuid);
    obj_bishop_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_bishop_black_1.transform.set_position([-1.45, -1.15, -9.4]);
    obj_bishop_black_1.transform.set_rotation([0.0, -45.0, 0.0]);
    let mut obj_bishop_black_2 = obj_bishop_black_1.clone();
    obj_bishop_black_2.set_name("obj_bishop_black_2".to_string());
    obj_bishop_black_2.transform.set_position([1.45, -1.15, -9.4]);
    obj_bishop_black_2.transform.set_rotation([0.0, 45.0, 0.0]);
    let mut obj_king_black = Object::load_from_gltf_resource(example_resources::chess_king_gltf());
    obj_king_black.set_name("obj_king_black".to_string());
    obj_king_black.add_material(figures_black_material.uuid);
    obj_king_black.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_king_black.transform.set_position([-0.5, -1.15, -9.4]);
    let mut obj_queen_black = Object::load_from_gltf_resource(example_resources::chess_queen_gltf());
    obj_queen_black.set_name("obj_queen_black".to_string());
    obj_queen_black.add_material(figures_black_material.uuid);
    obj_queen_black.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_queen_black.transform.set_position([0.5, -1.15, -9.4]);
    let mut obj_knight_black_1 = Object::load_from_gltf_resource(example_resources::chess_knight_gltf());
    obj_knight_black_1.set_name("obj_knight_black_1".to_string());
    obj_knight_black_1.add_material(figures_black_material.uuid);
    obj_knight_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_knight_black_1.transform.set_position([2.5, -1.15, -9.4]);
    obj_knight_black_1.transform.set_rotation([0.0, 90.0, 0.0]);
    let mut obj_knight_black_2 = obj_knight_black_1.clone();
    obj_knight_black_2.set_name("obj_knight_black_2".to_string());
    obj_knight_black_2.transform.set_position([-2.5, -1.15, -9.4]);
    let mut obj_rook_black_1 = Object::load_from_gltf_resource(example_resources::chess_rook_gltf());
    obj_rook_black_1.set_name("obj_rook_black_1".to_string());
    obj_rook_black_1.add_material(figures_black_material.uuid);
    obj_rook_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_rook_black_1.transform.set_position([3.5, -1.15, -9.4]);
    let mut obj_rook_black_2 = obj_rook_black_1.clone();
    obj_rook_black_2.set_name("obj_rook_black_2".to_string());
    obj_rook_black_2.transform.set_position([-3.5, -1.15, -9.4]);
    let mut obj_pawn_black_1 = Object::load_from_gltf_resource(example_resources::chess_pawn_gltf());
    obj_pawn_black_1.set_name("obj_pawn_black_1".to_string());
    obj_pawn_black_1.add_material(figures_black_material.uuid);
    obj_pawn_black_1.get_shapes_mut()[0].set_material_from_object_list(0);
    obj_pawn_black_1.transform.set_position([-3.5, -1.15, -8.5]);
    let mut obj_pawn_black_2 = obj_pawn_black_1.clone();
    obj_pawn_black_2.transform.set_position([-2.5, -1.15, -8.5]);
    obj_pawn_black_2.set_name("obj_pawn_black_2".to_string());
    let mut obj_pawn_black_3 = obj_pawn_black_1.clone();
    obj_pawn_black_3.transform.set_position([-1.5, -1.15, -8.5]);
    obj_pawn_black_3.set_name("obj_pawn_black_3".to_string());
    let mut obj_pawn_black_4 = obj_pawn_black_1.clone();
    obj_pawn_black_4.transform.set_position([-0.5, -1.15, -8.5]);
    obj_pawn_black_4.set_name("obj_pawn_black_4".to_string());
    let mut obj_pawn_black_5 = obj_pawn_black_1.clone();
    obj_pawn_black_5.transform.set_position([0.5, -1.15, -8.5]);
    obj_pawn_black_5.set_name("obj_pawn_black_5".to_string());
    let mut obj_pawn_black_6 = obj_pawn_black_1.clone();
    obj_pawn_black_6.transform.set_position([1.5, -1.15, -8.5]);
    obj_pawn_black_6.set_name("obj_pawn_black_6".to_string());
    let mut obj_pawn_black_7 = obj_pawn_black_1.clone();
    obj_pawn_black_7.transform.set_position([2.5, -1.15, -8.5]);
    obj_pawn_black_7.set_name("obj_pawn_black_7".to_string());
    let mut obj_pawn_black_8 = obj_pawn_black_1.clone();
    obj_pawn_black_8.transform.set_position([3.5, -1.15, -8.5]);
    obj_pawn_black_8.set_name("obj_pawn_black_8".to_string());

    // adding objects to app_state
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

    // adding materials to appstate
    app_state.add_material(board_material);
    app_state.add_material(figures_white_material);
    app_state.add_material(figures_black_material);
}

fn initialize_landscape(app_state: &mut AppState, event_loop: &EventLoop) {
    // ground_material with geometry grass shader
    let mut ground_material = Material::default(shader::Shader::from_strings(resources::vertex_shader(), resources::fragment_shader(), Some(resources::geometry_grass_shader())), &event_loop.display);

    //ground_material without geometry shader - way faster, since geometry shader are a very slow feature
    //let mut ground_material = Material::lit_pbr(event_loop.get_display_clone(), false);

    let mut tree_material_opaque = Material::default(shader::Shader::from_strings(resources::vertex_wind_shader(), resources::fragment_shader(), None), &event_loop.display);
    let mut tree_material_transparent = Material::default(shader::Shader::from_strings(resources::vertex_wind_shader(), resources::fragment_shader(), None), &event_loop.display);
    tree_material_transparent.set_transparency(true);
    tree_material_opaque.set_name("mat_tree_opaque");
    tree_material_transparent.set_name("mat_tree_transparent");
    ground_material.set_name("mat_terrain");

    let mut tex_ground_albedo = texture::Texture::from_resource(event_loop.get_display_reference(), example_resources::terrain_albedo());
    let mut tex_ground_normal = texture::Texture::from_resource(event_loop.get_display_reference(), example_resources::terrain_albedo());
    let mut tex_ground_metallic = texture::Texture::from_resource(event_loop.get_display_reference(), example_resources::terrain_albedo());
    let mut tex_ground_roughness = texture::Texture::from_resource(event_loop.get_display_reference(), example_resources::terrain_albedo());
    tex_ground_albedo.set_tileable(true);
    tex_ground_normal.set_tileable(true);
    tex_ground_metallic.set_tileable(true);
    tex_ground_roughness.set_tileable(true);
    ground_material.set_texture(tex_ground_albedo, enigma_3d::material::TextureType::Albedo);
    ground_material.set_texture(tex_ground_normal, enigma_3d::material::TextureType::Normal);
    ground_material.set_texture(tex_ground_metallic, enigma_3d::material::TextureType::Metallic);
    ground_material.set_texture(tex_ground_roughness, enigma_3d::material::TextureType::Roughness);

    tree_material_opaque.set_texture_from_resource(example_resources::tree_albedo(), material::TextureType::Albedo);
    tree_material_opaque.set_texture_from_resource(example_resources::tree_normal(), material::TextureType::Normal);
    tree_material_opaque.set_texture_from_resource(example_resources::tree_roughness(), material::TextureType::Roughness);

    tree_material_transparent.set_texture_from_resource(example_resources::tree_albedo(), material::TextureType::Albedo);
    tree_material_transparent.set_texture_from_resource(example_resources::tree_normal(), material::TextureType::Normal);
    tree_material_transparent.set_texture_from_resource(example_resources::tree_roughness(), material::TextureType::Roughness);

    let mut obj_terrain = object::Object::load_from_gltf_resource(example_resources::terrain());
    obj_terrain.set_name("obj_terrain".to_string());
    obj_terrain.set_collision(false);
    obj_terrain.add_material(ground_material.uuid);
    obj_terrain.transform.set_position([0.0, -1.5, -6.0]);
    obj_terrain.transform.set_rotation([0.0, -70.0, 0.0]);

    let mut obj_tree = object::Object::load_from_gltf_resource(example_resources::tree());
    obj_tree.set_name("obj_tree".to_string());
    obj_tree.set_collision(false);
    // we add both, the transparent and the opaque material uuid to the object
    obj_tree.add_material(tree_material_opaque.uuid);
    obj_tree.add_material(tree_material_transparent.uuid);
    obj_tree.transform.set_position([-5.0, -1.5, -10.0]);
    obj_tree.transform.set_rotation([0.0, 25.0, 0.0]);
    obj_tree.transform.set_scale([2.5, 2.5, 2.5]);

    app_state.add_material(ground_material);
    app_state.add_material(tree_material_opaque);
    app_state.add_material(tree_material_transparent);

    // we assign the materials to the individual shapes of the object. we need to know how many shapes an object has
    // in this case, the tree object has 2 shapes one for the bark and one for the leafs. the leafs are shape 0
    obj_tree.get_shapes_mut()[0].set_material_from_object_list(1);
    obj_tree.get_shapes_mut()[1].set_material_from_object_list(0);

    app_state.add_object(obj_terrain);
    app_state.add_object(obj_tree);
}

fn main() {
    // create an enigma eventloop and appstate
    //let event_loop = enigma_3d::EventLoop::new("Enigma 3D Renderer - Chessboard", 1080, 720);
    let event_loop = enigma_3d::EventLoop::new("Enigma 3D Renderer - Chessboard", 1920, 1080);
    let mut app_state = enigma_3d::AppState::new();
    // set the icon from the resources
    event_loop.set_icon_from_resource(resources::icon());
    // some default event setups like e.g. selection
    enigma_3d::init_default(&mut app_state);
    //initialize board and landscape
    initialize_board(&mut app_state, &event_loop);
    initialize_landscape(&mut app_state, &event_loop);
    // create a bunch of lights
    let light1 = enigma_3d::light::Light::new([1.0, 1.0, 5.0], [0.0, 1.0, 0.0], 100.0, None, false);
    let light2 = enigma_3d::light::Light::new([5.0, 1.0, 1.0], [1.0, 0.0, 0.0], 100.0, None, false);
    let light3 = enigma_3d::light::Light::new([-5.0, 1.0, 1.0], [0.0, 0.0, 1.0], 100.0, None, false);
    let light4 = enigma_3d::light::Light::new([-2.5, 0.0, -9.5], [1.0, 0.0, 0.0], 300.0, None, false);

    let light5 = enigma_3d::light::Light::new([1.0, 2.0, -8.0], [0.0, 1.0, 0.0], 100.0, None, false);
    let light6 = enigma_3d::light::Light::new([5.0, 2.0, -8.0], [1.0, 0.0, 0.0], 100.0, None, false);

    let ambient_light = enigma_3d::light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.2, None, false);

    // add the lights to the app state
    app_state.add_light(light1, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light2, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light3, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light4, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light5, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light6, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(ambient_light, enigma_3d::light::LightEmissionType::Ambient); // only one ambient light is supported atm

    // create and add a camera to the app state
    let camera = Camera::new(Some([-4.3, 3.0, 1.8]), Some([-9.3, -14.01, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // add post processing effects
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::bloom::Bloom::new(&event_loop.display.clone(), 0.999, 15)));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::depth_fog::DepthFog::new(&event_loop.display, 0.2, 60.0, 500.0, [0.3, 0.3, 0.75], 1.0)));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::vignette::Vignette::new(&event_loop.display.clone(), 0.2, 0.5, [0.0, 0.0, 0.0], 0.8)));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::lens_dirt::LensDirt::new(&event_loop.display, resources::lens_dirt_texture(), 2.0, [800.0, 800.0], 2.0)));

    //add one ui function to the app state. multiple ui functions can be added modular
    app_state.inject_gui(Arc::new(enigma_ui_function));

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}
