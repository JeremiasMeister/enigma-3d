use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, VirtualKeyCode};
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::Material;
use enigma_3d::particle::ParticleSystem;
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
    // No texture set → albedo is None → render_one returns early (silent skip).
    // To see particles, set a real texture on these materials (a 32x32 white circle PNG works).
    let mut sprite_mat = Material::particle_sprite(display.clone());
    sprite_mat.set_name("particle_sprite");
    app_state.add_material(sprite_mat);

    let mut ribbon_mat = Material::particle_ribbon(display.clone());
    ribbon_mat.set_name("particle_ribbon");
    app_state.add_material(ribbon_mat);

    let sprite_mat_id = app_state.materials.iter().find(|m| m.name == "particle_sprite").map(|m| m.uuid).unwrap();
    let ribbon_mat_id = app_state.materials.iter().find(|m| m.name == "particle_ribbon").map(|m| m.uuid).unwrap();

    // Templates for sub-emitters and click-to-spawn
    let mut smoke = ParticleSystem::from_file("assets/vfx/fireball_smoke.json").expect("load fireball_smoke");
    smoke.set_material(sprite_mat_id);
    app_state.add_particle_system_template(smoke);

    let mut dust = ParticleSystem::from_file("assets/vfx/spark_dust.json").expect("load spark_dust");
    dust.set_material(sprite_mat_id);
    app_state.add_particle_system_template(dust);

    let mut sparks = ParticleSystem::from_file("assets/vfx/hit_sparks.json").expect("load hit_sparks");
    sparks.set_material(ribbon_mat_id);
    app_state.add_particle_system_template(sparks);

    // Active fireball at origin + 2y
    let mut fireball = ParticleSystem::from_file("assets/vfx/fireball.json").expect("load fireball");
    fireball.set_material(sprite_mat_id);
    fireball.set_position([0.0, 2.0, 0.0]);
    app_state.add_particle_system(fireball);

    // Active magic orb
    let mut orb = ParticleSystem::from_file("assets/vfx/magic_orb.json").expect("load magic_orb");
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
