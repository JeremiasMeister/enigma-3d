use std::sync::Arc;
use enigma_3d::{AppState, EventLoop, example_resources, resources};
use enigma_3d::ui;
use enigma_3d::camera::Camera;
use enigma_3d::material::{Material, TextureType};
use enigma_3d::object::Object;
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::audio::AudioClip;
use enigma_3d::postprocessing;
use uuid::Uuid;

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

#[allow(dead_code)]
const WAVE_INTERVAL: f32 = 8.0;
#[allow(dead_code)]
const PAWN_SPEED: f32 = 1.2;
#[allow(dead_code)]
const PROJECTILE_SPEED: f32 = 20.0;
#[allow(dead_code)]
const PROJECTILE_MAX_RANGE: f32 = 35.0;
const STARTING_LIVES: u32 = 3;
#[allow(dead_code)]
const PAWN_DEATH_Z: f32 = 5.0;
const PAWN_SPAWN_Z: f32 = -14.0;

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
enum GamePhase { Menu, Playing, GameOver }

#[derive(Clone)]
struct GameState {
    phase: GamePhase,
    score: u32,
    lives: u32,
    wave: u32,
    wave_timer: f32,
    /// (uuid, velocity_xyz, distance_traveled)
    projectile_ids: Vec<(Uuid, [f32; 3], f32)>,
    pawn_ids: Vec<Uuid>,
    pawn_material_uuid: Uuid,
    #[allow(dead_code)]
    projectile_material_uuid: Uuid,
}

impl GameState {
    fn new(pawn_mat: Uuid, proj_mat: Uuid) -> Self {
        Self {
            phase: GamePhase::Menu,
            score: 0,
            lives: STARTING_LIVES,
            wave: 1,
            wave_timer: 0.0,
            projectile_ids: Vec::new(),
            pawn_ids: Vec::new(),
            pawn_material_uuid: pawn_mat,
            projectile_material_uuid: proj_mat,
        }
    }

    fn reset(&mut self) {
        self.score = 0;
        self.lives = STARTING_LIVES;
        self.wave = 1;
        self.wave_timer = 0.0;
        self.projectile_ids.clear();
        self.pawn_ids.clear();
    }
}

fn find_material_uuid(app_state: &AppState, name: &str) -> Uuid {
    app_state.materials.iter()
        .find(|m| m.name == name)
        .expect("material not found")
        .uuid
}

fn spawn_wave(app_state: &mut AppState, gs: &mut GameState) {
    let count = 3 + gs.wave;
    for i in 0..count {
        let x = -3.5 + (i as f32 % 8.0) * 1.0;
        let mut pawn = Object::load_from_gltf_resource(example_resources::chess_pawn_gltf(), None);
        pawn.set_name(format!("pawn_{i}"));
        pawn.set_collision(false);
        pawn.add_material(gs.pawn_material_uuid);
        pawn.get_shapes_mut()[0].set_material_from_object_list(0);
        pawn.transform.set_position([x, -1.15, PAWN_SPAWN_Z]);
        let uuid = pawn.get_unique_id();
        gs.pawn_ids.push(uuid);
        app_state.add_object(pawn);
    }
}

fn reset_game(app_state: &mut AppState, gs: &mut GameState) {
    let to_remove: Vec<Uuid> = gs.pawn_ids.iter()
        .chain(gs.projectile_ids.iter().map(|(id, _, _)| id))
        .copied()
        .collect();
    app_state.objects.retain(|o| !to_remove.contains(&o.get_unique_id()));

    gs.reset();
    spawn_wave(app_state, gs);
}

fn pawn_rush_ui(ctx: &ui::Context, app_state: &mut AppState) {
    let mut gs = match app_state.get_state_data_value::<GameState>("game_state") {
        Some(g) => g.clone(),
        None => return,
    };

    if gs.phase == GamePhase::Menu {
        ui::Window::new("Pawn Rush")
            .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("PAWN RUSH");
                ui.separator();
                ui.label("Chess pawns are marching toward you.");
                ui.label("Left-click to fire. Stop them before they reach you.");
                ui.label("You have ♥♥♥ lives.");
                ui.separator();
                if ui.button("  Start Game  ").clicked() {
                    reset_game(app_state, &mut gs);
                    gs.phase = GamePhase::Playing;
                }
            });
        app_state.set_state_data_value("game_state", Box::new(gs));
    }
}

fn main() {
    let event_loop = EventLoop::new("Pawn Rush", 1080, 720);
    let mut app_state = AppState::new();
    event_loop.set_icon_from_resource(resources::icon());

    initialize_scene(&mut app_state, &event_loop);

    let pawn_mat_uuid = find_material_uuid(&app_state, "mat_pawn");
    let proj_mat_uuid = find_material_uuid(&app_state, "mat_projectile");
    let gs = GameState::new(pawn_mat_uuid, proj_mat_uuid);
    app_state.add_state_data("game_state", Box::new(gs));

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

    app_state.inject_gui(Arc::new(pawn_rush_ui));

    event_loop.run(app_state.convert_to_arc_mutex());
}
