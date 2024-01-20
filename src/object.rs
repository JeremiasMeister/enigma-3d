use std::vec::Vec;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use crate::geometry::{BoundingBox, Vertex};
use crate::material::Material;
use nalgebra::{Vector3, Matrix4, Translation3, UnitQuaternion, Point3};
use crate::{debug_geo, geometry};
use uuid::Uuid;


use std::fs::File;
use std::io::BufReader;
use obj::{load_obj, Obj};


use crate::camera::Camera;
use crate::light::Light;

pub struct Object {
    pub name: String,
    pub transform: Transform,
    shapes: Vec<Shape>,
    materials: Vec<Material>,
    light: Option<Light>,
    camera: Option<Camera>,
    bounding_box: Option<geometry::BoundingBox>,
    unique_id: Uuid,
}

pub struct Shape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_index: usize,
}

impl Shape {
    pub fn new() -> Self {
        Shape {
            vertices: Vec::new(),
            indices: Vec::new(),
            material_index: 0,
        }
    }

    pub fn from_vertices_indices(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Shape {
            vertices,
            indices,
            material_index: 0,
        }
    }

    pub fn default() -> Self {
        let triangle = debug_geo::TRIANGLE;
        let mut shape = Shape::new();
        shape.vertices = triangle.to_vec();
        for i in 0..triangle.iter().len() {
            shape.indices.push(i as u32);
        }
        shape
    }

    pub fn get_vertex_buffer(&self, display: Display<WindowSurface>) -> glium::VertexBuffer<Vertex> {
        glium::VertexBuffer::new(&display, &self.vertices).unwrap()
    }

    pub fn get_index_buffer(&self, display: Display<WindowSurface>) -> glium::IndexBuffer<u32> {
        glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &self.indices).unwrap()
    }

    pub fn set_material_from_object_list(&mut self, material_index: usize) {
        self.material_index = material_index;
    }
}

impl Object {
    pub fn new(name: Option<String>) -> Self {
        let mut object = Object {
            name: name.unwrap_or_else(|| String::from("Object")),
            transform: Transform::new(),
            shapes: Vec::new(),
            materials: Vec::new(),
            light: None,
            camera: None,
            bounding_box: None,
            unique_id: Uuid::new_v4(), //generating unique id for object
        };
        object.calculate_bounding_box();
        object
    }

    pub fn get_unique_id(&self) -> Uuid {
        self.unique_id
    }

    fn calculate_bounding_box(&mut self) -> BoundingBox {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for shape in self.get_shapes().iter() {
            for vertex in shape.vertices.iter() {
                min_x = min_x.min(vertex.position[0]);
                min_y = min_y.min(vertex.position[1]);
                min_z = min_z.min(vertex.position[2]);
                max_x = max_x.max(vertex.position[0]);
                max_y = max_y.max(vertex.position[1]);
                max_z = max_z.max(vertex.position[2]);
            }
        }

        let min_point = Point3::new(min_x, min_y, min_z);
        let max_point = Point3::new(max_x, max_y, max_z);

        let center = Point3::new(
            (min_point.x + max_point.x) / 2.0,
            (min_point.y + max_point.y) / 2.0,
            (min_point.z + max_point.z) / 2.0,
        );
        self.transform.update();
        let transformed_center = self.transform.matrix.transform_point(&center);
        let transformed_width = (max_x - min_x) * self.transform.get_scale().x;
        let transformed_height = (max_y - min_y) * self.transform.get_scale().y;
        let transformed_depth = (max_z - min_z) * self.transform.get_scale().z;

        let aabb = BoundingBox {
            center: Vector3::from([transformed_center.x, transformed_center.y, transformed_center.z]),
            width: transformed_width,
            height: transformed_height,
            depth: transformed_depth,
        };
        self.bounding_box = Some(aabb);
        aabb
    }

    pub fn default() -> Self {
        let mut object = Object::new(None);
        object.add_shape(Shape::default());
        object
    }

