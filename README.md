enigma is my first attempt to do a little graphics API for Rust.
Please be aware that I'm not a professional graphics programmer, so the code is most likely butchering some conventions. I also don't take care of performance at the moment. That said, I have the following features working.

- Model loading from GLTF and OBJ
- Opaque and Transparent rendering
- Material, Shader, Shape, Object Abstractions
- PBR Shading
- Texturing, Normals and Vertex Colors
- up to 4 point lights per object
- one ambient light
- a Camera
- a simple Event system to inject functions and Keyboard presses into the event loop. Atm events get processed one by one in sequence
- a simple Update system to inject functions into the update loop. Atm, the functions get processes one by one in sequence
- Screen to World positions, including a selection system
- Postprocessing
- Skybox and Sky reflections
- egui integration for a simple UI
- loading resources from the 'include_bytes!' macro to include them in the built application
- adding and carrying an arbitrary amount of data within the `app_state`


A first little game, developed with enigma, can be found here: https://github.com/JeremiasMeister/enigma-flappy-bird


The API is quite straightforward and easy to use; see the example below.

**PBR Bloom postprocess and transparent objects**
![image](https://github.com/JeremiasMeister/enigma/assets/85162425/1d465331-c442-4c95-a472-ecfb9e58950c)
**Also added an outline postprocess as an example how to handle the depth buffer in postprocessing**
![image](https://github.com/JeremiasMeister/enigma/assets/19373094/75aac3e0-50d9-42cf-b896-b727289189e9)
**Some more postprocessing in form of a black and white shader and a red outline instead of a black one**
![image](https://github.com/JeremiasMeister/enigma/assets/19373094/9003a00e-f52c-4692-b7b7-e387b780d456)

***
    // create an enigma event loop and app state
    let event_loop = enigma::EventLoop::new("Enigma 3D Renderer Window");
    let mut app_state = enigma::AppState::new();

    //Some default event setups like selection
    enigma::init_default(&mut app_state);

    let mut material = enigma::material::Material::lit_pbr(event_loop.display.clone());
    material.set_texture_from_file("res/textures/uv_checker.png", enigma::material::TextureType::Albedo);

    //Create a default object
    let mut object = Object::load_from_gltf("res/models/suzanne.gltf");

    // set the material
    object.add_material(material);
    object.get_shapes_mut()[0].set_material_from_object_list(0);

    object.name = "Suzanne".to_string();
    object.transform.set_position([0.0, 0.0, -2.0]);

    // adding all the objects
    app_state.add_object(object);

    // add lighting
    let light1 = enigma::light::Light {
        position: [1.0, 1.0, 5.0],
        color: [0.0, 1.0, 0.0],
        intensity: 100.0,
    };
    let light2 = enigma::light::Light {
        position: [5.0, 1.0, 1.0],
        color: [1.0, 0.0, 0.0],
        intensity: 100.0,
    };
    let light3 = enigma::light::Light {
        position: [0.0, 1.0, 5.0],
        color: [0.0, 0.0, 1.0],
        intensity: 100.0,
    };
    let ambient_light = enigma::light::Light {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 1.0, 1.0],
        intensity: 0.10,
    };
    app_state.add_light(light1, enigma::light::LightType::Point);
    app_state.add_light(light2, enigma::light::LightType::Point);
    app_state.add_light(light3, enigma::light::LightType::Point);

    app_state.add_light(ambient_light, enigma::light::LightType::Ambient);

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

    // add post-processing
    //app_state.add_post_process(Box::new(GrayScale::new(&event_loop.display.clone())));
    app_state.add_post_process(Box::new(Bloom::new(&event_loop.display.clone(), 0.9, 15)));

    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
  ***
