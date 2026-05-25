use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, EventModifiers, VirtualKeyCode};
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::{Material, TextureType};
use enigma_3d::particle::ParticleSystem;
use enigma_3d::particle::config::{FlipbookConfig, RenderStyle, BlendMode};
use enigma_3d::particle::{EmitterShape, InitialVelocity, ColorRange, Range, Gradient};
use enigma_3d::example_resources;
use rand::Rng;
use winit::event::MouseButton;
use enigma_3d::audio::AudioClip;
use enigma_3d::texture::Texture;

/// Component marker placed on temporary lights so an update function can age them out.
struct TempLightTimer {
    remaining: f32,
    total: f32,
    base_intensity: f32,
}

/// Ray-vs-ground-plane (y=0) intersection. Returns None if the ray doesn't hit (parallel,
/// behind us, or hit is absurdly far away).
fn ray_hit_ground(origin: [f32; 3], dir: [f32; 3]) -> Option<[f32; 3]> {
    if dir[1].abs() < 1e-4 { return None; }
    let t = -origin[1] / dir[1];
    if t <= 0.0 || t > 200.0 { return None; }
    Some([
        origin[0] + dir[0] * t,
        0.0,
        origin[2] + dir[2] * t,
    ])
}

fn spawn_burst_on_click(app: &mut AppState) {
    // Screen-center raycast: shoot a ray from the camera position in the direction
    // the camera is looking. Independent of mouse position, so it works the same
    // whether the cursor is locked, unlocked, or hovering anywhere on screen.
    let camera = match app.camera.as_ref() { Some(c) => c, None => return };
    let origin = camera.get_position();
    let dir = camera.calculate_direction_vector();

    // Either hit the ground or fall back to a point 8m in front of the camera.
    let pos = ray_hit_ground(origin, dir).unwrap_or_else(|| {
        const FALLBACK_DISTANCE: f32 = 8.0;
        [
            origin[0] + dir[0] * FALLBACK_DISTANCE,
            origin[1] + dir[1] * FALLBACK_DISTANCE,
            origin[2] + dir[2] * FALLBACK_DISTANCE,
        ]
    });

    // Spawn at slightly-above-ground so particles aren't half-buried.
    let spawn_y = pos[1] + 0.5;
    let xform = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [pos[0], spawn_y, pos[2], 1.0],
    ];
    let _ = app.emit_particle_system("explosion", xform);
    let _ = app.emit_particle_system("hit_sparks", xform);

    // Bright temporary point light that fades over ~0.45s.
    const TEMP_LIGHT_LIFE: f32 = 0.45;
    const TEMP_LIGHT_INTENSITY: f32 = 35.0;
    let mut temp = Light::new(
        [pos[0], spawn_y + 0.6, pos[2]],
        [1.0, 0.6, 0.2],
        TEMP_LIGHT_INTENSITY,
        None,
        false,
    );
    temp.set_component(TempLightTimer {
        remaining: TEMP_LIGHT_LIFE,
        total: TEMP_LIGHT_LIFE,
        base_intensity: TEMP_LIGHT_INTENSITY,
    });
    app.add_light(temp, LightEmissionType::Source);
}

/// Per-frame: fade and reap expired temporary lights.
fn tick_temp_lights(app: &mut AppState) {
    let dt = app.delta_time;
    for light in app.light.iter_mut() {
        let (remaining, total, base) = match light.get_component::<TempLightTimer>() {
            Some(t) => (t.remaining, t.total, t.base_intensity),
            None => continue,
        };
        let new_remaining = remaining - dt;
        let scale = (new_remaining / total).max(0.0);
        light.intensity = base * scale;
        if let Some(t) = light.get_component_mut::<TempLightTimer>() {
            t.remaining = new_remaining;
        }
    }
    app.light.retain(|l| match l.get_component::<TempLightTimer>() {
        Some(t) => t.remaining > 0.0,
        None => true,
    });
}

fn show_particle_counter(ctx: &egui::Context, app: &mut AppState) {
    egui::Window::new("Particles").show(ctx, |ui| {
        let total: usize = app.particle_systems.iter().map(|s| s.live_count()).sum();
        ui.label(format!("Active systems: {}", app.particle_systems.len()));
        ui.label(format!("Total live particles: {}", total));
    });
}

