use nalgebra::Vector3;
use crate::AppState;
use crate::collision_world::RayCast;

pub fn select_object(app_state: &mut AppState) {
    match app_state.camera {
        Some(camera) => {
            let world_space_mouse_position = app_state.get_mouse_position().get_world_position(&camera, camera.near);
            let direction_vector = Vector3::from(camera.calculate_direction_vector());
            println!("Mouse position: {:?}", world_space_mouse_position);
            println!("Direction vector: {:?}", direction_vector);
            let mut raycast = RayCast::new(
                world_space_mouse_position,
                direction_vector,
                100.0,
                true,
            );
            raycast.cast(app_state);
            for (id, _) in raycast.get_intersection_map().iter() {
                for object in app_state.get_objects_mut() {
                    if object.get_unique_id() == *id {
                        println!("Selected object: {}", object.name)
                    }
                }
            }
        },
        None => {
            println!("No camera found to cast from, could not select object");
        }
    }
}