    pub fn update(&mut self) {
        self.transform.update();
        self.materials.iter_mut().for_each(|x| x.update());
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn get_vertex_buffers(&self) -> Vec<glium::VertexBuffer<Vertex>> {
        let shapes = self.get_shapes();
        let mut buffer = Vec::new();
        for shape in shapes.iter() {
            let material = &self.materials[shape.material_index];
            let vertex = glium::VertexBuffer::new(&material.display, &shape.vertices).unwrap();
            buffer.push(vertex);
        }
        buffer
    }

    pub fn get_index_buffers(&self) -> Vec<glium::IndexBuffer<u32>> {
        let shapes = self.get_shapes();
        let mut buffer = Vec::new();
        for shape in shapes.iter() {
            let material = &self.materials[shape.material_index];
            let index = glium::IndexBuffer::new(&material.display, glium::index::PrimitiveType::TrianglesList, &shape.indices).unwrap();
            buffer.push(index);
        }
        buffer
    }
    pub fn get_bounding_box(&mut self) -> BoundingBox {
        self.calculate_bounding_box()
    }

    pub fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    pub fn get_materials_mut(&mut self) -> &mut Vec<Material> {
        &mut self.materials
    }

    pub fn add_material(&mut self, material: Material) {
        self.materials.push(material);
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

    pub fn load_from_obj(path: &str) -> Self {
        let input = BufReader::new(File::open(path).expect("Failed to open file"));
        let obj: Obj = load_obj(input).unwrap();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for vert in obj.vertices.iter() {
            let vertex = geometry::Vertex { position: vert.position, color: [1.0, 1.0, 1.0], texcoord: [0.0, 0.0], normal: vert.normal };
            vertices.push(vertex);
        }
        for index in obj.indices.iter() {
            indices.push((*index).into());
        }

        let shape = Shape::from_vertices_indices(vertices, indices);
        let mut object = Object::new(obj.name);
        object.add_shape(shape);
        object
    }

    pub fn load_from_gltf(path: &str) -> Self {
        let (gltf, buffers, _) = gltf::import(path).expect("Failed to import gltf file");

        let mut object = Object::new(Some(String::from(path)));

        for mesh in gltf.meshes() {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let positions = reader.read_positions().unwrap();
                let normals = reader.read_normals().unwrap();
                let tex_coords = reader.read_tex_coords(0).unwrap().into_f32();
                let prim_indices = reader.read_indices().unwrap().into_u32();

                let mut flipped_tex_coords: Vec<[f32;2]> = Vec::new();
                // flip tex_coords
                for mut tex_coord in tex_coords.into_iter() {
                    tex_coord[1] = 1.0 - tex_coord[1];
                    flipped_tex_coords.push(tex_coord);
                }

                for ((position, normal), tex_coord) in positions.zip(normals).zip(flipped_tex_coords) {
                    let vertex = geometry::Vertex { position, color: [1.0, 1.0, 1.0], texcoord: tex_coord, normal };
                    vertices.push(vertex);
                }

                prim_indices.for_each(|index| indices.push(index));
            }
            let shape = Shape::from_vertices_indices(vertices, indices);
            object.add_shape(shape);
        }
        object
    }
}


#[derive(Copy, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    // radian angles
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


    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = Vector3::from(position);
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.position.clone()
    }

    pub fn set_rotation(&mut self, rotation: [f32; 3]) {
        let radians = rotation.iter().map(|x| x.to_radians()).collect::<Vec<f32>>();
        self.rotation = Vector3::from([radians[0], radians[1], radians[2]]);
    }

    pub fn rotate(&mut self, rotation: [f32; 3]) {
        let cur_r = self.get_rotation();
        let additive_rotation = [cur_r.x + rotation[0], cur_r.y + rotation[1], cur_r.z + rotation[2]];
        let radians = additive_rotation.iter().map(|x| x.to_radians()).collect::<Vec<f32>>();
        self.rotation = Vector3::from([radians[0], radians[1], radians[2]]);
    }

    pub fn move_dir(&mut self, position: [f32; 3]) {
        let cur_p = self.get_position();
        let additive_position = [cur_p.x + position[0], cur_p.y + position[1], cur_p.z + position[2]];
        self.position = Vector3::from(additive_position);
    }

    pub fn get_rotation(&self) -> Vector3<f32> {
        let x = self.rotation.x.to_degrees();
        let y = self.rotation.y.to_degrees();
        let z = self.rotation.z.to_degrees();
        Vector3::from([x, y, z])
    }

    pub fn set_scale(&mut self, scale: [f32; 3]) {
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
