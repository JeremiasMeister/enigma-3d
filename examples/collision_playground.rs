//! Collision query playground.
//!
//! Demonstrates the non-character-controller collision APIs:
//!   1. **Mouse ray-pick** — left-click in the scene → `RayCast` through the
//!      cursor → name of the hit object shown in the HUD.
//!   2. **Overlap zone** — a static sphere volume runs `ShapeOverlap` every
//!      frame. When any object is inside it, the zone's material colour
//!      changes from blue to red.
//!   3. **`ShapeCast`** — a moving sphere sweeps along +X. The HUD shows the
//!      time-of-impact and the name of the object it would hit; when a hit
//!      occurs the sphere resets to the start of the sweep.
//!
//! The camera is a free-fly cam: WASD + Space/LCtrl to move, hold right
//! mouse + Ctrl to rotate. Left-click is reserved for ray-picking.
//!
//! Run with: `cargo run --example=collision_playground --features=examples`

use std::sync::Arc;

use enigma_3d::camera::Camera;
use enigma_3d::collision_world::{Collider, RayCast, ShapeCast, ShapeOverlap};
use enigma_3d::default_events;
use enigma_3d::event::{EventCharacteristic, EventModifiers, MouseButton, VirtualKeyCode};
use enigma_3d::geometry::BoundingBox;
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::Material;
use enigma_3d::object::Object;
use enigma_3d::{AppState, EventLoop};
use nalgebra::Vector3;
use uuid::Uuid;

// ----------------------------------------------------------------------------
// Scene-state structs stored in `app_state` state_data
// ----------------------------------------------------------------------------

/// Tracked UUIDs for objects the demos manipulate every frame.
struct SceneIds {
    overlap_zone: Uuid,
    sweep_probe: Uuid,
    // Static targets that the sweep probe may collide with along its path.
    sweep_target_a: Uuid,
    sweep_target_b: Uuid,
}

/// Latest readouts from the per-frame queries — surfaced via egui in the HUD.
#[derive(Default, Clone)]
struct Hud {
    pick_name: String,
    pick_point: Option<[f32; 3]>,
    overlap_count: usize,
    overlap_first_name: String,
    sweep_toi: Option<f32>,
    sweep_hit_name: String,
}

/// Animation state for the sweep probe.
struct SweepState {
    /// Current X position of the swept sphere (starts at `start_x`, marches
    /// toward `end_x` every frame; resets on hit or when it reaches the end).
    x: f32,
    start_x: f32,
    end_x: f32,
    speed: f32,
    radius: f32,
}

const SCENE_IDS_KEY: &str = "scene_ids";
const HUD_KEY: &str = "hud";
const SWEEP_KEY: &str = "sweep";

// ----------------------------------------------------------------------------
// Per-frame update functions
// ----------------------------------------------------------------------------

