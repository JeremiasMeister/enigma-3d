use uuid::Uuid;
use crate::AppState;
use crate::collision_world::RayCast;

pub fn select_object(app_state: &mut AppState){
    app_state.object_selection.clear();
    select_object_single(app_state);
}

pub fn select_object_add(app_state: &mut AppState){
    select_object_single(app_state);
}

fn select_object_single(app_state: &mut AppState) {
    match app_state.camera {
        Some(camera) => {
            let world_space_mouse_position = app_state.get_mouse_state().get_world_position(&camera);
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
                    println!("Selected Objects: {:?}", app_state.object_selection)
                }
                None => ()
            }
        },
        None => {
            println!("No camera found to cast from, could not select object");
        }
    }
}