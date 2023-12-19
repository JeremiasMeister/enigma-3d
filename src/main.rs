use enigma::{shader, material, geometry};
mod debug_geo;
use glium::{IndexBuffer, VertexBuffer};




fn main() {
    // create an enigma eventloop and appstate
    let event_loop = enigma::EventLoop::new("Enigma Test Window");
    let mut app_state = enigma::AppState::new();

    // load enigma shader
    let shader = shader::Shader::from_files("res/shader/enigma_vertex_shader.glsl", "res/shader/enigma_fragment_shader.glsl");

    // TODO: collect the actual model data later
    let shapes = debug_geo::get_debug_shapes();
    let mut materials: Vec<material::Material> = Vec::new();
    for _ in shapes.iter() {
        let mut material = material::Material::default(shader.clone(), event_loop.get_display_clone());
        material.set_color([1.0, 1.0, 1.0]);
        material.set_texture_from_file("res/textures/uv_checker.png", material::TextureType::Albedo);
        materials.push(material);
    }


    // create buffer lists
    let mut vertex_buffers: Vec<VertexBuffer<geometry::Vertex>> = Vec::new();
    let mut index_buffers: Vec<IndexBuffer<u32>> = Vec::new();
    for (shape, material) in shapes.iter().zip(materials.iter()) {
        let indices = shape.iter().map(|x| x.index).collect::<Vec<u32>>();
        let vertex = VertexBuffer::new(&material.display, shape).unwrap();
        let index = IndexBuffer::new(&material.display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();
        vertex_buffers.push(vertex);
        index_buffers.push(index);
    }

    // adding all the buffers
    app_state.extend_vertex_buffers(vertex_buffers);
    app_state.extend_index_buffers(index_buffers);
    app_state.extend_materials(materials);

    event_loop.run(app_state);

}
