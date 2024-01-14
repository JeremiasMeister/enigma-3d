use nalgebra::Vector3;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Copy, Clone)]
pub struct BoundingSphere{
    pub center: Vector3<f32>, //relative to the objects position
    pub radius: f32,
}

glium::implement_vertex!(Vertex, position, texcoord, color, normal);
