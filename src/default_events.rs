use nalgebra::Vector3;
use uuid::Uuid;
use crate::{AppState, smart_format};
use crate::collision_world::{Collider, RayCast, ShapeCast};
use crate::logging::{EnigmaError, EnigmaMessage};

pub fn select_object(app_state: &mut AppState){
    app_state.object_selection.clear();
    select_object_single(app_state);
}

pub fn select_object_add(app_state: &mut AppState){
    select_object_single(app_state);
}

fn select_object_single(app_state: &mut AppState) {
    match app_state.camera.as_ref() {
        Some(camera) => {
            let world_space_mouse_position = app_state.get_mouse_state().get_world_position(camera);
            let mut raycast = RayCast::new(
                world_space_mouse_position.0,
                world_space_mouse_position.1,
                100.0,
            );
            raycast.cast(app_state);
            let intersection_map = raycast.get_intersection_map();
            match intersection_map.last() {
                Some((id, _)) => {
                    let mut ids: Vec<Uuid> = app_state.object_selection.clone();
                    for object in app_state.get_objects_mut() {
                        if object.get_unique_id() == *id {
                            if !ids.contains(&object.get_unique_id()) {
                                ids.push(object.get_unique_id());
                                break;
                            }
                        }
                    }
                    app_state.object_selection = ids;
                    // we wanna print in the console that we selected an object but not write to disk to not bloat the log file
                    EnigmaMessage::new(Some(smart_format!("Selected Objects: {:?}", app_state.object_selection).as_str()), true).log()
                }
                None => ()
            }
        },
        None => {
            EnigmaError::new(Some("No camera found to cast from, could not select object"), true).log();
        }
    }
}

pub fn camera_fly_forward(app_state: &mut AppState) {
    let delta_time = app_state.delta_time;
    let speed = *app_state.get_state_data_value::<f32>("camera_move_speed").expect("failed to get camera speed from state data");
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.forward() * -speed * delta_time;
        camera.transform.move_dir_vector(direction);
    }
}

pub fn camera_fly_backward(app_state: &mut AppState) {
    let delta_time = app_state.delta_time;
    let speed = *app_state.get_state_data_value::<f32>("camera_move_speed").expect("failed to get camera speed from state data");
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.forward() * speed * delta_time;
        camera.transform.move_dir_vector(direction);
    }
}

pub fn camera_fly_left(app_state: &mut AppState) {
    let delta_time = app_state.delta_time;
    let speed = *app_state.get_state_data_value::<f32>("camera_move_speed").expect("failed to get camera speed from state data");
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.left() * speed * delta_time;
        camera.transform.move_dir_vector(direction);
    }
}

pub fn camera_fly_right(app_state: &mut AppState) {
    let delta_time = app_state.delta_time;
    let speed = *app_state.get_state_data_value::<f32>("camera_move_speed").expect("failed to get camera speed from state data");
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = camera.transform.left() * -speed * delta_time;
        camera.transform.move_dir_vector(direction);
    }
}

pub fn camera_up(app_state: &mut AppState){
    let delta_time = app_state.delta_time;
    let speed = *app_state.get_state_data_value::<f32>("camera_move_speed").expect("failed to get camera speed from state data");
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = Vector3::new(0.0,1.0,0.0) * speed * delta_time;
        camera.transform.move_dir_vector(direction);
    }
}

pub fn camera_down(app_state: &mut AppState){
    let delta_time = app_state.delta_time;
    let speed = *app_state.get_state_data_value::<f32>("camera_move_speed").expect("failed to get camera speed from state data");
    if let Some(camera) = app_state.get_camera_mut() {
        let direction = Vector3::new(0.0,1.0,0.0) * -speed * delta_time;
        camera.transform.move_dir_vector(direction);
    }
}

