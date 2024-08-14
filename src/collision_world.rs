use std::fmt::Debug;
use crate::AppState;
use nalgebra::{Matrix4, Point3, Vector3, Vector4};
use uuid::Uuid;
use crate::camera::Camera;
use crate::geometry::BoundingBox;


pub struct MouseState {
    pub current_position: (f64, f64),
    previous_position: (f64, f64),
    delta: (f64, f64),
    pub world_space: Vector3<f32>,
}

pub struct RayCast {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
    intersection_objects: indexmap::IndexMap<Uuid, Vector3<f32>>
}

pub fn is_colliding(aabb1: &BoundingBox, aabb2: &BoundingBox) -> bool {
    let aabb1_min = aabb1.min_point();
    let aabb1_max = aabb1.max_point();
    let aabb2_min = aabb2.min_point();
    let aabb2_max = aabb2.max_point();

    if aabb1_min.x > aabb2_max.x || aabb1_max.x < aabb2_min.x {
        return false;
    }

    if aabb1_min.y > aabb2_max.y || aabb1_max.y < aabb2_min.y {
        return false;
    }

    if aabb1_min.z > aabb2_max.z || aabb1_max.z < aabb2_min.z {
        return false;
    }

    true
}

impl Debug for MouseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MousePosition")
            .field("current_position", &self.current_position)
            .field("previous_position", &self.previous_position)
            .field("delta", &self.delta)
            .field("world_space", &self.world_space)
            .finish()
    }
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            current_position: (0.0, 0.0),
            previous_position: (0.0, 0.0),
            delta: (0.0,0.0),
            world_space: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn get_world_position(&self, camera: &Camera) -> (Vector3<f32>, Vector3<f32>) {
        let clip_space_x = (self.current_position.0 as f32 / camera.width) * 2.0 - 1.0;
        let clip_space_y = 1.0-(self.current_position.1 as f32 / camera.height) * 2.0;
        let clip_space_z = -1.0;
        let clip_space_coord: Vector4<f32> = Vector4::new(clip_space_x, clip_space_y, clip_space_z, 1.0);
        let view_space_coord = Matrix4::from(camera.get_projection_matrix()).try_inverse().unwrap().transform_point(&Point3::from_homogeneous(clip_space_coord).unwrap());
        let world_space_coord = Matrix4::from(camera.get_view_matrix()).try_inverse().unwrap().transform_point(&view_space_coord);
        let world_space_point: Point3<f32> = world_space_coord.xyz().into();
        let ray_direction: Vector3<f32> = (world_space_point - camera.transform.get_position()).coords.normalize();

        (world_space_point.coords, ray_direction)
    }

    pub fn get_screen_position(&self) -> (f64, f64) {
        self.current_position
    }

    pub fn update_position(&mut self, new_position: (f64, f64)) {
        self.previous_position = self.current_position;
        self.current_position = new_position;
        self.delta = (
            self.current_position.0 - self.previous_position.0,
            self.current_position.1 - self.previous_position.1,
        );
    }
    pub fn get_delta(&self) -> (f64, f64) {
        self.delta
    }
}

impl RayCast {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>, length: f32) -> Self {
        Self {
            origin,
            direction,
            length,
            intersection_objects: indexmap::IndexMap::new(),
        }
    }

    pub fn get_intersection_map(&self) -> &indexmap::IndexMap<Uuid, Vector3<f32>> {
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
            if object.get_collision() == &false{
                continue
            }
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