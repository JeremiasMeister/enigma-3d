use std::collections::HashMap;
use std::fmt::Debug;
use crate::AppState;
use nalgebra::{Matrix4, Point3, Vector3, Vector4};
use uuid::Uuid;
use crate::camera::Camera;
use crate::geometry::BoundingBox;

pub struct MousePosition {
    pub screen_space: (f64, f64),
    pub world_space: Vector3<f32>,
}

pub struct RayCast {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
    intersection_objects: HashMap<Uuid, Vector3<f32>>
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

    pub fn get_world_position(&self, camera: &Camera) -> (Vector3<f32>, Vector3<f32>) {
        let clip_space_x = (self.screen_space.0 as f32 / camera.width) * 2.0 - 1.0;
        let clip_space_y = 1.0-(self.screen_space.1 as f32 / camera.height) * 2.0;
        let clip_space_z = -1.0;
        let clip_space_coord: Vector4<f32> = Vector4::new(clip_space_x, clip_space_y, clip_space_z, 1.0);
        let view_space_coord = Matrix4::from(camera.get_projection_matrix()).try_inverse().unwrap().transform_point(&Point3::from_homogeneous(clip_space_coord).unwrap());
        let world_space_coord = Matrix4::from(camera.get_view_matrix()).try_inverse().unwrap().transform_point(&view_space_coord);
        let world_space_point: Point3<f32> = world_space_coord.xyz().into();
        let ray_direction: Vector3<f32> = (world_space_point - camera.transform.get_position()).coords.normalize();

        (world_space_point.coords, ray_direction)
    }

    pub fn get_screen_position(&self) -> (f64, f64) {
        self.screen_space
    }

    pub fn set_screen_position(&mut self, position: (f64, f64)) {
        self.screen_space = position;
    }
}

impl RayCast {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>, length: f32) -> Self {
        Self {
            origin,
            direction,
            length,
            intersection_objects: HashMap::new(),
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