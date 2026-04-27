use std::sync::Arc;
use enigma_3d::{AppState, event, EventLoop, example_resources, init_default};
use enigma_3d::event::EventCharacteristic;
use enigma_3d::light::{Light, LightEmissionType};
use enigma_3d::material::{Material, TextureType};
use enigma_3d::object::Object;

pub fn debug_single_bone(app_state: &mut AppState){
    match app_state.get_object("Knight") {
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
    match app_state.get_object("Knight") {
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
    match app_state.get_object_mut("Knight") {
        Some(knight) => {
            match knight.get_current_animation() {
                Some(_) => knight.stop_animation(),
                None => knight.play_animation("Armature|mixamo.com|Layer0", true)
            }
        }
        None => ()
    }
}

fn main() {
    let event_loop = EventLoop::new("Skinned Mesh Rendering", 1080, 720);
    let mut app_state = AppState::new();
    init_default(&mut app_state);

    let mut knight = Object::load_from_gltf_resource(example_resources::skinned_knight(), None);
    knight.set_name("Knight".to_string());
    match knight.try_fix_object() {
        Ok(m) => m.log(),
        Err(e) => e.log()
    }
    knight.transform.set_position([0.0,-1.0,-2.0]);
    let mut mat = Material::lit_pbr(event_loop.get_display_clone(), false);
    mat.set_texture_from_resource(example_resources::skinned_knight_albedo(), TextureType::Albedo);
    mat.set_texture_from_resource(example_resources::skinned_knight_normal(), TextureType::Normal);
    mat.set_texture_from_resource(example_resources::skinned_knight_roughness(), TextureType::Roughness);
    knight.add_material(mat.uuid);

    app_state.add_material(mat);
    app_state.add_object(knight);


    let mut light = Light::default();
    let mut light2 = Light::default();
    light.intensity = 3f32;
    light2.intensity = 300f32;
    light2.position = [2f32, 3f32, 0f32];
    app_state.add_light(light, LightEmissionType::Ambient);
    app_state.add_light(light2, LightEmissionType::Source);

    // functions
    app_state.inject_event(EventCharacteristic::KeyPress(event::VirtualKeyCode::P),Arc::new(print_rig_data), None);
    app_state.inject_event(EventCharacteristic::KeyPress(event::VirtualKeyCode::X), Arc::new(toggle_animation), None);


    event_loop.run(app_state.convert_to_arc_mutex());
}