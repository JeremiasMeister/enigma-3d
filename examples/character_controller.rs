//! Kinematic character controller — collider showcase room.
//!
//! WASD to move, space to jump, mouse to look. The player is a capsule. The
//! scene contains one obstacle of every collider primitive enigma-3d supports,
//! arranged in a rough circle around the spawn point so you can walk into each
//! one and verify the sliding behaviour:
//!
//!   * AABB box (axis-aligned)
//!   * Sphere
//!   * Rotated OBB (oriented 30° around Y)
//!   * Mesh collider (triangle soup built from the visual mesh)
//!   * Heightfield (a small terrain patch off to the side)
//!   * Infinite plane ground (the fallback floor)
//!
//! Visuals here are intentionally crude — bright unlit colours so each
//! obstacle is easy to identify, not pretty. The point of the example is the
//! collision behaviour.
//!
//! Run with: `cargo run --example=character_controller --features=examples`
use std::sync::Arc;

use enigma_3d::camera::Camera;
use enigma_3d::collision_world::{Collider, MeshCollider};
use enigma_3d::default_events::{
    self,
    character_jump,
    character_move_backward,
    character_move_forward,
    character_move_left,
    character_move_right,
    character_update,
    CharacterState,
    CHARACTER_STATE_KEY,
};
use enigma_3d::event::{EventCharacteristic, VirtualKeyCode};
use enigma_3d::geometry::BoundingBox;
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::Material;
use enigma_3d::object::Object;
use enigma_3d::terrain::{Terrain, TerrainConfig};
use enigma_3d::{AppState, EventLoop};
use nalgebra::{UnitQuaternion, Vector3};

