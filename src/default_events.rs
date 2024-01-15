use nalgebra::Vector3;
use uuid::Uuid;
use crate::AppState;
use crate::collision_world::RayCast;

pub fn select_object(app_state: &mut AppState) {
    app_state.object_selection.clear();
    match app_state.camera {
        Some(camera) => {
            let world_space_mouse_position = app_state.get_mouse_position().get_world_position(&camera);
            let mut raycast = RayCast::new(
                world_space_mouse_position.0,
                world_space_mouse_position.1,
                100.0,
                true,
            );
            raycast.cast(app_state);
            for (id, _) in raycast.get_intersection_map().iter() {
                let mut ids: Vec<Uuid> = Vec::new();
                for object in app_state.get_objects_mut() {
                    if object.get_unique_id() == *id {
                        ids.push(object.get_unique_id());
                    }
                }
                app_state.object_selection = ids;
                println!("Selected object(s): {:?}", app_state.object_selection);
            }
        },
        None => {
            println!("No camera found to cast from, could not select object");
        }
    }
}