/// Move the overlap zone sphere up and down a little so the user can see it
/// breathe, and resolve whether anything else is inside it. Updates the HUD
/// readout and recolours the zone's material from blue (empty) to red
/// (overlapping).
fn update_overlap_zone(app_state: &mut AppState) {
    let ids = match app_state.get_state_data_value::<SceneIds>(SCENE_IDS_KEY) {
        Some(ids) => SceneIds {
            overlap_zone: ids.overlap_zone,
            sweep_probe: ids.sweep_probe,
            sweep_target_a: ids.sweep_target_a,
            sweep_target_b: ids.sweep_target_b,
        },
        None => return,
    };

    // Snapshot the zone's world-space center for the query. Use the actual
    // object's transform so the query stays in sync with any future movement.
    let (zone_center, zone_radius) = match app_state.get_object_by_uuid(&ids.overlap_zone) {
        Some(obj) => {
            let p = obj.transform.get_position();
            (Vector3::new(p.x, p.y, p.z), 1.6_f32)
        }
        None => return,
    };

    let zone_shape = Collider::Sphere {
        center: zone_center,
        radius: zone_radius,
    };

    // ShapeOverlap will also hit the zone object itself if it has a collider;
    // we deliberately leave the zone without a collider so it's a pure visual
    // marker. Anything else with collision enabled is fair game.
    let overlap = ShapeOverlap::new(zone_shape);
    let hits = overlap.query(app_state);

    let overlap_count = hits.len();
    let first_name = hits
        .iter()
        .find_map(|h| {
            app_state
                .get_object_by_uuid(&h.object_id)
                .map(|o| o.get_name().clone())
        })
        .unwrap_or_default();

    // Re-tint the zone's material: red while overlapping, blue while empty.
    // Material lookup goes through the object's material list so we don't
    // have to thread material UUIDs into the HUD struct.
    if let Some(zone_obj) = app_state.get_object_by_uuid(&ids.overlap_zone) {
        if let Some(mat_uuid) = zone_obj.get_materials().first().copied() {
            // Look up the mutable Material on the app state.
            let new_color = if overlap_count > 0 {
                [1.0, 0.25, 0.25]
            } else {
                [0.25, 0.55, 1.0]
            };
            for m in app_state.materials.iter_mut() {
                if m.uuid == mat_uuid {
                    m.set_color(new_color);
                    break;
                }
            }
        }
    }

    if let Some(hud) = app_state.get_state_data_value_mut::<Hud>(HUD_KEY) {
        hud.overlap_count = overlap_count;
        hud.overlap_first_name = first_name;
    }
    // Silence unused warning — sweep_probe / targets are used in update_sweep.
    let _ = (ids.sweep_probe, ids.sweep_target_a, ids.sweep_target_b);
}

/// Sweep the probe sphere along +X. Each frame we cast a `ShapeCast` from the
/// current position for the remaining distance and update the HUD with the
/// result. The visual sphere then steps forward by `speed * delta_time`. On
/// hit or when we run past `end_x`, the sphere resets to `start_x`.
fn update_sweep(app_state: &mut AppState) {
    let dt = app_state.delta_time;
    let ids = match app_state.get_state_data_value::<SceneIds>(SCENE_IDS_KEY) {
        Some(ids) => ids.sweep_probe,
        None => return,
    };

    // Pull mutable sweep state out as a snapshot, then write it back.
    let mut sweep = match app_state.get_state_data_value_mut::<SweepState>(SWEEP_KEY) {
        Some(s) => SweepState {
            x: s.x,
            start_x: s.start_x,
            end_x: s.end_x,
            speed: s.speed,
            radius: s.radius,
        },
        None => return,
    };

    // Cast from current world position for the *remaining* distance — that
    // way the reported TOI is fresh every frame relative to the sphere's
    // present location, not the spawn point.
    let current_pos = Vector3::new(sweep.x, 1.0, -8.0);
    let remaining = (sweep.end_x - sweep.x).max(0.0);
    let probe_shape = Collider::Sphere {
        center: current_pos,
        radius: sweep.radius,
    };
    let cast = ShapeCast::new(probe_shape, Vector3::new(1.0, 0.0, 0.0), remaining)
        .with_steps(32);
    let hit = cast.cast(app_state);

    // Update the visible sphere and step forward.
    let mut hit_name = String::new();
    let mut hit_toi: Option<f32> = None;
    if let Some(h) = &hit {
        hit_toi = Some(h.toi);
        if let Some(obj) = app_state.get_object_by_uuid(&h.object_id) {
            hit_name = obj.get_name().clone();
        }
    }

    // Reset if we hit something (with a tiny epsilon so we don't oscillate
    // right at the contact boundary) or if we've traversed the full sweep.
    let reached_end = sweep.x >= sweep.end_x - 0.001;
    let hit_close = matches!(hit_toi, Some(t) if t <= 0.02);
    if reached_end || hit_close {
        sweep.x = sweep.start_x;
    } else {
        sweep.x += sweep.speed * dt;
    }

    // Push position to the visible object.
    if let Some(obj) = app_state.get_object_by_uuid_mut(ids) {
        obj.transform.set_position([sweep.x, 1.0, -8.0]);
    }

    if let Some(hud) = app_state.get_state_data_value_mut::<Hud>(HUD_KEY) {
        hud.sweep_toi = hit_toi;
        hud.sweep_hit_name = hit_name;
    }

    // Write the snapshot back.
    if let Some(s) = app_state.get_state_data_value_mut::<SweepState>(SWEEP_KEY) {
        *s = sweep;
    }
}