/// Ground-locked WASD movement — projects the camera forward/left vectors onto the XZ plane
/// so the player cannot gain or lose altitude through WASD while looking up or down.
pub fn camera_walk_forward(app_state: &mut AppState) {
    let dt = app_state.delta_time;
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(10.0);
    if let Some(cam) = app_state.camera.as_mut() {
        let f = cam.transform.forward();
        let len = (f.x * f.x + f.z * f.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([-f.x / len * speed * dt, 0.0, -f.z / len * speed * dt]);
        }
    }
}

pub fn camera_walk_backward(app_state: &mut AppState) {
    let dt = app_state.delta_time;
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(10.0);
    if let Some(cam) = app_state.camera.as_mut() {
        let f = cam.transform.forward();
        let len = (f.x * f.x + f.z * f.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([f.x / len * speed * dt, 0.0, f.z / len * speed * dt]);
        }
    }
}

pub fn camera_walk_left(app_state: &mut AppState) {
    let dt = app_state.delta_time;
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(10.0);
    if let Some(cam) = app_state.camera.as_mut() {
        let l = cam.transform.left();
        let len = (l.x * l.x + l.z * l.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([l.x / len * speed * dt, 0.0, l.z / len * speed * dt]);
        }
    }
}

pub fn camera_walk_right(app_state: &mut AppState) {
    let dt = app_state.delta_time;
    let speed = app_state.get_state_data_value::<f32>("camera_move_speed").copied().unwrap_or(10.0);
    if let Some(cam) = app_state.camera.as_mut() {
        let l = cam.transform.left();
        let len = (l.x * l.x + l.z * l.z).sqrt();
        if len > 0.001 {
            cam.transform.move_dir_array([-l.x / len * speed * dt, 0.0, -l.z / len * speed * dt]);
        }
    }
}

