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
use itertools::enumerate;
use obj::{load_obj, Obj};

pub struct Object {
    pub name: String,
    pub transform: Transform,
    shapes: Vec<Shape>,
    materials: Vec<Material>,
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
            bounding_box: None,
            unique_id: Uuid::new_v4(), //generating unique id for object
        };
        object.calculate_bounding_box();
        object
    }

    pub fn primitive_cube(size: f32, invert_normals: bool) -> Self {
        let mut object = Object::new(Some(String::from("Cube")));
        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Define the vertices for each face of the cube
        let half_size = size / 2.0;
        let positions = [
            // Front face
            [-half_size, -half_size, half_size], // Bottom-left
            [half_size, -half_size, half_size],  // Bottom-right
            [half_size, half_size, half_size],   // Top-right
            [-half_size, half_size, half_size],  // Top-left
            // Back face
            [-half_size, -half_size, -half_size], // Bottom-left
            [-half_size, half_size, -half_size],  // Top-left
            [half_size, half_size, -half_size],   // Top-right
            [half_size, -half_size, -half_size],  // Bottom-right
        ];

        let mut normals = [
            [0.0, 0.0, 1.0],  // Front face
            [0.0, 0.0, -1.0], // Back face
            [1.0, 0.0, 0.0],  // Right face
            [-1.0, 0.0, 0.0], // Left face
            [0.0, 1.0, 0.0],  // Top face
            [0.0, -1.0, 0.0], // Bottom face
        ];

        if invert_normals {
            for normal in normals.iter_mut() {
                normal[0] *= -1.0;
                normal[1] *= -1.0;
                normal[2] *= -1.0;
            }
        }

        // Create each face in CW order
        let mut face_indices = [
            [0, 1, 2, 3], // Front face
            [4, 5, 6, 7], // Back face
            [1, 7, 6, 2], // Right face
            [4, 0, 3, 5], // Left face
            [3, 2, 6, 5], // Top face
            [4, 7, 1, 0], // Bottom face
        ];

        //invert the winding order of the faces
        if invert_normals {
            for face in face_indices.iter_mut() {
                face.reverse();
            }
        }

        //proper tex_coords per face
        let tex_coords = [
            [0.0, 0.0], // Front face
            [1.0, 0.0], // Back face
            [0.0, 0.0], // Right face
            [1.0, 0.0], // Left face
            [0.0, 1.0], // Top face
            [0.0, 0.0], // Bottom face
        ];

        for &normal in &normals {
            for &face in &face_indices {
                vertices.extend(face.iter().map(|&i| {
                    let position = positions[i];
                    Vertex {
                        position,
                        color: [1.0, 1.0, 1.0],
                        texcoord: tex_coords.iter().map(|x| x.to_owned()).collect::<Vec<[f32; 2]>>()[0],
                        normal,
                    }
                }));
            }
        }

        // Each face is two triangles, so 6 indices per face
        for (face_index, _) in face_indices.iter().enumerate() {
            let base_index = (face_index * 4) as u32;
            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3, // First triangle
            ]);
        }

        let shape = Shape::from_vertices_indices(vertices, indices);
        object.add_shape(shape);
        object
    }


    pub fn primitive_plane(size_x: i32, size_z: i32) -> Self {
        let mut object = Object::new(Some(String::from("Plane")));
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for x in 0..size_x {
            for z in 0..size_z {
                vertices.push(Vertex { position: [x as f32, 0.0, z as f32], color: [1.0, 1.0, 1.0], texcoord: [0.0, 0.0], normal: [0.0, 1.0, 0.0] });
                vertices.push(Vertex { position: [x as f32 + 1.0, 0.0, z as f32], color: [1.0, 1.0, 1.0], texcoord: [1.0, 0.0], normal: [0.0, 1.0, 0.0] });
                vertices.push(Vertex { position: [x as f32 + 1.0, 0.0, z as f32 + 1.0], color: [1.0, 1.0, 1.0], texcoord: [1.0, 1.0], normal: [0.0, 1.0, 0.0] });
                vertices.push(Vertex { position: [x as f32, 0.0, z as f32 + 1.0], color: [1.0, 1.0, 1.0], texcoord: [0.0, 1.0], normal: [0.0, 1.0, 0.0] });
                let index = (x * size_z + z) * 4;
                // Reverse the order of the last three indices for each triangle to change the winding order
                indices.push(index.try_into().unwrap());
                indices.push((index + 2).try_into().unwrap()); // Swapped from index + 1 to index + 2
                indices.push((index + 1).try_into().unwrap()); // Swapped from index + 2 to index + 1
                indices.push(index.try_into().unwrap());
                indices.push((index + 3).try_into().unwrap()); // Swapped from index + 2 to index + 3
                indices.push((index + 2).try_into().unwrap()); // Swapped from index + 3 to index + 2
            }
        }
        let shape = Shape::from_vertices_indices(vertices, indices);
        object.add_shape(shape);
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

    pub fn get_closest_lights(&self, lights: &Vec<crate::light::Light>) -> (Vec<usize>, Vec<crate::light::Light>) {
        let mut closest_lights = Vec::new();
        let mut closest_light_indices = Vec::new();

        //collect the four closest lights to the object
        for (light_index, light) in enumerate(lights.iter()) {
            let light_pos = light.position;
            let object_pos = self.transform.get_position();
            let distance = (Vector3::from(light_pos) - object_pos).magnitude();
            if closest_lights.len() < 4 {
                closest_lights.push((light.clone(), distance));
                closest_light_indices.push(light_index);
            } else {
                let mut max_distance = 0.0;
                let mut max_index = 0;
                for (index, (_, distance)) in closest_lights.iter().enumerate() {
                    if *distance > max_distance {
                        max_distance = *distance;
                        max_index = index;
                    }
                }
                if distance < max_distance {
                    closest_lights[max_index] = (light.clone(), distance.clone());
                    closest_light_indices[max_index] = light_index;
                }
            }
        }
        (closest_light_indices, closest_lights.iter().map(|(light, _)| light.clone()).collect())
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

                let mut flipped_tex_coords: Vec<[f32; 2]> = Vec::new();
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
        self.matrix = scale_matrix * rotation_matrix * translation_matrix;
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

    pub fn get_matrix(&mut self) -> Matrix4<f32> {
        self.update();
        self.matrix
    }

    pub fn look_at(&mut self, target: Vector3<f32>, up_vector: Vector3<f32>) {
        // Convert positions to Point3 for clarity, though not strictly necessary here
        let position = Point3::from(self.position);
        let target_point = Point3::from(target);

        // Compute the forward vector from the object to the target
        let direction = (target_point - position).normalize();

        // Use the provided up_vector instead of a hardcoded value
        let up = up_vector.normalize();

        // Compute the rotation as a quaternion
        let rotation_quaternion = UnitQuaternion::face_towards(&direction, &up);

        // Convert quaternion to Euler angles
        let euler_angles = rotation_quaternion.euler_angles();

        // Set the rotation; convert to degrees if necessary
        self.rotation = Vector3::from([
            euler_angles.0.to_degrees(),
            euler_angles.1.to_degrees(),
            euler_angles.2.to_degrees(),
        ]);
    }

}
