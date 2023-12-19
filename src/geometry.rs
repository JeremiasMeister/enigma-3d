#[derive(Copy, Clone)]
pub struct Vertex{
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub index: u32,

}

glium::implement_vertex!(Vertex, position, texcoord, color, normal, index);


//uniforms

