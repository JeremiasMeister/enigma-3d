//! Kinematic character controller demo.
//!
//! WASD to move, space to jump, mouse to look. The player is a capsule;
//! the world is a ground plane plus a few static box obstacles.
//!
//! The camera is a separate FPS flycam (mouse-look only — does NOT follow the
//! capsule). This is intentional for a minimal v1 demo: it lets you orbit and
//! verify the capsule behaves correctly under gravity, jump, and sliding
//! collision without needing a follow-cam implementation.
//!
//! Run with: `cargo run --example=character_controller --features=examples`
use std::sync::Arc;

use enigma_3d::camera::Camera;
use enigma_3d::collision_world::Collider;
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
use enigma_3d::object::Object;
use enigma_3d::{AppState, EventLoop};
use nalgebra::Vector3;

fn main() {
    // ---- Window + AppState bootstrap ----------------------------------------
    let event_loop = EventLoop::new("Character Controller Demo", 1280, 720);
    let mut app_state = AppState::new();

    // FPS-style state-data the default-events helpers read.
    app_state.add_state_data("camera_move_speed", Box::new(10.0f32));
    app_state.add_state_data("camera_rotate_speed", Box::new(0.002f32));

    // ---- Camera -------------------------------------------------------------
    // Set the camera up and back so the player capsule is visible in front.
    let camera = Camera::new(
        Some([0.0, 3.0, 8.0]),
        Some([0.0, 0.0, 0.0]),
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

    // ---- Static box obstacles ------------------------------------------------
    // Three AABB boxes arranged in a row in front of the spawn point. The
    // collider extents are local-space; world position comes from the object's
    // transform.
    for (i, x) in [-3.0_f32, 0.0, 3.0].iter().enumerate() {
        let mut box_obj = Object::new(Some(format!("box_{}", i)));
        box_obj.transform.set_position([*x, 0.5, 4.0]);
        box_obj.set_collider(Collider::Aabb(BoundingBox::new(
            [-1.0, -0.5, -1.0],
            [1.0, 0.5, 1.0],
        )));
        app_state.add_object(box_obj);
    }

    // ---- Ground plane --------------------------------------------------------
    // A Plane collider catches gravity. The plane is implicitly infinite, so a
    // single object covers the whole floor.
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
