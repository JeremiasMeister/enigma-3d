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


use crate::camera::Camera;
use crate::light::Light;

pub struct Object {
    name: String,
    pub transform: Transform,
    shapes: Vec<Shape>,
    materials: Vec<Material>,
    light: Option<Light>,
    camera: Option<Camera>,
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
    pub fn new(name: Option<String>) -> Self {
        Object {
            name: name.unwrap_or_else(|| String::from("Object")),
            transform: Transform::new(),
            shapes: Vec::new(),
            materials: Vec::new(),
            light: None,
            camera: None,
        }
    }

    pub fn default(display: Display<WindowSurface>) -> Self {
        let mut object = Object::new(None);
        object.add_shape(Shape::default(), Material::default(Shader::default(), display));
        object
    }

    pub fn update(&mut self) {
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
        let shapes = self.get_shapes();
        let mut buffer = Vec::new();
        for (shape, material) in shapes.iter().zip(self.materials.iter()) {
            let vertex = glium::VertexBuffer::new(&material.display, &shape.vertices).unwrap();
            buffer.push(vertex);
        }
        buffer
    }

    pub fn get_index_buffers(&self) -> Vec<glium::IndexBuffer<u32>> {
        let shapes = self.get_shapes();
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

    pub fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    pub fn get_materials_mut(&mut self) -> &mut Vec<Material> {
        &mut self.materials
    }

    pub fn get_shapes(&self) -> &Vec<Shape> {
        &self.shapes
    }

    pub fn get_shapes_mut(&mut self) -> &mut Vec<Shape> {
        &mut self.shapes
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_light(&self) -> &Option<Light> {
        &self.light
    }

    pub fn set_light(&mut self, light: Light) {
        self.light = Some(light);
    }

    pub fn get_camera(&self) -> &Option<Camera> {
        &self.camera
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    pub fn load_from_obj(path: &str, display: Display<WindowSurface>, material: Option<Material>) -> Self {
        let input = BufReader::new(File::open(path).expect("Failed to open file"));
        let obj: Obj = load_obj(input).unwrap();
        let mut vertices = Vec::new();
        for (vert, index) in obj.vertices.iter().zip(obj.indices.iter()) {
            let vertex = geometry::Vertex { position: vert.position, color: [1.0, 1.0, 1.0], texcoord: [0.0, 0.0], normal: vert.normal, index: (*index).into() };
            vertices.push(vertex);
        }

        let shape = Shape::from_vertices(vertices);

        let mut object = Object::new(obj.name);
        match material {
            Some(material) => object.add_shape(shape, material),
            None => object.add_shape(shape, Material::lit_pbr(display)),
        }
        object
    }
}


#[derive(Copy, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>, // radian angles
    pub scale: Vector3<f32>,
    pub matrix: Matrix4<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            matrix: Matrix4::identity(),
        }
    }
    pub fn update(&mut self) {
        let scale_matrix = Matrix4::new_nonuniform_scaling(&self.scale);
        let rotation_matrix = UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z).to_homogeneous();
        let translation_matrix = Translation3::from(self.position).to_homogeneous();
        // Scale, then rotate, then translate
        self.matrix = translation_matrix * rotation_matrix * scale_matrix;
    }


    pub fn set_position(&mut self, position: [f32; 3]){
        self.position = Vector3::from(position);
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.position.clone()
    }

    pub fn set_rotation(&mut self, rotation: [f32; 3]) {
        let radians = rotation.iter().map(|x| x.to_radians()).collect::<Vec<f32>>();
        self.rotation = Vector3::from([radians[0], radians[1], radians[2]]);
    }

    pub fn get_rotation(&self) -> Vector3<f32> {
        let x = self.rotation.x.to_degrees();
        let y = self.rotation.y.to_degrees();
        let z = self.rotation.z.to_degrees();
        Vector3::from([x, y, z])
    }

    pub fn set_scale(&mut self, scale: [f32; 3]){
        self.scale = Vector3::from(scale);
    }

    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale.clone()
    }

    pub fn get_matrix(&mut self) -> [[f32; 4]; 4] {
        self.update();
        self.matrix.into()
    }
}
