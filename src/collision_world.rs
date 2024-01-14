use std::collections::HashMap;
use std::fmt::Debug;
use crate::{AppState};
use nalgebra::Vector3;
use uuid::Uuid;
use crate::camera::Camera;
use crate::geometry::{BoundingBox};

pub struct MousePosition {
    pub screen_space: (f64, f64),
    pub world_space: Vector3<f32>,
}

pub struct RayCast {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
    intersection_objects: HashMap<Uuid, Vector3<f32>>,
    block_first_hit: bool,
}

impl Debug for MousePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MousePosition")
            .field("screen_space", &self.screen_space)
            .field("world_space", &self.world_space)
            .finish()
    }
}

impl MousePosition {
    pub fn new() -> Self {
        Self {
            screen_space: (0.0, 0.0),
            world_space: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn get_world_position(&self, camera: &Camera, depth: f32) -> Vector3<f32> {
        // Convert screen coordinates to normalized device coordinates (NDC)
        let x = (2.0 * self.screen_space.0 as f32) / camera.width - 1.0;
        let y = 1.0 - (2.0 * self.screen_space.1 as f32) / camera.height;
        let ndc = nalgebra::Vector4::new(x, y, -1.0, 1.0); // Using -1.0 for Z to get a point on the near plane

        // Convert NDC to camera/eye space
        let projection_matrix = nalgebra::Matrix4::from(camera.get_projection_matrix());
        let inverse_projection_matrix = projection_matrix.try_inverse().unwrap();
        let eye_space = inverse_projection_matrix * ndc;

        // Adjust the point to the specified depth along the camera's forward vector
        let normalized_device_coordinates = nalgebra::Vector3::new(eye_space.x, eye_space.y, eye_space.z) / eye_space.w;
        let camera_space_point = normalized_device_coordinates * depth;

        // Convert from camera/eye space to world space
        let view_matrix = nalgebra::Matrix4::from(camera.get_view_matrix());
        let inverse_view_matrix = view_matrix.try_inverse().unwrap();
        let world_space_point = inverse_view_matrix.transform_point(&nalgebra::Point3::from(camera_space_point));

        Vector3::new(world_space_point.x, world_space_point.y, world_space_point.z)
    }


    pub fn get_screen_position(&self) -> (f64, f64) {
        self.screen_space
    }

    pub fn set_screen_position(&mut self, position: (f64, f64)) {
        self.screen_space = position;
    }
}

impl RayCast {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>, length: f32, block_first_hit: bool) -> Self {
        Self {
            origin,
            direction,
            length,
            intersection_objects: HashMap::new(),
            block_first_hit
        }
    }

    pub fn get_intersection_map(&self) -> &HashMap<Uuid, Vector3<f32>> {
        &self.intersection_objects
    }

    pub fn get_intersection_uuids(&self) -> Vec<Uuid> {
        let mut uuids = Vec::new();
        for (uuid, _) in self.intersection_objects.iter() {
            uuids.push(*uuid);
        }
        uuids
    }

    pub fn get_intersection_points(&self) -> Vec<Vector3<f32>> {
        let mut points = Vec::new();
        for (_, point) in self.intersection_objects.iter() {
            points.push(*point);
        }
        points
    }

    pub fn cast(&mut self, app_state: &mut AppState) {
        for object in app_state.objects.iter_mut() {
            let aabb = object.get_bounding_box();
            match self.intersects_bounding_box(&aabb) {
                Some(intersection_point) => {
                    self.intersection_objects.insert(object.get_unique_id(), intersection_point);
                },
                None => {}
            }
        }
    }

    fn intersects_bounding_box(&self, bounding_box: &BoundingBox) -> Option<Vector3<f32>> {
        let inv_direction = Vector3::new(1.0 / self.direction.x, 1.0 / self.direction.y, 1.0 / self.direction.z);
        let sign = [
            (inv_direction.x < 0.0) as usize,
            (inv_direction.y < 0.0) as usize,
            (inv_direction.z < 0.0) as usize,
        ];

        let bbox = [bounding_box.min_point(), bounding_box.max_point()];
        let mut tmin = (bbox[sign[0]].x - self.origin.x) * inv_direction.x;
        let mut tmax = (bbox[1 - sign[0]].x - self.origin.x) * inv_direction.x;
        let tymin = (bbox[sign[1]].y - self.origin.y) * inv_direction.y;
        let tymax = (bbox[1 - sign[1]].y - self.origin.y) * inv_direction.y;

        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let tzmin = (bbox[sign[2]].z - self.origin.z) * inv_direction.z;
        let tzmax = (bbox[1 - sign[2]].z - self.origin.z) * inv_direction.z;

        if (tmin > tzmax) || (tzmin > tmax) {
            return None;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        if tzmax < tmax {
            tmax = tzmax;
        }

        if (tmin < self.length) && (tmax > 0.0) {
            return Some(self.origin + tmin * self.direction);
        }

        None
    }
}