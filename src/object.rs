use glium::Display;
use glium::glutin::surface::WindowSurface;
use crate::geometry::Vertex;
use crate::material::Material;
use crate::shader::Shader;
use nalgebra::{Vector3, Matrix4, Translation3, UnitQuaternion, Point3};
use crate::{debug_geo, geometry};

use std::fs::File;
use std::io::BufReader;
use obj::{load_obj, Obj};

pub struct Object {
    pub transform: Transform,
    pub shapes: Vec<Shape>,
    pub materials: Vec<Material>,
}

pub struct Shape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Shape {

    pub fn new() -> Self {
        Shape {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn from_vertices(vertices: Vec<Vertex>) -> Self {
        let mut shape = Shape::new();
        shape.vertices = vertices;
        for vertex in shape.vertices.iter() {
            shape.indices.push(vertex.index);
        }
        shape
    }

    pub fn default() -> Self {
        let triangle = debug_geo::TRIANGLE;
        let mut shape = Shape::new();
        shape.vertices = triangle.to_vec();
        for vertex in triangle.iter() {
            shape.indices.push(vertex.index);
        }
        shape
    }

    pub fn get_vertex_buffer(&self, display: Display<WindowSurface>) -> glium::VertexBuffer<Vertex> {
        glium::VertexBuffer::new(&display, &self.vertices).unwrap()
    }

    pub fn get_index_buffer(&self, display: Display<WindowSurface>) -> glium::IndexBuffer<u32> {
        glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &self.indices).unwrap()
    }
}

impl Object {
    pub fn new() -> Self {
        Object {
            transform: Transform::new(),
            shapes: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn default(display: Display<WindowSurface>) -> Self {
        let mut object = Object::new();
        object.add_shape(Shape::default(), Material::default(Shader::default(), display));
        object
    }

    pub fn update(&mut self){
        self.transform.update();
        self.materials.iter_mut().for_each(|x| x.update());
    }

    //TODO: Get rid of potential material duplication by mapping materials to multiple shapes
    pub fn add_shape(&mut self, shape: Shape, material: Material) {
        self.shapes.push(shape);
        self.materials.push(material);
    }

    pub fn get_transformed_shapes(&self) -> Vec<Shape> {
        let mut shapes = Vec::new();
        for shape in self.shapes.iter() {
            let mut vertices = Vec::new();
            for vertex in shape.vertices.iter() {
                let mut vertex = vertex.clone();
                let position_point = Point3::from(Vector3::from(vertex.position));
                vertex.position = self.transform.matrix.transform_point(&position_point).into();
                vertices.push(vertex);
            }
            shapes.push(Shape::from_vertices(vertices));
        }
        shapes
    }

    pub fn get_vertex_buffers(&self) -> Vec<glium::VertexBuffer<Vertex>> {
        let shapes = self.get_transformed_shapes();
        let mut buffer = Vec::new();
        for (shape, material) in shapes.iter().zip(self.materials.iter()) {
            let vertex = glium::VertexBuffer::new(&material.display, &shape.vertices).unwrap();
            buffer.push(vertex);
        }
        buffer
    }

    pub fn get_index_buffers(&self) -> Vec<glium::IndexBuffer<u32>> {
        let shapes = self.get_transformed_shapes();
        let mut buffer = Vec::new();
        for (shape, material) in shapes.iter().zip(self.materials.iter()) {
            let mut indices = Vec::new();
            for vertex in shape.vertices.iter() {
                indices.push(vertex.index);
            }
            let index = glium::IndexBuffer::new(&material.display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();
            buffer.push(index);
        }
        buffer
    }

    pub fn load_from_obj(path: &str, display: Display<WindowSurface>) -> Self {
        let input = BufReader::new(File::open(path).unwrap());
        let obj: Obj = load_obj(input).unwrap();
        let mut vertices = Vec::new();
        for (vert, index) in obj.vertices.iter().zip(obj.indices.iter()) {
            let vertex = geometry::Vertex { position: vert.position, color: [1.0, 1.0, 1.0], texcoord: [0.0,0.0], normal: vert.normal, index: (*index).into() };
            vertices.push(vertex);
        }

        let shape = Shape::from_vertices(vertices);

        let mut object = Object::new();
        object.add_shape(shape, Material::default(Shader::default(), display));
        object
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

    pub fn set_position(&mut self, position: [f32; 3]){
        self.position = Vector3::from(position);
        self.update();
    }

    pub fn set_rotation(&mut self, rotation: [f32; 3]){
        self.rotation = Vector3::from(rotation);
        self.update();
    }

    pub fn set_scale(&mut self, scale: [f32; 3]){
        self.scale = Vector3::from(scale);
        self.update();
    }
}
