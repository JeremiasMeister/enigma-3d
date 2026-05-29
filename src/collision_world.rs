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

/// Returns (closest_point_on_segment, parameter_t_in_0_1) for the given point.
fn closest_point_on_segment(
    p: &Vector3<f32>,
    a: &Vector3<f32>,
    b: &Vector3<f32>,
) -> (Vector3<f32>, f32) {
    let ab = b - a;
    let t = (p - a).dot(&ab) / ab.norm_squared().max(1e-12);
    let t = t.clamp(0.0, 1.0);
    (a + ab * t, t)
}

/// Returns the closest points and distance between two segments.
fn closest_points_segment_segment(
    a0: &Vector3<f32>, a1: &Vector3<f32>,
    b0: &Vector3<f32>, b1: &Vector3<f32>,
) -> (Vector3<f32>, Vector3<f32>) {
    // Real-Time Collision Detection §5.1.9 — clamped solve for two segment parameters.
    let d1 = a1 - a0;
    let d2 = b1 - b0;
    let r = a0 - b0;
    let a = d1.norm_squared();
    let e = d2.norm_squared();
    let f = d2.dot(&r);
    let (s, t);
    if a <= 1e-12 && e <= 1e-12 {
        return (*a0, *b0);
    }
    if a <= 1e-12 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(&r);
        if e <= 1e-12 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(&d2);
            let denom = a * e - b * b;
            let s0 = if denom.abs() > 1e-12 { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
            let t0 = (b * s0 + f) / e;
            let (s1, t1) = if t0 < 0.0 {
                (((-c) / a).clamp(0.0, 1.0), 0.0)
            } else if t0 > 1.0 {
                (((b - c) / a).clamp(0.0, 1.0), 1.0)
            } else {
                (s0, t0)
            };
            s = s1; t = t1;
        }
    }
    (a0 + d1 * s, b0 + d2 * t)
}

fn obb_axes(rot: &UnitQuaternion<f32>) -> [Vector3<f32>; 3] {
    let m = rot.to_rotation_matrix();
    [m * Vector3::x(), m * Vector3::y(), m * Vector3::z()]
}

fn obb_project(center: &Vector3<f32>, axes: &[Vector3<f32>; 3], half: &Vector3<f32>, axis: &Vector3<f32>) -> (f32, f32) {
    let c = center.dot(axis);
    let r = half.x * axes[0].dot(axis).abs()
          + half.y * axes[1].dot(axis).abs()
          + half.z * axes[2].dot(axis).abs();
    (c - r, c + r)
}

/// Returns Some((axis, penetration_depth)) if the OBBs overlap, where axis is the
/// minimum-penetration separating axis (pointing from box 1 toward box 2 by convention),
/// and penetration_depth is the overlap along that axis.
/// Returns None if the OBBs are separated.
fn sat_obb_obb(c1: &Vector3<f32>, a1: &[Vector3<f32>;3], h1: &Vector3<f32>,
               c2: &Vector3<f32>, a2: &[Vector3<f32>;3], h2: &Vector3<f32>)
               -> Option<(Vector3<f32>, f32)> {
    let mut axes: Vec<Vector3<f32>> = Vec::with_capacity(15);
    axes.extend_from_slice(a1);
    axes.extend_from_slice(a2);
    for i in 0..3 { for j in 0..3 {
        let cross = a1[i].cross(&a2[j]);
        if cross.norm_squared() > 1e-10 { axes.push(cross.normalize()); }
    }}
    let mut best_axis = Vector3::new(0.0, 1.0, 0.0);
    let mut best_penetration = f32::INFINITY;
    for axis in &axes {
        let (min1, max1) = obb_project(c1, a1, h1, axis);
        let (min2, max2) = obb_project(c2, a2, h2, axis);
        if max1 < min2 || max2 < min1 { return None; }
        // Overlap along this axis:
        let overlap = (max1 - min2).min(max2 - min1);
        if overlap < best_penetration {
            best_penetration = overlap;
            // Convention: axis points from box 1 toward box 2
            let center_dir = c2 - c1;
            best_axis = if center_dir.dot(axis) < 0.0 { -axis } else { *axis };
        }
    }
    Some((best_axis, best_penetration))
}