/// Left-click handler — ray-cast through the cursor and report the first hit
/// object's name into the HUD.
fn ray_pick_on_click(app_state: &mut AppState) {
    let (origin, direction) = match app_state.camera.as_ref() {
        Some(cam) => app_state.get_mouse_state().get_world_position(cam),
        None => return,
    };
    let mut ray = RayCast::new(origin, direction, 200.0);
    ray.cast(app_state);

    // Grab the *first* intersection (insertion order); RayCast collects all.
    let first = ray.get_intersection_map().iter().next()
        .map(|(uuid, point)| (*uuid, *point));

    let mut point_arr: Option<[f32; 3]> = None;
    let name = if let Some((uuid, point)) = first {
        point_arr = Some([point.x, point.y, point.z]);
        if uuid == Uuid::nil() {
            "<terrain>".to_string()
        } else if let Some(obj) = app_state.get_object_by_uuid(&uuid) {
            obj.get_name().clone()
        } else {
            "<unknown>".to_string()
        }
    } else {
        "<no hit>".to_string()
    };

    if let Some(hud) = app_state.get_state_data_value_mut::<Hud>(HUD_KEY) {
        hud.pick_name = name;
        hud.pick_point = point_arr;
    }
}

// ----------------------------------------------------------------------------
// GUI
// ----------------------------------------------------------------------------

fn collision_hud(ctx: &egui::Context, app_state: &mut AppState) {
    let hud_snapshot = app_state.get_state_data_value::<Hud>(HUD_KEY).cloned();

    egui::Window::new("Collision Playground")
        .default_width(280.0)
        .show(ctx, |ui| {
            ui.label("Free-fly cam: WASD + Space/LCtrl to move, hold Right-Click + Ctrl to look.");
            ui.label("Left-click = ray-pick the object under the cursor.");
            ui.separator();

            ui.heading("Ray-pick");
            if let Some(h) = &hud_snapshot {
                ui.label(format!("Hit: {}", if h.pick_name.is_empty() { "<none yet>".into() } else { h.pick_name.clone() }));
                if let Some(p) = h.pick_point {
                    ui.label(format!("Point: ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]));
                }
            }
            ui.separator();

            ui.heading("Overlap zone");
            if let Some(h) = &hud_snapshot {
                ui.label(format!("Objects inside zone: {}", h.overlap_count));
                if !h.overlap_first_name.is_empty() {
                    ui.label(format!("First: {}", h.overlap_first_name));
                }
                if h.overlap_count > 0 {
                    ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "ZONE TRIGGERED");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(60, 140, 220), "zone empty");
                }
            }
            ui.separator();

            ui.heading("Shape sweep");
            if let Some(h) = &hud_snapshot {
                match h.sweep_toi {
                    Some(t) => ui.label(format!("TOI: {:.3}", t)),
                    None => ui.label("TOI: <no hit on remaining path>"),
                };
                if !h.sweep_hit_name.is_empty() {
                    ui.label(format!("Would hit: {}", h.sweep_hit_name));
                }
            }
        });
}

// ----------------------------------------------------------------------------
// Bootstrap
// ----------------------------------------------------------------------------

