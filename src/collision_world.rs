use std::fmt::Debug;
use crate::AppState;
use nalgebra::{Matrix4, Point3, UnitQuaternion, Vector3, Vector4};
use uuid::Uuid;
use crate::camera::Camera;
use crate::geometry::BoundingBox;
use crate::object::Transform;


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

/// A single contact result from any collision query.
#[derive(Clone, Debug)]
pub struct Hit {
    /// UUID of the hit object, or `Uuid::nil()` if the hit is against terrain.
    pub object_id: Uuid,
    /// World-space contact point.
    pub point: Vector3<f32>,
    /// World-space contact normal, pointing away from the hit surface.
    pub normal: Vector3<f32>,
    /// Time of impact along a sweep, in [0.0, 1.0]. For overlap queries, 0.0.
    pub toi: f32,
}

/// A triangle-soup collider, stored in local (object) space.
/// Static-only: meshes are queried *against* but do not move and collide with other meshes.
#[derive(Clone, Debug)]
pub struct MeshCollider {
    pub triangles: Vec<[Vector3<f32>; 3]>,
}

impl MeshCollider {
    pub fn from_triangles(triangles: Vec<[Vector3<f32>; 3]>) -> Self {
        Self { triangles }
    }
}

/// All collider shapes supported by collision_world.
/// When stored on an `Object`, positions are in local (object-relative) space.
/// When passed to a query function directly, positions are in world space.
#[derive(Clone, Debug)]
pub enum Collider {
    Aabb(BoundingBox),
    Sphere   { center: Vector3<f32>, radius: f32 },
    Capsule  { a: Vector3<f32>, b: Vector3<f32>, radius: f32 },
    Obb      { center: Vector3<f32>, half_extents: Vector3<f32>, rotation: UnitQuaternion<f32> },
    Plane    { normal: Vector3<f32>, offset: f32 },
    Mesh(MeshCollider),
    Heightfield,
}

impl Collider {
    /// Transforms a local-space collider into world space using the object's transform.
    /// For Plane, Mesh, and Heightfield, the result equals the input (those are world-space
    /// by convention, regardless of which object owns them).
    pub fn to_world_space(&self, transform: &Transform) -> Collider {
        let pos: Vector3<f32> = transform.get_position();
        let scale: Vector3<f32> = transform.get_scale();
        // Transform.rotation is Euler angles in radians (Vector3<f32>); see object.rs Transform.
        let rot_euler: Vector3<f32> = transform.rotation;
        let rot = UnitQuaternion::from_euler_angles(rot_euler.x, rot_euler.y, rot_euler.z);
        let uniform_scale = (scale.x + scale.y + scale.z) / 3.0;

        match self {
            Collider::Aabb(bb) => {
                // Transform bbox corners and rebuild AABB. Loses precision on rotation —
                // for rotated objects, prefer Obb.
                let min = bb.min_point();
                let max = bb.max_point();
                let corners = [
                    Vector3::new(min.x, min.y, min.z),
                    Vector3::new(max.x, min.y, min.z),
                    Vector3::new(min.x, max.y, min.z),
                    Vector3::new(max.x, max.y, min.z),
                    Vector3::new(min.x, min.y, max.z),
                    Vector3::new(max.x, min.y, max.z),
                    Vector3::new(min.x, max.y, max.z),
                    Vector3::new(max.x, max.y, max.z),
                ];
                let mut wmin = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
                let mut wmax = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
                for c in corners.iter() {
                    let w = pos + rot * c.component_mul(&scale);
                    wmin = wmin.inf(&w);
                    wmax = wmax.sup(&w);
                }
                Collider::Aabb(BoundingBox::new(
                    [wmin.x, wmin.y, wmin.z],
                    [wmax.x, wmax.y, wmax.z],
                ))
            }
            Collider::Sphere { center, radius } => Collider::Sphere {
                center: pos + rot * center.component_mul(&scale),
                radius: radius * uniform_scale.abs(),
            },
            Collider::Capsule { a, b, radius } => Collider::Capsule {
                a: pos + rot * a.component_mul(&scale),
                b: pos + rot * b.component_mul(&scale),
                radius: radius * uniform_scale.abs(),
            },
            Collider::Obb { center, half_extents, rotation } => Collider::Obb {
                center: pos + rot * center.component_mul(&scale),
                half_extents: half_extents.component_mul(&scale.abs()),
                rotation: rot * rotation,
            },
            Collider::Plane { .. } | Collider::Mesh(_) | Collider::Heightfield => self.clone(),
        }
    }
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
    }

    pub fn add_raw_delta(&mut self, dx: f64, dy: f64) {
        self.delta = (self.delta.0 + dx, self.delta.1 + dy);
    }

    pub fn reset_delta(&mut self) {
        self.delta = (0.0, 0.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Vector3, UnitQuaternion};

    #[test]
    fn collider_variants_construct() {
        let _a = Collider::Aabb(BoundingBox::new([0.0,0.0,0.0],[1.0,1.0,1.0]));
        let _s = Collider::Sphere { center: Vector3::new(0.0,0.0,0.0), radius: 1.0 };
        let _c = Collider::Capsule {
            a: Vector3::new(0.0,0.0,0.0),
            b: Vector3::new(0.0,1.0,0.0),
            radius: 0.5,
        };
        let _o = Collider::Obb {
            center: Vector3::new(0.0,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::identity(),
        };
        let _p = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let _m = Collider::Mesh(MeshCollider::from_triangles(vec![[
            Vector3::new(0.0,0.0,0.0),
            Vector3::new(1.0,0.0,0.0),
            Vector3::new(0.0,1.0,0.0),
        ]]));
        let _h = Collider::Heightfield;
    }

    #[test]
    fn collider_to_world_space_translates_sphere() {
        use crate::object::Transform;
        let local = Collider::Sphere { center: Vector3::new(0.0, 0.0, 0.0), radius: 1.0 };
        let mut t = Transform::new();
        t.set_position([5.0, 0.0, 0.0]);
        let world = local.to_world_space(&t);
        match world {
            Collider::Sphere { center, radius } => {
                assert!((center - Vector3::new(5.0, 0.0, 0.0)).norm() < 1e-5);
                assert!((radius - 1.0).abs() < 1e-5);
            }
            _ => panic!("expected Sphere"),
        }
    }

    #[test]
    fn collider_to_world_space_translates_capsule() {
        use crate::object::Transform;
        let local = Collider::Capsule {
            a: Vector3::new(0.0, 0.0, 0.0),
            b: Vector3::new(0.0, 2.0, 0.0),
            radius: 0.5,
        };
        let mut t = Transform::new();
        t.set_position([10.0, 0.0, 0.0]);
        let world = local.to_world_space(&t);
        if let Collider::Capsule { a, b, .. } = world {
            assert!((a - Vector3::new(10.0, 0.0, 0.0)).norm() < 1e-5);
            assert!((b - Vector3::new(10.0, 2.0, 0.0)).norm() < 1e-5);
        } else { panic!() }
    }

    #[test]
    fn collider_to_world_space_plane_unchanged() {
        use crate::object::Transform;
        let local = Collider::Plane { normal: Vector3::new(0.0, 1.0, 0.0), offset: 3.0 };
        let mut t = Transform::new();
        t.set_position([100.0, 100.0, 100.0]);
        let world = local.to_world_space(&t);
        if let Collider::Plane { normal, offset } = world {
            assert!((normal - Vector3::new(0.0, 1.0, 0.0)).norm() < 1e-5);
            assert!((offset - 3.0).abs() < 1e-5);
        } else { panic!() }
    }
}