/// Eye-height clamp: prevent the FPS camera from sinking below the ground plane.
fn keep_camera_above_ground(app: &mut AppState) {
    const EYE_HEIGHT: f32 = 1.6;
    if let Some(cam) = app.camera.as_mut() {
        let p = cam.get_position();
        if p[1] < EYE_HEIGHT {
            cam.set_position([p[0], EYE_HEIGHT, p[2]]);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new("Particles Demo", 1280, 720);
    let mut app_state = AppState::new();
    app_state.cursor_locked = true;
    enigma_3d::init_default(&mut app_state);
    let display = event_loop.get_display_clone();

    // black skybox
    app_state.set_skybox_from_texture(Texture::colored_texture(event_loop.get_display_reference(),[13,13,28,255], Some("Skybox".to_string())), &event_loop);

    // Camera positioning
    if let Some(cam) = app_state.camera.as_mut() {
        cam.set_position([0.0, 2.0, 6.0]);
        cam.set_far(200.0);
    }

    // Flat ground plane at y=0 so the scene has spatial reference.
    let mut terrain_mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    terrain_mat.set_texture_from_resource(example_resources::terrain_albedo(), TextureType::Albedo);
    terrain_mat.set_texture_from_resource(example_resources::terrain_metallic(), TextureType::Metallic);
    terrain_mat.set_texture_from_resource(example_resources::terrain_normal(), TextureType::Normal);
    terrain_mat.set_texture_from_resource(example_resources::terrain_roughness(), TextureType::Roughness);
    let mut terrain = enigma_3d::terrain::Terrain::new(
        &event_loop.get_display_clone(),
        enigma_3d::terrain::TerrainConfig {
            width: 100.0,
            depth: 100.0,
            resolution: 64,
            max_height: 0.0,
            custom_noise: Some(Box::new(|_x, _z| 0.0)),
            // Neutral grey vertex colors so colored lights read as their hue.
            // The terrain shader does (ambient + diffuse) * v_color, so the
            // default green grass color was tinting everything.
            color_flat_low:  [0.55, 0.55, 0.55],
            color_flat_high: [0.55, 0.55, 0.55],
            color_slope:     [0.40, 0.40, 0.40],
            ..Default::default()
        },
    );
    terrain.set_material(terrain_mat);
    app_state.set_terrain(terrain);

    // Post-processing: bloom picks up HDR particle colors as glow; vignette for framing.
    app_state.add_post_process(Box::new(
        enigma_3d::postprocessing::bloom::Bloom::new(&display, 0.97, 12)
    ));
    app_state.add_post_process(Box::new(
        enigma_3d::postprocessing::vignette::Vignette::new(&display, 0.35, 0.45, [0.0, 0.0, 0.0], 0.7)
    ));

    // Lighting: a dim sun + ambient, plus a warm point light on the fireball
    // and a cool one on the aura. The engine caps source lights at 4 total —
    // sun + fireball + aura = 3, leaving 1 slot for click-spawned temp lights.
    let sun = Light::new([10.0, 20.0, 10.0], [1.0, 0.97, 0.90], 60.0, None, false);
    app_state.add_light(sun, LightEmissionType::Source);
    let ambient = Light::new([0.0, 0.0, 0.0], [0.4, 0.4, 0.55], 0.3, None, false);
    app_state.add_light(ambient, LightEmissionType::Ambient);
    let fireball_light = Light::new([0.0, 2.0, 0.0], [1.0, 0.5, 0.15], 40.0, None, false);
    app_state.add_light(fireball_light, LightEmissionType::Source);
    let aura_light = Light::new([-6.0, 1.2, 0.0], [0.6, 0.4, 1.0], 30.0, None, false);
    app_state.add_light(aura_light, LightEmissionType::Source);

    // Two sprite materials — one per atlas — plus a ribbon material for trails.
    // The 4x4 sprite_sheet drives the persistent fireball; the 5x5 explosion atlas
    // drives the click-to-spawn explosion burst.
    let mut sheet_mat = Material::particle_sprite(display.clone());
    sheet_mat.set_name("particle_sheet_4x4");
    sheet_mat.set_texture_from_resource(example_resources::vfx_sprite_sheet(), TextureType::Albedo);
    app_state.add_material(sheet_mat);

    let mut explosion_mat = Material::particle_sprite(display.clone());
    explosion_mat.set_name("particle_explosion_5x5");
    explosion_mat.set_texture_from_resource(example_resources::vfx_explosion_flipbook(), TextureType::Albedo);
    app_state.add_material(explosion_mat);

    let mut ribbon_mat = Material::particle_ribbon(display.clone());
    ribbon_mat.set_name("particle_ribbon");
    ribbon_mat.set_texture_from_resource(example_resources::vfx_ribbon_texture(), TextureType::Albedo);
    app_state.add_material(ribbon_mat);

    let sheet_mat_id = app_state.materials.iter().find(|m| m.name == "particle_sheet_4x4").map(|m| m.uuid).unwrap();
    let explosion_mat_id = app_state.materials.iter().find(|m| m.name == "particle_explosion_5x5").map(|m| m.uuid).unwrap();
    let ribbon_mat_id = app_state.materials.iter().find(|m| m.name == "particle_ribbon").map(|m| m.uuid).unwrap();

    // Lock each sprite-sheet-based system to a single random cell of the 4x4 atlas.
    // Result: all particles in the fireball share one sprite; smoke uses a different
    // one; dust uses another. fixed_frame overrides any per-particle frame state.
    let mut rng = rand::thread_rng();
    let sheet_locked = |frame: u32| FlipbookConfig {
        cols: 4, rows: 4, frame_count: 16,
        fps: 0.0,
        blend: false,
        randomize_start_frame: false,
        fixed_frame: Some(frame),
    };

    // Templates for sub-emitters and click-to-spawn (loaded from embedded JSON resources).
    let mut smoke = ParticleSystem::from_resource(example_resources::vfx_fireball_smoke()).expect("load fireball_smoke");
    smoke.set_material(sheet_mat_id);
    smoke.set_flipbook(sheet_locked(5));
    app_state.add_particle_system_template(smoke);

    let mut dust = ParticleSystem::from_resource(example_resources::vfx_spark_dust()).expect("load spark_dust");
    dust.set_material(sheet_mat_id);
    dust.set_flipbook(sheet_locked(rng.gen_range(0..16)));
    app_state.add_particle_system_template(dust);

    let mut sparks = ParticleSystem::from_resource(example_resources::vfx_hit_sparks()).expect("load hit_sparks");
    sparks.set_material(ribbon_mat_id);
    app_state.add_particle_system_template(sparks);

    // Explosion template (5x5 atlas, one-shot burst, plays the atlas once).
    let mut explosion = ParticleSystem::new("explosion");
    explosion.set_position([5.0,0.0,-5.0]);
    explosion.set_max_particles(16);
    explosion.set_looped(true);
    explosion.set_duration(0.1);
    explosion.add_burst(0.0, 8);
    explosion.set_emission_rate(0.0);
    explosion.set_emitter_shape(EmitterShape::Sphere { radius: 0.2 });
    explosion.set_initial_velocity(InitialVelocity::Outward { speed: Range::new(0.5, 1.5) });
    explosion.set_initial_lifetime(0.7, 0.9);   // matches 25 frames / 30 fps ≈ 0.83s
    explosion.set_initial_size(1.2, 1.8);
    explosion.set_initial_rotation(0.0, 6.283);
    explosion.set_initial_color(ColorRange::Hdr { rgb: [1.0, 0.7, 0.3], intensity: 8.0, alpha: 1.0 });
    explosion.set_drag(2.0);
    explosion.set_material(explosion_mat_id);
    explosion.set_render_style(RenderStyle::Sprite {
        flipbook: None,
        blend_mode: BlendMode::Additive,
        soft_particles: true,
        soft_fade_distance: 0.3,
        velocity_stretch: 0.0,
    });
    explosion.set_flipbook(FlipbookConfig {
        cols: 5,
        rows: 5,
        frame_count: 25,
        fps: 30.0,
        blend: false,
        randomize_start_frame: false,  // explosion plays from frame 0 — it's a one-shot animation
        fixed_frame: None,             // animate through frames, don't lock to one
    });
    app_state.add_particle_system_template(explosion);

    // Active fireball: locked to one random cell of the 4x4 atlas for the whole system.
    let mut fireball = ParticleSystem::from_resource(example_resources::vfx_fireball()).expect("load fireball");
    fireball.set_material(sheet_mat_id);
    fireball.set_position([0.0, 2.0, 0.0]);
    fireball.set_flipbook(sheet_locked(14));
    app_state.add_particle_system(fireball);

    // Active magic orb (ribbon — no flipbook)
    let mut orb = ParticleSystem::from_resource(example_resources::vfx_magic_orb()).expect("load magic_orb");
    orb.set_material(ribbon_mat_id);
    orb.set_position([-3.0, 2.0, 0.0]);
    app_state.add_particle_system(orb);

    // Soft purple aura (sprite, additive) — sits low so it reads as a glowing dome.
    let mut aura = ParticleSystem::from_resource(example_resources::vfx_aura()).expect("load aura");
    aura.set_material(sheet_mat_id);
    aura.set_flipbook(sheet_locked(14));
    aura.set_position([-6.0, 1.0, 0.0]);
    app_state.add_particle_system(aura);

    // Teal/cyan ribbon vortex — many trails swirling around a vertical axis.
    let mut vortex = ParticleSystem::from_resource(example_resources::vfx_vortex_ribbon()).expect("load vortex_ribbon");
    vortex.set_material(ribbon_mat_id);
    vortex.set_position([3.0, 1.0, 0.0]);
    app_state.add_particle_system(vortex);

    // Crackling lightning bolts — short-lived bright ribbons twisting upward via curl noise.
    let mut lightning = ParticleSystem::from_resource(example_resources::vfx_lightning()).expect("load lightning");
    lightning.set_material(ribbon_mat_id);
    lightning.set_position([6.0, 0.5, 0.0]);
    app_state.add_particle_system(lightning);

    // Starfield overhead. Thin slab of near-stationary points high above the player.
    // Twinkle comes from an oscillating alpha curve in color_over_life; varied lifetimes
    // (3–7s) mean stars desync naturally so the field shimmers rather than pulsing in unison.
    // Note: color_over_life overwrites per-particle RGB, so we bake one cool-white palette
    // into the gradient — true per-star color variation isn't possible while gradient is on.
    let mut stars = ParticleSystem::new("stars");
    stars.set_material(sheet_mat_id);
    stars.set_position([0.0, 20.0, 0.0]);
    stars.set_max_particles(600);
    stars.set_looped(true);
    stars.set_duration(1.0);
    stars.set_emission_rate(120.0); // ~600 capacity / ~5s mean life ≈ 120/s
    // Thin horizontal slab — wider than the 100×100 terrain so stars cover the visible sky dome.
    stars.set_emitter_shape(EmitterShape::Box { half_extents: [80.0, 6.0, 80.0] });
    // Effectively stationary — speed zero keeps stars fixed while alive.
    stars.set_initial_velocity(InitialVelocity::Direction {
        direction: [0.0, 1.0, 0.0],
        speed: Range::new(0.0, 0.0),
    });
    stars.set_initial_lifetime(3.0, 7.0);
    stars.set_initial_size(0.55, 0.8);
    // Cool-white with a blue tint, HDR-bright so bloom picks them up.
    // RGB = [0.82, 0.90, 1.0] × intensity 4.0 = [3.28, 3.60, 4.0].
    // Alpha oscillates 3× across each lifetime → twinkle. First/last keys at 0 hide spawn/death.
    stars.set_color_over_life(Gradient::new(vec![
        (0.00, [3.28, 3.60, 4.0, 0.0]),
        (0.10, [3.28, 3.60, 4.0, 1.0]),
        (0.25, [3.28, 3.60, 4.0, 0.25]),
        (0.40, [3.28, 3.60, 4.0, 1.0]),
        (0.55, [3.28, 3.60, 4.0, 0.20]),
        (0.70, [3.28, 3.60, 4.0, 1.0]),
        (0.85, [3.28, 3.60, 4.0, 0.30]),
        (1.00, [3.28, 3.60, 4.0, 0.0]),
    ]));
    stars.set_render_style(RenderStyle::Sprite {
        flipbook: None,
        blend_mode: BlendMode::Additive,
        soft_particles: true,
        soft_fade_distance: 0.4,
        velocity_stretch: 0.0,
    });
    // Must come after set_render_style — that call replaces the whole render style
    // and would wipe the flipbook back to None. Lock to cell 0 of the 4×4 atlas.
    stars.set_flipbook(sheet_locked(0));
    app_state.add_particle_system(stars);

    // Camera controls
    app_state.set_state_data_value("camera_move_speed", Box::new(5.0f32));
    app_state.set_state_data_value("camera_rotate_speed", Box::new(0.002f32));
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::W), Arc::new(default_events::camera_walk_forward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::S), Arc::new(default_events::camera_walk_backward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::A), Arc::new(default_events::camera_walk_left), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::D), Arc::new(default_events::camera_walk_right), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::Space), Arc::new(default_events::camera_up), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::Space), Arc::new(default_events::camera_down), Some(EventModifiers::new(false, true, false)));
    app_state.inject_update_function(Arc::new(default_events::camera_fps_look));
    app_state.inject_update_function(Arc::new(keep_camera_above_ground));
    app_state.inject_update_function(Arc::new(tick_temp_lights));

    // Click-to-spawn sparks
    app_state.inject_event(
        EventCharacteristic::MousePress(MouseButton::Left),
        Arc::new(spawn_burst_on_click),
        None,
    );

    // GUI counter
    app_state.inject_gui(Arc::new(show_particle_counter));
    let background_sound = AudioClip::from_resource(example_resources::background_music(), "background_music");
    app_state.add_audio(background_sound);
    app_state.play_audio_loop("background_music");

    event_loop.run(app_state.convert_to_arc_mutex());
}
