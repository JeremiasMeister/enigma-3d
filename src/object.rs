use crate::geometry::Vertex;
use crate::material::Material;
use nalgebra::{Vector3, Matrix4, Translation3, UnitQuaternion, Point3};

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

    //TODO: Get rid of potential material duplication by mapping materials to multiple shapes
    pub fn add_shape(&mut self, shape: Shape, material: Material) {
        self.shapes.push(shape);
        self.materials.push(material);
    }

    pub fn get_transformed_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        for shape in self.shapes.iter() {
            for vertex in shape.vertices.iter() {
                let mut vertex = vertex.clone();
                let position_point = Point3::from(Vector3::from(vertex.position));
                vertex.position = self.transform.matrix.transform_point(&position_point).into();
                vertices.push(vertex);
            }
        }
        vertices
    }
}


pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>, // Euler angles
    pub scale: Vector3<f32>,
    pub matrix: Matrix4<f32>,
}

impl Transform{
    pub fn new() -> Self{
        Transform{
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            matrix: Matrix4::identity(),
        }
    }
    pub fn update(&mut self){
        let translation = Translation3::from(self.position);
        let rotation = UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let scale = Matrix4::new_nonuniform_scaling(&self.scale);
        self.matrix = translation.to_homogeneous() * rotation.to_homogeneous() * scale;
    }
}