fn main() {
    // ---- Window + AppState bootstrap ----------------------------------------
    let event_loop = EventLoop::new("Character Controller — Collider Showcase", 1280, 720);
    let mut app_state = AppState::new();

    // FPS-style state-data the default-events helpers read.
    app_state.add_state_data("camera_move_speed", Box::new(10.0f32));
    app_state.add_state_data("camera_rotate_speed", Box::new(0.002f32));

    // ---- Camera -------------------------------------------------------------
    // Up and back so the spawn area + all obstacles fit in the initial view.
    let camera = Camera::new(
        Some([0.0, 6.0, 14.0]),
        Some([-15.0, 0.0, 0.0]),
        Some(75.0),
        Some(16.0 / 9.0),
        Some(0.01),
        Some(1024.0),
    );
    app_state.set_camera(camera);

    // ---- Lights -------------------------------------------------------------
    let key_light = Light::new([5.0, 8.0, 5.0], [1.0, 0.97, 0.9], 200.0, None, false);
    let fill_light = Light::new([-4.0, 4.0, 2.0], [0.6, 0.7, 1.0], 80.0, None, false);
    let ambient = Light::new([0.0, 0.0, 0.0], [0.5, 0.55, 0.65], 0.4, None, false);
    app_state.add_light(key_light, LightEmissionType::Source);
    app_state.add_light(fill_light, LightEmissionType::Source);
    app_state.add_light(ambient, LightEmissionType::Ambient);

    // ---- Bright unlit materials for each obstacle ----------------------------
    // One colour per collider type so they're easy to tell apart at a glance.
    let display = event_loop.get_display_clone();

    let mut aabb_mat = Material::unlit(display.clone(), false);
    aabb_mat.set_color([0.95, 0.35, 0.30]); // red
    aabb_mat.set_name("mat_aabb");
    let aabb_mat_id = aabb_mat.uuid;

    let mut sphere_mat = Material::unlit(display.clone(), false);
    sphere_mat.set_color([0.30, 0.85, 0.45]); // green
    sphere_mat.set_name("mat_sphere");
    let sphere_mat_id = sphere_mat.uuid;

    let mut obb_mat = Material::unlit(display.clone(), false);
    obb_mat.set_color([0.35, 0.55, 0.95]); // blue
    obb_mat.set_name("mat_obb");
    let obb_mat_id = obb_mat.uuid;

    let mut mesh_mat = Material::unlit(display.clone(), false);
    mesh_mat.set_color([0.95, 0.85, 0.30]); // yellow
    mesh_mat.set_name("mat_mesh");
    let mesh_mat_id = mesh_mat.uuid;

    app_state.add_material(aabb_mat);
    app_state.add_material(sphere_mat);
    app_state.add_material(obb_mat);
    app_state.add_material(mesh_mat);

    // ---- Player object -------------------------------------------------------
    // The capsule collider is in object-local space — the character_update
    // function translates it into world space using the object's transform
    // each frame.
    let mut player = Object::new(Some("player".to_string()));
    player.transform.set_position([0.0, 5.0, 0.0]);
    player.set_collider(Collider::Capsule {
        a: Vector3::new(0.0, 0.5, 0.0),
        b: Vector3::new(0.0, 1.5, 0.0),
        radius: 0.4,
    });
    let player_id = player.get_unique_id();
    app_state.add_object(player);

    // ---- Obstacle 1: AABB box ------------------------------------------------
    // Axis-aligned box at (-4, 0.5, 3). Visual cube + matching AABB collider.
    {
        let mut obj = Object::cube(1.0); // half-extent 1.0 → 2×2×2 cube
        obj.set_name("obstacle_aabb".to_string());
        obj.transform.set_position([-4.0, 1.0, 3.0]);
        obj.add_material(aabb_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-1.0, -1.0, -1.0],
            [1.0, 1.0, 1.0],
        )));
        app_state.add_object(obj);
    }

    // ---- Obstacle 2: Sphere --------------------------------------------------
    // Sphere mesh + Collider::Sphere of matching radius.
    {
        let radius = 1.2_f32;
        let mut obj = Object::sphere(radius, 16, 24);
        obj.set_name("obstacle_sphere".to_string());
        obj.transform.set_position([0.0, radius, 5.0]);
        obj.add_material(sphere_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        obj.set_collider(Collider::Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius,
        });
        app_state.add_object(obj);
    }

    // ---- Obstacle 3: Rotated OBB --------------------------------------------
    // Visual cube rotated 30° around Y, with a matching Collider::Obb. AABB
    // colliders bloat under rotation; OBB tracks the rotated bounds exactly.
    {
        let mut obj = Object::cube(1.0);
        obj.set_name("obstacle_obb".to_string());
        obj.transform.set_position([4.0, 1.0, 3.0]);
        obj.transform.set_rotation([0.0, 30.0, 0.0]); // degrees → radians inside
        obj.add_material(obb_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        // Local-space OBB centered on the cube. `to_world_space` composes this
        // with the owning object's transform every frame, so the rotation here
        // can be identity — the obstacle's own rotation rotates the collider.
        obj.set_collider(Collider::Obb {
            center: Vector3::new(0.0, 0.0, 0.0),
            half_extents: Vector3::new(1.0, 1.0, 1.0),
            rotation: UnitQuaternion::identity(),
        });
        app_state.add_object(obj);
    }

    // ---- Obstacle 4: Mesh collider ------------------------------------------
    // Same visual as the sphere obstacle, but the collider is a triangle-soup
    // mesh derived from the visual geometry. This is the expensive but exact
    // option — useful for non-convex hand-authored geometry.
    {
        let radius = 1.0_f32;
        let mut obj = Object::sphere(radius, 12, 16);
        obj.set_name("obstacle_mesh".to_string());
        obj.transform.set_position([-3.0, radius, -3.0]);
        obj.add_material(mesh_mat_id);
        obj.get_shapes_mut()[0].set_material_from_object_list(0);
        // Build the mesh collider from the visual mesh, then assign it.
        let mesh_collider = MeshCollider::from_object_mesh(&obj);
        obj.set_collider(Collider::Mesh(mesh_collider));
        app_state.add_object(obj);
    }

    // ---- Obstacle 5: Heightfield (small terrain patch) -----------------------
    // A small terrain off to the player's left. The `Collider::Heightfield`
    // marker on the terrain's owning side tells queries to dispatch through
    // `AppState::get_terrain()` for height/normal sampling. (Terrain is set
    // directly on AppState; the `Heightfield` collider lives on objects that
    // want to register interest — here we don't need a wrapper object since
    // the character controller queries the terrain directly via app_state.)
    let terrain = Terrain::new(
        &display,
        TerrainConfig {
            width: 16.0,
            depth: 16.0,
            resolution: 64,
            max_height: 2.5,
            noise_scale: 0.18,
            ..TerrainConfig::default()
        },
    );
    let mut terrain = terrain;
    // Offset the terrain so it sits next to the spawn point rather than
    // overlapping the infinite ground plane.
    terrain.set_position([14.0, 0.0, 0.0]);
    app_state.set_terrain(terrain);

    // ---- Ground plane --------------------------------------------------------
    // A Plane collider catches gravity for the main play area. The plane is
    // implicitly infinite, so a single object covers the whole floor outside
    // the terrain patch.
    let mut ground = Object::new(Some("ground".to_string()));
    ground.set_collider(Collider::Plane {
        normal: Vector3::new(0.0, 1.0, 0.0),
        offset: 0.0,
    });
    app_state.add_object(ground);

    // ---- Character controller state -----------------------------------------
    let cs = CharacterState::new(player_id);
    app_state.add_state_data(CHARACTER_STATE_KEY, Box::new(cs));

    // ---- Input bindings ------------------------------------------------------
    // KeyDown = held (fires every frame while held) — matches the input
    // accumulation pattern in character_*_move_* functions.
    app_state.inject_event(
        EventCharacteristic::KeyDown(VirtualKeyCode::W),
        Arc::new(character_move_forward),
        None,
    );
    app_state.inject_event(
        EventCharacteristic::KeyDown(VirtualKeyCode::S),
        Arc::new(character_move_backward),
        None,
    );
    app_state.inject_event(
        EventCharacteristic::KeyDown(VirtualKeyCode::A),
        Arc::new(character_move_left),
        None,
    );
    app_state.inject_event(
        EventCharacteristic::KeyDown(VirtualKeyCode::D),
        Arc::new(character_move_right),
        None,
    );
    // Jump is KeyPress (single edge) so holding space doesn't auto-jump.
    app_state.inject_event(
        EventCharacteristic::KeyPress(VirtualKeyCode::Space),
        Arc::new(character_jump),
        None,
    );

    // ---- Update functions ----------------------------------------------------
    // Order matters: mouse-look runs first so the camera forward used by the
    // character move functions reflects the player's current view direction.
    app_state.inject_update_function(Arc::new(default_events::camera_fps_look));
    app_state.inject_update_function(Arc::new(character_update));

    app_state.cursor_locked = true;

    event_loop.run(app_state.convert_to_arc_mutex());
}