/// FPS mouse-look — applies raw mouse delta (pixels, not a rate) to yaw and pitch,
/// clamping pitch to ±90° to prevent flipping. Reads sensitivity from `camera_rotate_speed`
/// state data (default 0.002). Does not multiply by delta_time — mouse delta is already
/// frame-relative.
pub fn camera_fps_look(app_state: &mut AppState) {
    if app_state.modifiers.ctrl { return; }
    let delta = app_state.get_mouse_state().get_delta();
    let sensitivity = app_state.get_state_data_value::<f32>("camera_rotate_speed").copied().unwrap_or(0.002);
    if let Some(cam) = app_state.camera.as_mut() {
        cam.transform.rotation.y -= delta.0 as f32 * sensitivity;
        cam.transform.rotation.x -= delta.1 as f32 * sensitivity;
        cam.transform.rotation.x = cam.transform.rotation.x
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
}

pub fn camera_rotate(app_state: &mut AppState) {
    let mouse_delta = app_state.get_mouse_state().get_delta();
    let sensitivity = *app_state.get_state_data_value::<f32>("camera_rotate_speed").expect("failed to get camera rotate speed from state data") * app_state.delta_time;
    if let Some(camera) = app_state.get_camera_mut() {
        // Convert delta to radians and apply a sensitivity factor
        let (delta_yaw, delta_pitch) = (
            mouse_delta.0 as f32 * sensitivity,
            mouse_delta.1 as f32 * sensitivity
        );

        // Update camera rotation
        let mut rotation = camera.transform.rotation;

        // Yaw rotation (around Y-axis)
        rotation.y -= delta_yaw;

        // Pitch rotation (around X-axis)
        rotation.x -= delta_pitch;

        // Clamp pitch to prevent camera flipping
        rotation.x = rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        // Apply the new rotation
        camera.transform.rotation = rotation;
    }
}

// ---------------------------------------------------------------------------
// Kinematic character controller
// ---------------------------------------------------------------------------

/// State for a kinematic character controller. Stored in `app_state.state_data`
/// under the key [`CHARACTER_STATE_KEY`] (or a custom key if multiple characters).
///
/// The `capsule` is in *local* space — its world-space position is derived by
/// translating by the character object's transform every frame.
pub struct CharacterState {
    /// Object UUID this controller drives. Position is written to `object.transform`.
    pub object_id: Uuid,
    /// Capsule shape (local to the object's transform — typically a vertical capsule).
    pub capsule: Collider,
    pub velocity: Vector3<f32>,
    pub grounded: bool,
    pub move_speed: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
}

impl CharacterState {
    /// Construct a default controller for the given object. The default capsule is a
    /// 1.0m tall capsule with 0.4m radius, centered around y=1.0 in object space.
    pub fn new(object_id: Uuid) -> Self {
        Self {
            object_id,
            capsule: Collider::Capsule {
                a: Vector3::new(0.0, 0.5, 0.0),
                b: Vector3::new(0.0, 1.5, 0.0),
                radius: 0.4,
            },
            velocity: Vector3::zeros(),
            grounded: false,
            move_speed: 5.0,
            jump_velocity: 5.0,
            gravity: -20.0,
        }
    }
}

/// State-data key under which a `CharacterState` is stored on `AppState`.
pub const CHARACTER_STATE_KEY: &str = "character_state";

/// Translate a capsule's endpoints by `pos`. Used to lift a local-space capsule
/// into world space for collision queries. Non-capsule colliders are returned
/// unchanged.
fn translate_capsule(capsule: &Collider, pos: Vector3<f32>) -> Collider {
    if let Collider::Capsule { a, b, radius } = capsule {
        Collider::Capsule { a: a + pos, b: b + pos, radius: *radius }
    } else {
        capsule.clone()
    }
}

/// Overwrite horizontal (XZ) velocity to point in `world_dir` at `move_speed`.
/// If `world_dir` is near-zero, XZ velocity is zeroed.
fn add_horizontal_velocity(app_state: &mut AppState, world_dir: Vector3<f32>) {
    if let Some(cs) = app_state.get_state_data_value_mut::<CharacterState>(CHARACTER_STATE_KEY) {
        let dir = Vector3::new(world_dir.x, 0.0, world_dir.z);
        let n = dir.norm();
        if n > 1e-6 {
            let v = dir / n * cs.move_speed;
            cs.velocity.x = v.x;
            cs.velocity.z = v.z;
        } else {
            cs.velocity.x = 0.0;
            cs.velocity.z = 0.0;
        }
    }
}

/// Drive the character forward along the camera's view direction (ground-projected).
pub fn character_move_forward(app_state: &mut AppState) {
    let cam_fwd = app_state.camera.as_ref().map(|c| c.transform.forward());
    if let Some(f) = cam_fwd {
        // Camera "forward" actually points behind the camera in this engine (see
        // `camera_walk_forward`), so negate to get the look direction.
        add_horizontal_velocity(app_state, Vector3::new(-f.x, 0.0, -f.z));
    }
}

/// Drive the character backward along the camera's view direction (ground-projected).
pub fn character_move_backward(app_state: &mut AppState) {
    let cam_fwd = app_state.camera.as_ref().map(|c| c.transform.forward());
    if let Some(f) = cam_fwd {
        add_horizontal_velocity(app_state, Vector3::new(f.x, 0.0, f.z));
    }
}

/// Strafe left relative to the camera (ground-projected).
pub fn character_move_left(app_state: &mut AppState) {
    let cam_left = app_state.camera.as_ref().map(|c| c.transform.left());
    if let Some(l) = cam_left {
        add_horizontal_velocity(app_state, Vector3::new(l.x, 0.0, l.z));
    }
}

/// Strafe right relative to the camera (ground-projected).
pub fn character_move_right(app_state: &mut AppState) {
    let cam_left = app_state.camera.as_ref().map(|c| c.transform.left());
    if let Some(l) = cam_left {
        add_horizontal_velocity(app_state, Vector3::new(-l.x, 0.0, -l.z));
    }
}

/// Apply jump velocity, but only when the controller is grounded. Clears `grounded`
/// so a subsequent ground check has to re-establish it.
pub fn character_jump(app_state: &mut AppState) {
    if let Some(cs) = app_state.get_state_data_value_mut::<CharacterState>(CHARACTER_STATE_KEY) {
        if cs.grounded {
            cs.velocity.y = cs.jump_velocity;
            cs.grounded = false;
        }
    }
}

/// Per-frame integrator for the character controller:
/// 1. Apply gravity to vertical velocity.
/// 2. Sweep the capsule along `velocity * dt` with up to 3 cast-and-slide iterations.
///    On hit: advance to just before TOI, project remaining motion onto the contact
///    tangent (`v - (v·n) * n`), and kill velocity component into the surface.
/// 3. Re-check ground with a small (0.05m) downward sweep.
/// 4. Write position back to the character object's transform, and write velocity +
///    grounded back to `CharacterState`.
///
/// Register with `app_state.inject_update_function(Arc::new(character_update))`.
pub fn character_update(app_state: &mut AppState) {
    let dt = app_state.delta_time;

    // 1. Snapshot the bits of state we need so AppState is free to be borrowed
    //    mutably by ShapeCast::cast below. `grounded` is recomputed every frame by
    //    the downward sweep below, so we don't need the previous value here.
    let (mut velocity, capsule, object_id, gravity) = {
        let Some(cs) = app_state.get_state_data_value::<CharacterState>(CHARACTER_STATE_KEY) else {
            return;
        };
        (cs.velocity, cs.capsule.clone(), cs.object_id, cs.gravity)
    };

    // Integrate gravity.
    velocity.y += gravity * dt;
    let mut remaining = velocity * dt;

    // 2. Read current object position (immutable borrow on objects only).
    let mut pos = {
        let obj = app_state.objects.iter().find(|o| o.get_unique_id() == object_id);
        match obj {
            Some(o) => o.transform.get_position(),
            None => return,
        }
    };

    // 3. Iteratively cast + slide. Max 3 iterations to avoid infinite loops in corners.
    for _ in 0..3 {
        let dist = remaining.norm();
        if dist < 1e-5 { break; }
        let dir = remaining / dist;
        let world_capsule = translate_capsule(&capsule, pos);
        let cast = ShapeCast::new(world_capsule, dir, dist);
        match cast.cast(app_state) {
            Some(hit) if hit.toi < 1.0 => {
                // Advance to just before the hit.
                let advance = (hit.toi * dist - 1e-3).max(0.0);
                pos += dir * advance;
                // Project remaining motion along the contact tangent.
                let consumed = advance / dist;
                let left = remaining * (1.0 - consumed);
                let tangent = left - hit.normal * left.dot(&hit.normal);
                remaining = tangent;
                // Kill the velocity component pointing into the surface (so gravity
                // doesn't keep accumulating downward into the floor, etc.).
                velocity = velocity - hit.normal * velocity.dot(&hit.normal);
            }
            _ => {
                pos += remaining;
                break;
            }
        }
    }

    // 4. Ground check — a small downward sweep. "Grounded" means we hit something
    //    with an upward-facing normal.
    let world_capsule = translate_capsule(&capsule, pos);
    let down_cast = ShapeCast::new(world_capsule, Vector3::new(0.0, -1.0, 0.0), 0.05);
    let grounded = down_cast.cast(app_state).map_or(false, |h| h.normal.y > 0.5);

    // 5. Write back position to the object and velocity/grounded to CharacterState.
    if let Some(obj) = app_state.objects.iter_mut().find(|o| o.get_unique_id() == object_id) {
        obj.transform.set_position([pos.x, pos.y, pos.z]);
    }
    if let Some(cs) = app_state.get_state_data_value_mut::<CharacterState>(CHARACTER_STATE_KEY) {
        cs.velocity = velocity;
        cs.grounded = grounded;
    }
}