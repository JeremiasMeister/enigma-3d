use std::sync::Arc;
use enigma_3d::camera::Camera;
use enigma_3d::{AppState, example_resources, light, material, object, resources};
use enigma_3d::event::EventCharacteristic;
use enigma_3d::light::LightEmissionType;
use enigma_3d::material::TextureType;

pub fn debug_single_bone(app_state: &mut AppState){
    match app_state.get_object("knight") {
        Some(knight) => {
            let skel = knight.get_skeleton();
            match skel {
                Some(skeleton) => {
                    let bone = &skeleton.bones[5];
                    println!("Bone Pos: {} ", bone.inverse_bind_pose);
                }
                None => ()
            }
        }
        None => ()
    }
}

pub fn print_rig_data(app_state: &mut AppState) {
    match app_state.get_object("knight") {
        Some(knight) => {
            let anims = knight.get_animations();
            println!("ANIMATION:\n__________________________________________________________________________{}","");
            for anim in anims{
                println!("Animation {} -> {}", anim.1.name, anim.1.channels.len());
                for channel in &anim.1.channels{
                    println!("Channel Bone ID: {}", channel.bone_id)
                }
            }
            println!("SKELETON:\n__________________________________________________________________________{}","");
            match knight.get_skeleton() {
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

pub fn print_selected_objects(app_state: &mut AppState){
    for id in &app_state.object_selection{
        let obj = &app_state.get_object_by_uuid(&id);
        match obj {
            Some(o) => println!("Selected: {}", o.name),
            None => ()
        }
    }
}

fn main() {
    let event_loop = enigma_3d::EventLoop::new("Enigma 3D Skinned Mesh Example", 1080, 720);
    let mut app_state = enigma_3d::AppState::new();
    // set the icon from the resources
    event_loop.set_icon_from_resource(resources::icon());
    // some default event setups like e.g. selection
    enigma_3d::init_default(&mut app_state);

    // create and add a camera to the app state
    let camera = Camera::new(Some([0.0, 1.5, 1.0]), Some([-20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // load knight material
    let mut material = material::Material::lit_pbr(event_loop.get_display_clone(), false);
    material.set_name("knight_material");
    material.set_texture_from_resource(example_resources::skinned_knight_albedo(), TextureType::Albedo);
    material.set_texture_from_resource(example_resources::skinned_knight_normal(), TextureType::Normal);
    material.set_texture_from_resource(example_resources::skinned_knight_roughness(), TextureType::Roughness);

    // load knight model
    let mut knight = object::Object::load_from_gltf_resource(example_resources::skinned_knight(), None);
    knight.try_fix_object().expect("failed to fix object");
    knight.set_name("knight".to_string());
    let scaler = 1.0;
    knight.transform.set_scale([scaler,scaler,scaler]);
    knight.add_material(material.uuid);

    for (anim, _) in knight.get_animations(){
        println!("{}", anim);
    }

    knight.play_animation("MyAnimation", true);


    // create some lighting
    let main_light = light::Light::new([0.0,3.0,2.0], [1.0,1.0,1.0], 80.0, None, false);
    let fill_light = light::Light::new([-0.5,1.5,2.0], [1.0,1.0,1.0], 80.0, None, false);
    let ambient_light = light::Light::new([0.0,0.0,0.0], [1.0,1.0,1.0], 0.5, None, false);

    app_state.add_light(main_light, LightEmissionType::Source);
    app_state.add_light(fill_light, LightEmissionType::Source);
    app_state.add_light(ambient_light, LightEmissionType::Ambient);

    app_state.inject_event(EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::P), Arc::new(print_rig_data), None);
    app_state.inject_event(EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::P), Arc::new(print_selected_objects), None);

    //app_state.inject_update_function(Arc::new(debug_single_bone));

    app_state.add_material(material);
    app_state.add_object(knight);

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}