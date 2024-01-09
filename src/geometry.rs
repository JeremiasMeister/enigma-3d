#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub index: u32, //TODO: Index should not be stored on the vertex but in the shape
}

glium::implement_vertex!(Vertex, position, texcoord, color, normal, index);


