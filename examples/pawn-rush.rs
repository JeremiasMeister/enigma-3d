use std::sync::Arc;
use enigma_3d::{AppState, EventLoop, example_resources, resources};
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
use rand::Rng;

// ── Movement ─────────────────────────────────────────────────────────────────
// Custom ground-locked variants: flatten the camera's forward/left vectors to
// XZ so the player can never gain or lose altitude through WASD.

fn walk_forward(app_state: &mut AppState) {
    if app_state.get_state_data_value::<GameState>("game_state").map(|gs| gs.phase.clone()) != Some(GamePhase::Playing) { return; }
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(50.0);
    let dt = app_state.delta_time;
    if let Some(cam) = app_state.get_camera_mut() {
        let f = cam.transform.forward(); // points "behind" camera in this engine
        let len = (f.x * f.x + f.z * f.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([-f.x / len * speed * dt, 0.0, -f.z / len * speed * dt]);
        }
    }
}

fn walk_backward(app_state: &mut AppState) {
    if app_state.get_state_data_value::<GameState>("game_state").map(|gs| gs.phase.clone()) != Some(GamePhase::Playing) { return; }
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(50.0);
    let dt = app_state.delta_time;
    if let Some(cam) = app_state.get_camera_mut() {
        let f = cam.transform.forward();
        let len = (f.x * f.x + f.z * f.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([f.x / len * speed * dt, 0.0, f.z / len * speed * dt]);
        }
    }
}

fn walk_left(app_state: &mut AppState) {
    if app_state.get_state_data_value::<GameState>("game_state").map(|gs| gs.phase.clone()) != Some(GamePhase::Playing) { return; }
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(50.0);
    let dt = app_state.delta_time;
    if let Some(cam) = app_state.get_camera_mut() {
        let l = cam.transform.left();
        let len = (l.x * l.x + l.z * l.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([l.x / len * speed * dt, 0.0, l.z / len * speed * dt]);
        }
    }
}

fn walk_right(app_state: &mut AppState) {
    if app_state.get_state_data_value::<GameState>("game_state").map(|gs| gs.phase.clone()) != Some(GamePhase::Playing) { return; }
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(50.0);
    let dt = app_state.delta_time;
    if let Some(cam) = app_state.get_camera_mut() {
        let l = cam.transform.left();
        let len = (l.x * l.x + l.z * l.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([-l.x / len * speed * dt, 0.0, -l.z / len * speed * dt]);
        }
    }
}

// ── Component data structs ────────────────────────────────────────────────────

#[derive(Clone)]
struct PawnData {
    kind: PieceKind,
    speed_mult: f32,
    health: i32,
    wander_dir: [f32; 2],   // normalized XZ direction
    wander_timer: f32,       // seconds until next direction change
}

#[derive(Clone)]
struct ProjectileData {
    velocity: [f32; 3],
    distance: f32,
    speed: f32,
}

#[derive(Clone)]
struct AmmoPickupData {
    ammo: u32,
    bob_phase: f32,
}

// ── Scene setup ───────────────────────────────────────────────────────────────

