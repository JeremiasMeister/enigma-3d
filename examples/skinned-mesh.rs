use std::sync::Arc;
use egui::Key::N;
use winit::event::VirtualKeyCode;
use enigma_3d::camera::Camera;
use enigma_3d::{AppState, EventLoop, example_resources, init_default, light, material, object, resources};
use enigma_3d::event::EventCharacteristic;
use enigma_3d::light::LightEmissionType;
use enigma_3d::logging::{EnigmaError, EnigmaMessage};
use enigma_3d::material::{Material, TextureType};
use enigma_3d::object::Object;

pub fn debug_single_bone(app_state: &mut AppState){
    match app_state.get_object("Wiggle") {
        Some(wiggle) => {
            let skel = wiggle.get_skeleton();
            match skel {
                Some(skeleton) => {
                    let bone = &skeleton.bones[2];
                    println!("Bone Pos: {} ", bone.inverse_bind_pose);
                }
                None => ()
            }
        }
        None => ()
    }
}

pub fn print_rig_data(app_state: &mut AppState) {
    match app_state.get_object("Wiggle") {
        Some(wiggle) => {
            let anims = wiggle.get_animations();
            println!("ANIMATION:\n__________________________________________________________________________{}","");
            for anim in anims{
                println!("Animation {} -> {}", anim.1.name, anim.1.channels.len());
                for channel in &anim.1.channels{
                    println!("Channel Bone ID: {}, keys {}", channel.bone_id, channel.keyframes.len())
                }
            }
            println!("SKELETON:\n__________________________________________________________________________{}","");
            match wiggle.get_skeleton() {
                Some(skeleton) => {
                    for bone in &skeleton.bones {
                        println!("Bone: {} -> Matrix: {} -> ID: {} -> Parent: {}", bone.name, bone.inverse_bind_pose, bone.id, bone.parent_id.unwrap_or_else(||0).to_string());
                    }
                }
                None => ()
            }
        }
        None => ()
    }
}

fn toggle_animation(app_state: &mut AppState){
    match app_state.get_object_mut("Wiggle") {
        Some(wiggle) => {
            match wiggle.get_current_animation() {
                Some(_) => wiggle.stop_animation(),
                None => wiggle.play_animation("Wiggle", true)
            }
        }
        None => ()
    }
}

fn main() {
    let event_loop = EventLoop::new("Skinned Mesh Rendering", 1080, 720);
    let mut app_state = AppState::new();
    init_default(&mut app_state);

    let mut wiggle = Object::load_from_gltf_resource(example_resources::skinned_wiggle(), None);
    wiggle.set_name("Wiggle".to_string());
    match wiggle.try_fix_object() {
        Ok(m) => m.log(),
        Err(e) => e.log()
    }
    wiggle.transform.set_position([0.0,-1.0,-2.0]);
    let mat = Material::unlit(event_loop.get_display_clone(), false);
    wiggle.add_material(mat.uuid);

    app_state.add_material(mat);
    app_state.add_object(wiggle);

    // functions
    app_state.inject_event(EventCharacteristic::KeyPress(VirtualKeyCode::P),Arc::new(print_rig_data), None);
    app_state.inject_event(EventCharacteristic::KeyPress(VirtualKeyCode::X), Arc::new(toggle_animation), None);


    event_loop.run(app_state.convert_to_arc_mutex());
}