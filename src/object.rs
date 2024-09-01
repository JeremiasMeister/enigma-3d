use std::collections::HashMap;
use std::vec::Vec;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use crate::geometry::{BoneTransforms, BoundingBox, Vertex};
use nalgebra::{Vector3, Matrix4, Translation3, UnitQuaternion, Point3};
use crate::{animation, debug_geo, geometry};
use uuid::Uuid;


use std::fs::File;
use std::io::BufReader;
use glium::uniforms::UniformBuffer;
use nalgebra_glm::normalize;
use obj::{load_obj, Obj};
use serde::{Deserialize, Serialize};

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
    animations: HashMap<String, animation::AnimationSerializer>,
    skeleton: Option<animation::SkeletonSerializer>,
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
    skeleton: Option<animation::Skeleton>,
    current_animation: Option<String>,
    animation_time: f32,
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
            animation_time: 0.0,
        };
        object.calculate_bounding_box();
        object
    }

    pub fn to_serializer(&self) -> ObjectSerializer {
        let name = self.name.clone();
        let transform = self.transform.to_serializer();
        let mut animations = HashMap::new();
        for (n, a) in &self.animations {
            animations.insert(n.to_string(), a.to_serializer());
        }
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
            animations,
            skeleton: match &self.skeleton {
                Some(skeleton) => Some(skeleton.to_serializer()),
                None => None
            },
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

        let mut animations = HashMap::new();
        for (n, s) in serializer.animations {
            let anim = animation::Animation::from_serializer(s);
            animations.insert(n, anim);
        }
        object.animations = animations;
        object.skeleton = match serializer.skeleton {
            Some(s) => Some(animation::Skeleton::from_serializer(s)),
            None => None
        };
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
        if let Some(anim_name) = &self.current_animation {
            let animation = self.animations.get(anim_name);
            match animation {
                Some(anim) => {
                    self.animation_time += delta_time;
                    if self.animation_time > anim.duration {
                        self.animation_time %= anim.duration;
                    }
                }
                None => self.animation_time = 0.0
            }
        }
    }

    pub fn get_bone_transform_buffer(&self, display: &Display<WindowSurface>) -> UniformBuffer<BoneTransforms> {
        let mut bone_transform_data = BoneTransforms {
            bone_transforms: [
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ]; 128
            ],
        };
        if let Some(skeleton) = &self.skeleton {
            if let Some(anim_name) = &self.current_animation {
                match self.animations.get(anim_name) {
                    Some(animation) => {
                        for (i, bone) in skeleton.bones.iter().take(128).enumerate() {
                            // taking only 100 as a hard capped bone limit
                            let local_transform = self.interpolate_keyframes(animation, i, self.animation_time);
                            let parent_transform = Matrix4::from(bone.parent_id.map_or([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]], |parent_id| bone_transform_data.bone_transforms[parent_id]));
                            let global_transform = parent_transform * local_transform;
                            bone_transform_data.bone_transforms[i] = (global_transform * bone.inverse_bind_pose).into();
                        }
                    }
                    None => ()
                }
            } else {
                // No animation is playing, return the skeleton's bones in their bind pose
                for (i, bone) in skeleton.bones.iter().take(128).enumerate() {
                    let parent_transform = Matrix4::from(bone.parent_id.map_or(
                        [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
                        |parent_id| bone_transform_data.bone_transforms[parent_id]
                    ));
                    let global_transform = parent_transform; // No local transform when in bind pose
                    bone_transform_data.bone_transforms[i] = (global_transform * bone.inverse_bind_pose).into();
                }
            }
        }

        UniformBuffer::new(display, bone_transform_data).expect("Failed to create BoneTransform Buffer")
    }

    fn interpolate_keyframes(&self, animation: &animation::Animation, bone_id: usize, time: f32) -> Matrix4<f32> {
        // Implement keyframe interpolation logic here
        // This will depend on how your keyframes are stored and what interpolation method you want to use
        // For simplicity, let's assume linear interpolation between two keyframes

        if let Some(channel) = animation.channels.iter().find(|c| c.bone_id == bone_id) {
            let mut prev_keyframe = &channel.keyframes[0];
            let mut next_keyframe = prev_keyframe;

            for keyframe in &channel.keyframes {
                if keyframe.time > time {
                    next_keyframe = keyframe;
                    break;
                }
                prev_keyframe = keyframe;
            }

            let t = (time - prev_keyframe.time) / (next_keyframe.time - prev_keyframe.time);

            // Interpolate between prev_keyframe and next_keyframe based on t
            // This is a simplified example, you might need to handle different transform types
            match (&prev_keyframe.transform, &next_keyframe.transform) {
                (animation::AnimationTransform::Translation(prev), animation::AnimationTransform::Translation(next)) => {
                    let interpolated = [
                        prev[0] + (next[0] - prev[0]) * t,
                        prev[1] + (next[1] - prev[1]) * t,
                        prev[2] + (next[2] - prev[2]) * t,
                    ];
                    Matrix4::from([
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [interpolated[0], interpolated[1], interpolated[2], 1.0],
                    ])
                }
                // Handle other transform types (rotation, scale) similarly
                _ => Matrix4::from([
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ]) // Return identity matrix if no matching transform type
            }
        } else {
            Matrix4::from([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]]) // Return identity matrix if no animation channel for this bone
        }
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

    fn load_from_gltf_internal(content: (gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>), mut object: Object) -> Self {
        let (gltf, buffers, _images) = content;
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

        if let Some(skin) = gltf.skins().next() {
            let skeleton = Object::load_skeleton_internal(&gltf, &skin, &buffers);
            object.skeleton = Some(skeleton);
        }
        let animations = gltf.animations();
        for (i, animation) in animations.enumerate() {
            let loaded_anim = Object::load_animation_internal(&animation, &buffers, i);
            object.animations.insert(loaded_anim.name.clone(), loaded_anim);
        }
        object
    }

    fn load_skeleton_internal(document: &gltf::Document, skin: &gltf::Skin, buffers: &[gltf::buffer::Data]) -> animation::Skeleton {
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));

        // Get joints from the skin
        let joints: Vec<gltf::Node> = skin.joints().collect();

        // Read inverse bind matrices
        let inverse_bind_matrices: Vec<Matrix4<f32>> = reader.read_inverse_bind_matrices()
            .map(|iter| iter.map(Matrix4::from).collect())
            .unwrap_or_else(|| vec![Matrix4::identity(); joints.len()]);

        // Create a map of child to parent relationships
        let mut parent_map = HashMap::new();
        for node in document.nodes() {
            for child in node.children() {
                parent_map.insert(child.index(), node.index());
            }
        }

        let bones = joints.into_iter().enumerate().zip(inverse_bind_matrices).map(|((id, joint), ibm)| {
            animation::Bone {
                name: joint.name().unwrap_or("").to_string(),
                id,
                parent_id: parent_map.get(&joint.index()).cloned(),
                inverse_bind_pose: ibm,
            }
        }).collect();

        animation::Skeleton { bones }
    }

    fn load_animation_internal(anim: &gltf::Animation, buffers: &[gltf::buffer::Data], padding: usize) -> animation::Animation {
        let mut channels = Vec::new();
        let mut duration: f32 = 0.0;
        let name = match anim.name() {
            Some(n) => n.to_string(),
            None => format!("animation_{}", padding)
        };
        for channel in anim.channels() {
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            let bone_id = channel.target().node().index();
            let mut keyframes = Vec::new();
            if let (Some(times), Some(outputs)) = (reader.read_inputs(), reader.read_outputs()) {
                let times: Vec<f32> = times.collect();
                // Update max_time
                if let Some(&channel_duration) = times.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)) {
                    duration = duration.max(channel_duration);
                }
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translations) => {
                        for (i, translation) in translations.enumerate() {
                            let translation = Vector3::from(translation);
                            keyframes.push(animation::AnimationKeyframe {
                                time: times[i],
                                transform: animation::AnimationTransform::Translation(translation.into()),
                            });
                        }
                    }
                    gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                        for (i, rotation) in rotations.into_f32().enumerate() {
                            let rotation = UnitQuaternion::from_quaternion(
                                nalgebra::Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2])
                            );
                            keyframes.push(animation::AnimationKeyframe {
                                time: times[i],
                                transform: animation::AnimationTransform::Rotation([rotation[0], rotation[1], rotation[2], rotation[3]]),
                            });
                        }
                    }
                    gltf::animation::util::ReadOutputs::Scales(scales) => {
                        for (i, scale) in scales.enumerate() {
                            let scale = Vector3::from(scale);
                            keyframes.push(animation::AnimationKeyframe {
                                time: times[i],
                                transform: animation::AnimationTransform::Scale(scale.into()),
                            });
                        }
                    }
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {
                        // Handle morph target weights if needed
                        // For now, we'll just ignore these
                    }
                }
            }

            if !keyframes.is_empty() {
                channels.push(animation::AnimationChannel { bone_id, keyframes });
            }
        }

        animation::Animation {
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
