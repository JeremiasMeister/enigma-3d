enigma-3d is my first attempt to do a little graphics API and game engine for Rust.
Please be aware that I'm not a professional graphics programmer, so the code is most likely butchering some conventions. I also don't take care of performance at the moment. That said, I have the following features working:

### Feature List:
- Model loading from GLTF and OBJ
- Opaque and Transparent rendering
- `Material`, `Shader`, `Shape`, `Object` Abstractions
- PBR Shading
- 3 step customizable Render pipeline: Vertex -> Geometry -> Fragment 
- Texturing, Normals and Vertex Colors
- up to 4 point lights per object
- one ambient light
- a `Camera`
- a simple Event system to inject functions and Keyboard presses and KeyCode modifiers into the `EventLoop`. Atm events get processed one by one in sequence
- a simple Update system to inject functions into the update loop. Atm, the functions get processes one by one in sequence
- Screen to World positions, including a selection system
- Postprocessing
- Skybox and Sky reflections
- `egui` integration for a simple UI
- loading resources from the `include_bytes!` and `include_str!` macro to include them in the built application
- adding and carrying an arbitrary amount of data within the `AppState`
- serialize currently loaded `AppState` to json and inject serialized `AppState` into running one.
- simple Audio implementation to trigger oneshot or loop-able audio from resources or a file.
- Optimization: Textures are cached
- Optimization: Materials are shared in between Objects and managed via the `AppState`
- Optimisation: Cloned Objects are batched into one Draw Call via GPU Instancing 

### How to Install and Run:
Installing the Library in its latest release is quite straight forward, you should be able to just run `cargo add enigma-3d`. from there, you have access to the library for your codebase.
The current release should include most of the base features, but it might be a little outdated regarding optimization. I have enough for another release soon.

When it comes to running the examples they are hidden behind a feature flag due to its size, which would bloat the package. After cloning the repository onto your machine, running `cargo run --example=engine --features=example` or `cargo run --example=chessboard --features=example` should to the trick.
Cargo should take care of all the dependencies for you.

You can find a no-cut tutorial series on enigma-3d on my YouTube channel: https://youtube.com/playlist?list=PLheMI-GTxzlQQ8mYQQSXrSLxUziGqAHMp&si=usk-C3MHyYuPZCE2