fn initialize_scene(app_state: &mut AppState, event_loop: &EventLoop) {
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

    let mut pawn_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    pawn_mat.set_name("mat_pawn");
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_white_albedo(), TextureType::Albedo);
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_normal(), TextureType::Normal);
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_metallic(), TextureType::Metallic);
    pawn_mat.set_texture_from_resource(example_resources::chess_figures_white_roughness(), TextureType::Roughness);

    let mut proj_mat = Material::unlit(event_loop.get_display_clone(), false);
    proj_mat.set_name("mat_projectile");
    proj_mat.set_color([0.82, 0.84, 0.92]);

    let mut pickup_mat = Material::unlit(event_loop.get_display_clone(), false);
    pickup_mat.set_name("mat_pickup");
    pickup_mat.set_color([1.0, 0.75, 0.1]);
    let pickup_mat_uuid = pickup_mat.uuid;

    // weapon shares the same unlit material but with a slightly different hue
    let tree_opaque_uuid = tree_mat_opaque.uuid;
    let tree_transparent_uuid = tree_mat_transparent.uuid;

    app_state.add_material(terrain_mat);
    app_state.add_material(tree_mat_opaque);
    app_state.add_material(tree_mat_transparent);
    app_state.add_material(pawn_mat);
    app_state.add_material(proj_mat);
    app_state.add_material(pickup_mat);
    app_state.add_state_data("pickup_mat_uuid", Box::new(pickup_mat_uuid));

    app_state.add_object(terrain);

    let tree_positions: [([f32; 3], [f32; 3], f32); 18] = [
        // inner ring
        ([-6.0,  -2.0, -14.0], [0.0,   0.0, 0.0], 4.5),
        ([ 8.0,  -2.0, -10.0], [0.0,  40.0, 0.0], 4.2),
        ([-12.0, -2.0,  -4.0], [0.0, -20.0, 0.0], 4.8),
        ([ 5.0,  -2.0,  10.0], [0.0,  70.0, 0.0], 4.0),
        ([-4.0,  -2.0,  14.0], [0.0, 150.0, 0.0], 5.0),
        // mid ring
        ([-22.0, -2.0, -30.0], [0.0,  15.0, 0.0], 5.5),
        ([ 18.0, -2.0, -26.0], [0.0,  80.0, 0.0], 5.0),
        ([-30.0, -2.0,  -8.0], [0.0, -35.0, 0.0], 6.0),
        ([ 28.0, -2.0,   6.0], [0.0, 110.0, 0.0], 5.2),
        ([-18.0, -2.0,  28.0], [0.0, 200.0, 0.0], 5.8),
        ([ 10.0, -2.0,  32.0], [0.0, -60.0, 0.0], 5.4),
        // outer ring (near walls)
        ([-42.0, -2.0, -42.0], [0.0,  25.0, 0.0], 7.0),
        ([ 38.0, -2.0, -38.0], [0.0,  95.0, 0.0], 6.5),
        ([-44.0, -2.0,   2.0], [0.0, -10.0, 0.0], 7.5),
        ([ 42.0, -2.0,  18.0], [0.0, 130.0, 0.0], 6.8),
        ([-20.0, -2.0,  44.0], [0.0, 175.0, 0.0], 7.2),
        ([ 22.0, -2.0, -44.0], [0.0,  55.0, 0.0], 6.6),
        ([  0.0, -2.0,  46.0], [0.0, -80.0, 0.0], 7.0),
    ];
    for (pos, rot, scale) in &tree_positions {
        let mut t = Object::load_from_gltf_resource(example_resources::tree(), None);
        t.set_collision(false);
        t.add_material(tree_opaque_uuid);
        t.add_material(tree_transparent_uuid);
        t.get_shapes_mut()[0].set_material_from_object_list(1);
        t.get_shapes_mut()[1].set_material_from_object_list(0);
        t.transform.set_position(*pos);
        t.transform.set_rotation(*rot);
        t.transform.set_scale([*scale, *scale, *scale]);
        app_state.add_object(t);
    }

    // FPS weapon — gun model, positioned in camera-local space every frame.
    // Start it off-screen; game_update moves it into view during Playing.
    let mut gun_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    gun_mat.set_name("mat_gun");
    gun_mat.set_color([0.08, 0.08, 0.08]);
    gun_mat.set_roughness_strength(1.0);
    gun_mat.set_metallic_strength(0.0);
    let gun_mat_uuid = gun_mat.uuid;
    app_state.add_material(gun_mat);

    let mut weapon = Object::load_from_gltf_resource(example_resources::gun_gltf(), None);
    weapon.set_name("weapon".to_string());
    weapon.set_collision(false);
    weapon.add_material(gun_mat_uuid);
    weapon.get_shapes_mut()[0].set_material_from_object_list(0);
    weapon.transform.set_position([0.0, -1000.0, 0.0]);
    weapon.transform.set_scale([1.0, 1.0, 1.0]);
    let weapon_uuid = weapon.get_unique_id();
    app_state.add_object(weapon);
    app_state.add_state_data("weapon_uuid", Box::new(weapon_uuid));

    // Fireflies — 20 glowing spheres scattered across the full arena
    let mut firefly_mat = Material::unlit(event_loop.get_display_clone(), false);
    firefly_mat.set_name("mat_firefly");
    firefly_mat.set_color([0.55, 1.0, 0.35]);
    let firefly_mat_uuid = firefly_mat.uuid;
    app_state.add_material(firefly_mat);

    let spread = ARENA_HALF - WALL_THICK - 2.0;
    let mut rng = rand::thread_rng();
    let mut firefly_entries: Vec<FireflyEntry> = Vec::new();
    for i in 0..20usize {
        let bx = rng.gen_range(-spread..spread);
        let by = rng.gen_range(1.5f32..3.5f32);
        let bz = rng.gen_range(-spread..spread);

        let mut sphere = Object::sphere(0.12, 8, 12);
        sphere.set_name(format!("firefly_{i}"));
        sphere.set_collision(false);
        sphere.add_material(firefly_mat_uuid);
        sphere.get_shapes_mut()[0].set_material_from_object_list(0);
        sphere.transform.set_position([bx, by, bz]);
        let uuid = sphere.get_unique_id();
        app_state.add_object(sphere);

        let light_idx = app_state.light.len();
        app_state.add_light(
            Light::new([bx, by, bz], [0.5, 1.0, 0.3], 60.0, None, false),
            LightEmissionType::Source,
        );

        firefly_entries.push(FireflyEntry {
            uuid,
            light_idx,
            base_x: bx,
            base_y: by,
            base_z: bz,
            phase: i as f32 * 0.63,  // golden-ratio-ish stagger
        });
    }
    app_state.add_state_data("fireflies", Box::new(firefly_entries));

    // ── Arena walls ───────────────────────────────────────────────────────────
    let mut wall_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    wall_mat.set_name("mat_wall");
    wall_mat.set_color([0.55, 0.50, 0.42]);
    let wall_mat_uuid = wall_mat.uuid;
    app_state.add_material(wall_mat);

    // (position, scale) for N/S/E/W walls
    let wall_defs: [([f32; 3], [f32; 3]); 4] = [
        ([0.0, 0.5, -ARENA_HALF],  [ARENA_HALF * 2.0 + WALL_THICK, WALL_HEIGHT, WALL_THICK]),  // north
        ([0.0, 0.5,  ARENA_HALF],  [ARENA_HALF * 2.0 + WALL_THICK, WALL_HEIGHT, WALL_THICK]),  // south
        ([-ARENA_HALF, 0.5, 0.0],  [WALL_THICK, WALL_HEIGHT, ARENA_HALF * 2.0 + WALL_THICK]),  // west
        ([ ARENA_HALF, 0.5, 0.0],  [WALL_THICK, WALL_HEIGHT, ARENA_HALF * 2.0 + WALL_THICK]),  // east
    ];

    for (i, (pos, scale)) in wall_defs.iter().enumerate() {
        let mut wall = Object::cube(0.5);
        wall.set_name(format!("wall_{i}"));
        wall.set_collision(false);
        wall.add_material(wall_mat_uuid);
        wall.get_shapes_mut()[0].set_material_from_object_list(0);
        wall.transform.set_position(*pos);
        wall.transform.set_scale(*scale);
        app_state.add_object(wall);
    }
}

// ── Constants ─────────────────────────────────────────────────────────────────

const WAVE_INTERVAL: f32 = 8.0;
const PAWN_SPEED: f32 = 4.0;
const PROJECTILE_SPEED: f32 = 80.0;
const PROJECTILE_MAX_RANGE: f32 = 80.0;
const MAX_PROJECTILES: usize = 20;
const STARTING_LIVES: u32 = 3;
const MAG_SIZE: u32 = 10;
const STARTING_TOTAL_AMMO: u32 = 50;
const MAX_TOTAL_AMMO: u32 = 90;
const RELOAD_TIME: f32 = 1.8;
const PICKUP_AMMO: u32 = 5;
const PICKUP_COLLECT_RADIUS: f32 = 2.2;
const ARENA_HALF: f32 = 55.0;   // 110×110 play area
const WALL_HEIGHT: f32 = 5.0;
const WALL_THICK: f32 = 0.6;
const PAWN_DETECTION_RADIUS: f32 = 25.0;
const PAWN_CAPTURE_RADIUS: f32 = 2.5;
const AIM_DOT_THRESHOLD: f32 = 0.97;
const CAMERA_HEIGHT: f32 = 3.5;
const BOB_FREQ: f32 = 9.0;   // radians per second (step cycle)
const BOB_AMP: f32 = 0.055;  // world units up/down

// ── Game state ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum GamePhase { Menu, Playing, GameOver }

#[derive(Clone, Copy, PartialEq)]
enum PieceKind { Pawn, Bishop, Rook, Queen, King, Knight }

