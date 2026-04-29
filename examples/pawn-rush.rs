use std::sync::Arc;
use enigma_3d::{AppState, EventLoop, example_resources, resources, default_events};
use enigma_3d::event;
use enigma_3d::ui;
use enigma_3d::camera::Camera;
use enigma_3d::material::{Material, TextureType};
use enigma_3d::object::Object;
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::audio::AudioClip;
use enigma_3d::postprocessing;
use enigma_3d::collision_world::is_colliding;
use enigma_3d::geometry::BoundingBox;
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

const WAVE_INTERVAL: f32 = 8.0;
const PAWN_SPEED: f32 = 1.2;
const PROJECTILE_SPEED: f32 = 20.0;
const PROJECTILE_MAX_RANGE: f32 = 35.0;
const MAX_PROJECTILES: usize = 20;
const STARTING_LIVES: u32 = 3;
const PAWN_SPAWN_RADIUS: f32 = 18.0;
const PAWN_CAPTURE_RADIUS: f32 = 2.5;

#[derive(Clone, PartialEq)]
enum GamePhase { Menu, Playing, GameOver }

#[derive(Clone)]
struct GameState {
    phase: GamePhase,
    score: u32,
    lives: u32,
    wave: u32,
    wave_timer: f32,
    /// (uuid, velocity_xyz, distance_traveled, speed)
    projectile_ids: Vec<(Uuid, [f32; 3], f32, f32)>,
    pawn_ids: Vec<Uuid>,
    pawn_material_uuid: Uuid,
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
    let cam_pos = app_state.camera.map(|c| c.get_position()).unwrap_or([0.0, 3.5, 8.0]);
    let count = 3 + gs.wave;
    for i in 0..count {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / count as f32;
        let x = cam_pos[0] + PAWN_SPAWN_RADIUS * angle.cos();
        let z = cam_pos[2] + PAWN_SPAWN_RADIUS * angle.sin();
        let mut pawn = Object::load_from_gltf_resource(example_resources::chess_pawn_gltf(), None);
        pawn.set_name(format!("pawn_{i}"));
        pawn.set_collision(false);
        pawn.add_material(gs.pawn_material_uuid);
        pawn.get_shapes_mut()[0].set_material_from_object_list(0);
        pawn.transform.set_position([x, -1.15, z]);
        let uuid = pawn.get_unique_id();
        gs.pawn_ids.push(uuid);
        app_state.add_object(pawn);
    }
}

fn reset_game(app_state: &mut AppState, gs: &mut GameState) {
    let to_remove: Vec<Uuid> = gs.pawn_ids.iter()
        .chain(gs.projectile_ids.iter().map(|(id, _, _, _)| id))
        .copied()
        .collect();
    app_state.objects.retain(|o| !to_remove.contains(&o.get_unique_id()));

    gs.reset();
    spawn_wave(app_state, gs);
}

fn game_update(app_state: &mut AppState) {
    let mut gs = match app_state.get_state_data_value::<GameState>("game_state") {
        Some(g) => g.clone(),
        None => return,
    };

    if gs.phase != GamePhase::Playing {
        return;
    }

    let dt = app_state.delta_time;
    let wave_speed = 1.0 + (gs.wave as f32 - 1.0) * 0.2;
    let cam_pos = app_state.camera.map(|c| c.get_position()).unwrap_or([0.0, 3.5, 8.0]);

    // wave timer
    gs.wave_timer += dt;
    if gs.wave_timer >= WAVE_INTERVAL {
        gs.wave_timer = 0.0;
        gs.wave += 1;
        spawn_wave(app_state, &mut gs);
    }

    // move pawns toward camera (XZ plane only)
    for uuid in &gs.pawn_ids {
        if let Some(obj) = app_state.get_object_by_uuid_mut(*uuid) {
            let pos = obj.transform.get_position();
            let dx = cam_pos[0] - pos.x;
            let dz = cam_pos[2] - pos.z;
            let len = (dx * dx + dz * dz).sqrt();
            if len > 0.01 {
                let step = PAWN_SPEED * wave_speed * dt / len;
                obj.transform.move_dir_array([dx * step, 0.0, dz * step]);
            }
        }
    }

    // pawns that reach the camera — collect captured UUIDs first, then remove
    let escaped: Vec<Uuid> = gs.pawn_ids.iter()
        .copied()
        .filter(|uuid| {
            app_state.get_object_by_uuid(uuid)
                .map(|o| {
                    let pos = o.transform.get_position();
                    let dx = pos.x - cam_pos[0];
                    let dz = pos.z - cam_pos[2];
                    (dx * dx + dz * dz).sqrt() < PAWN_CAPTURE_RADIUS
                })
                .unwrap_or(false)
        })
        .collect();

    gs.lives = gs.lives.saturating_sub(escaped.len() as u32);
    app_state.objects.retain(|o| !escaped.contains(&o.get_unique_id()));
    gs.pawn_ids.retain(|id| !escaped.contains(id));

    if gs.lives == 0 {
        gs.phase = GamePhase::GameOver;
        app_state.set_state_data_value("game_state", Box::new(gs));
        return;
    }

    // move projectiles and track distance
    for (uuid, vel, dist, speed) in &mut gs.projectile_ids {
        if let Some(obj) = app_state.get_object_by_uuid_mut(*uuid) {
            obj.transform.move_dir_array([vel[0] * dt, vel[1] * dt, vel[2] * dt]);
            *dist += *speed * dt;
        }
    }

    // collision detection — collect bounding boxes, then check pairs
    let proj_bbs: Vec<(Uuid, BoundingBox)> = gs.projectile_ids.iter()
        .filter_map(|(uuid, _, _, _)| {
            app_state.get_object_by_uuid_mut(*uuid)
                .map(|o| (*uuid, o.get_bounding_box()))
        })
        .collect();

    let pawn_bbs: Vec<(Uuid, BoundingBox)> = gs.pawn_ids.iter()
        .filter_map(|uuid| {
            app_state.get_object_by_uuid_mut(*uuid)
                .map(|o| (*uuid, o.get_bounding_box()))
        })
        .collect();

    let mut kill_projectiles: Vec<Uuid> = Vec::new();
    let mut kill_pawns: Vec<Uuid> = Vec::new();

    'outer: for (proj_uuid, proj_bb) in &proj_bbs {
        for (pawn_uuid, pawn_bb) in &pawn_bbs {
            if kill_pawns.contains(pawn_uuid) {
                continue;
            }
            if is_colliding(proj_bb, pawn_bb) {
                kill_projectiles.push(*proj_uuid);
                kill_pawns.push(*pawn_uuid);
                gs.score += 1;
                app_state.play_audio_once("hit");
                continue 'outer;
            }
        }
    }

    app_state.objects.retain(|o| !kill_projectiles.contains(&o.get_unique_id()));
    gs.projectile_ids.retain(|(id, _, _, _)| !kill_projectiles.contains(id));
    app_state.objects.retain(|o| !kill_pawns.contains(&o.get_unique_id()));
    gs.pawn_ids.retain(|id| !kill_pawns.contains(id));

    // remove out-of-range projectiles
    let expired: Vec<Uuid> = gs.projectile_ids.iter()
        .filter(|(_, _, d, _)| *d > PROJECTILE_MAX_RANGE)
        .map(|(id, _, _, _)| *id)
        .collect();
    app_state.objects.retain(|o| !expired.contains(&o.get_unique_id()));
    gs.projectile_ids.retain(|(id, _, _, _)| !expired.contains(id));

    app_state.set_state_data_value("game_state", Box::new(gs));
}

