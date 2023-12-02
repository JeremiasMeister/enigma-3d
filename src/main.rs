mod shader;
mod material;
mod geometry;
mod texture;
mod debug_geo;

use glium::{Surface, uniform, VertexBuffer};
use crate::geometry::Vertex;



fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build();
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Enigma - Render 3D")
        .build(&event_loop);

    let shader = shader::Shader::from_files("res/shader/enigma_vertex_shader.glsl", "res/shader/enigma_fragment_shader.glsl");
    let program = glium::Program::from_source(&display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), None).expect("Failed to compile shader program");


    // TODO: collect the actual model data later
    let shapes = debug_geo::get_debug_shapes();

    let vertex_buffers: Vec<VertexBuffer<Vertex>> = shapes.iter().map(|shape| {
        VertexBuffer::new(&display, shape).unwrap()
    }).collect();
    let indices: glium::index::NoIndices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    //define shader uniforms
    let mut time: f32 = 0.0;
    let matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0 , 0.0, 0.0, 1.0f32],
    ];


    event_loop.run(move |event, _, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                // This event is sent by the OS when you close the Window, or request the program to quit via the taskbar.
                winit::event::WindowEvent::CloseRequested => window_target.set_exit(),
                _ => (),
            },
            winit::event::Event::RedrawEventsCleared => {
                _window.request_redraw();
            },
            winit::event::Event::RedrawRequested(_) => {
                // Update the time uniform
                time += 0.001;
                let uniforms = glium::uniform! {
                    time: time,
                    matrix: matrix,
                };
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                for buffer in vertex_buffers.iter() {
                    target.draw(buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
                }
                target.finish().unwrap();
            },
            _ => (),
        }
    });
}
