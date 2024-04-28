use std::fmt::Debug;
use nalgebra::{Matrix4, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Plane {
    pub normal: [f32; 3],
    pub distance: f32,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

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

glium::implement_vertex!(Vertex, position, texcoord, color, normal);

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

impl Frustum {
    pub fn new(view_projection_matrix: Matrix4<f32>) -> Self {
        let m = view_projection_matrix.as_slice();
        let mut planes = [
            Plane::new(
                Vector3::new(m[3] + m[0], m[7] + m[4], m[11] + m[8]),
                m[15] + m[12],
            ), // Left
            Plane::new(
                Vector3::new(m[3] - m[0], m[7] - m[4], m[11] - m[8]),
                m[15] - m[12],
            ), // Right
            Plane::new(
                Vector3::new(m[3] + m[1], m[7] + m[5], m[11] + m[9]),
                m[15] + m[13],
            ), // Bottom
            Plane::new(
                Vector3::new(m[3] - m[1], m[7] - m[5], m[11] - m[9]),
                m[15] - m[13],
            ), // Top
            Plane::new(
                Vector3::new(m[3] + m[2], m[7] + m[6], m[11] + m[10]),
                m[15] + m[14],
            ), // Near
            Plane::new(
                Vector3::new(m[3] - m[2], m[7] - m[6], m[11] - m[10]),
                m[15] - m[14],
            ), // Far
        ];
        // Normalize all planes
        for plane in &mut planes {
            plane.normalize();
        }

        Self {
            planes,
        }
    }

    pub fn update(&mut self, view_projection_matrix: Matrix4<f32>) {
        let m = view_projection_matrix.as_slice();
        self.planes[0] = Plane::new(
            Vector3::new(m[3] + m[0], m[7] + m[4], m[11] + m[8]),
            m[15] + m[12],
        ); // Left
        self.planes[1] = Plane::new(
            Vector3::new(m[3] - m[0], m[7] - m[4], m[11] - m[8]),
            m[15] - m[12],
        ); // Right
        self.planes[2] = Plane::new(
            Vector3::new(m[3] + m[1], m[7] + m[5], m[11] + m[9]),
            m[15] + m[13],
        ); // Bottom
        self.planes[3] = Plane::new(
            Vector3::new(m[3] - m[1], m[7] - m[5], m[11] - m[9]),
            m[15] - m[13],
        ); // Top
        self.planes[4] = Plane::new(
            Vector3::new(m[3] + m[2], m[7] + m[6], m[11] + m[10]),
            m[15] + m[14],
        ); // Near
        self.planes[5] = Plane::new(
            Vector3::new(m[3] - m[2], m[7] - m[6], m[11] - m[10]),
            m[15] - m[14],
        ); // Far
    }

    pub fn default() -> Self {
        Self {
            planes: [
                Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0),
                Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0),
                Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0),
                Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0),
                Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0),
                Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0),
            ],
        }

    }
}

impl Plane {
    pub fn new(normal: Vector3<f32>, distance: f32) -> Self {
        let len = normal.magnitude();
        Plane {
            normal: (normal / len).into(),
            distance: distance / len,
        }
    }

    fn normalize(&mut self) {
        let mag = (self.normal[0].powi(2) + self.normal[1].powi(2) + self.normal[2].powi(2)).sqrt();
        self.normal[0] /= mag;
        self.normal[1] /= mag;
        self.normal[2] /= mag;
        self.distance /= mag;
    }

    pub fn intersects(&self, bounding_box: &BoundingBox) -> bool {
        let min = bounding_box.min_point();
        let max = bounding_box.max_point();

        // Choose the point most positive along the plane's normal direction
        let p_vertex = Vector3::new(
            if self.normal[0] >= 0.0 { max.x } else { min.x },
            if self.normal[1] >= 0.0 { max.y } else { min.y },
            if self.normal[2] >= 0.0 { max.z } else { min.z },
        );

        // Compute the dot product of the plane's normal and p_vertex, check against plane's distance
        let d = Vector3::from(self.normal).dot(&p_vertex) + self.distance;  // Assuming 'self.distance' is the plane's distance from the origin

        // If d is greater than zero, the bounding box intersects or is in front of the plane
        d >= 0.0
    }
}