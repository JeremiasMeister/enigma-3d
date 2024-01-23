Enigma is my first attempt to do a little graphics API for Rust.
Please be aware, I'm not a professional graphics programmer, so the code is most likely butchering some conventions. I also don't take care of performance at the moment. that said I have the following features working

- Model loading from GLTF and OBJ
- Material, Shader, Shape, Object Abstractions
- PBR Shading
- Texturing, Normals and Vertex Colors
- up to 4 point lights per object
- one ambient light
- a Camera
- a simple Event system to inject functions and Keyboard presses into the event loop. Atm events get processed one by one in sequence
- a simple Update system to inject functions into the update loop. Atm, the functions get processes one by one in sequence
- Screen to World positions including a selection system
- Postprocessing
- Skybox and Sky reflections


the API is quite straight forward and easy to use, see the example below

![image](https://github.com/JeremiasMeister/enigma/assets/85162425/f6c09279-63f4-4e7f-81aa-277a62e42a66)

***
    // create an enigma eventloop and appstate
    let event_loop = enigma::EventLoop::new("Enigma 3D Renderer Window");
    let mut app_state = enigma::AppState::new();

    // some default event setups like selection
    app_state.set_renderscale(2);
    enigma::init_default(&mut app_state);

    let mut material = enigma::material::Material::lit_pbr(event_loop.display.clone());
    material.set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);

    // create a default object
    let mut object = Object::load_from_gltf("res/models/suzanne.gltf");

    // set the material
    object.add_material(material);
    object.get_shapes_mut()[0].set_material_from_object_list(0);

    object.name = "Suzanne".to_string();
    object.transform.set_position([0.0, 1.0, -2.0]);
    object.transform.set_scale([0.5, 0.5, 0.5]);

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
        intensity: 0.50,
    };
    app_state.set_light(light, enigma::light::LightType::Point);
    app_state.set_light(ambient_light, enigma::light::LightType::Ambient);

    // add a camera
    let camera = Camera::new(Some([0.0, 1.0, 1.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
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
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Space),
        Arc::new(spawn_object),
    );

    // add update
    app_state.inject_update_function(Arc::new(hopping_objects));

    // add post processing
    app_state.add_post_process(Box::new(GrayScale::new(&event_loop.display.clone())));

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
  ***