**The Chessboard Example**
![image](https://github.com/user-attachments/assets/59d69fc6-e6ba-4854-a7bd-d266bd52855e)
**The Chessboard Example with a Geometry Grass shader for the ground and a Tree wind shader**
![image](https://github.com/user-attachments/assets/908f0264-b90c-42cf-8f99-f29317c8ee05)
**PBR Bloom postprocess and transparent objects**
![image](https://github.com/JeremiasMeister/enigma/assets/85162425/1d465331-c442-4c95-a472-ecfb9e58950c)
**Some more postprocessing in form of a black and white shader and a red outline instead of a black one**
![image](https://github.com/JeremiasMeister/enigma/assets/19373094/9003a00e-f52c-4692-b7b7-e387b780d456)

### Example Game:
A first little game, developed with enigma, can be found here: https://github.com/JeremiasMeister/enigma-flappy-bird
I just recently gave it an upgrade to the newest enigma-3d version, v0.2.13
![Screenshot 2025-06-10 075524](https://github.com/user-attachments/assets/c7c1ccc2-7dac-4e62-bfa0-c654e82e011b)



### engine.rs example, main function:
The API is quite straightforward and easy to use; see the example below.
```rust
fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma_3d::EventLoop::new("Enigma 3D Renderer Window", 1080, 720);
    let mut app_state = enigma_3d::AppState::new();

    // set the icon from the resources
    event_loop.set_icon_from_resource(resources::icon());

    // some default event setups like e.g. selection
    enigma_3d::init_default(&mut app_state);

    // create a material and assign the UV checker texture from resources
    let mut material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    material.set_texture_from_resource(example_resources::uv_checker(), enigma_3d::material::TextureType::Albedo);
    material.set_name("opaque_mat");

    let mut transparent_material = enigma_3d::material::Material::lit_pbr(event_loop.get_display_clone(), true);
    transparent_material.set_transparency_strength(0.2);
    transparent_material.set_texture_from_resource(example_resources::uv_checker(), enigma_3d::material::TextureType::Albedo);
    transparent_material.set_name("transparent_mat");


    // create an object, and load the Suzanne model from resources
    let mut object = Object::load_from_gltf_resource(example_resources::suzanne(), None);

    // set the material to the suzan object to the first shape (submesh) slot
    object.add_material(material.uuid);

    /// this step is optional, if you only have one shape in the object, but if you have multiple you might want to assign
    /// the material to the correct shape
    object.get_shapes_mut()[0].set_material_from_object_list(0);

    // set the name and position of the object
    object.name = "Suzanne".to_string();
    object.transform.set_position([0.0, 0.0, -2.0]);

    // adding the object to the app state
    app_state.add_object(object);

    //also add materials to appstate
    app_state.add_material(material);
    app_state.add_material(transparent_material);

    // create a bunch of lights
    let light1 = enigma_3d::light::Light::new([1.0, 1.0, 5.0], [0.0, 1.0, 0.0], 100.0, Some([1.0, 0.0, 0.0]), false);
    let light2 = enigma_3d::light::Light::new([5.0, 1.0, 1.0], [1.0, 0.0, 0.0], 100.0, None, false);
    let light3 = enigma_3d::light::Light::new([-5.0, 1.0, 1.0], [0.0, 0.0, 1.0], 100.0, None, false);
    let ambient_light = enigma_3d::light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.1, None, false);

    // add the lights to the app state
    app_state.add_light(light1, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light2, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(light3, enigma_3d::light::LightEmissionType::Source);
    app_state.add_light(ambient_light, enigma_3d::light::LightEmissionType::Ambient); // only one ambient light is supported atm

    // create and add a camera to the app state
    let camera = Camera::new(Some([0.0, 1.0, 1.0]), Some([20.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    // add events
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Left),
        Arc::new(rotate_left),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Right),
        Arc::new(rotate_right),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Up),
        Arc::new(rotate_up),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Down),
        Arc::new(rotate_down),
        None,
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Right),
        Arc::new(roll_right),
        Some(EventModifiers::new(true, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Left),
        Arc::new(roll_left),
        Some(EventModifiers::new(true, false, false)),
    );
    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::E),
        Arc::new(spawn_object),
        None,
    );

    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::S),
        Arc::new(save_app_state),
        Some(EventModifiers::new(true, false, false)),
    );

    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::O),
        Arc::new(load_app_state),
        Some(EventModifiers::new(true, false, false)),
    );

    app_state.inject_event(
        event::EventCharacteristic::KeyPress(event::VirtualKeyCode::N),
        Arc::new(reset),
        Some(EventModifiers::new(true, false, false)),
    );

    // add update functions
    app_state.inject_update_function(Arc::new(hopping_objects));
    app_state.inject_update_function(Arc::new(print_data));

    // add post processing effects
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::bloom::Bloom::new(&event_loop.display.clone(), 0.9, 15)));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::edge::Edge::new(&event_loop.display.clone(), 0.8, [1.0, 0.0, 0.0])));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::lens_dirt::LensDirt::new(&event_loop.display, resources::lens_dirt_texture(), 2.0, [800.0, 800.0], 2.0)));
    app_state.add_post_process(Box::new(enigma_3d::postprocessing::vignette::Vignette::new(&event_loop.display.clone(), 0.2, 0.5, [0.0, 0.0, 0.0], 0.8)));


    //add one ui function to the app state. multiple ui functions can be added modularly
    app_state.inject_gui(Arc::new(enigma_ui_function));


    // add some arbitrary state data. This can be used to store any kind of data in the app state
    // game globals, or other data that needs to be shared between different parts of the application
    app_state.add_state_data("intdata", Box::new(10i32));
    app_state.add_state_data("stringdata", Box::new("Hello World".to_string() as String));
    app_state.add_state_data("booldata", Box::new(true as bool));

    // adding some audio
    let background_sound = AudioClip::from_resource(example_resources::background_music(), "background_music");
    let spawn_click = AudioClip::from_resource(example_resources::click_sound(), "spawn_click");
    app_state.add_audio(background_sound);
    app_state.add_audio(spawn_click);
    // trigger background music
    app_state.play_audio_loop("background_music");


    // run the event loop
    event_loop.run(app_state.convert_to_arc_mutex());
}

```