fn main() {
    let event_loop = EventLoop::new("Collision Playground", 1280, 720);
    let mut app_state = AppState::new();
    let display = event_loop.get_display_clone();

    // ---- Camera + lights ---------------------------------------------------
    let camera = Camera::new(
        Some([0.0, 8.0, 14.0]),
        Some([-20.0, 0.0, 0.0]),
        Some(75.0),
        Some(16.0 / 9.0),
        Some(0.01),
        Some(1024.0),
    );
    app_state.set_camera(camera);

    let key = Light::new([6.0, 10.0, 6.0], [1.0, 0.97, 0.9], 220.0, None, false);
    let fill = Light::new([-5.0, 4.0, -2.0], [0.6, 0.7, 1.0], 80.0, None, false);
    let ambient = Light::new([0.0, 0.0, 0.0], [0.55, 0.6, 0.7], 0.5, None, false);
    app_state.add_light(key, LightEmissionType::Source);
    app_state.add_light(fill, LightEmissionType::Source);
    app_state.add_light(ambient, LightEmissionType::Ambient);

    // ---- Materials ---------------------------------------------------------
    // Bright unlit colours, one per scene role.
    let mut ground_mat = Material::unlit(display.clone(), false);
    ground_mat.set_color([0.18, 0.20, 0.24]);
    ground_mat.set_name("mat_ground");
    let ground_mat_id = ground_mat.uuid;

    let mut pick_mat_a = Material::unlit(display.clone(), false);
    pick_mat_a.set_color([0.85, 0.45, 0.25]);
    pick_mat_a.set_name("mat_pick_a");
    let pick_mat_a_id = pick_mat_a.uuid;

    let mut pick_mat_b = Material::unlit(display.clone(), false);
    pick_mat_b.set_color([0.25, 0.85, 0.55]);
    pick_mat_b.set_name("mat_pick_b");
    let pick_mat_b_id = pick_mat_b.uuid;

    let mut pick_mat_c = Material::unlit(display.clone(), false);
    pick_mat_c.set_color([0.85, 0.85, 0.25]);
    pick_mat_c.set_name("mat_pick_c");
    let pick_mat_c_id = pick_mat_c.uuid;

    // Overlap zone — starts blue, recoloured red while triggered.
    let mut zone_mat = Material::unlit(display.clone(), true);
    zone_mat.set_color([0.25, 0.55, 1.0]);
    zone_mat.set_transparency_strength(0.45);
    zone_mat.set_name("mat_zone");
    let zone_mat_id = zone_mat.uuid;

    // Sweep probe + targets.
    let mut probe_mat = Material::unlit(display.clone(), false);
    probe_mat.set_color([1.0, 0.4, 0.85]);
    probe_mat.set_name("mat_probe");
    let probe_mat_id = probe_mat.uuid;

    let mut target_mat = Material::unlit(display.clone(), false);
    target_mat.set_color([0.5, 0.5, 0.5]);
    target_mat.set_name("mat_target");
    let target_mat_id = target_mat.uuid;

    app_state.add_material(ground_mat);
    app_state.add_material(pick_mat_a);
    app_state.add_material(pick_mat_b);
    app_state.add_material(pick_mat_c);
    app_state.add_material(zone_mat);
    app_state.add_material(probe_mat);
    app_state.add_material(target_mat);

    // ---- Ground (visual only — no collider so ray-pick reports objects) ----
    {
        let mut ground = Object::cube(0.5);
        ground.set_name("ground_plate".to_string());
        ground.transform.set_position([0.0, -0.5, 0.0]);
        ground.transform.set_scale([40.0, 1.0, 40.0]);
        ground.add_material(ground_mat_id);
        ground.get_shapes_mut()[0].set_material_from_object_list(0);
        // Give it an AABB collider so ray-picks against the ground hit
        // something nameable.
        ground.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.5, -0.5, -0.5],
            [0.5, 0.5, 0.5],
        )));
        app_state.add_object(ground);
    }

    // ---- Pickable objects (demo 1) ----------------------------------------
    // A row of three objects in front of the camera — distinct shapes so the
    // user can click around and see different names in the HUD.
    {
        let mut obj = Object::cube(1.0);
        obj.set_name("pick_cube".to_string());
        obj.transform.set_position([-5.0, 1.0, 3.0]);
        obj.add_material(pick_mat_a_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-1.0, -1.0, -1.0],
            [1.0, 1.0, 1.0],
        )));
        app_state.add_object(obj);
    }
    {
        let r = 1.2_f32;
        let mut obj = Object::sphere(r, 16, 24);
        obj.set_name("pick_sphere".to_string());
        obj.transform.set_position([0.0, r, 3.0]);
        obj.add_material(pick_mat_b_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: r,
        });
        app_state.add_object(obj);
    }
    {
        let mut obj = Object::cube(0.7);
        obj.set_name("pick_pillar".to_string());
        obj.transform.set_position([5.0, 2.0, 3.0]);
        obj.transform.set_scale([1.0, 3.0, 1.0]);
        obj.add_material(pick_mat_c_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.7, -0.7, -0.7],
            [0.7, 0.7, 0.7],
        )));
        app_state.add_object(obj);
    }

    // ---- Overlap zone (demo 2) --------------------------------------------
    // Visual sphere placed near the origin; no collider on the zone itself so
    // its `ShapeOverlap` doesn't trivially hit itself. The test objects below
    // start inside its radius so the effect is visible the moment the demo
    // launches.
    let overlap_zone_id;
    {
        let mut zone = Object::sphere(1.6, 24, 32);
        zone.set_name("overlap_zone".to_string());
        zone.transform.set_position([-6.0, 1.6, -4.0]);
        zone.add_material(zone_mat_id);
        zone.get_shapes_mut()[0].set_material_from_object_list(0);
        // Intentionally no collider so the zone is purely a query volume.
        zone.set_collision(false);
        overlap_zone_id = zone.get_unique_id();
        app_state.add_object(zone);
    }

    // Two probe cubes inside the zone, one outside, so on launch the HUD
    // shows the zone triggered. (User can drag them by tweaking source if
    // they want; runtime dragging would need a follow-up feature.)
    {
        let mut obj = Object::cube(0.4);
        obj.set_name("in_zone_a".to_string());
        obj.transform.set_position([-6.2, 1.5, -4.2]);
        obj.add_material(pick_mat_a_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.4, -0.4, -0.4],
            [0.4, 0.4, 0.4],
        )));
        app_state.add_object(obj);
    }
    {
        let mut obj = Object::cube(0.4);
        obj.set_name("in_zone_b".to_string());
        obj.transform.set_position([-5.5, 2.0, -3.6]);
        obj.add_material(pick_mat_b_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.4, -0.4, -0.4],
            [0.4, 0.4, 0.4],
        )));
        app_state.add_object(obj);
    }
    {
        // A cube safely outside the zone — for visual reference.
        let mut obj = Object::cube(0.4);
        obj.set_name("out_of_zone".to_string());
        obj.transform.set_position([-2.0, 0.4, -4.0]);
        obj.add_material(pick_mat_c_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.4, -0.4, -0.4],
            [0.4, 0.4, 0.4],
        )));
        app_state.add_object(obj);
    }

    // ---- Shape sweep demo (demo 3) ----------------------------------------
    // Two static "target" cubes along the +X path of the swept probe sphere.
    let sweep_target_a_id;
    let sweep_target_b_id;
    {
        let mut obj = Object::cube(0.6);
        obj.set_name("sweep_target_A".to_string());
        obj.transform.set_position([2.0, 1.0, -8.0]);
        obj.add_material(target_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.6, -0.6, -0.6],
            [0.6, 0.6, 0.6],
        )));
        sweep_target_a_id = obj.get_unique_id();
        app_state.add_object(obj);
    }
    {
        let mut obj = Object::cube(0.6);
        obj.set_name("sweep_target_B".to_string());
        obj.transform.set_position([6.5, 1.0, -8.0]);
        obj.add_material(target_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-0.6, -0.6, -0.6],
            [0.6, 0.6, 0.6],
        )));
        sweep_target_b_id = obj.get_unique_id();
        app_state.add_object(obj);
    }

    // The probe — visual sphere; collider intentionally omitted so its own
    // ShapeCast doesn't self-intersect (collider is built fresh each frame in
    // the update function from the visual position).
    let sweep_probe_id;
    let probe_radius = 0.5_f32;
    let probe_start_x = -8.0_f32;
    let probe_end_x = 10.0_f32;
    {
        let mut obj = Object::sphere(probe_radius, 12, 16);
        obj.set_name("sweep_probe".to_string());
        obj.transform.set_position([probe_start_x, 1.0, -8.0]);
        obj.add_material(probe_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        // No collider so the probe isn't hit by overlap/cast queries against it.
        obj.set_collision(false);
        sweep_probe_id = obj.get_unique_id();
        app_state.add_object(obj);
    }

    // ---- State data --------------------------------------------------------
    app_state.add_state_data(
        SCENE_IDS_KEY,
        Box::new(SceneIds {
            overlap_zone: overlap_zone_id,
            sweep_probe: sweep_probe_id,
            sweep_target_a: sweep_target_a_id,
            sweep_target_b: sweep_target_b_id,
        }),
    );
    app_state.add_state_data(HUD_KEY, Box::new(Hud::default()));
    app_state.add_state_data(
        SWEEP_KEY,
        Box::new(SweepState {
            x: probe_start_x,
            start_x: probe_start_x,
            end_x: probe_end_x,
            speed: 3.5,
            radius: probe_radius,
        }),
    );

    // Camera speed for the fly-cam helpers.
    app_state.add_state_data("camera_move_speed", Box::new(10.0f32));
    app_state.add_state_data("camera_rotate_speed", Box::new(2.0f32));

    // ---- Input bindings ----------------------------------------------------
    // Free-fly cam: WASD on KeyPress (same as `init_default`), Space/Ctrl
    // for vertical, right-click + Ctrl for rotation. We do NOT call
    // `init_default` because it binds MousePress(Left) to object selection,
    // and we want that for the ray-pick demo instead.
    app_state.inject_event(
        EventCharacteristic::KeyPress(VirtualKeyCode::W),
        Arc::new(default_events::camera_fly_forward),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        EventCharacteristic::KeyPress(VirtualKeyCode::A),
        Arc::new(default_events::camera_fly_left),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        EventCharacteristic::KeyPress(VirtualKeyCode::S),
        Arc::new(default_events::camera_fly_backward),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        EventCharacteristic::KeyPress(VirtualKeyCode::D),
        Arc::new(default_events::camera_fly_right),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        EventCharacteristic::KeyPress(VirtualKeyCode::Space),
        Arc::new(default_events::camera_up),
        Some(EventModifiers::new(false, false, false)),
    );
    app_state.inject_event(
        EventCharacteristic::KeyDown(VirtualKeyCode::LControl),
        Arc::new(default_events::camera_down),
        None,
    );
    app_state.inject_event(
        EventCharacteristic::MouseDown(MouseButton::Right),
        Arc::new(default_events::camera_rotate),
        Some(EventModifiers::new(true, false, false)),
    );

    // Left-click → ray-pick.
    app_state.inject_event(
        EventCharacteristic::MousePress(MouseButton::Left),
        Arc::new(ray_pick_on_click),
        None,
    );

    // Per-frame updates.
    app_state.inject_update_function(Arc::new(update_overlap_zone));
    app_state.inject_update_function(Arc::new(update_sweep));

    // HUD.
    app_state.inject_gui(Arc::new(collision_hud));

    event_loop.run(app_state.convert_to_arc_mutex());
}
