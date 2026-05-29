//! Frustum culling demo.
//!
//! A 10x10 grid of colored cubes spread over a wide XZ area lets you watch the
//! visible-vs-total count drop as you rotate the camera away from the scene.
//! Press `F` to toggle culling on/off and confirm the count snaps back to the
//! total when it is disabled.
//!
//! Controls:
//!   WASD                — flycam horizontal movement
//!   Space / Ctrl+Space  — flycam up / down
//!   Right-click + drag  — rotate camera (with Ctrl, per the engine default)
//!   F                   — toggle frustum culling

use std::sync::Arc;

use enigma_3d::camera::Camera;
use enigma_3d::event;
use enigma_3d::event::EventModifiers;
use enigma_3d::material::Material;
use enigma_3d::object::Object;
use enigma_3d::AppState;

const GRID_SIZE: i32 = 10;
const GRID_SPACING: f32 = 4.0;

fn toggle_culling(app_state: &mut AppState) {
    app_state.frustum_culling_enabled = !app_state.frustum_culling_enabled;
    if !app_state.frustum_culling_enabled {
        // When disabling culling, force everything visible so the scene is
        // immediately complete again — `cull_objects` early-returns and would
        // otherwise leave Object.visible at whatever it was last frame.
        for obj in app_state.objects.iter_mut() {
            obj.visible = true;
        }
    }
}

fn frustum_ui(ctx: &egui::Context, app_state: &mut AppState) {
    egui::Window::new("Frustum Culling Demo")
        .default_width(260.0)
        .show(ctx, |ui| {
            let on = app_state.frustum_culling_enabled;
            ui.heading(if on { "Culling: ON" } else { "Culling: OFF" });
            ui.separator();
            ui.label(format!(
                "Visible: {} / {}",
                app_state.cull_stats.visible, app_state.cull_stats.total
            ));
            ui.separator();
            ui.label("F           toggle culling");
            ui.label("WASD        fly horizontally");
            ui.label("Space       fly up");
            ui.label("Ctrl+Space  fly down");
            ui.label("Ctrl+RMB    rotate camera");
        });
}

fn main() {
    let event_loop = enigma_3d::EventLoop::new("Enigma 3D — Frustum Culling Demo", 1280, 720);
    let mut app_state = AppState::new();

    enigma_3d::init_default(&mut app_state);

    // One material per row gives a clearly visible color gradient across the grid
    // so you can tell which cubes are being culled. Unlit keeps the demo cheap.
    let mut row_material_ids = Vec::with_capacity(GRID_SIZE as usize);
    for row in 0..GRID_SIZE {
        let mut mat = Material::unlit(event_loop.get_display_clone(), false);
        let t = row as f32 / (GRID_SIZE - 1).max(1) as f32;
        // Smooth red -> green -> blue ramp across rows.
        let r = (1.0 - t).clamp(0.0, 1.0);
        let g = (1.0 - (t - 0.5).abs() * 2.0).clamp(0.0, 1.0);
        let b = t.clamp(0.0, 1.0);
        mat.set_color([r, g, b]);
        mat.set_name(&format!("cube_row_{}_mat", row));
        row_material_ids.push(mat.uuid);
        app_state.add_material(mat);
    }

    // 10x10 grid of cubes — 100 objects so the visible-count change is dramatic.
    let half = (GRID_SIZE as f32 - 1.0) * 0.5;
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let mut cube = Object::cube(0.6);
            cube.name = format!("cube_{}_{}", row, col);
            let x = (col as f32 - half) * GRID_SPACING;
            let z = (row as f32 - half) * GRID_SPACING;
            cube.transform.set_position([x, 0.0, z]);
            cube.add_material(row_material_ids[row as usize]);
            cube.get_shapes_mut()[0].set_material_from_object_list(0);
            app_state.add_object(cube);
        }
    }

    // A handful of spheres at varied heights so the depth/scale is obvious.
    for i in 0..16 {
        let mut sphere = Object::sphere(0.8, 16, 24);
        sphere.name = format!("sphere_{}", i);
        let angle = i as f32 / 16.0 * std::f32::consts::TAU;
        let radius = (GRID_SIZE as f32 * GRID_SPACING) * 0.35;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        sphere.transform.set_position([x, 2.5, z]);
        let mat_id = row_material_ids[i as usize % row_material_ids.len()];
        sphere.add_material(mat_id);
        sphere.get_shapes_mut()[0].set_material_from_object_list(0);
        app_state.add_object(sphere);
    }

    // A bit of ambient so unlit looks fine alongside any future lit assets.
    let ambient = enigma_3d::light::Light::new(
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        0.4,
        None,
        false,
    );
    app_state.add_light(ambient, enigma_3d::light::LightEmissionType::Ambient);

    // Camera placed high and looking down-ish so the grid is fully framed at start.
    let camera = Camera::new(
        Some([0.0, 12.0, 25.0]),
        Some([-0.35, 0.0, 0.0]),
        Some(75.0),
        Some(1280.0 / 720.0),
        Some(0.1),
        Some(500.0),
    );
    app_state.set_camera(camera);

    // F toggles culling.
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::F),
        Arc::new(toggle_culling),
        Some(EventModifiers::new(false, false, false)),
    );

    app_state.inject_gui(Arc::new(frustum_ui));

    event_loop.run(app_state.convert_to_arc_mutex());
}
