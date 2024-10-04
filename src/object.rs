use std::collections::HashMap;
use std::vec::Vec;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use crate::geometry::{BoneTransforms, BoundingBox, Vertex};
use nalgebra::{Vector3, Matrix4, Translation3, UnitQuaternion, Point3};
use crate::{animation, debug_geo, geometry, smart_format};
use uuid::Uuid;


use std::fs::File;
use std::io::BufReader;
use glium::uniforms::UniformBuffer;
use gltf::buffer::Data;
use nalgebra_glm::normalize;
use obj::{load_obj, Obj};
use serde::{Deserialize, Serialize};
use crate::animation::{Animation, AnimationChannel, AnimationKeyframe, AnimationState, Bone, MAX_BONES, Skeleton};
use crate::gizmo::Gizmo;
use crate::logging::{EnigmaError, EnigmaWarning};

pub struct ObjectInstance {
    pub vertex_buffers: Vec<(glium::vertex::VertexBufferAny, usize)>,
    pub index_buffers: Vec<glium::IndexBuffer<u32>>,
    pub instance_matrices: Vec<[[f32; 4]; 4]>,
    pub instance_attributes: glium::VertexBuffer<geometry::InstanceAttribute>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectSerializer {
    pub name: String,
    pub transform: TransformSerializer,
    collision: bool,
    shapes: Vec<Shape>,
    materials: Vec<String>,
    unique_id: String,
    cloned_id: String,
}

pub struct Object {
    pub name: String,
    pub transform: Transform,
    collision: bool,
    shapes: Vec<Shape>,
    materials: Vec<Uuid>,
    bounding_box: Option<geometry::BoundingBox>,
    unique_id: Uuid,
    cloned_id: Uuid,
    animations: HashMap<String, animation::Animation>,
    pub (crate) skeleton: Option<animation::Skeleton>,
    current_animation: Option<AnimationState>,
}

impl Clone for Object {
    fn clone(&self) -> Self {
        //creating new object
        let mut new_object = Object::new(Some(self.name.clone()));

        //setting transform for new object
        new_object.transform.set_position(self.transform.get_position().into());
        new_object.transform.set_rotation(self.transform.get_rotation().into());
        new_object.transform.set_scale(self.transform.get_scale().into());

        //cloning shapes
        for shape in self.shapes.iter() {
            let mut new_shape = Shape::new();
            new_shape.vertices = shape.vertices.clone();
            new_shape.indices = shape.indices.clone();
            new_shape.material_index = shape.material_index;
            new_object.add_shape(new_shape);
        }

        //cloning materials
        new_object.materials = self.materials.clone();

        new_object.bounding_box = self.bounding_box.clone();
        new_object.unique_id = Uuid::new_v4();
        new_object.cloned_id = self.unique_id;
        new_object.animations = self.animations.clone();
        new_object.skeleton = self.skeleton.clone();
        new_object
    }
}

#[derive(Serialize, Deserialize)]
pub struct Shape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_index: usize,
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        Shape {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            material_index: self.material_index,
        }
    }
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

impl ObjectInstance {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            vertex_buffers: Vec::new(),
            index_buffers: Vec::new(),
            instance_matrices: Vec::new(),
            instance_attributes: glium::vertex::VertexBuffer::dynamic(display, &Vec::new()).expect("Building ObjectInstance, Per Instance Attribute could not be created"),
        }
    }

    pub fn set_vertex_buffers(&mut self, buffers: Vec<(glium::vertex::VertexBufferAny, usize)>) {
        self.vertex_buffers = buffers;
    }

    pub fn set_index_buffers(&mut self, buffers: Vec<glium::IndexBuffer<u32>>) {
        self.index_buffers = buffers;
    }

    pub fn add_instance(&mut self, instance: [[f32; 4]; 4]) {
        self.instance_matrices.push(instance);
    }
}

