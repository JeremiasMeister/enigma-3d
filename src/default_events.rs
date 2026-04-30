use nalgebra::Vector3;
use uuid::Uuid;
use crate::{AppState, smart_format};
use crate::collision_world::RayCast;
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