use std::collections::HashMap;
use std::fmt::Debug;
use crate::{AppState};
use nalgebra::Vector3;
use uuid::Uuid;
use crate::camera::Camera;
use crate::geometry::BoundingSphere;

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

    pub fn get_world_position(&self, camera: &Camera) -> Vector3<f32> {
        let x = (2.0 * self.screen_space.0 as f32) / camera.width - 1.0;
        let y = 1.0 - (2.0 * self.screen_space.1 as f32) / camera.height;
        let ndc: nalgebra::Vector4<f32> = nalgebra::Vector4::new(x, y, 1.0, 1.0); // Assuming Z as 1.0 for the far plane

        let projection_matrix: nalgebra::Matrix4<f32> = nalgebra::Matrix4::from(camera.get_projection_matrix());
        let view_matrix: nalgebra::Matrix4<f32> = nalgebra::Matrix4::from(camera.get_view_matrix());

        let view_space = projection_matrix.try_inverse().unwrap() * ndc;

        let world_space = view_matrix.try_inverse().unwrap() * view_space;

        Vector3::new(world_space.x, world_space.y, world_space.z)
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
        for mut object in app_state.objects.iter_mut() {
            let collision_sphere = object.get_bounding_sphere();
            match self.intersects(&collision_sphere) {
                Some(intersection_point) => {
                    self.intersection_objects.insert(object.get_unique_id(), intersection_point);
                },
                None => {}
            }
        }
    }

    fn intersects(&self, sphere: &BoundingSphere) -> Option<Vector3<f32>> {
        let ray_to_sphere = sphere.center - self.origin; // Convert origin to array for subtraction
        let ray_direction = self.direction; // Convert direction to array

        let a = ray_direction.dot(&ray_direction);
        let b = 2.0 * ray_to_sphere.dot(&ray_direction);
        let c = ray_to_sphere.dot(&ray_to_sphere) - sphere.radius * sphere.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant >= 0.0 {
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-b - sqrt_discriminant) / (2.0 * a);
            let t2 = (-b + sqrt_discriminant) / (2.0 * a);

            if t1 >= 0.0 && (t1 <= self.length || !self.block_first_hit) {
                let intersection_point = self.origin + t1 * self.direction;
                return Some(Vector3::from(intersection_point));
            }

            if t2 >= 0.0 && (t2 <= self.length || !self.block_first_hit) {
                let intersection_point = self.origin + t2 * self.direction;
                return Some(Vector3::from(intersection_point));
            }
        }

        None
    }
}