impl Object {
    pub fn new(name: Option<String>) -> Self {
        let uuid = Uuid::new_v4();
        let mut object = Object {
            name: name.unwrap_or_else(|| String::from("Object")),
            transform: Transform::new(),
            shapes: Vec::new(),
            materials: Vec::new(),
            bounding_box: None,
            unique_id: uuid,
            cloned_id: uuid,
            collision: true,
            animations: HashMap::new(),
            skeleton: None,
            current_animation: None,
        };
        object.calculate_bounding_box();
        object
    }

    pub fn to_serializer(&self) -> ObjectSerializer {
        let name = self.name.clone();
        let transform = self.transform.to_serializer();
        let shapes = self.shapes.clone();
        let materials = self.materials.iter().map(|x| x.to_string()).collect();
        let unique_id = self.unique_id.to_string();
        let cloned_id = self.cloned_id.to_string();
        ObjectSerializer {
            name,
            transform,
            shapes,
            materials,
            unique_id,
            cloned_id,
            collision: self.collision,
        }
    }

    pub fn from_serializer(serializer: ObjectSerializer) -> Self {
        let mut object = Object::new(Some(serializer.name));
        object.transform = Transform::from_serializer(serializer.transform);
        object.shapes = serializer.shapes;
        for mat in serializer.materials {
            object.add_material(Uuid::parse_str(mat.as_str()).expect("failed to parse material uuid"));
        }
        object.unique_id = uuid::Uuid::parse_str(serializer.unique_id.as_str()).unwrap();
        object.cloned_id = uuid::Uuid::parse_str(serializer.cloned_id.as_str()).unwrap();
        object.collision = serializer.collision;
        object.calculate_bounding_box();
        object
    }

    pub fn set_collision(&mut self, collision: bool) {
        self.collision = collision;
    }

    pub fn get_collision(&self) -> &bool {
        &self.collision
    }

    pub fn get_unique_id(&self) -> Uuid {
        self.unique_id
    }

    pub fn get_instance_id(&self) -> Uuid {
        self.cloned_id
    }

