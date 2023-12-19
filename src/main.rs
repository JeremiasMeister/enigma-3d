use enigma::{shader, material, geometry, EventLoop, object};
mod debug_geo;
use enigma::object::Shape;

fn default_object(event_loop: &EventLoop) -> object::Object {
    let object = object::Object::default(event_loop.get_display_clone());
    object
}

fn debug_shapes(event_loop: &EventLoop) -> object::Object {
    let mut object = object::Object::new();
    let verts = debug_geo::get_debug_shapes();
    let shape1 = Shape::from_vertices(verts[0].to_vec());
    let shape2 = Shape::from_vertices(verts[1].to_vec());
    let material1 = material::Material::lit_pbr(event_loop.get_display_clone());
    let material2 = material::Material::default(shader::Shader::default(), event_loop.get_display_clone());
    object.add_shape(shape1, material1);
    object.add_shape(shape2, material2);
    object
}


fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma::EventLoop::new("Enigma 3D Renderer Window");
    let mut app_state = enigma::AppState::new();

    // create a default object
    let mut object1 = default_object(&event_loop);
    let mut object2 = debug_shapes(&event_loop);
    let mut object3 = object::Object::load_from_obj("res/models/suzanne.obj", event_loop.get_display_clone(), Some(material::Material::lit_pbr(event_loop.get_display_clone())));

    object1.transform.set_position([0.5, 0.5, 0.0]);
    object1.transform.set_rotation([0.0, 0.0, 45.0]);
    object1.transform.set_scale([0.5, 0.5, 0.5]);

    object1.materials[0].set_color([0.0, 1.0, 0.0]);

    object2.transform.set_position([-0.5, -0.5, 0.0]);
    object2.transform.set_rotation([0.0, 0.0, -45.0]);
    object2.transform.set_scale([0.5, 0.5, 0.5]);


    object2.materials[0].set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);
    object2.materials[0].set_color([1.0, 0.0, 0.0]);

    object3.transform.set_position([0.0, 0.0, 0.0]);
    object3.transform.set_rotation([0.0, 0.0, 180.0]);
    object3.transform.set_scale([0.5, 0.5, 0.5]);

    // adding all the objects
    app_state.add_object(object1);
    app_state.add_object(object2);
    app_state.add_object(object3);


    // run the event loop
    event_loop.run(app_state);
}
