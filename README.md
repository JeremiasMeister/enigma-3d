Enigma is my first attempt to do a little graphics API for Rust.
Please be aware, I'm not a professional graphics programmer, so the code is most likely butchering some conventions. I also don't take care of performance at the moment. that said I have the following features working

- Model loading from GLTF and OBJ
- Material, Shader, Shape, Object Abstractions
- PBR Shading
- Texturing, Normals and Vertex Colors
- one Point Light and one Ambient Light
- a Camera
- a simple Event system to add functions to Keyboard presses. Atm events get processed one by one in sequence

the API is quite straight forward and easy to use, see the example below

  ![image](https://github.com/JeremiasMeister/enigma/assets/19373094/0580fc5e-2cbe-42b5-ae50-f65346235c39)

***
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
    let camera = Camera::new(Some([0.0, 1., 0.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // add events
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::A),
        Arc::new(rotate_left),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::D),
        Arc::new(rotate_right),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::W),
        Arc::new(rotate_up),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::S),
        Arc::new(rotate_down),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::E),
        Arc::new(roll_right),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Q),
        Arc::new(roll_left),
    );

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
  ***