    pub fn break_instance(&mut self) {
        self.cloned_id = self.unique_id;
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

    fn update_animation_internal(&mut self, delta_time: f32) {
        if let Some(anim_state) = &mut self.current_animation {
            if let Some(animation) = self.animations.get(&anim_state.name) {
                anim_state.time += delta_time * anim_state.speed;
                if anim_state.time >= animation.duration {
                    if anim_state.looping {
                        anim_state.time %= animation.duration;
                    } else {
                        anim_state.time = animation.duration;
                    }
                }
            }
        }
    }

    pub fn has_skeletal_animation(&self) -> bool {
        self.skeleton.is_some() && !self.animations.is_empty()
    }

    pub fn is_animation_playing(&self) -> bool {
        self.current_animation.is_some()
    }

    pub fn visualize_skeleton(&mut self, gizmo: &mut Gizmo) {
        if let Some(skeleton) = &self.skeleton {
            let animated_transforms = self.get_animated_bone_transforms();

            for (i, bone) in skeleton.bones.iter().enumerate() {
                let bone_transform = animated_transforms[i];
                let bone_position = self.transform.get_matrix_object() * bone_transform.column(3);
                let bone_point = Point3::new(bone_position[0], bone_position[1], bone_position[2]);

                // Draw a small circle for each bone
                gizmo.draw_position(bone_point, 0.02, 8, [1.0,1.0,1.0,1.0]);

                // Draw bone axes
                let x_axis = bone_transform.column(0).xyz().normalize() * 0.05;
                let y_axis = bone_transform.column(1).xyz().normalize() * 0.05;
                let z_axis = bone_transform.column(2).xyz().normalize() * 0.05;
                gizmo.draw_line(bone_point, bone_point + x_axis, [1.0, 0.0, 0.0, 1.0]);
                gizmo.draw_line(bone_point, bone_point + y_axis, [0.0, 1.0, 0.0, 1.0]);
                gizmo.draw_line(bone_point, bone_point + z_axis, [0.0, 0.0, 1.0, 1.0]);

                // Draw a line to the parent bone
                if let Some(parent_id) = bone.parent_id {
                    let parent_transform = animated_transforms[parent_id];
                    let parent_position = self.transform.get_matrix_object() * parent_transform.column(3);
                    let parent_point = Point3::new(parent_position[0], parent_position[1], parent_position[2]);

                    gizmo.draw_line(bone_point, parent_point, [1.0, 1.0, 1.0, 1.0]);
                }
            }
        }
    }

    pub fn visualize_bind_skeleton(&self, gizmo: &mut Gizmo) {
        if let Some(skeleton) = &self.skeleton {
            for (_i, bone) in skeleton.bones.iter().enumerate() {
                // Calculate the bone's position in bind pose
                let bind_pose = bone.inverse_bind_pose.try_inverse().unwrap_or(Matrix4::identity());
                let bone_position = bind_pose.column(3).xyz();
                let bone_point = Point3::from(bone_position);

                // Draw a small sphere for each bone
                gizmo.draw_position(bone_point, 0.02, 8, [1.0,1.0,1.0,1.0]);

                // Draw bone axes
                let x_axis = bind_pose.column(0).xyz().normalize() * 0.05;
                let y_axis = bind_pose.column(1).xyz().normalize() * 0.05;
                let z_axis = bind_pose.column(2).xyz().normalize() * 0.05;
                gizmo.draw_line(bone_point, bone_point + x_axis, [1.0, 0.0, 0.0, 1.0]);
                gizmo.draw_line(bone_point, bone_point + y_axis, [0.0, 1.0, 0.0, 1.0]);
                gizmo.draw_line(bone_point, bone_point + z_axis, [0.0, 0.0, 1.0, 1.0]);

                // Draw a line to the parent bone
                if let Some(parent_id) = bone.parent_id {
                    let parent_bone = &skeleton.bones[parent_id];
                    let parent_bind_pose = parent_bone.inverse_bind_pose.try_inverse().unwrap_or(Matrix4::identity());
                    let parent_position = parent_bind_pose.column(3).xyz();
                    let parent_point = Point3::from(parent_position);

                    gizmo.draw_line(bone_point, parent_point, [0.5, 0.5, 0.5, 1.0]);
                }
            }
        }
    }

    fn get_animated_bone_transforms(&self) -> Vec<Matrix4<f32>> {
        let mut transforms = Vec::new();

        if let Some(skeleton) = &self.skeleton {
            if let Some(animation_state) = &self.current_animation {
                if let Some(animation) = self.animations.get(&animation_state.name) {
                    for (i, bone) in skeleton.bones.iter().enumerate() {
                        let local_transform = animation::interpolate_keyframes(animation, i, animation_state.time);
                        let parent_transform = bone.parent_id
                            .map(|id| transforms[id])
                            .unwrap_or_else(Matrix4::identity);

                        let global_transform = parent_transform * local_transform;
                        transforms.push(global_transform);
                    }
                }
            } else {
                // If no animation is playing, use bind pose
                for bone in &skeleton.bones {
                    let bind_pose = bone.inverse_bind_pose.try_inverse().unwrap_or(Matrix4::identity());
                    let parent_transform = bone.parent_id
                        .map(|id| transforms[id])
                        .unwrap_or_else(Matrix4::identity);
                    transforms.push(parent_transform * bind_pose);
                }
            }
        }

        transforms
    }

    pub fn get_bone_transform_buffer(&self, display: &Display<WindowSurface>) -> UniformBuffer<BoneTransforms> {
        let mut logger = EnigmaWarning::new(None, true);
        let mut bone_transform_data = BoneTransforms {
            bone_transforms: [[[0.0; 4]; 4]; MAX_BONES],
        };

        logger.extent(smart_format!("Entering get_bone_transform_buffer for object: {}", self.name).as_str());

        if let Some(skeleton) = &self.skeleton {
            let mut global_transforms = vec![Matrix4::identity(); skeleton.bones.len()];

            if let Some(anim_state) = &self.current_animation {
                if let Some(animation) = self.animations.get(anim_state.name.as_str()) {
                    logger.extent(smart_format!("Applying animation: {}", anim_state.name).as_str());

                    for (i, bone) in skeleton.bones.iter().enumerate() {
                        let local_transform = animation::interpolate_keyframes(animation, i, anim_state.time);
                        let parent_transform: Matrix4<f32> = bone.parent_id
                            .map(|id| global_transforms[id])
                            .unwrap_or_else(Matrix4::identity);

                        global_transforms[i] = parent_transform * local_transform;

                        // Calculate final transform for skinning
                        let final_transform: Matrix4<f32> = global_transforms[i] * bone.inverse_bind_pose;

                        bone_transform_data.bone_transforms[i] = final_transform.into();
                        if i < 5 {  // Log only first 5 bones to avoid spam
                            logger.extent(smart_format!(
                            "Bone {}: Local: {:?}, Global: {:?}, Final: {:?}",
                            i, local_transform, global_transforms[i], final_transform
                        ).as_str());
                        }
                    }
                } else {
                    logger.extent(smart_format!("Animation not found: {}", anim_state.name).as_str());
                }
            } else {
                logger.extent("No animation playing, using bind pose");

                for (i, bone) in skeleton.bones.iter().enumerate() {
                    let parent_transform = bone.parent_id
                        .map(|id| global_transforms[id])
                        .unwrap_or_else(Matrix4::identity);

                    // Use the inverse of the inverse bind pose to get the bind pose
                    let bind_pose = bone.inverse_bind_pose.try_inverse().unwrap_or(Matrix4::identity());

                    global_transforms[i] = parent_transform * bind_pose;

                    // For bind pose, the final transform is identity
                    let final_transform: Matrix4<f32> = global_transforms[i] * bone.inverse_bind_pose;

                    bone_transform_data.bone_transforms[i] = final_transform.into();
                }
            }
        } else {
            logger.extent("No skeleton found, using identity transforms");
        }

        //logger.log();
        UniformBuffer::new(display, bone_transform_data).expect("Failed to create BoneTransform Buffer")
    }



    pub fn play_animation(&mut self, name: &str, looping: bool) {
        if let Some(_) = self.animations.get(name) {
            self.current_animation = Some(AnimationState {
                name: name.to_string(),
                time: 0.0,
                speed: 1.0,
                looping,
            });
        }
    }

    pub fn stop_animation(&mut self) {
        self.current_animation = None;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.transform.update();
        if self.skeleton.is_some() && self.current_animation.is_some() {
            self.update_animation_internal(delta_time);
        }
    }

    pub fn get_closest_lights(&self, lights: &Vec<crate::light::Light>) -> Vec<crate::light::Light> {
        let mut closest_lights = Vec::new();

        //collect the four closest lights to the object
        for light in lights.iter() {
            let light_pos = light.position;
            let object_pos = self.transform.get_position();
            let distance = (Vector3::from(light_pos) - object_pos).magnitude();
            if closest_lights.len() < 4 {
                closest_lights.push((light.clone(), distance));
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
                }
            }
        }
        closest_lights.iter().map(|(light, _)| light.clone()).collect()
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn get_vertex_buffers(&self, display: &Display<WindowSurface>) -> Vec<(glium::vertex::VertexBufferAny, usize)> {
        let shapes = self.get_shapes();
        let mut buffer = Vec::new();
        for shape in shapes.iter() {
            let vertex: glium::vertex::VertexBufferAny = glium::VertexBuffer::new(display, &shape.vertices).unwrap().into();
            buffer.push((vertex, shape.material_index));
        }
        buffer
    }

    pub fn get_index_buffers(&self, display: &Display<WindowSurface>) -> Vec<glium::IndexBuffer<u32>> {
        let shapes = self.get_shapes();
        let mut buffer = Vec::new();
        for shape in shapes.iter() {
            let index = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &shape.indices).unwrap();
            buffer.push(index);
        }
        buffer
    }
    pub fn get_bounding_box(&mut self) -> BoundingBox {
        self.calculate_bounding_box()
    }

