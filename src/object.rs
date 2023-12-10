use crate::geometry::Vertex;
use crate::material::Material;

pub struct Object {
    pub transform: Transform,
    pub shapes: Vec<Shape>,
    pub materials: Vec<Material>,
}

pub struct Shape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Object {
    pub fn new() -> Self {
        Object {
            transform: Transform::new(),
            shapes: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn get_transformed_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        for shape in self.shapes.iter() {
            for vertex in shape.vertices.iter() {
                let mut vertex = vertex.clone();
                vertex.position = vertex.position * self.transform.matrix;
                vertices.push(vertex);
            }
        }
        vertices
    }
}


pub struct Transform{
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub matrix: [[f32; 4]; 4],
}

impl Transform{
    pub fn new() -> Self{
        Transform{
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            matrix: [[1.0, 0.0, 0.0, 0.0],
                     [0.0, 1.0, 0.0, 0.0],
                     [0.0, 0.0, 1.0, 0.0],
                     [0.0, 0.0, 0.0, 1.0]],
        }
    }
    pub fn update(&mut self){
        self.matrix = [
            [self.scale[0], 0.0, 0.0, 0.0],
            [0.0, self.scale[1], 0.0, 0.0],
            [0.0, 0.0, self.scale[2], 0.0],
            [self.position[0], self.position[1], self.position[2], 1.0],
        ];
    }
}