impl PieceKind {
    fn speed_mult(self) -> f32 {
        match self {
            PieceKind::Pawn   => 1.0,
            PieceKind::Bishop => 0.9,
            PieceKind::Rook   => 0.85,
            PieceKind::Queen  => 1.35,
            PieceKind::King   => 0.65,
            PieceKind::Knight => 1.2,
        }
    }
    fn scale(self) -> f32 {
        match self {
            PieceKind::Pawn   => 3.0,
            PieceKind::Bishop => 3.2,
            PieceKind::Rook   => 3.2,
            PieceKind::Queen  => 3.1,
            PieceKind::King   => 3.1,
            PieceKind::Knight => 2.8,
        }
    }
    fn name(self) -> &'static str {
        match self {
            PieceKind::Pawn   => "pawn",
            PieceKind::Bishop => "bishop",
            PieceKind::Rook   => "rook",
            PieceKind::Queen  => "queen",
            PieceKind::King   => "king",
            PieceKind::Knight => "knight",
        }
    }
    fn load(self) -> &'static [u8] {
        match self {
            PieceKind::Pawn   => example_resources::chess_pawn_gltf(),
            PieceKind::Bishop => example_resources::chess_bishop_gltf(),
            PieceKind::Rook   => example_resources::chess_rook_gltf(),
            PieceKind::Queen  => example_resources::chess_queen_gltf(),
            PieceKind::King   => example_resources::chess_king_gltf(),
            PieceKind::Knight => example_resources::chess_knight_gltf(),
        }
    }
    fn max_health(self) -> i32 {
        match self {
            PieceKind::Pawn   => 1,
            PieceKind::Bishop => 2,
            PieceKind::Knight => 2,
            PieceKind::Rook   => 3,
            PieceKind::Queen  => 5,
            PieceKind::King   => 7,
        }
    }
    fn drop_chance(self) -> f32 {
        match self {
            PieceKind::Pawn   => 0.20,
            PieceKind::Bishop => 0.35,
            PieceKind::Knight => 0.35,
            PieceKind::Rook   => 0.50,
            PieceKind::Queen  => 0.65,
            PieceKind::King   => 0.80,
        }
    }
}

#[derive(Clone)]
struct FireflyEntry {
    uuid: Uuid,
    light_idx: usize,
    base_x: f32,
    base_y: f32,
    base_z: f32,
    phase: f32,
}

