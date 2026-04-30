use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, VirtualKeyCode};
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::terrain::{Terrain, TerrainConfig, gradient_noise};

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

pub fn create_eroded_noise() -> Box<dyn Fn(f32, f32) -> f32 + Send + Sync> {
    Box::new(|x, z| {
        let frequency = 0.006; // Lower frequency = broader, less "frantic" hills
        let octaves = 8;
        let lacunarity = 2.1;
        let gain = 0.45;       // Lower gain = smaller high-frequency details

        let mut amp = 1.0;     // Keep amp at 1.0 for easier normalization
        let mut freq = frequency;
        let mut value = 0.0;
        let mut weight = 1.0;

        for i in 0..octaves {
            let noise_val = gradient_noise(x * freq, z * freq);

            // --- THE SOFTENING FIX ---
            // Instead of 1.0 - abs(n), we mix standard noise with rigid noise
            // or just use a softer ridge function:
            let mut signal = 1.0 - noise_val.abs();

            // Soften the "peak" of the ridge so it isn't a needle point
            signal = signal * (1.0 - (i as f32 * 0.05));

            signal *= weight;

            // Erosion: ensure valleys are smoother than peaks
            weight = signal.clamp(0.0, 1.0);

            value += signal * amp;
            freq *= lacunarity;
            amp *= gain;
        }

        // --- SCALE ADJUSTMENT ---
        // result.powf(1.5) makes things very spiky.
        // Use a value closer to 1.0 or 1.2 for "worn" mountains.
        value.powf(1.1) * 50.0
    })
}

fn main() {
    let event_loop = EventLoop::new("Terrain Demo", 1280, 720);
    let mut app_state = AppState::new();

    enigma_3d::init_default(&mut app_state);

    // Camera — slightly elevated, looking toward the terrain
    app_state.set_state_data_value("camera_move_speed", Box::new(10.0f32));
    app_state.set_state_data_value("camera_rotate_speed", Box::new(0.002f32));
    app_state.camera.as_mut().map(|cam| {cam.set_far(2048f32)});

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
            width:             2048.0,
            depth:             2048.0,
            resolution:        2048,
            slope_threshold:    0.75,
            max_height:        200.0,
            custom_noise: Some(Box::new(create_eroded_noise())),
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

    app_state.cursor_locked = true;

    event_loop.run(app_state.convert_to_arc_mutex());
}
