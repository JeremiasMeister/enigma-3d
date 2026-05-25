use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, VirtualKeyCode};
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::{Material, TextureType};
use enigma_3d::particle::ParticleSystem;
use enigma_3d::particle::config::FlipbookConfig;
use enigma_3d::resources;
use winit::event::MouseButton;

fn spawn_sparks_on_click(app: &mut AppState) {
    // Spawn at (2, 1, 0) for v1 — a real raycast from mouse pos comes later.
    let xform = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [2.0, 1.0, 0.0, 1.0],
    ];
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

    // Materials — particles use particle_sprite / particle_ribbon presets.
    // The sprite material uses the embedded explosion flipbook atlas.
    let mut sprite_mat = Material::particle_sprite(display.clone());
    sprite_mat.set_name("particle_sprite");
    sprite_mat.set_texture_from_resource(resources::vfx_explosion_flipbook(), TextureType::Albedo);
    app_state.add_material(sprite_mat);

    let mut ribbon_mat = Material::particle_ribbon(display.clone());
    ribbon_mat.set_name("particle_ribbon");
    app_state.add_material(ribbon_mat);

    let sprite_mat_id = app_state.materials.iter().find(|m| m.name == "particle_sprite").map(|m| m.uuid).unwrap();
    let ribbon_mat_id = app_state.materials.iter().find(|m| m.name == "particle_ribbon").map(|m| m.uuid).unwrap();

    // Templates for sub-emitters and click-to-spawn (loaded from embedded JSON resources)
    let mut smoke = ParticleSystem::from_resource(resources::vfx_fireball_smoke()).expect("load fireball_smoke");
    smoke.set_material(sprite_mat_id);
    app_state.add_particle_system_template(smoke);

    let mut dust = ParticleSystem::from_resource(resources::vfx_spark_dust()).expect("load spark_dust");
    dust.set_material(sprite_mat_id);
    app_state.add_particle_system_template(dust);

    let mut sparks = ParticleSystem::from_resource(resources::vfx_hit_sparks()).expect("load hit_sparks");
    sparks.set_material(ribbon_mat_id);
    app_state.add_particle_system_template(sparks);

    // Active fireball at origin + 2y, with the explosion atlas as a flipbook.
    // Adjust cols/rows/frame_count to match the actual atlas layout if it isn't 4x4.
    let mut fireball = ParticleSystem::from_resource(resources::vfx_fireball()).expect("load fireball");
    fireball.set_material(sprite_mat_id);
    fireball.set_position([0.0, 2.0, 0.0]);
    fireball.set_flipbook(FlipbookConfig {
        cols: 4,
        rows: 4,
        frame_count: 16,
        fps: 30.0,
        blend: false,
        randomize_start_frame: true,
    });
    app_state.add_particle_system(fireball);

    // Active magic orb
    let mut orb = ParticleSystem::from_resource(resources::vfx_magic_orb()).expect("load magic_orb");
    orb.set_material(ribbon_mat_id);
    orb.set_position([-3.0, 2.0, 0.0]);
    app_state.add_particle_system(orb);

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
