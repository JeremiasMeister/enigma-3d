use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::camera::Camera;
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, VirtualKeyCode};
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::terrain::{Terrain, TerrainConfig};

fn ground_camera(app_state: &mut AppState) {
    if let Some(terrain) = app_state.get_terrain() {
        let pos = app_state.camera.as_ref().map(|c| c.get_position());
        if let Some([x, _, z]) = pos {
            let ground = terrain.get_height(x, z);
            if let Some(cam) = app_state.camera.as_mut() {
                let p = cam.get_position();
                cam.set_position([p[0], ground + 1.7, p[2]]);
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new("Terrain Demo", 1280, 720);
    let mut app_state = AppState::new();

    enigma_3d::init_default(&mut app_state);

    // Camera — slightly elevated, looking toward the terrain
    let camera = Camera::new(
        Some([0.0, 5.0, 30.0]),
        Some([-10.0, 180.0, 0.0]),
        Some(75.0),
        Some(1280.0 / 720.0),
        Some(0.1),
        Some(1024.0),
    );
    app_state.set_camera(camera);
    app_state.add_state_data("camera_move_speed", Box::new(12.0f32));
    app_state.add_state_data("camera_rotate_speed", Box::new(0.002f32));

    // Sun
    let sun = Light::new([80.0, 120.0, 60.0], [1.0, 0.97, 0.90], 2800.0, None, false);
    app_state.add_light(sun, LightEmissionType::Source);

    // Ambient fill
    let ambient = Light::new([0.0, 0.0, 0.0], [0.45, 0.55, 0.75], 0.35, None, false);
    app_state.add_light(ambient, LightEmissionType::Ambient);

    // Terrain
    let terrain = Terrain::new(
        &event_loop.get_display_clone(),
        TerrainConfig {
            width:             200.0,
            depth:             200.0,
            max_height:        20.0,
            resolution:        128,
            noise_scale:       0.025,
            noise_octaves:     5,
            ..TerrainConfig::default()
        },
    );
    app_state.set_terrain(terrain);

    // FPS movement (KeyDown = held, fires every frame)
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::W), Arc::new(default_events::camera_walk_forward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::S), Arc::new(default_events::camera_walk_backward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::A), Arc::new(default_events::camera_walk_left), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::D), Arc::new(default_events::camera_walk_right), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::Space), Arc::new(default_events::camera_up), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::LControl), Arc::new(default_events::camera_down), None);

    // Mouse look + terrain grounding both run as update functions
    app_state.inject_update_function(Arc::new(default_events::camera_fps_look));
    app_state.inject_update_function(Arc::new(ground_camera));

    event_loop.run(app_state.convert_to_arc_mutex());
}