/// Returns a `Hit` (with `object_id` left as `Uuid::nil()` for the caller to fill in,
/// `toi = 0.0`, contact point and normal computed) when the two world-space shapes overlap.
/// `None` if they do not overlap.
///
/// For pairs marked as out-of-scope in the spec (mesh-vs-mesh family,
/// or static-vs-static), returns `None` with a warning log.
pub fn overlap_pair(a: &Collider, b: &Collider) -> Option<Hit> {
    use Collider::*;
    match (a, b) {
        (Sphere { center: c1, radius: r1 }, Sphere { center: c2, radius: r2 }) => {
            let d = c2 - c1;
            let dist = d.norm();
            let sum = r1 + r2;
            if dist <= sum {
                let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit {
                    object_id: Uuid::nil(),
                    point: c1 + normal * *r1,
                    normal,
                    toi: 0.0,
                })
            } else { None }
        }
        (Aabb(b1), Aabb(b2)) => {
            if is_colliding(b1, b2) {
                let min1 = b1.min_point();
                let max1 = b1.max_point();
                let min2 = b2.min_point();
                let max2 = b2.max_point();
                // Overlap region:
                let omin = Vector3::new(min1.x.max(min2.x), min1.y.max(min2.y), min1.z.max(min2.z));
                let omax = Vector3::new(max1.x.min(max2.x), max1.y.min(max2.y), max1.z.min(max2.z));
                let extent = omax - omin;
                // Pick the axis of *minimum* overlap → that's the axis of minimum penetration.
                let normal = if extent.x <= extent.y && extent.x <= extent.z {
                    let c1x = (min1.x + max1.x) * 0.5;
                    let c2x = (min2.x + max2.x) * 0.5;
                    Vector3::new(if c2x >= c1x { 1.0 } else { -1.0 }, 0.0, 0.0)
                } else if extent.y <= extent.z {
                    let c1y = (min1.y + max1.y) * 0.5;
                    let c2y = (min2.y + max2.y) * 0.5;
                    Vector3::new(0.0, if c2y >= c1y { 1.0 } else { -1.0 }, 0.0)
                } else {
                    let c1z = (min1.z + max1.z) * 0.5;
                    let c2z = (min2.z + max2.z) * 0.5;
                    Vector3::new(0.0, 0.0, if c2z >= c1z { 1.0 } else { -1.0 })
                };
                let point = (omin + omax) * 0.5;
                Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 })
            } else { None }
        }
        (Aabb(bb), Sphere { center, radius }) | (Sphere { center, radius }, Aabb(bb)) => {
            let min = Vector3::from(bb.min_point());
            let max = Vector3::from(bb.max_point());
            let closest = Vector3::new(
                center.x.clamp(min.x, max.x),
                center.y.clamp(min.y, max.y),
                center.z.clamp(min.z, max.z),
            );
            let d = center - closest;
            let dist = d.norm();
            if dist <= *radius {
                let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit { object_id: Uuid::nil(), point: closest, normal, toi: 0.0 })
            } else { None }
        }
        (Capsule { a, b, radius: cr }, Sphere { center, radius: sr })
        | (Sphere { center, radius: sr }, Capsule { a, b, radius: cr }) => {
            let (closest, _) = closest_point_on_segment(center, a, b);
            let d = center - closest;
            let dist = d.norm();
            let sum = cr + sr;
            if dist <= sum {
                let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit { object_id: Uuid::nil(), point: closest + normal * *cr, normal, toi: 0.0 })
            } else { None }
        }
        (Capsule { a: a1, b: b1, radius: r1 }, Capsule { a: a2, b: b2, radius: r2 }) => {
            let (p1, p2) = closest_points_segment_segment(a1, b1, a2, b2);
            let d = p2 - p1;
            let dist = d.norm();
            let sum = r1 + r2;
            if dist <= sum {
                let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit { object_id: Uuid::nil(), point: p1 + normal * *r1, normal, toi: 0.0 })
            } else { None }
        }
        (Capsule { a, b, radius: cr }, Aabb(bb))
        | (Aabb(bb), Capsule { a, b, radius: cr }) => {
            let min = Vector3::from(bb.min_point());
            let max = Vector3::from(bb.max_point());
            // Sample the segment at a few parameters; find min distance to box.
            // v1 approximation: 9 samples is adequate for character-scale capsules against world-scale boxes.
            // Tunnel risk on thin obstacles (<25cm) — bump sample count or switch to analytic for those cases.
            let mut best_dist = f32::INFINITY;
            let mut best_point = *a;
            let mut best_closest = *a;
            for i in 0..=8 {
                let t = i as f32 / 8.0;
                let p = a + (b - a) * t;
                let c = Vector3::new(p.x.clamp(min.x, max.x), p.y.clamp(min.y, max.y), p.z.clamp(min.z, max.z));
                let d = (p - c).norm();
                if d < best_dist { best_dist = d; best_point = p; best_closest = c; }
            }
            if best_dist <= *cr {
                let d = best_point - best_closest;
                let normal = if best_dist > 1e-6 { d / best_dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit { object_id: Uuid::nil(), point: best_closest, normal, toi: 0.0 })
            } else { None }
        }
        (Obb { center: c1, half_extents: h1, rotation: r1 },
         Obb { center: c2, half_extents: h2, rotation: r2 }) => {
            let a1 = obb_axes(r1);
            let a2 = obb_axes(r2);
            sat_obb_obb(c1, &a1, h1, c2, &a2, h2).map(|(normal, _depth)| {
                let mid = (c1 + c2) * 0.5;
                Hit { object_id: Uuid::nil(), point: mid, normal, toi: 0.0 }
            })
        }
        (Obb { center, half_extents, rotation }, Sphere { center: sc, radius: sr })
        | (Sphere { center: sc, radius: sr }, Obb { center, half_extents, rotation }) => {
            let inv_rot = rotation.inverse();
            let local = inv_rot * (sc - center);
            let clamped = Vector3::new(
                local.x.clamp(-half_extents.x, half_extents.x),
                local.y.clamp(-half_extents.y, half_extents.y),
                local.z.clamp(-half_extents.z, half_extents.z),
            );
            let closest_world = center + rotation * clamped;
            let d = sc - closest_world;
            let dist = d.norm();
            if dist <= *sr {
                let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit { object_id: Uuid::nil(), point: closest_world, normal, toi: 0.0 })
            } else { None }
        }
        (Obb { center: oc, half_extents: oh, rotation: orot }, Aabb(bb))
        | (Aabb(bb), Obb { center: oc, half_extents: oh, rotation: orot }) => {
            let bmin = Vector3::from(bb.min_point());
            let bmax = Vector3::from(bb.max_point());
            let bc = (bmin + bmax) * 0.5;
            let bh = (bmax - bmin) * 0.5;
            let aabb_axes = [Vector3::x(), Vector3::y(), Vector3::z()];
            let obb_a = obb_axes(orot);
            sat_obb_obb(oc, &obb_a, oh, &bc, &aabb_axes, &bh).map(|(normal, _depth)| {
                let mid = (oc + bc) * 0.5;
                Hit { object_id: Uuid::nil(), point: mid, normal, toi: 0.0 }
            })
        }
        (Obb { center, half_extents, rotation }, Capsule { a, b, radius })
        | (Capsule { a, b, radius }, Obb { center, half_extents, rotation }) => {
            let inv_rot = rotation.inverse();
            let mut best_dist = f32::INFINITY;
            let mut best_point = *a;
            let mut best_closest = *a;
            for i in 0..=8 {
                let t = i as f32 / 8.0;
                let p = a + (b - a) * t;
                let local = inv_rot * (p - center);
                let clamped = Vector3::new(
                    local.x.clamp(-half_extents.x, half_extents.x),
                    local.y.clamp(-half_extents.y, half_extents.y),
                    local.z.clamp(-half_extents.z, half_extents.z),
                );
                let closest_world = center + rotation * clamped;
                let d = (p - closest_world).norm();
                if d < best_dist { best_dist = d; best_point = p; best_closest = closest_world; }
            }
            if best_dist <= *radius {
                let d = best_point - best_closest;
                let normal = if best_dist > 1e-6 { d / best_dist } else { Vector3::new(0.0,1.0,0.0) };
                Some(Hit { object_id: Uuid::nil(), point: best_closest, normal, toi: 0.0 })
            } else { None }
        }
        (Plane { normal, offset }, Sphere { center, radius })
        | (Sphere { center, radius }, Plane { normal, offset }) => {
            let d = normal.dot(center) - *offset;
            if d.abs() <= *radius {
                Some(Hit {
                    object_id: Uuid::nil(),
                    point: center - normal * d,
                    normal: *normal,
                    toi: 0.0,
                })
            } else { None }
        }
        (Plane { normal, offset }, Capsule { a, b, radius })
        | (Capsule { a, b, radius }, Plane { normal, offset }) => {
            let da = normal.dot(a) - *offset;
            let db = normal.dot(b) - *offset;
            let dmin = da.min(db);
            let dmax = da.max(db);
            // Overlap iff the signed-distance interval intersects [-radius, +radius].
            if dmin <= *radius && dmax >= -*radius {
                // Pick the endpoint closest to the plane (smallest |d|) for the contact reference.
                let closer = if da.abs() < db.abs() { a } else { b };
                let d = normal.dot(closer) - *offset;
                Some(Hit {
                    object_id: Uuid::nil(),
                    point: closer - normal * d,
                    normal: *normal,
                    toi: 0.0,
                })
            } else { None }
        }
        (Plane { normal, offset }, Aabb(bb))
        | (Aabb(bb), Plane { normal, offset }) => {
            let min = Vector3::from(bb.min_point());
            let max = Vector3::from(bb.max_point());
            let center = (min + max) * 0.5;
            let half = (max - min) * 0.5;
            let r = half.x * normal.x.abs() + half.y * normal.y.abs() + half.z * normal.z.abs();
            let d = normal.dot(&center) - *offset;
            if d.abs() <= r {
                Some(Hit {
                    object_id: Uuid::nil(),
                    point: center - normal * d,
                    normal: *normal,
                    toi: 0.0,
                })
            } else { None }
        }
        (Plane { normal, offset }, Obb { center, half_extents, rotation })
        | (Obb { center, half_extents, rotation }, Plane { normal, offset }) => {
            let a = obb_axes(rotation);
            let r = half_extents.x * a[0].dot(normal).abs()
                  + half_extents.y * a[1].dot(normal).abs()
                  + half_extents.z * a[2].dot(normal).abs();
            let d = normal.dot(center) - *offset;
            if d.abs() <= r {
                Some(Hit {
                    object_id: Uuid::nil(),
                    point: center - normal * d,
                    normal: *normal,
                    toi: 0.0,
                })
            } else { None }
        }
        (Plane { .. }, Plane { .. })
        | (Plane { .. }, Mesh(_)) | (Mesh(_), Plane { .. })
        | (Plane { .. }, Heightfield) | (Heightfield, Plane { .. })
        | (Mesh(_), Mesh(_))
        | (Mesh(_), Heightfield) | (Heightfield, Mesh(_))
        | (Heightfield, Heightfield) => {
            crate::logging::EnigmaWarning::new(
                Some("collision_world::overlap_pair called with static-vs-static pair (unsupported in v1)"),
                true,
            ).log();
            None
        }
        _ => None,  // Mesh / Heightfield arms added in later tasks
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

    #[test]
    fn sphere_sphere_overlap() {
        let s1 = Collider::Sphere { center: Vector3::new(0.0,0.0,0.0), radius: 1.0 };
        let s2_hit = Collider::Sphere { center: Vector3::new(1.5,0.0,0.0), radius: 1.0 };
        let s2_miss = Collider::Sphere { center: Vector3::new(3.0,0.0,0.0), radius: 1.0 };
        assert!(overlap_pair(&s1, &s2_hit).is_some());
        assert!(overlap_pair(&s1, &s2_miss).is_none());
    }

    #[test]
    fn aabb_aabb_overlap_via_pair() {
        let a = Collider::Aabb(BoundingBox::new([0.0,0.0,0.0], [1.0,1.0,1.0]));
        let b_hit = Collider::Aabb(BoundingBox::new([0.5,0.5,0.5], [2.0,2.0,2.0]));
        let b_miss = Collider::Aabb(BoundingBox::new([2.0,2.0,2.0], [3.0,3.0,3.0]));
        assert!(overlap_pair(&a, &b_hit).is_some());
        assert!(overlap_pair(&a, &b_miss).is_none());
    }

    #[test]
    fn sphere_aabb_overlap() {
        let aabb = Collider::Aabb(BoundingBox::new([-1.0,-1.0,-1.0], [1.0,1.0,1.0]));
        let s_hit = Collider::Sphere { center: Vector3::new(1.4,0.0,0.0), radius: 0.5 };
        let s_miss = Collider::Sphere { center: Vector3::new(3.0,0.0,0.0), radius: 0.5 };
        assert!(overlap_pair(&aabb, &s_hit).is_some());
        assert!(overlap_pair(&s_hit, &aabb).is_some(), "symmetry");
        assert!(overlap_pair(&aabb, &s_miss).is_none());
    }

    #[test]
    fn capsule_sphere_overlap() {
        let cap = Collider::Capsule {
            a: Vector3::new(0.0,0.0,0.0),
            b: Vector3::new(0.0,2.0,0.0),
            radius: 0.5,
        };
        let s_hit = Collider::Sphere { center: Vector3::new(0.8,1.0,0.0), radius: 0.5 };
        let s_miss = Collider::Sphere { center: Vector3::new(3.0,1.0,0.0), radius: 0.5 };
        assert!(overlap_pair(&cap, &s_hit).is_some());
        assert!(overlap_pair(&s_hit, &cap).is_some(), "symmetry");
        assert!(overlap_pair(&cap, &s_miss).is_none());
    }

    #[test]
    fn capsule_capsule_overlap() {
        let c1 = Collider::Capsule { a: Vector3::new(0.0,0.0,0.0), b: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
        let c2_hit = Collider::Capsule { a: Vector3::new(0.8,0.5,0.0), b: Vector3::new(0.8,2.5,0.0), radius: 0.5 };
        let c2_miss = Collider::Capsule { a: Vector3::new(5.0,0.0,0.0), b: Vector3::new(5.0,2.0,0.0), radius: 0.5 };
        assert!(overlap_pair(&c1, &c2_hit).is_some());
        assert!(overlap_pair(&c1, &c2_miss).is_none());
    }

    #[test]
    fn capsule_aabb_overlap() {
        let cap = Collider::Capsule { a: Vector3::new(0.0,0.0,0.0), b: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
        let bb_hit = Collider::Aabb(BoundingBox::new([0.3, 0.5, -0.5], [2.0, 1.5, 0.5]));
        let bb_miss = Collider::Aabb(BoundingBox::new([5.0, 0.0, 0.0], [6.0, 1.0, 1.0]));
        assert!(overlap_pair(&cap, &bb_hit).is_some());
        assert!(overlap_pair(&cap, &bb_miss).is_none());
    }

    #[test]
    fn obb_obb_overlap() {
        let o1 = Collider::Obb {
            center: Vector3::new(0.0,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::identity(),
        };
        let o2_hit = Collider::Obb {
            center: Vector3::new(1.5,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.5),
        };
        let o2_miss = Collider::Obb {
            center: Vector3::new(5.0,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::identity(),
        };
        assert!(overlap_pair(&o1, &o2_hit).is_some());
        assert!(overlap_pair(&o1, &o2_miss).is_none());
    }

    #[test]
    fn obb_sphere_overlap() {
        let o = Collider::Obb {
            center: Vector3::new(0.0,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f32::consts::FRAC_PI_4),
        };
        let s_hit = Collider::Sphere { center: Vector3::new(1.4,0.0,0.0), radius: 0.5 };
        let s_miss = Collider::Sphere { center: Vector3::new(5.0,0.0,0.0), radius: 0.5 };
        assert!(overlap_pair(&o, &s_hit).is_some());
        assert!(overlap_pair(&o, &s_miss).is_none());
    }

    #[test]
    fn obb_aabb_overlap() {
        let o = Collider::Obb {
            center: Vector3::new(0.0,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.3),
        };
        let bb_hit = Collider::Aabb(BoundingBox::new([0.5,-0.5,-0.5], [2.0,0.5,0.5]));
        let bb_miss = Collider::Aabb(BoundingBox::new([5.0,5.0,5.0], [6.0,6.0,6.0]));
        assert!(overlap_pair(&o, &bb_hit).is_some());
        assert!(overlap_pair(&o, &bb_miss).is_none());
    }

    #[test]
    fn aabb_aabb_overlap_normal_is_min_penetration_axis() {
        // Two boxes overlapping by 0.1 along X (small) and 0.5 along Y/Z (large).
        // The min-penetration axis is X → normal should be along ±X.
        let a = Collider::Aabb(BoundingBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));
        let b = Collider::Aabb(BoundingBox::new([0.9, 0.5, 0.5], [1.9, 1.5, 1.5]));
        let hit = overlap_pair(&a, &b).expect("should overlap");
        assert!(hit.normal.x.abs() > 0.99, "expected X-aligned normal, got {:?}", hit.normal);
        assert!(hit.normal.y.abs() < 0.01);
        assert!(hit.normal.z.abs() < 0.01);
        // b is to the +X of a, so normal pointing from a→b should be +X.
        assert!(hit.normal.x > 0.0);
    }

    #[test]
    fn obb_obb_overlap_normal_is_min_penetration_axis() {
        // Two identity-rotated OBBs (= AABBs) overlapping slightly along X.
        let o1 = Collider::Obb {
            center: Vector3::new(0.0, 0.0, 0.0),
            half_extents: Vector3::new(1.0, 1.0, 1.0),
            rotation: UnitQuaternion::identity(),
        };
        let o2 = Collider::Obb {
            center: Vector3::new(1.9, 0.0, 0.0),       // overlap 0.1 along X
            half_extents: Vector3::new(1.0, 1.0, 1.0),
            rotation: UnitQuaternion::identity(),
        };
        let hit = overlap_pair(&o1, &o2).expect("should overlap");
        assert!(hit.normal.x.abs() > 0.99, "expected X-aligned normal, got {:?}", hit.normal);
        assert!(hit.normal.x > 0.0, "o2 is +X of o1, normal should point +X");
    }

    #[test]
    fn obb_capsule_overlap() {
        let o = Collider::Obb {
            center: Vector3::new(0.0,0.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::identity(),
        };
        let cap_hit = Collider::Capsule { a: Vector3::new(0.5,2.0,0.0), b: Vector3::new(0.5,0.0,0.0), radius: 0.4 };
        let cap_miss = Collider::Capsule { a: Vector3::new(5.0,0.0,0.0), b: Vector3::new(5.0,2.0,0.0), radius: 0.4 };
        assert!(overlap_pair(&o, &cap_hit).is_some());
        assert!(overlap_pair(&o, &cap_miss).is_none());
    }

    #[test]
    fn sphere_plane_overlap() {
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let s_hit = Collider::Sphere { center: Vector3::new(0.0,0.3,0.0), radius: 0.5 };
        let s_miss = Collider::Sphere { center: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
        assert!(overlap_pair(&plane, &s_hit).is_some());
        assert!(overlap_pair(&plane, &s_miss).is_none());
    }

    #[test]
    fn capsule_plane_overlap() {
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let cap_hit = Collider::Capsule { a: Vector3::new(0.0,0.3,0.0), b: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
        let cap_miss = Collider::Capsule { a: Vector3::new(0.0,2.0,0.0), b: Vector3::new(0.0,3.0,0.0), radius: 0.4 };
        assert!(overlap_pair(&plane, &cap_hit).is_some());
        assert!(overlap_pair(&plane, &cap_miss).is_none());
    }

    #[test]
    fn aabb_plane_overlap() {
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let bb_hit = Collider::Aabb(BoundingBox::new([-1.0,-0.5,-1.0], [1.0,0.5,1.0]));
        let bb_miss = Collider::Aabb(BoundingBox::new([-1.0,1.0,-1.0], [1.0,2.0,1.0]));
        assert!(overlap_pair(&plane, &bb_hit).is_some());
        assert!(overlap_pair(&plane, &bb_miss).is_none());
    }

    #[test]
    fn obb_plane_overlap() {
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let o_hit = Collider::Obb {
            center: Vector3::new(0.0,0.5,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::identity(),
        };
        let o_miss = Collider::Obb {
            center: Vector3::new(0.0,3.0,0.0),
            half_extents: Vector3::new(1.0,1.0,1.0),
            rotation: UnitQuaternion::identity(),
        };
        assert!(overlap_pair(&plane, &o_hit).is_some());
        assert!(overlap_pair(&plane, &o_miss).is_none());
    }

    #[test]
    fn capsule_plane_misses_when_fully_below() {
        // Both endpoints clearly below the ground plane — should be a miss.
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let cap = Collider::Capsule {
            a: Vector3::new(0.0, -3.0, 0.0),
            b: Vector3::new(0.0, -2.0, 0.0),
            radius: 0.5,
        };
        assert!(overlap_pair(&plane, &cap).is_none(),
            "capsule fully below plane should not overlap");
        assert!(overlap_pair(&cap, &plane).is_none(), "symmetric");
    }

    #[test]
    fn capsule_plane_hits_when_crossing() {
        // Capsule straddles the plane — clear hit.
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let cap = Collider::Capsule {
            a: Vector3::new(0.0, -1.0, 0.0),
            b: Vector3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };
        assert!(overlap_pair(&plane, &cap).is_some());
    }

    #[test]
    fn aabb_plane_misses_when_fully_below() {
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let bb = Collider::Aabb(BoundingBox::new([-1.0,-3.0,-1.0], [1.0,-2.0,1.0]));
        assert!(overlap_pair(&plane, &bb).is_none(), "AABB below plane should not overlap");
    }

    #[test]
    fn obb_plane_misses_when_fully_below() {
        let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
        let o = Collider::Obb {
            center: Vector3::new(0.0, -3.0, 0.0),
            half_extents: Vector3::new(1.0, 0.5, 1.0),
            rotation: UnitQuaternion::identity(),
        };
        assert!(overlap_pair(&plane, &o).is_none(), "OBB below plane should not overlap");
    }
}