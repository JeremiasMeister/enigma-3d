use std::collections::HashMap;
use crate::object;
use nalgebra::Vector3;

struct RayCast<'a> {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
    intersection_objects: HashMap<Vector3<f32>, &'a object::Object>
}

impl RayCast<'_> {

}