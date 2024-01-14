use nalgebra::Vector3;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Copy, Clone)]
pub struct BoundingBox {
    pub center: Vector3<f32>,
    //relative to the objects position
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

pub struct BoundingBoxMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

glium::implement_vertex!(Vertex, position, texcoord, color, normal);


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
}