    pub fn get_materials(&self) -> &Vec<Uuid> {
        &self.materials
    }

    pub fn get_materials_mut(&mut self) -> &mut Vec<Uuid> {
        &mut self.materials
    }

    pub fn add_material(&mut self, material: Uuid) {
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

    pub fn get_animations(&self) -> &HashMap<String, animation::Animation> {
        &self.animations
    }
    pub fn get_animations_mut(&mut self) -> &mut HashMap<String, animation::Animation> {
        &mut self.animations
    }

    pub fn get_skeleton(&self) -> &Option<animation::Skeleton> {
        &self.skeleton
    }

    pub fn get_skeleton_mut(&mut self) -> &mut Option<animation::Skeleton> {
        &mut self.skeleton
    }

    pub fn try_fix_object(&mut self) -> Result<(), EnigmaError> {
        let mut errors = EnigmaError::new(None, true);
        if let Some(skeleton) = &mut self.skeleton {
            match skeleton.try_fix() {
                Ok(_) => {}
                Err(e) => errors.merge(e),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn load_from_obj(path: &str) -> Self {
        let input = BufReader::new(File::open(path).expect("Failed to open file"));
        let obj: Obj = load_obj(input).unwrap();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for vert in obj.vertices.iter() {
            let vertex = geometry::Vertex { position: vert.position, color: [1.0, 1.0, 1.0], texcoord: [0.0, 0.0], normal: vert.normal, bone_indices: [0, 0, 0, 0], bone_weights: [0.0, 0.0, 0.0, 0.0] };
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

    pub fn load_from_gltf_resource(data: &[u8]) -> Self {
        let (gltf, buffers, images) = gltf::import_slice(data).expect("Failed to import gltf file"); // gltf::import(path).expect("Failed to import gltf file");
        let object = Object::new(Some(String::from("INTERNAL ENIGMA RESOURCE")));
        Object::load_from_gltf_internal((gltf, buffers, images), object)
    }

    pub fn load_from_gltf(path: &str) -> Self {
        let (gltf, buffers, images) = gltf::import(path).expect("Failed to import gltf file");
        let object = Object::new(Some(String::from(path)));
        Object::load_from_gltf_internal((gltf, buffers, images), object)
    }

    fn load_from_gltf_internal(
        content: (gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>),
        mut object: Object,
    ) -> Self {
        let (gltf, buffers, _images) = content;

        // Load skeleton first
        let mut node_index_to_bone_index = HashMap::new();

        if let Some(skin) = gltf.skins().next() {
            let skeleton = Object::load_skeleton_internal(&gltf, &skin, &buffers);
            match skeleton.validate() {
                Err(e) => e.log(),
                Ok(_) => (),
            }

            // Build node_index_to_bone_index mapping
            for bone in &skeleton.bones {
                node_index_to_bone_index.insert(bone.node_index, bone.id);
            }

            object.skeleton = Some(skeleton);
        }

        // Now load meshes
        for mesh in gltf.meshes() {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| &data[..]));

                let positions = reader.read_positions().unwrap();
                let normals = reader.read_normals().unwrap();
                let tex_coords = reader.read_tex_coords(0).unwrap().into_f32();
                let prim_indices = reader.read_indices().unwrap().into_u32();

                // Read skinning data
                let joints = reader.read_joints(0).map(|j| j.into_u16());
                let weights = reader.read_weights(0).map(|w| w.into_f32());

                let mut flipped_tex_coords: Vec<[f32; 2]> = Vec::new();
                for mut tex_coord in tex_coords.into_iter() {
                    tex_coord[1] = 1.0 - tex_coord[1];
                    flipped_tex_coords.push(tex_coord);
                }

                let mut joint_data = joints.map(|j| j.map(|arr| [arr[0] as u32, arr[1] as u32, arr[2] as u32, arr[3] as u32]));
                let mut weight_data = weights;

                for ((position, normal), tex_coord) in positions.zip(normals).zip(flipped_tex_coords) {

                    let bone_indices = joint_data.as_mut().and_then(|j| j.next()).unwrap_or([0; 4]);
                    let bone_weight = weight_data.as_mut().and_then(|w| w.next()).unwrap_or([0.0; 4]);

                    // Map joint indices (node indices) to bone indices
                    let bone_indices = [
                        *node_index_to_bone_index.get(&(bone_indices[0] as usize)).unwrap_or(&0) as u32,
                        *node_index_to_bone_index.get(&(bone_indices[1] as usize)).unwrap_or(&0) as u32,
                        *node_index_to_bone_index.get(&(bone_indices[2] as usize)).unwrap_or(&0) as u32,
                        *node_index_to_bone_index.get(&(bone_indices[3] as usize)).unwrap_or(&0) as u32,
                    ];

                    let vertex = Vertex {
                        position,
                        texcoord: tex_coord,
                        color: [1.0, 1.0, 1.0],
                        normal,
                        bone_indices,
                        bone_weights: bone_weight,
                    };
                    vertices.push(vertex);
                }

                indices.extend(prim_indices);
            }
            let shape = Shape::from_vertices_indices(vertices, indices);
            object.add_shape(shape);
        }

        // Load animations
        let animations = gltf.animations();
        for (i, animation) in animations.enumerate() {
            let loaded_anim = Object::load_animation_internal(&animation, &buffers, i);
            object.animations.insert(loaded_anim.name.clone(), loaded_anim);
        }

        object
    }

    fn find_parent_bone_id(
        node_index: usize,
        node_parent_map: &HashMap<usize, usize>,
        joint_node_indices: &[usize],
    ) -> Option<usize> {
        let mut current_node_index = node_index;
        while let Some(&parent_node_index) = node_parent_map.get(&current_node_index) {
            if let Some(parent_bone_index) = joint_node_indices
                .iter()
                .position(|&idx| idx == parent_node_index)
            {
                return Some(parent_bone_index);
            }
            current_node_index = parent_node_index;
        }
        None
    }

    fn load_skeleton_internal(
        document: &gltf::Document,
        skin: &gltf::Skin,
        buffers: &[Data],
    ) -> Skeleton {
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));

        // Collect joint node indices
        let joint_node_indices: Vec<usize> = skin.joints().map(|j| j.index()).collect();

        // Read inverse bind matrices
        let inverse_bind_matrices: Vec<Matrix4<f32>> = reader
            .read_inverse_bind_matrices()
            .map(|matrices| matrices.map(Matrix4::from).collect())
            .unwrap_or_else(|| vec![Matrix4::identity(); joint_node_indices.len()]);

        // Ensure consistency
        assert_eq!(joint_node_indices.len(), inverse_bind_matrices.len());

        // Build a map from node index to parent node index
        let mut node_parent_map = HashMap::new();
        for node in document.nodes() {
            for child in node.children() {
                node_parent_map.insert(child.index(), node.index());
            }
        }

        // Build bones
        let bones: Vec<Bone> = joint_node_indices
            .iter()
            .enumerate()
            .map(|(bone_index, &node_index)| {
                // Get the node corresponding to this joint
                let node = document.nodes().find(|n| n.index() == node_index).unwrap();

                // Find the parent bone index
                let parent_bone_id =
                    Object::find_parent_bone_id(node_index, &node_parent_map, &joint_node_indices);

                Bone {
                    name: node.name().unwrap_or("").to_string(),
                    id: bone_index,
                    node_index,  // Store node index
                    parent_id: parent_bone_id,
                    inverse_bind_pose: inverse_bind_matrices[bone_index],
                }
            })
            .collect();

        Skeleton { bones }
    }

    pub fn load_animation_internal(
        anim: &gltf::Animation,
        buffers: &[Data],
        padding: usize
    ) -> Animation {
        let mut bone_channels: HashMap<usize, AnimationChannel> = HashMap::new();
        let mut duration: f32 = 0.0;
        let name = anim.name().map_or_else(
            || format!("animation_{}", padding),
            |n| n.to_string(),
        );

        for channel in anim.channels() {
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            let bone_id = channel.target().node().index();

            if let (Some(times), Some(outputs)) = (reader.read_inputs(), reader.read_outputs()) {
                let times: Vec<f32> = times.collect();

                if let Some(&channel_duration) = times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                    duration = duration.max(channel_duration);
                }

                // Get or create the AnimationChannel for this bone
                let animation_channel = bone_channels
                    .entry(bone_id)
                    .or_insert_with(|| AnimationChannel {
                        bone_id,
                        translations: Vec::new(),
                        rotations: Vec::new(),
                        scales: Vec::new(),
                    });

                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translations) => {
                        for (i, translation) in translations.enumerate() {
                            let translation = Vector3::from(translation) * 0.01;
                            let keyframe = AnimationKeyframe {
                                time: times[i],
                                value: translation.into(),
                            };
                            animation_channel.translations.push(keyframe);
                        }
                    }
                    gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                        for (i, rotation) in rotations.into_f32().enumerate() {
                            let keyframe = AnimationKeyframe {
                                time: times[i],
                                value: rotation,
                            };
                            animation_channel.rotations.push(keyframe);
                        }
                    }
                    gltf::animation::util::ReadOutputs::Scales(scales) => {
                        for (i, scale) in scales.enumerate() {
                            let scale = Vector3::from(scale);
                            let keyframe = AnimationKeyframe {
                                time: times[i],
                                value: scale.into(),
                            };
                            animation_channel.scales.push(keyframe);
                        }
                    }
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {
                        // Handle morph target weights if needed
                    }
                }
            }
        }

        // Sort keyframes in each AnimationChannel
        let channels = bone_channels
            .into_iter()
            .map(|(_, mut channel)| {
                channel.translations.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                channel.rotations.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                channel.scales.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                channel
            })
            .collect();

        Animation {
            name,
            duration,
            channels,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransformSerializer {
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
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

    pub fn forward(&self) -> Vector3<f32> {
        // return the forward vector of the transform with positive z being forward
        let rotation = UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let forward = rotation * Vector3::new(0.0, 0.0, 1.0);
        normalize(&forward)
    }

    pub fn left(&self) -> Vector3<f32> {
        // return the left vector of the transform with positive x being left
        let rotation = UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let left = rotation * Vector3::new(-1.0, 0.0, 0.0);
        normalize(&left)
    }

    pub fn up(&self) -> Vector3<f32> {
        // return the up vector of the transform with positive y being up
        let rotation = UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let up = rotation * Vector3::new(0.0, 1.0, 0.0);
        normalize(&up)
    }

    pub fn from_serializer(serializer: TransformSerializer) -> Self {
        let mut t = Transform::new();
        t.set_position(serializer.position);
        t.set_rotation(serializer.rotation);
        t.set_scale(serializer.scale);
        t
    }

    pub fn to_serializer(&self) -> TransformSerializer {
        TransformSerializer {
            position: self.get_position().into(),
            rotation: self.get_rotation().into(),
            scale: self.get_scale().into(),
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

    pub fn move_dir_array(&mut self, position: [f32; 3]) {
        let cur_p = self.get_position();
        let additive_position = [cur_p.x + position[0], cur_p.y + position[1], cur_p.z + position[2]];
        self.position = Vector3::from(additive_position);
    }

    pub fn move_dir_vector(&mut self, direction: Vector3<f32>) {
        self.position += direction;
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

    pub fn get_matrix_object(&mut self) -> Matrix4<f32> {
        self.update();
        self.matrix
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let position = self.get_position().lerp(&other.get_position(), t);
        let scale = self.get_scale().lerp(&other.get_scale(), t);
        let rotation = self.get_rotation().slerp(&other.get_rotation(), t);

        let mut result = Self::new();
        result.set_position(position.into());
        result.set_scale(scale.into());
        result.set_rotation(rotation.into());
        result
    }
}