#[derive(Clone)]
struct GameState {
    phase: GamePhase,
    score: u32,
    lives: u32,
    wave: u32,
    wave_timer: f32,
    projectile_ids: Vec<Uuid>,
    pawn_ids: Vec<Uuid>,
    pickup_ids: Vec<Uuid>,
    pawn_material_uuid: Uuid,
    projectile_material_uuid: Uuid,
    volume: f32,
    aimed_pawn: Option<Uuid>,
    bob_phase: f32,
    bob_active: f32,
    recoil: f32,
    ammo: u32,
    total_ammo: u32,
    reloading: bool,
    reload_timer: f32,
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
            pickup_ids: Vec::new(),
            pawn_material_uuid: pawn_mat,
            projectile_material_uuid: proj_mat,
            volume: 1.0,
            aimed_pawn: None,
            bob_phase: 0.0,
            bob_active: 0.0,
            recoil: 0.0,
            ammo: MAG_SIZE,
            total_ammo: STARTING_TOTAL_AMMO,
            reloading: false,
            reload_timer: 0.0,
        }
    }

    fn reset(&mut self) {
        self.score = 0;
        self.lives = STARTING_LIVES;
        self.wave = 1;
        self.wave_timer = 0.0;
        self.projectile_ids.clear();
        self.pawn_ids.clear();
        self.pickup_ids.clear();
        self.aimed_pawn = None;
        self.bob_phase = 0.0;
        self.bob_active = 0.0;
        self.recoil = 0.0;
        self.ammo = MAG_SIZE;
        self.total_ammo = STARTING_TOTAL_AMMO;
        self.reloading = false;
        self.reload_timer = 0.0;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_material_uuid(app_state: &AppState, name: &str) -> Uuid {
    app_state.materials.iter()
        .find(|m| m.name == name)
        .expect("material not found")
        .uuid
}

fn spawn_wave(app_state: &mut AppState, gs: &mut GameState) {
    let count = 4 + gs.wave * 2;
    let mut rng = rand::thread_rng();
    let all_kinds = [
        PieceKind::Pawn, PieceKind::Bishop, PieceKind::Rook,
        PieceKind::Queen, PieceKind::King, PieceKind::Knight,
    ];

    let spawn_inner = ARENA_HALF - WALL_THICK - 0.5;
    for i in 0..count {
        let (x, z) = match rng.gen_range(0u8..4) {
            0 => (rng.gen_range(-spawn_inner..spawn_inner), -spawn_inner),  // north edge
            1 => (rng.gen_range(-spawn_inner..spawn_inner),  spawn_inner),  // south edge
            2 => (-spawn_inner, rng.gen_range(-spawn_inner..spawn_inner)),  // west edge
            _ => ( spawn_inner, rng.gen_range(-spawn_inner..spawn_inner)),  // east edge
        };

        let kind = all_kinds[rng.gen_range(0..all_kinds.len())];
        let sc = kind.scale();

        let mut piece = Object::load_from_gltf_resource(kind.load(), None);
        piece.set_name(format!("{}_{i}", kind.name()));
        piece.set_collision(false);
        piece.add_material(gs.pawn_material_uuid);
        piece.get_shapes_mut()[0].set_material_from_object_list(0);
        piece.transform.set_position([x, -1.15, z]);
        piece.transform.set_scale([sc, sc, sc]);
        let uuid = piece.get_unique_id();
        let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
        piece.set_component(PawnData {
            kind,
            speed_mult: kind.speed_mult(),
            health: kind.max_health(),
            wander_dir: [angle.cos(), angle.sin()],
            wander_timer: rng.gen_range(0.5f32..1.5f32),
        });
        gs.pawn_ids.push(uuid);
        app_state.add_object(piece);
    }
}

fn start_reload(gs: &mut GameState) {
    if gs.reloading || gs.total_ammo == 0 || gs.ammo == MAG_SIZE { return; }
    gs.reloading = true;
    gs.reload_timer = RELOAD_TIME;
}

fn reload_weapon(app_state: &mut AppState) {
    if let Some(gs) = app_state.get_state_data_value::<GameState>("game_state") {
        let mut gs = gs.clone();
        if gs.phase == GamePhase::Playing { start_reload(&mut gs); }
        app_state.set_state_data_value("game_state", Box::new(gs));
    }
}

fn reset_game(app_state: &mut AppState, gs: &mut GameState) {
    let to_remove: Vec<Uuid> = gs.pawn_ids.iter().copied()
        .chain(gs.projectile_ids.iter().copied())
        .chain(gs.pickup_ids.iter().copied())
        .collect();
    app_state.objects.retain(|o| !to_remove.contains(&o.get_unique_id()));
    gs.reset();
    spawn_wave(app_state, gs);
}

// ── Per-frame update ──────────────────────────────────────────────────────────

fn game_update(app_state: &mut AppState) {
    let mut gs = match app_state.get_state_data_value::<GameState>("game_state") {
        Some(g) => g.clone(),
        None => return,
    };
    let weapon_uuid = app_state.get_state_data_value::<Uuid>("weapon_uuid").copied();

    let dt = app_state.delta_time;
    let cam_pos = app_state.camera.as_ref().map(|c| c.get_position()).unwrap_or([0.0, CAMERA_HEIGHT, 0.0]);
    let cam_fwd = app_state.camera.as_ref().map(|c| c.calculate_direction_vector()).unwrap_or([0.0, 0.0, -1.0]);

    // ── Head bob ──────────────────────────────────────────────────────────────
    let is_moving = app_state.held_keys.iter().any(|k| matches!(
        k,
        event::VirtualKeyCode::W | event::VirtualKeyCode::S |
        event::VirtualKeyCode::A | event::VirtualKeyCode::D
    ));
    if is_moving {
        gs.bob_active = (gs.bob_active + dt * 8.0).min(1.0);
        gs.bob_phase += dt * BOB_FREQ;
    } else {
        gs.bob_active = (gs.bob_active - dt * 6.0).max(0.0);
    }
    let bob_y = gs.bob_phase.sin() * BOB_AMP * gs.bob_active;

    // Clamp camera to ground height + bob — prevents any vertical drift
    if let Some(cam) = app_state.get_camera_mut() {
        let p = cam.transform.get_position();
        cam.transform.set_position([p.x, CAMERA_HEIGHT + bob_y, p.z]);
    }

    // Clamp player inside arena walls
    if gs.phase == GamePhase::Playing {
        if let Some(cam) = app_state.get_camera_mut() {
            let p = cam.transform.get_position();
            let inner = ARENA_HALF - WALL_THICK / 2.0;
            cam.transform.set_position([
                p.x.clamp(-inner, inner),
                p.y,
                p.z.clamp(-inner, inner),
            ]);
        }
    }

    // lock cursor during gameplay, release in menu/gameover
    app_state.cursor_locked = gs.phase == GamePhase::Playing;

    // ── Firefly animation (always, not gated on Playing) ──────────────────────
    let t = app_state.time;
    if let Some(fireflies) = app_state.get_state_data_value::<Vec<FireflyEntry>>("fireflies") {
        let fireflies = fireflies.clone();
        for ff in &fireflies {
            let hover_y = ff.base_y + (t * 1.3 + ff.phase).sin() * 0.35;
            let drift_x = ff.base_x + (t * 0.7 + ff.phase * 1.7).cos() * 0.4;
            let drift_z = ff.base_z + (t * 0.5 + ff.phase * 2.1).sin() * 0.4;
            if let Some(obj) = app_state.get_object_by_uuid_mut(ff.uuid) {
                obj.transform.set_position([drift_x, hover_y, drift_z]);
            }
            if let Some(light) = app_state.light.get_mut(ff.light_idx) {
                light.position = [drift_x, hover_y, drift_z];
            }
        }
    }

    // ── FPS weapon ────────────────────────────────────────────────────────────
    // Position in camera-local space: right=0.20, down=0.13, forward=0.40.
    // Camera-local "down" = -camera_up = -cross(fwd, right), so the cube stays
    // locked to the lower-right corner of the view frustum regardless of pitch.
    if let Some(wid) = weapon_uuid {
        // Extract all camera data before taking a mutable borrow on app_state
        let weapon_data = app_state.camera.as_ref().map(|cam| {
            let cp = cam.get_position();
            let cf = cam.calculate_direction_vector();
            let rx = -cf[2];
            let right_z =  cf[0];
            let r_len = (rx * rx + right_z * right_z).sqrt();
            let (rx, right_z) = if r_len > 0.001 { (rx / r_len, right_z / r_len) } else { (-1.0, 0.0) };
            let cam_up = cam.transform.up();
            let yaw_rad   = cam.transform.rotation.y;
            let pitch_rad = cam.transform.rotation.x;
            (cp, cf, rx, right_z, cam_up, yaw_rad, pitch_rad)
        });

        // ── Reload timer ──────────────────────────────────────────────────────
        if gs.reloading {
            gs.reload_timer -= dt;
            if gs.reload_timer <= 0.0 {
                let needed = MAG_SIZE - gs.ammo;
                let refill = needed.min(gs.total_ammo);
                gs.ammo += refill;
                gs.total_ammo -= refill;
                gs.reloading = false;
                gs.reload_timer = 0.0;
            }
        }

        let mut muzzle_world: Option<[f32; 3]> = None;

        if let Some((cp, cf, rx, right_z, cam_up, yaw_rad, pitch_rad)) = weapon_data {
            const FWD: f32   = 0.28;
            const RIGHT: f32 = 0.20;
            const DOWN: f32  = 0.13;

            // Decay recoil
            gs.recoil = (gs.recoil - dt * 8.0).max(0.0);
            let fwd_actual = FWD - gs.recoil * 0.10;

            // Reload animation: smooth dip using sin arc (0→1→0 over reload duration)
            let reload_anim = if gs.reloading {
                (gs.reload_timer / RELOAD_TIME * std::f32::consts::PI).sin()
            } else {
                0.0
            };
            let reload_lower = reload_anim * 0.18;   // drop weapon

            if let Some(weapon) = app_state.get_object_by_uuid_mut(wid) {
                if gs.phase == GamePhase::Playing {
                    let wx = cp[0] + rx * RIGHT       - cam_up.x * (DOWN + reload_lower) + cf[0] * fwd_actual;
                    let wy = cp[1]                    - cam_up.y * (DOWN + reload_lower) + cf[1] * fwd_actual + bob_y * 0.5;
                    let wz = cp[2] + right_z * RIGHT  - cam_up.z * (DOWN + reload_lower) + cf[2] * fwd_actual;
                    weapon.transform.set_position([wx, wy, wz]);
                    weapon.transform.set_scale([0.22, 0.22, 0.22]);

                    // Zero-roll decomposition: point +X barrel along camera forward without barrel spin.
                    // Raw yaw/pitch avoids the asin branch discontinuity that caused rolling along barrel length.
                    let pitch_animated = pitch_rad
                        + gs.recoil * 18.0_f32.to_radians()
                        - reload_anim * 55.0_f32.to_radians();
                    let sy = yaw_rad.sin();
                    let cy = yaw_rad.cos();
                    let sp = pitch_animated.sin();
                    let cp = pitch_animated.cos();
                    let denom = ((sy * sy * cp * cp) + (sp * sp)).sqrt();
                    let gx = (cy * sp).atan2(-sy).to_degrees();
                    let gy = (cy * cp).atan2(denom).to_degrees();
                    let gz = sp.atan2(-sy * cp).to_degrees();
                    weapon.transform.set_rotation([gx, gy, gz]);

                    // Muzzle is ~0.30 units forward of the weapon center along view direction
                    muzzle_world = Some([
                        wx + cf[0] * 0.30,
                        wy + cf[1] * 0.30,
                        wz + cf[2] * 0.30,
                    ]);
                } else {
                    weapon.transform.set_position([0.0, -1000.0, 0.0]);
                }
            }
        }

        if let Some(mp) = muzzle_world {
            app_state.set_state_data_value("muzzle_pos", Box::new(mp));
        }
    }

    if gs.phase != GamePhase::Playing {
        app_state.set_state_data_value("game_state", Box::new(gs));
        return;
    }

    // ── Sun follows camera ────────────────────────────────────────────────────
    if let Some(sun) = app_state.light.first_mut() {
        sun.position = [cam_pos[0], cam_pos[1] + 25.0, cam_pos[2]];
    }

    // ── Wave timer ────────────────────────────────────────────────────────────
    gs.wave_timer += dt;
    let current_wave_interval = (WAVE_INTERVAL - (gs.wave as f32 - 1.0) * 0.4).max(3.0);
    if gs.wave_timer >= current_wave_interval {
        gs.wave_timer = 0.0;
        gs.wave += 1;
        spawn_wave(app_state, &mut gs);
    }

    // ── Aimed-at detection ────────────────────────────────────────────────────
    let mut new_aimed: Option<Uuid> = None;
    let mut best_dot = AIM_DOT_THRESHOLD;
    for uuid in &gs.pawn_ids {
        if let Some(obj) = app_state.get_object_by_uuid(uuid) {
            let pos = obj.transform.get_position();
            let to_x = pos.x - cam_pos[0];
            let to_y = pos.y - cam_pos[1];
            let to_z = pos.z - cam_pos[2];
            let len = (to_x*to_x + to_y*to_y + to_z*to_z).sqrt();
            if len < 0.01 { continue; }
            let dot = cam_fwd[0]*to_x/len + cam_fwd[1]*to_y/len + cam_fwd[2]*to_z/len;
            if dot > best_dot {
                best_dot = dot;
                new_aimed = Some(*uuid);
            }
        }
    }
    gs.aimed_pawn = new_aimed;

    // ── Move pawns ────────────────────────────────────────────────────────────
    let wave_speed = 1.0 + (gs.wave as f32 - 1.0) * 0.2;
    let mut rng = rand::thread_rng();
    for uuid in &gs.pawn_ids {
        if let Some(obj) = app_state.get_object_by_uuid_mut(*uuid) {
            let pos = obj.transform.get_position();
            let dx = cam_pos[0] - pos.x;
            let dz = cam_pos[2] - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            // Read current wander state (copy out so borrow ends)
            let (speed_mult, mut wdir, mut wtimer) = obj
                .get_component::<PawnData>()
                .map(|d| (d.speed_mult, d.wander_dir, d.wander_timer))
                .unwrap_or((1.0, [1.0, 0.0], 1.0));

            let (move_x, move_z) = if dist < PAWN_DETECTION_RADIUS {
                // Chase: move directly toward player
                if dist > 0.01 { (dx / dist, dz / dist) } else { (0.0, 0.0) }
            } else {
                // Wander: tick timer, bounce off walls, pick new dir when expired
                wtimer -= dt;

                let wall_inner = ARENA_HALF - WALL_THICK / 2.0 - 0.5;
                if pos.x < -wall_inner { wdir[0] =  wdir[0].abs(); }
                if pos.x >  wall_inner { wdir[0] = -wdir[0].abs(); }
                if pos.z < -wall_inner { wdir[1] =  wdir[1].abs(); }
                if pos.z >  wall_inner { wdir[1] = -wdir[1].abs(); }

                if wtimer <= 0.0 {
                    let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
                    wdir = [angle.cos(), angle.sin()];
                    wtimer = rng.gen_range(0.8f32..2.0f32);
                }

                (wdir[0], wdir[1])
            };

            // Write back updated wander state
            if let Some(data) = obj.get_component_mut::<PawnData>() {
                data.wander_dir = wdir;
                data.wander_timer = wtimer;
            }

            // Face movement direction
            if move_x.abs() > 0.01 || move_z.abs() > 0.01 {
                let angle = move_x.atan2(move_z).to_degrees();
                obj.transform.set_rotation([0.0, angle + 90.0, 0.0]);
            }

            // Apply movement
            let step = PAWN_SPEED * wave_speed * speed_mult * dt;
            obj.transform.move_dir_array([move_x * step, 0.0, move_z * step]);
        }
    }

    // ── Escaped pawns (reach camera) ──────────────────────────────────────────
    let escaped: Vec<Uuid> = gs.pawn_ids.iter()
        .filter(|uuid| {
            app_state.get_object_by_uuid(*uuid)
                .map(|o| {
                    let pos = o.transform.get_position();
                    let dx = pos.x - cam_pos[0];
                    let dz = pos.z - cam_pos[2];
                    (dx * dx + dz * dz).sqrt() < PAWN_CAPTURE_RADIUS
                })
                .unwrap_or(false)
        })
        .copied()
        .collect();

    gs.lives = gs.lives.saturating_sub(escaped.len() as u32);
    app_state.objects.retain(|o| !escaped.contains(&o.get_unique_id()));
    gs.pawn_ids.retain(|id| !escaped.contains(id));

    if gs.lives == 0 {
        gs.phase = GamePhase::GameOver;
        app_state.set_state_data_value("game_state", Box::new(gs));
        return;
    }

    // ── Move projectiles ──────────────────────────────────────────────────────
    for uuid in &gs.projectile_ids {
        if let Some(obj) = app_state.get_object_by_uuid_mut(*uuid) {
            let (vel, spd) = obj.get_component::<ProjectileData>()
                .map(|d| (d.velocity, d.speed))
                .unwrap_or(([0.0; 3], 0.0));
            obj.transform.move_dir_array([vel[0] * dt, vel[1] * dt, vel[2] * dt]);
            if let Some(data) = obj.get_component_mut::<ProjectileData>() {
                data.distance += spd * dt;
            }
        }
    }

    // ── Collision detection ───────────────────────────────────────────────────
    let proj_bbs: Vec<(Uuid, BoundingBox)> = gs.projectile_ids.iter()
        .filter_map(|uuid| {
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

    // First pass: find which pawns are hit this frame (one projectile per pawn)
    let mut kill_projectiles: Vec<Uuid> = Vec::new();
    let mut hit_pawns: Vec<Uuid> = Vec::new();

    'outer: for (proj_uuid, proj_bb) in &proj_bbs {
        for (pawn_uuid, pawn_bb) in &pawn_bbs {
            if hit_pawns.contains(pawn_uuid) { continue; }
            if is_colliding(proj_bb, pawn_bb) {
                kill_projectiles.push(*proj_uuid);
                hit_pawns.push(*pawn_uuid);
                app_state.play_audio_once("break");
                continue 'outer;
            }
        }
    }

    // Second pass: decrement health, collect info for dead pawns
    let mut kill_pawns: Vec<Uuid> = Vec::new();
    let mut kill_pawn_info: Vec<([f32; 3], PieceKind)> = Vec::new();
    for pawn_uuid in &hit_pawns {
        if let Some(obj) = app_state.get_object_by_uuid_mut(*pawn_uuid) {
            let pos = obj.transform.get_position();
            let (dead, kind) = if let Some(data) = obj.get_component_mut::<PawnData>() {
                data.health -= 1;
                (data.health <= 0, data.kind)
            } else {
                (true, PieceKind::Pawn)
            };
            if dead {
                kill_pawn_info.push(([pos.x, 0.3, pos.z], kind));
                kill_pawns.push(*pawn_uuid);
                gs.score += 1;
            }
        }
    }

    app_state.objects.retain(|o| !kill_projectiles.contains(&o.get_unique_id()));
    gs.projectile_ids.retain(|id| !kill_projectiles.contains(id));
    app_state.objects.retain(|o| !kill_pawns.contains(&o.get_unique_id()));
    gs.pawn_ids.retain(|id| !kill_pawns.contains(id));

    // ── Spawn ammo pickups at killed enemy positions (with drop chance) ────────
    let pickup_mat_uuid = app_state.get_state_data_value::<Uuid>("pickup_mat_uuid").copied();
    if let Some(mat_uuid) = pickup_mat_uuid {
        let mut rng = rand::thread_rng();
        for (pos, kind) in kill_pawn_info {
            if rng.gen::<f32>() >= kind.drop_chance() { continue; }
            let mut pickup = Object::load_from_gltf_resource(example_resources::bullet_gltf(), None);
            pickup.set_name("ammo_pickup".to_string());
            pickup.set_collision(false);
            pickup.add_material(mat_uuid);
            pickup.get_shapes_mut()[0].set_material_from_object_list(0);
            pickup.transform.set_position(pos);
            pickup.transform.set_scale([0.3, 0.3, 0.3]);
            let phase = rng.gen_range(0.0f32..std::f32::consts::TAU);
            pickup.set_component(AmmoPickupData { ammo: PICKUP_AMMO, bob_phase: phase });
            let uuid = pickup.get_unique_id();
            gs.pickup_ids.push(uuid);
            app_state.add_object(pickup);
        }
    }

    // ── Animate pickups + check collection ────────────────────────────────────
    let t = app_state.time;
    let mut collected_pickups: Vec<Uuid> = Vec::new();
    for uuid in &gs.pickup_ids {
        if let Some(obj) = app_state.get_object_by_uuid_mut(*uuid) {
            let phase = obj.get_component::<AmmoPickupData>().map(|d| d.bob_phase).unwrap_or(0.0);
            let base_y = 0.3;
            let hover_y = base_y + (t * 2.5 + phase).sin() * 0.15;
            let pos = obj.transform.get_position();
            obj.transform.set_position([pos.x, hover_y, pos.z]);
            obj.transform.rotate([0.0, 200.0 * dt, 0.0]);

            let dx = pos.x - cam_pos[0];
            let dz = pos.z - cam_pos[2];
            if (dx * dx + dz * dz).sqrt() < PICKUP_COLLECT_RADIUS {
                collected_pickups.push(*uuid);
            }
        }
    }
    for uuid in &collected_pickups {
        if let Some(obj) = app_state.get_object_by_uuid(uuid) {
            let ammo = obj.get_component::<AmmoPickupData>().map(|d| d.ammo).unwrap_or(0);
            gs.total_ammo = (gs.total_ammo + ammo).min(MAX_TOTAL_AMMO);
            app_state.play_audio_once("pickup");
        }
    }
    app_state.objects.retain(|o| !collected_pickups.contains(&o.get_unique_id()));
    gs.pickup_ids.retain(|id| !collected_pickups.contains(id));

    // ── Expire out-of-range projectiles ───────────────────────────────────────
    let expired: Vec<Uuid> = gs.projectile_ids.iter()
        .filter(|uuid| {
            app_state.get_object_by_uuid(*uuid)
                .and_then(|o| o.get_component::<ProjectileData>())
                .map(|d| d.distance > PROJECTILE_MAX_RANGE)
                .unwrap_or(false)
        })
        .copied()
        .collect();
    app_state.objects.retain(|o| !expired.contains(&o.get_unique_id()));
    gs.projectile_ids.retain(|id| !expired.contains(id));

    app_state.set_state_data_value("game_state", Box::new(gs));
}

// ── UI ────────────────────────────────────────────────────────────────────────

fn chess_visuals() -> ui::Visuals {
    let mut v = ui::Visuals::dark();
    v.window_fill = ui::Color32::from_rgb(18, 14, 10);
    v.window_stroke = ui::Stroke::new(1.5, ui::Color32::from_rgb(180, 140, 60));
    v.override_text_color = Some(ui::Color32::from_rgb(220, 195, 130));
    v.widgets.inactive.bg_fill = ui::Color32::from_rgb(35, 27, 15);
    v.widgets.inactive.fg_stroke = ui::Stroke::new(1.0, ui::Color32::from_rgb(180, 140, 60));
    v.widgets.hovered.bg_fill = ui::Color32::from_rgb(80, 60, 20);
    v.widgets.hovered.fg_stroke = ui::Stroke::new(1.5, ui::Color32::from_rgb(240, 200, 80));
    v.widgets.active.bg_fill = ui::Color32::from_rgb(120, 90, 25);
    v.widgets.active.fg_stroke = ui::Stroke::new(2.0, ui::Color32::from_rgb(255, 220, 100));
    v.selection.bg_fill = ui::Color32::from_rgb(100, 75, 20);
    v
}

fn pawn_rush_ui(ctx: &ui::Context, app_state: &mut AppState) {
    ctx.set_visuals(chess_visuals());

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
                    ui.heading(ui::RichText::new("♟  PAWN RUSH  ♟")
                        .size(28.0)
                        .color(ui::Color32::from_rgb(240, 200, 80)));
                    ui.separator();
                    ui.label("Chess pieces are marching toward you.");
                    ui.label("Left-click to fire. Survive the waves.");
                    ui.label("WASD to move. Move mouse to look.");
                    ui.label("You have ♥♥♥ lives.");
                    ui.add_space(6.0);
                    ui.separator();
                    ui.label("Volume");
                    let prev_volume = gs.volume;
                    ui.add(ui::Slider::new(&mut gs.volume, 0.0..=1.0).show_value(false));
                    if (gs.volume - prev_volume).abs() > f32::EPSILON {
                        app_state.set_audio_volume("bgm", gs.volume);
                        app_state.set_audio_volume("hit", gs.volume);
                        app_state.set_audio_volume("shot", gs.volume);
                        app_state.set_audio_volume("break", gs.volume);
                        app_state.set_audio_volume("pickup", gs.volume);
                    }
                    ui.separator();
                    if ui.button(ui::RichText::new("  ▶  Start Game  ").size(16.0)
                            .color(ui::Color32::from_rgb(255, 220, 80))).clicked() {
                        reset_game(app_state, &mut gs);
                        gs.phase = GamePhase::Playing;
                    }
                });
            app_state.set_state_data_value("game_state", Box::new(gs));
        }

        GamePhase::Playing => {
            let crosshair_color = if gs.aimed_pawn.is_some() {
                ui::Color32::from_rgb(255, 80, 60)
            } else {
                ui::Color32::from_rgba_premultiplied(255, 255, 255, 180)
            };
            ui::Area::new("crosshair")
                .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(ui::RichText::new("⊕").size(22.0).color(crosshair_color));
                });
            ui::Window::new("HUD")
                .anchor(ui::Align2::LEFT_TOP, [10.0, 10.0])
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.label(ui::RichText::new(format!("Score: {}", gs.score))
                        .color(ui::Color32::from_rgb(240, 200, 80)));
                    ui.label(ui::RichText::new(format!("Wave:  {}", gs.wave))
                        .color(ui::Color32::from_rgb(180, 160, 100)));
                });
            ui::Window::new("HUD_BOTTOM")
                .anchor(ui::Align2::LEFT_BOTTOM, [16.0, -16.0])
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .frame(ui::Frame::none()
                    .fill(ui::Color32::from_rgba_unmultiplied(0, 0, 0, 140))
                    .inner_margin(ui::Margin::symmetric(14.0, 10.0))
                    .rounding(8.0))
                .show(ctx, |ui| {
                    ui.set_min_width(180.0);
                    // ── Health bar ───────────────────────────────────────────
                    ui.label(ui::RichText::new("HEALTH").size(11.0)
                        .color(ui::Color32::from_rgb(160, 160, 160)));
                    let health_ratio = gs.lives as f32 / STARTING_LIVES as f32;
                    let bar_color = if health_ratio > 0.6 {
                        ui::Color32::from_rgb(80, 200, 80)
                    } else if health_ratio > 0.3 {
                        ui::Color32::from_rgb(220, 180, 50)
                    } else {
                        ui::Color32::from_rgb(220, 60, 60)
                    };
                    let (bar_rect, _) = ui.allocate_exact_size(
                        ui::Vec2::new(180.0, 10.0),
                        ui::Sense::hover(),
                    );
                    ui.painter().rect_filled(bar_rect, 4.0, ui::Color32::from_rgb(50, 50, 50));
                    if health_ratio > 0.0 {
                        let filled = ui::Rect::from_min_size(
                            bar_rect.min,
                            ui::Vec2::new(bar_rect.width() * health_ratio, bar_rect.height()),
                        );
                        ui.painter().rect_filled(filled, 4.0, bar_color);
                    }
                    ui.add_space(6.0);
                    // ── Ammo ─────────────────────────────────────────────────
                    if gs.reloading {
                        ui.label(ui::RichText::new("RELOADING")
                            .size(28.0)
                            .color(ui::Color32::from_rgb(220, 160, 50)));
                    } else {
                        let ammo_color = if gs.ammo == 0 {
                            ui::Color32::from_rgb(220, 60, 60)
                        } else if gs.ammo <= MAG_SIZE / 3 {
                            ui::Color32::from_rgb(220, 160, 50)
                        } else {
                            ui::Color32::WHITE
                        };
                        ui.horizontal(|ui| {
                            ui.label(ui::RichText::new(format!("{}", gs.ammo))
                                .size(48.0)
                                .color(ammo_color));
                            ui.vertical(|ui| {
                                ui.add_space(20.0);
                                ui.label(ui::RichText::new(format!("/ {}", MAG_SIZE))
                                    .size(16.0)
                                    .color(ui::Color32::from_rgb(160, 160, 160)));
                                ui.label(ui::RichText::new(format!("▪ {}", gs.total_ammo))
                                    .size(13.0)
                                    .color(ui::Color32::from_rgb(120, 140, 120)));
                            });
                        });
                    }
                });
        }

        GamePhase::GameOver => {
            ui::Window::new("Game Over")
                .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.heading(ui::RichText::new("✝  GAME OVER  ✝")
                        .size(28.0)
                        .color(ui::Color32::from_rgb(200, 60, 50)));
                    ui.separator();
                    ui.label(format!("Final Score: {}", gs.score));
                    ui.label(format!("Waves survived: {}", gs.wave.saturating_sub(1)));
                    ui.add_space(6.0);
                    ui.separator();
                    if ui.button(ui::RichText::new("  ▶  Play Again  ")
                            .color(ui::Color32::from_rgb(255, 220, 80))).clicked() {
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

// ── FPS look (mouse-driven camera rotation) ───────────────────────────────────

fn fps_look(app_state: &mut AppState) {
    let phase = app_state.get_state_data_value::<GameState>("game_state")
        .map(|gs| gs.phase.clone());
    if phase != Some(GamePhase::Playing) || app_state.modifiers.ctrl {
        return;
    }
    let delta = app_state.get_mouse_state().get_delta();
    // No delta_time: mouse delta is already pixels-moved-this-frame, not a rate.
    let sensitivity = app_state.get_state_data_value::<f32>("camera_rotate_speed")
        .copied()
        .unwrap_or(0.002);
    if let Some(cam) = app_state.get_camera_mut() {
        cam.transform.rotation.y -= delta.0 as f32 * sensitivity;
        cam.transform.rotation.x -= delta.1 as f32 * sensitivity;
        cam.transform.rotation.x = cam.transform.rotation.x
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
}

// ── Shoot ─────────────────────────────────────────────────────────────────────

fn fire_projectile(app_state: &mut AppState) {
    let mut gs = match app_state.get_state_data_value::<GameState>("game_state") {
        Some(g) => g.clone(),
        None => return,
    };

    if gs.phase != GamePhase::Playing { return; }
    if gs.reloading { return; }
    if gs.ammo == 0 {
        start_reload(&mut gs);
        app_state.set_state_data_value("game_state", Box::new(gs));
        return;
    }
    if gs.projectile_ids.len() >= MAX_PROJECTILES { return; }

    let cam = match app_state.camera.as_ref() {
        Some(c) => c,
        None => return,
    };
    let cam_pos = cam.get_position();
    let dir = cam.calculate_direction_vector();
    let velocity = [dir[0] * PROJECTILE_SPEED, dir[1] * PROJECTILE_SPEED, dir[2] * PROJECTILE_SPEED];

    let yaw   = dir[0].atan2(dir[2]).to_degrees() + 90.0;
    let pitch = (-dir[1]).atan2((dir[0]*dir[0] + dir[2]*dir[2]).sqrt()).to_degrees();

    let spawn_pos = app_state.get_state_data_value::<[f32; 3]>("muzzle_pos")
        .copied()
        .unwrap_or(cam_pos);

    let mut proj = Object::load_from_gltf_resource(example_resources::bullet_gltf(), None);
    proj.set_name("projectile".to_string());
    proj.set_collision(false);
    proj.add_material(gs.projectile_material_uuid);
    proj.get_shapes_mut()[0].set_material_from_object_list(0);
    proj.transform.set_position(spawn_pos);
    proj.transform.set_scale([0.1, 0.1, 0.1]);
    proj.transform.set_rotation([pitch, yaw, 0.0]);

    let speed = (velocity[0]*velocity[0] + velocity[1]*velocity[1] + velocity[2]*velocity[2]).sqrt();
    proj.set_component(ProjectileData { velocity, distance: 0.0, speed });
    let uuid = proj.get_unique_id();
    gs.projectile_ids.push(uuid);
    app_state.add_object(proj);
    app_state.play_audio_once("shot");
    gs.recoil = 1.0;
    gs.ammo -= 1;
    if gs.ammo == 0 { start_reload(&mut gs); }
    app_state.set_state_data_value("game_state", Box::new(gs));
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let event_loop = EventLoop::new("Pawn Rush", 1920, 1080);
    let mut app_state = AppState::new();
    event_loop.set_icon_from_resource(resources::icon());

    // Base lights first — initialize_scene uses light.len() to index firefly lights
    let sun = Light::new([0.0, 25.0, 0.0], [1.0, 0.95, 0.85], 2500.0, None, false);
    let fill = Light::new([-5.0, 3.0, 5.0], [0.4, 0.55, 1.0], 200.0, None, false);
    let ambient = Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.25, None, false);
    app_state.add_light(sun, LightEmissionType::Source);
    app_state.add_light(fill, LightEmissionType::Source);
    app_state.add_light(ambient, LightEmissionType::Ambient);

    initialize_scene(&mut app_state, &event_loop);

    let pawn_mat_uuid = find_material_uuid(&app_state, "mat_pawn");
    let proj_mat_uuid = find_material_uuid(&app_state, "mat_projectile");
    let gs = GameState::new(pawn_mat_uuid, proj_mat_uuid);
    app_state.add_state_data("game_state", Box::new(gs));

    let camera = Camera::new(
        Some([0.0, CAMERA_HEIGHT, 8.0]),
        Some([-20.0, 0.0, 0.0]),
        Some(80.0),
        Some(16.0 / 9.0),
        Some(0.01),
        Some(512.0),
    );
    app_state.set_camera(camera);

    let bgm = AudioClip::from_resource(example_resources::background_music(), "bgm");
    let hit_sound = AudioClip::from_resource(example_resources::click_sound(), "hit");
    let shot_sound = AudioClip::from_resource(example_resources::shot_sound(), "shot");
    let break_sound = AudioClip::from_resource(example_resources::break_sound(), "break");
    let pickup_sound = AudioClip::from_resource(example_resources::pickup_sound(), "pickup");
    app_state.add_audio(bgm);
    app_state.add_audio(hit_sound);
    app_state.add_audio(shot_sound);
    app_state.add_audio(break_sound);
    app_state.add_audio(pickup_sound);
    app_state.play_audio_loop("bgm");

    app_state.add_post_process(Box::new(
        postprocessing::bloom::Bloom::new(&event_loop.display.clone(), 0.97, 12)
    ));
    app_state.add_post_process(Box::new(
        postprocessing::depth_fog::DepthFog::new(&event_loop.display, 0.15, 80.0, 400.0, [0.1, 0.1, 0.2], 1.0)
    ));
    app_state.add_post_process(Box::new(
        postprocessing::vignette::Vignette::new(&event_loop.display.clone(), 0.3, 0.4, [0.0, 0.0, 0.0], 0.85)
    ));

    app_state.add_state_data("camera_move_speed", Box::new(15.0f32));
    app_state.add_state_data("camera_rotate_speed", Box::new(0.002f32));

    // WASD: KeyDown fires every frame while held, custom ground-locked movement
    app_state.inject_event(
        event::EventCharacteristic::KeyDown(event::VirtualKeyCode::W),
        Arc::new(walk_forward),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyDown(event::VirtualKeyCode::S),
        Arc::new(walk_backward),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyDown(event::VirtualKeyCode::A),
        Arc::new(walk_left),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyDown(event::VirtualKeyCode::D),
        Arc::new(walk_right),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::MousePress(event::MouseButton::Left),
        Arc::new(fire_projectile),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::R),
        Arc::new(reload_weapon),
        Some(event::EventModifiers::new(false, false, false)),
    );
    app_state.inject_update_function(Arc::new(fps_look));
    app_state.inject_update_function(Arc::new(game_update));
    app_state.inject_gui(Arc::new(pawn_rush_ui));

    event_loop.run(app_state.convert_to_arc_mutex());
}