fn pawn_rush_ui(ctx: &ui::Context, app_state: &mut AppState) {
    let mut gs = match app_state.get_state_data_value::<GameState>("game_state") {
        Some(g) => g.clone(),
        None => return,
    };

    match gs.phase {
        GamePhase::Menu => {
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

        GamePhase::Playing => {
            ui::Window::new("HUD")
                .anchor(ui::Align2::LEFT_TOP, [10.0, 10.0])
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.label(format!("Score: {}", gs.score));
                    ui.label(format!("Wave:  {}", gs.wave));
                    let filled = "♥".repeat(gs.lives.min(STARTING_LIVES) as usize);
                    let empty = "♡".repeat(STARTING_LIVES.saturating_sub(gs.lives) as usize);
                    ui.label(format!("{}{}", filled, empty));
                });
        }

        GamePhase::GameOver => {
            ui::Window::new("Game Over")
                .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.heading("GAME OVER");
                    ui.separator();
                    ui.label(format!("Final Score: {}", gs.score));
                    ui.label(format!("Waves survived: {}", gs.wave.saturating_sub(1)));
                    ui.separator();
                    if ui.button("  Play Again  ").clicked() {
                        reset_game(app_state, &mut gs);
                        gs.phase = GamePhase::Playing;
                    }
                    if ui.button("  Main Menu  ").clicked() {
                        reset_game(app_state, &mut gs);
                        gs.phase = GamePhase::Menu;
                    }
                });
            app_state.set_state_data_value("game_state", Box::new(gs));
        }
    }
}

fn fire_projectile(app_state: &mut AppState) {
    let mut gs = match app_state.get_state_data_value::<GameState>("game_state") {
        Some(g) => g.clone(),
        None => return,
    };

    if gs.phase != GamePhase::Playing {
        return;
    }

    if gs.projectile_ids.len() >= MAX_PROJECTILES {
        return;
    }

    let cam = match app_state.camera {
        Some(c) => c,
        None => return,
    };
    let cam_pos = cam.get_position();
    let (_, ray_dir) = app_state.get_mouse_state().get_world_position(&cam);
    let dir = [ray_dir.x, ray_dir.y, ray_dir.z];

    let velocity = [
        dir[0] * PROJECTILE_SPEED,
        dir[1] * PROJECTILE_SPEED,
        dir[2] * PROJECTILE_SPEED,
    ];

    let mut proj = Object::cube(0.08);
    proj.set_name("projectile".to_string());
    proj.set_collision(false);
    proj.add_material(gs.projectile_material_uuid);
    proj.get_shapes_mut()[0].set_material_from_object_list(0);
    proj.transform.set_position(cam_pos);

    let uuid = proj.get_unique_id();
    let speed = (velocity[0]*velocity[0] + velocity[1]*velocity[1] + velocity[2]*velocity[2]).sqrt();
    gs.projectile_ids.push((uuid, velocity, 0.0, speed));
    app_state.add_object(proj);
    app_state.play_audio_once("hit");

    app_state.set_state_data_value("game_state", Box::new(gs));
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

    app_state.add_state_data("camera_move_speed", Box::new(8.0f32));
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::W),
        Arc::new(default_events::camera_fly_forward),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::S),
        Arc::new(default_events::camera_fly_backward),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::A),
        Arc::new(default_events::camera_fly_left),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::D),
        Arc::new(default_events::camera_fly_right),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::MousePress(event::MouseButton::Left),
        Arc::new(fire_projectile),
        None,
    );
    app_state.inject_update_function(Arc::new(game_update));
    app_state.inject_gui(Arc::new(pawn_rush_ui));

    event_loop.run(app_state.convert_to_arc_mutex());
}
