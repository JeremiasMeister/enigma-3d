use enigma::object::Object;
use enigma::camera::Camera;

fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma::EventLoop::new("Enigma 3D Renderer Window");
    let mut app_state = enigma::AppState::new();

    // create a default object
    let mut object = Object::load_from_gltf("res/models/suzanne.gltf", event_loop.get_display_clone());


    object.transform.set_position([0.0, 0.0, -2.0]);

    object.get_materials_mut()[0].set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);

    // adding all the objects
    app_state.add_object(object);

    // add lighting
    let light = enigma::light::Light {
        position: [1.0, 1.0, 5.0],
        color: [1.0, 1.0, 1.0],
        intensity: 100.0,
    };
    let ambient_light = enigma::light::Light {
        position: [0.0, 0.0, 0.0],
        color: [0.35, 0.35, 1.0],
        intensity: 0.20,
    };
    app_state.set_light(light, enigma::light::LightType::Point);
    app_state.set_light(ambient_light, enigma::light::LightType::Ambient);

    // add a camera
    let camera = Camera::new(Some([0.0, 1., 0.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16./9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // run the event loop
    event_loop.run(app_state);

}
