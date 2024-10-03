use std::sync::Arc;
use enigma_3d::camera::Camera;
use enigma_3d::{AppState, example_resources, light, material, object, resources, smart_format};
use enigma_3d::event::EventCharacteristic;
use enigma_3d::light::LightEmissionType;
use enigma_3d::logging::{EnigmaMessage, EnigmaWarning};
use enigma_3d::material::TextureType;

pub fn debug_single_bone(app_state: &mut AppState){
    match app_state.get_object("knight") {
        Some(knight) => {
            let skel = knight.get_skeleton();
            match skel {
                Some(skeleton) => {
                    let bone = &skeleton.bones[5];
                    EnigmaMessage::new(Some(smart_format!("Bone Pos: {} ", bone.inverse_bind_pose).as_str()), true).log()
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
            let mut logger = EnigmaMessage::new(None, true);
            logger.extent("ANIMATION:\n__________________________________________________________________________");
            for anim in anims{
                logger.extent(smart_format!("Animation {} -> {}", anim.1.name, anim.1.channels.len()).as_str());
                for channel in &anim.1.channels{
                    logger.extent(smart_format!("Channel Bone ID: {}", channel.bone_id).as_str());
                }
            }
            logger.extent("SKELETON:\n__________________________________________________________________________");
            match knight.get_skeleton() {
                Some(skeleton) => {
                    for bone in &skeleton.bones {
                        logger.extent(smart_format!("Bone: {} -> Matrix: {} -> ID: {} -> Parent: {}", bone.name, bone.inverse_bind_pose, bone.id, bone.parent_id.unwrap_or_else(||0).to_string()).as_str());
                    }
                }
                None => ()
            }
            logger.log();
        }
        None => ()
    }
}

pub fn print_selected_objects(app_state: &mut AppState){
    for id in &app_state.object_selection{
        let obj = &app_state.get_object_by_uuid(&id);
        match obj {
            Some(o) => EnigmaMessage::new(Some(smart_format!("Selected {}", o.name).as_str()), true).log(),
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
    let camera = Camera::new(Some([0.0, 0.0, 1.0]), Some([0.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // load knight material
    let mut material = material::Material::lit_pbr(event_loop.get_display_clone(), false);
    material.set_name("knight_material");
    material.set_texture_from_resource(example_resources::skinned_knight_albedo(), TextureType::Albedo);
    material.set_texture_from_resource(example_resources::skinned_knight_normal(), TextureType::Normal);
    material.set_texture_from_resource(example_resources::skinned_knight_roughness(), TextureType::Roughness);

    let mut debug_mat = material::Material::unlit(event_loop.get_display_clone(), false);
    debug_mat.set_color([1.0,0.,0.]);

    // load knight model
    let mut knight = object::Object::load_from_gltf_resource(example_resources::skinned_knight());
    //match knight.try_fix_object() {
    //    Ok(_) => {},
    //    Err(e) => e.log()
    //}
    knight.set_name("knight".to_string());
    let scaler = 1.0;
    knight.transform.set_scale([scaler,scaler,scaler]);
    knight.transform.set_position([0.0,-0.38,0.0]);
    knight.add_material(material.uuid);

    let mut anim_logger = EnigmaWarning::new(None, true);
    for (anim, data) in knight.get_animations(){
        anim_logger.extent(&smart_format!("{}, {}", anim, data.channels.len()));
    }
    anim_logger.log();
    knight.play_animation("Armature|mixamo.com|Layer0", true);


    let mut base_loc = object::Object::default();
    base_loc.transform.set_scale([0.1,0.2,0.1]);
    base_loc.transform.set_position([0.0,0.0,0.0]);
    base_loc.transform.set_rotation([0.,180.,0.]);
    base_loc.add_shape(object::Shape::default());
    base_loc.add_material(debug_mat.uuid);
    app_state.add_object(base_loc);



    // create some lighting
    let main_light = light::Light::new([0.0,3.0,2.0], [1.0,1.0,1.0], 80.0, None, false);
    let fill_light = light::Light::new([-0.5,1.5,2.0], [1.0,1.0,1.0], 80.0, None, false);
    let ambient_light = light::Light::new([0.0,0.0,0.0], [1.0,1.0,1.0], 0.5, None, false);

    app_state.add_light(main_light, LightEmissionType::Source);
    app_state.add_light(fill_light, LightEmissionType::Source);
    app_state.add_light(ambient_light, LightEmissionType::Ambient);

    app_state.inject_event(EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::P), Arc::new(print_selected_objects), None);
    app_state.inject_event(EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::P), Arc::new(debug_single_bone), None);
    app_state.inject_event(EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::P), Arc::new(print_rig_data), None);

    app_state.add_material(material);
    app_state.add_material(debug_mat);
    app_state.add_object(knight);

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}