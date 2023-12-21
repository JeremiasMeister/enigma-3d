use enigma::{shader, material, geometry, EventLoop, object};
mod debug_geo;
use enigma::object::Shape;

fn default_object(event_loop: &EventLoop) -> object::Object {
    let object = object::Object::default(event_loop.get_display_clone());
    object
}

fn debug_shapes(event_loop: &EventLoop) -> object::Object {
    let mut object = object::Object::new(Some(String::from("Debug Shapes")));
    let verts = debug_geo::get_debug_shapes();
    let shape1 = Shape::from_vertices(verts[0].to_vec());
    let shape2 = Shape::from_vertices(verts[1].to_vec());
    let material1 = material::Material::lit_pbr(event_loop.get_display_clone());
    let material2 = material::Material::lit_pbr(event_loop.get_display_clone());
    object.add_shape(shape1, material1);
    object.add_shape(shape2, material2);
    object
}


fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma::EventLoop::new("Enigma 3D Renderer Window");
    let mut app_state = enigma::AppState::new();

    // create a default object
    let mut object = object::Object::load_from_obj("res/models/suzanne.obj", event_loop.get_display_clone(), Some(material::Material::lit_pbr(event_loop.get_display_clone())));
    let mut object2 = object::Object::load_from_obj("res/models/suzanne.obj", event_loop.get_display_clone(), Some(material::Material::lit_pbr(event_loop.get_display_clone())));
    let mut object3 = object::Object::load_from_obj("res/models/suzanne.obj", event_loop.get_display_clone(), Some(material::Material::lit_pbr(event_loop.get_display_clone())));
    let mut object4 = object::Object::load_from_obj("res/models/suzanne.obj", event_loop.get_display_clone(), Some(material::Material::lit_pbr(event_loop.get_display_clone())));
    let mut object5 = object::Object::load_from_obj("res/models/suzanne.obj", event_loop.get_display_clone(), Some(material::Material::lit_pbr(event_loop.get_display_clone())));

    object.transform.set_position([0.5, -0.5, 0.0]);
    object.transform.set_scale([0.2, 0.2, 0.2]);

    object2.transform.set_position([-0.5, -0.5, 0.0]);
    object2.transform.set_scale([0.2, 0.2, 0.2]);

    object3.transform.set_position([0.5, 0.5, 0.0]);
    object3.transform.set_scale([0.2, 0.2, 0.2]);

    object4.transform.set_position([-0.5, 0.5, 0.0]);
    object4.transform.set_scale([0.2, 0.2, 0.2]);

    object5.transform.set_position([0.0, 0.0, 0.0]);
    object5.transform.set_scale([0.2, 0.2, 0.2]);

    object.get_materials_mut()[0].set_color([1.0, 1.0, 0.5]);
    object.get_materials_mut()[0].set_roughness_strength(0.0);

    object2.get_materials_mut()[0].set_color([1.0, 0.5, 1.0]);
    object2.get_materials_mut()[0].set_roughness_strength(2.0);

    object3.get_materials_mut()[0].set_color([0.5, 1.0, 1.0]);
    object4.get_materials_mut()[0].set_color([1.0, 1.0, 1.0]);
    object5.get_materials_mut()[0].set_color([0.5, 0.5, 0.5]);

    // adding all the objects
    app_state.add_object(object);
    app_state.add_object(object2);
    app_state.add_object(object3);
    app_state.add_object(object4);
    app_state.add_object(object5);

    // add a light
    let light = enigma::light::Light {
        position: [2.0, 2.0, 0.0],
        color: [1.0, 1.0, 1.0],
        intensity: 10.0,
    };

    let ambient_light = enigma::light::Light {
        position: [0.0, 0.0, 0.0],
        color: [0.35, 0.35, 1.0],
        intensity: 0.20,
    };

    app_state.set_light(light, enigma::light::LightType::Point);
    app_state.set_light(ambient_light, enigma::light::LightType::Ambient);

    // run the event loop
    event_loop.run(app_state);
}
