use enigma_3d::{AppState, EventLoop, example_resources, resources};
use enigma_3d::camera::Camera;
use enigma_3d::material::{Material, TextureType};
use enigma_3d::object::Object;
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::audio::AudioClip;
use enigma_3d::postprocessing;

fn initialize_scene(app_state: &mut AppState, event_loop: &EventLoop) {
    // terrain
    let mut terrain_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    terrain_mat.set_name("mat_terrain");
    terrain_mat.set_texture_from_resource(example_resources::terrain_albedo(), TextureType::Albedo);
    terrain_mat.set_texture_from_resource(example_resources::terrain_normal(), TextureType::Normal);
    terrain_mat.set_texture_from_resource(example_resources::terrain_roughness(), TextureType::Roughness);

    let mut terrain = Object::load_from_gltf_resource(example_resources::terrain(), None);
    terrain.set_name("terrain".to_string());
    terrain.set_collision(false);
    terrain.add_material(terrain_mat.uuid);
    terrain.get_shapes_mut()[0].set_material_from_object_list(0);
    terrain.transform.set_position([0.0, -2.0, -6.0]);
    terrain.transform.set_rotation([0.0, -70.0, 0.0]);
    terrain.transform.set_scale([1.5, 1.5, 1.5]);

    // tree
    let mut tree_mat_opaque = Material::lit_pbr(event_loop.get_display_clone(), false);
    tree_mat_opaque.set_name("mat_tree_opaque");
    tree_mat_opaque.set_texture_from_resource(example_resources::tree_albedo(), TextureType::Albedo);
    tree_mat_opaque.set_texture_from_resource(example_resources::tree_normal(), TextureType::Normal);
    tree_mat_opaque.set_texture_from_resource(example_resources::tree_roughness(), TextureType::Roughness);

    let mut tree_mat_transparent = Material::lit_pbr(event_loop.get_display_clone(), true);
    tree_mat_transparent.set_name("mat_tree_transparent");
    tree_mat_transparent.set_texture_from_resource(example_resources::tree_albedo(), TextureType::Albedo);
    tree_mat_transparent.set_texture_from_resource(example_resources::tree_normal(), TextureType::Normal);
    tree_mat_transparent.set_texture_from_resource(example_resources::tree_roughness(), TextureType::Roughness);

    let mut tree = Object::load_from_gltf_resource(example_resources::tree(), None);
    tree.set_name("tree".to_string());
    tree.set_collision(false);
    tree.add_material(tree_mat_opaque.uuid);
    tree.add_material(tree_mat_transparent.uuid);
    tree.get_shapes_mut()[0].set_material_from_object_list(1);
    tree.get_shapes_mut()[1].set_material_from_object_list(0);
    tree.transform.set_position([-6.0, -2.0, -14.0]);
    tree.transform.set_scale([2.0, 2.0, 2.0]);

    // knight (decoration)
    let mut knight_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    knight_mat.set_name("mat_knight");
    knight_mat.set_texture_from_resource(example_resources::skinned_knight_albedo(), TextureType::Albedo);
    knight_mat.set_texture_from_resource(example_resources::skinned_knight_normal(), TextureType::Normal);
    knight_mat.set_texture_from_resource(example_resources::skinned_knight_roughness(), TextureType::Roughness);

    let mut knight = Object::load_from_gltf_resource(example_resources::skinned_knight(), None);
    knight.set_name("knight".to_string());
    knight.set_collision(false);
    knight.add_material(knight_mat.uuid);
    knight.transform.set_position([3.0, -2.0, -8.0]);
    knight.transform.set_rotation([0.0, -30.0, 0.0]);
    knight.transform.set_scale([2.5, 2.5, 2.5]);
    knight.play_animation("Armature|mixamo.com|Layer0", true);

    // pawn material (reused for all spawned pawns)
    let mut pawn_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    pawn_mat.set_name("mat_pawn");
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_white_albedo(), TextureType::Albedo);
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_normal(), TextureType::Normal);
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_metallic(), TextureType::Metallic);
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_white_roughness(), TextureType::Roughness);

    // projectile material (unlit orange — glows with bloom)
    let mut proj_mat = Material::unlit(event_loop.get_display_clone(), false);
    proj_mat.set_name("mat_projectile");
    proj_mat.set_color([1.0, 0.55, 0.0]);
    proj_mat.set_transparency_strength(1.0);

    app_state.add_material(terrain_mat);
    app_state.add_material(tree_mat_opaque);
    app_state.add_material(tree_mat_transparent);
    app_state.add_material(knight_mat);
    app_state.add_material(pawn_mat);
    app_state.add_material(proj_mat);

    app_state.add_object(terrain);
    app_state.add_object(tree);
    app_state.add_object(knight);
}

fn main() {
    let event_loop = EventLoop::new("Pawn Rush", 1080, 720);
    let mut app_state = AppState::new();
    event_loop.set_icon_from_resource(resources::icon());

    initialize_scene(&mut app_state, &event_loop);

    // lights
    let sun = Light::new([0.0, 8.0, 0.0], [1.0, 0.95, 0.85], 800.0, None, false);
    let fill = Light::new([-5.0, 3.0, 5.0], [0.4, 0.55, 1.0], 200.0, None, false);
    let ambient = Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.25, None, false);
    app_state.add_light(sun, LightEmissionType::Source);
    app_state.add_light(fill, LightEmissionType::Source);
    app_state.add_light(ambient, LightEmissionType::Ambient);

    // fixed camera
    let camera = Camera::new(
        Some([0.0, 3.5, 8.0]),
        Some([-20.0, 0.0, 0.0]),
        Some(80.0),
        Some(16.0 / 9.0),
        Some(0.01),
        Some(512.0),
    );
    app_state.set_camera(camera);

    // audio
    let bgm = AudioClip::from_resource(example_resources::background_music(), "bgm");
    let hit_sound = AudioClip::from_resource(example_resources::click_sound(), "hit");
    app_state.add_audio(bgm);
    app_state.add_audio(hit_sound);
    app_state.play_audio_loop("bgm");

    // post-processing
    app_state.add_post_process(Box::new(
        postprocessing::bloom::Bloom::new(&event_loop.display.clone(), 0.97, 12)
    ));
    app_state.add_post_process(Box::new(
        postprocessing::depth_fog::DepthFog::new(&event_loop.display, 0.15, 80.0, 400.0, [0.1, 0.1, 0.2], 1.0)
    ));
    app_state.add_post_process(Box::new(
        postprocessing::vignette::Vignette::new(&event_loop.display.clone(), 0.3, 0.4, [0.0, 0.0, 0.0], 0.85)
    ));

    event_loop.run(app_state.convert_to_arc_mutex());
}
