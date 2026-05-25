use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, VirtualKeyCode};
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::{Material, TextureType};
use enigma_3d::particle::ParticleSystem;
use enigma_3d::particle::config::{FlipbookConfig, RenderStyle, BlendMode};
use enigma_3d::particle::{EmitterShape, InitialVelocity, ColorRange, Range};
use enigma_3d::resources;
use rand::Rng;
use winit::event::MouseButton;

fn spawn_sparks_on_click(app: &mut AppState) {
    // Spawn at (2, 1, 0) for v1 — a real raycast from mouse pos comes later.
    let xform = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [2.0, 1.0, 0.0, 1.0],
    ];
    let _ = app.emit_particle_system("explosion", xform);
    let _ = app.emit_particle_system("hit_sparks", xform);
}

fn show_particle_counter(ctx: &egui::Context, app: &mut AppState) {
    egui::Window::new("Particles").show(ctx, |ui| {
        let total: usize = app.particle_systems.iter().map(|s| s.live_count()).sum();
        ui.label(format!("Active systems: {}", app.particle_systems.len()));
        ui.label(format!("Total live particles: {}", total));
    });
}

fn main() {
    let event_loop = EventLoop::new("Particles Demo", 1280, 720);
    let mut app_state = AppState::new();
    app_state.cursor_locked = true;
    enigma_3d::init_default(&mut app_state);
    let display = event_loop.get_display_clone();

    // Camera positioning
    if let Some(cam) = app_state.camera.as_mut() {
        cam.set_position([0.0, 2.0, 6.0]);
        cam.set_far(200.0);
    }

    // A gentle sun + ambient — but particles render after geometry, so this mainly affects any future ground/walls.
    let sun = Light::new([10.0, 20.0, 10.0], [1.0, 0.97, 0.90], 200.0, None, false);
    app_state.add_light(sun, LightEmissionType::Source);
    let ambient = Light::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.6], 0.4, None, false);
    app_state.add_light(ambient, LightEmissionType::Ambient);

    // Two sprite materials — one per atlas — plus a ribbon material for trails.
    // The 4x4 sprite_sheet drives the persistent fireball; the 5x5 explosion atlas
    // drives the click-to-spawn explosion burst.
    let mut sheet_mat = Material::particle_sprite(display.clone());
    sheet_mat.set_name("particle_sheet_4x4");
    sheet_mat.set_texture_from_resource(resources::vfx_sprite_sheet(), TextureType::Albedo);
    app_state.add_material(sheet_mat);

    let mut explosion_mat = Material::particle_sprite(display.clone());
    explosion_mat.set_name("particle_explosion_5x5");
    explosion_mat.set_texture_from_resource(resources::vfx_explosion_flipbook(), TextureType::Albedo);
    app_state.add_material(explosion_mat);

    let mut ribbon_mat = Material::particle_ribbon(display.clone());
    ribbon_mat.set_name("particle_ribbon");
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
    let mut smoke = ParticleSystem::from_resource(resources::vfx_fireball_smoke()).expect("load fireball_smoke");
    smoke.set_material(sheet_mat_id);
    smoke.set_flipbook(sheet_locked(5));
    app_state.add_particle_system_template(smoke);

    let mut dust = ParticleSystem::from_resource(resources::vfx_spark_dust()).expect("load spark_dust");
    dust.set_material(sheet_mat_id);
    dust.set_flipbook(sheet_locked(rng.gen_range(0..16)));
    app_state.add_particle_system_template(dust);

    let mut sparks = ParticleSystem::from_resource(resources::vfx_hit_sparks()).expect("load hit_sparks");
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
    let mut fireball = ParticleSystem::from_resource(resources::vfx_fireball()).expect("load fireball");
    fireball.set_material(sheet_mat_id);
    fireball.set_position([0.0, 2.0, 0.0]);
    fireball.set_flipbook(sheet_locked(14));
    app_state.add_particle_system(fireball);

    // Active magic orb (ribbon — no flipbook)
    let mut orb = ParticleSystem::from_resource(resources::vfx_magic_orb()).expect("load magic_orb");
    orb.set_material(ribbon_mat_id);
    orb.set_position([-3.0, 2.0, 0.0]);
    app_state.add_particle_system(orb);

    // Soft purple aura (sprite, additive) — sits low so it reads as a glowing dome.
    let mut aura = ParticleSystem::from_resource(resources::vfx_aura()).expect("load aura");
    aura.set_material(sheet_mat_id);
    aura.set_flipbook(sheet_locked(rng.gen_range(0..16)));
    aura.set_position([-6.0, 1.0, 0.0]);
    app_state.add_particle_system(aura);

    // Teal/cyan ribbon vortex — many trails swirling around a vertical axis.
    let mut vortex = ParticleSystem::from_resource(resources::vfx_vortex_ribbon()).expect("load vortex_ribbon");
    vortex.set_material(ribbon_mat_id);
    vortex.set_position([3.0, 1.0, 0.0]);
    app_state.add_particle_system(vortex);

    // Crackling lightning bolts — short-lived bright ribbons twisting upward via curl noise.
    let mut lightning = ParticleSystem::from_resource(resources::vfx_lightning()).expect("load lightning");
    lightning.set_material(ribbon_mat_id);
    lightning.set_position([6.0, 0.5, 0.0]);
    app_state.add_particle_system(lightning);

    // Camera controls
    app_state.set_state_data_value("camera_move_speed", Box::new(5.0f32));
    app_state.set_state_data_value("camera_rotate_speed", Box::new(0.002f32));
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::W), Arc::new(default_events::camera_walk_forward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::S), Arc::new(default_events::camera_walk_backward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::A), Arc::new(default_events::camera_walk_left), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::D), Arc::new(default_events::camera_walk_right), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::Space), Arc::new(default_events::camera_up), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::LControl), Arc::new(default_events::camera_down), None);
    app_state.inject_update_function(Arc::new(default_events::camera_fps_look));

    // Click-to-spawn sparks
    app_state.inject_event(
        EventCharacteristic::MousePress(MouseButton::Left),
        Arc::new(spawn_sparks_on_click),
        None,
    );

    // GUI counter
    app_state.inject_gui(Arc::new(show_particle_counter));

    event_loop.run(app_state.convert_to_arc_mutex());
}
