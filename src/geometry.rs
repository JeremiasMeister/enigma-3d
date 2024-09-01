use std::fmt::Debug;
use glium::{implement_uniform_block, implement_vertex};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub struct BoneTransforms {
    pub bone_transforms: [[[f32; 4]; 4]; 128]
}
implement_uniform_block!(BoneTransforms, bone_transforms);

#[derive(Copy, Clone)]
pub struct InstanceAttribute {
    pub model_matrix: [[f32; 4]; 4],
}
implement_vertex!(InstanceAttribute, model_matrix);

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub bone_indices: [u32; 4],
    pub bone_weights: [f32; 4],
}

glium::implement_vertex!(Vertex, position, texcoord, color, normal, bone_indices, bone_weights);


#[derive(Serialize, Deserialize)]
pub struct BoundingBoxSerializer {
    pub center: [f32; 3],
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

#[derive(Copy, Clone)]
pub struct BoundingBox {
    pub center: Vector3<f32>,
    //relative to the objects position
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BoundingBoxMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Debug for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vertex")
            .field("position", &self.position)
            .field("texcoord", &self.texcoord)
            .field("color", &self.color)
            .field("normal", &self.normal)
            .finish()
    }
}

impl BoundingBoxMesh {
    pub fn new(bounding_box: &BoundingBox) -> Self {
        let half_width = bounding_box.width / 2.0;
        let half_height = bounding_box.height / 2.0;
        let half_depth = bounding_box.depth / 2.0;

        let corners = [
            bounding_box.center + Vector3::new(-half_width, -half_height, -half_depth),
            bounding_box.center + Vector3::new(half_width, -half_height, -half_depth),
            bounding_box.center + Vector3::new(half_width, half_height, -half_depth),
            bounding_box.center + Vector3::new(-half_width, half_height, -half_depth),
            bounding_box.center + Vector3::new(-half_width, -half_height, half_depth),
            bounding_box.center + Vector3::new(half_width, -half_height, half_depth),
            bounding_box.center + Vector3::new(half_width, half_height, half_depth),
            bounding_box.center + Vector3::new(-half_width, half_height, half_depth),
        ];

        let mut vertices = Vec::new();

        for i in 0..corners.len() {
            let corner = corners[i];
            vertices.push(Vertex {
                position: [corner.x, corner.y, corner.z],
                texcoord: [0.0, 0.0],
                color: [1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
                bone_indices: [0, 0, 0, 0],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            });
        }
        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front face
            1, 5, 6, 6, 2, 1, // Right face
            5, 4, 7, 7, 6, 5, // Back face
            4, 0, 3, 3, 7, 4, // Left face
            3, 2, 6, 6, 7, 3, // Top face
            4, 5, 1, 1, 0, 4, // Bottom face
        ];

        Self {
            vertices,
            indices,
        }
    }
}

impl BoundingBox {
    // Returns the minimum point of the bounding box
    pub fn min_point(&self) -> Vector3<f32> {
        Vector3::new(
            self.center.x - self.width / 2.0,
            self.center.y - self.height / 2.0,
            self.center.z - self.depth / 2.0,
        )
    }

    // Returns the maximum point of the bounding box
    pub fn max_point(&self) -> Vector3<f32> {
        Vector3::new(
            self.center.x + self.width / 2.0,
            self.center.y + self.height / 2.0,
            self.center.z + self.depth / 2.0,
        )
    }

    pub fn to_serializer(&self) -> BoundingBoxSerializer {
        BoundingBoxSerializer {
            center: [self.center.x, self.center.y, self.center.z],
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }

    pub fn from_serializer(serializer: BoundingBoxSerializer) -> Self {
        Self {
            center: Vector3::new(serializer.center[0], serializer.center[1], serializer.center[2]),
            width: serializer.width,
            height: serializer.height,
            depth: serializer.depth,
        }
    }
}