mod shader;
mod material;
mod geometry;
mod texture;
mod debug_geo;

use glium::{Surface, VertexBuffer};
use crate::geometry::Vertex;


fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build();
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Enigma - Render 3D")
        .build(&event_loop);

    let shader = shader::Shader::from_files("res/shader/enigma_vertex_shader.glsl", "res/shader/enigma_fragment_shader.glsl");

    // TODO: collect the actual model data later
    let shapes = debug_geo::get_debug_shapes();
    let mut materials: Vec<material::Material> = Vec::new();
    for _ in shapes.iter() {
        let mut material = material::Material::default(shader.clone(), display.clone());
        material.set_color([1.0, 1.0, 1.0]);
        material.set_texture_from_file("res/textures/uv_checker.png", material::TextureType::Albedo);
        materials.push(material);
    }

    let mut vertex_buffers: Vec<VertexBuffer<Vertex>> = Vec::new();
    for (shape, material) in shapes.iter().zip(materials.iter()) {
        let buffer = VertexBuffer::new(&material.display, shape).unwrap();
        vertex_buffers.push(buffer);
    }
    let indices: glium::index::NoIndices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);


    event_loop.run(move |event, _, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                // This event is sent by the OS when you close the Window, or request the program to quit via the taskbar.
                winit::event::WindowEvent::CloseRequested => window_target.set_exit(),
                _ => (),
            },
            winit::event::Event::RedrawEventsCleared => {
                _window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                for (buffer, mut material) in vertex_buffers.iter().zip((materials.iter_mut())) {
                    material.update();
                    target.draw(buffer, &indices, &material.program, &material.get_uniforms(), &Default::default()).unwrap();
                }
                target.finish().unwrap();
            }
            _ => (),
        }
    });
}
