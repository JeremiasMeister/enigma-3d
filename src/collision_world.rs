use std::collections::HashMap;
use crate::{AppState, object};
use nalgebra::Vector3;
use uuid::Uuid;
use crate::geometry::BoundingSphere;

struct RayCast {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
    intersection_objects: HashMap<Uuid, Vector3<f32>>,
    block_first_hit: bool,
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

    pub fn intersects(&self, sphere: &BoundingSphere) -> Option<Vector3<f32>> {
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