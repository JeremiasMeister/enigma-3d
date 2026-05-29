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

    /// Builds a mesh collider from all triangles in the object's visual shapes.
    /// Triangles are stored in object-local space — the object's transform is
    /// applied at query time via `Collider::to_world_space`.
    pub fn from_object_mesh(object: &crate::object::Object) -> Self {
        let mut tris = Vec::new();
        for shape in object.shapes_ref() {
            let verts = &shape.vertices;
            for tri_indices in shape.indices.chunks_exact(3) {
                let v0 = verts[tri_indices[0] as usize].position;
                let v1 = verts[tri_indices[1] as usize].position;
                let v2 = verts[tri_indices[2] as usize].position;
                tris.push([
                    Vector3::new(v0[0], v0[1], v0[2]),
                    Vector3::new(v1[0], v1[1], v1[2]),
                    Vector3::new(v2[0], v2[1], v2[2]),
                ]);
            }
        }
        Self { triangles: tris }
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
            Collider::Mesh(m) => {
                // Mesh triangles are stored in object-local space (see
                // `MeshCollider::from_object_mesh`). Apply the owner's transform so query
                // arms can treat the mesh as world-space, consistent with all other
                // colliders.
                Collider::Mesh(MeshCollider {
                    triangles: m.triangles.iter().map(|tri| [
                        pos + rot * tri[0].component_mul(&scale),
                        pos + rot * tri[1].component_mul(&scale),
                        pos + rot * tri[2].component_mul(&scale),
                    ]).collect(),
                })
            }
            Collider::Plane { .. } | Collider::Heightfield => self.clone(),
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
        (Mesh(m), Sphere { center, radius }) | (Sphere { center, radius }, Mesh(m)) => {
            for tri in &m.triangles {
                if let Some((point, normal)) =
                    sphere_overlap_triangle(center, *radius, &tri[0], &tri[1], &tri[2])
                {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
        }
        (Mesh(m), Capsule { a, b, radius }) | (Capsule { a, b, radius }, Mesh(m)) => {
            // 9-sample sphere-vs-triangle along the capsule's segment — matches the
            // sample-count used by capsule-vs-AABB/OBB for consistency.
            for i in 0..=8 {
                let t = i as f32 / 8.0;
                let p = a + (b - a) * t;
                for tri in &m.triangles {
                    if let Some((point, normal)) =
                        sphere_overlap_triangle(&p, *radius, &tri[0], &tri[1], &tri[2])
                    {
                        return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                    }
                }
            }
            None
        }
        (Mesh(m), Aabb(bb)) | (Aabb(bb), Mesh(m)) => {
            // v1 approximation: treat the AABB as its bounding sphere. Cheaper than
            // analytic AABB-tri SAT and good enough for the typical use case
            // (medium-scale boxes vs. terrain-like meshes).
            let min = Vector3::from(bb.min_point());
            let max = Vector3::from(bb.max_point());
            let center = (min + max) * 0.5;
            let radius = (max - min).norm() * 0.5;
            for tri in &m.triangles {
                if let Some((point, normal)) =
                    sphere_overlap_triangle(&center, radius, &tri[0], &tri[1], &tri[2])
                {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
        }
        (Mesh(m), Obb { center, half_extents, .. })
        | (Obb { center, half_extents, .. }, Mesh(m)) => {
            // Same bounding-sphere approximation as the AABB arm.
            let radius = half_extents.norm();
            for tri in &m.triangles {
                if let Some((point, normal)) =
                    sphere_overlap_triangle(center, radius, &tri[0], &tri[1], &tri[2])
                {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
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
        // Heightfield-vs-dynamic shapes need access to the actual `Terrain` (heightmap, config,
        // position). `overlap_pair` only takes `Collider`s, so it can't resolve `Heightfield`.
        // Terrain handling lives in `ShapeOverlap::query` via `overlap_shape_terrain`, which has
        // an `AppState` and can call `Terrain::triangles_in_aabb`.
        (Heightfield, Sphere { .. }) | (Sphere { .. }, Heightfield)
        | (Heightfield, Capsule { .. }) | (Capsule { .. }, Heightfield)
        | (Heightfield, Aabb(_)) | (Aabb(_), Heightfield)
        | (Heightfield, Obb { .. }) | (Obb { .. }, Heightfield) => None,
    }
}

/// Möller-Trumbore ray-triangle intersection. Returns `Some(t)` where the hit point is
/// `origin + dir * t`, or `None` if the ray misses or hits behind the origin.
#[allow(dead_code)]
fn ray_triangle(
    origin: &Vector3<f32>,
    dir: &Vector3<f32>,
    v0: &Vector3<f32>,
    v1: &Vector3<f32>,
    v2: &Vector3<f32>,
) -> Option<f32> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = dir.cross(&e2);
    let det = e1.dot(&p);
    if det.abs() < 1e-8 { return None; }
    let inv = 1.0 / det;
    let s = origin - v0;
    let u = s.dot(&p) * inv;
    if u < 0.0 || u > 1.0 { return None; }
    let q = s.cross(&e1);
    let v = dir.dot(&q) * inv;
    if v < 0.0 || u + v > 1.0 { return None; }
    let t = e2.dot(&q) * inv;
    if t > 1e-6 { Some(t) } else { None }
}

/// Sphere-vs-triangle overlap via closest-point-on-triangle test (Real-Time Collision
/// Detection §5.1.5). Returns the contact point and outward normal when the sphere
/// overlaps the triangle.
fn sphere_overlap_triangle(
    center: &Vector3<f32>,
    radius: f32,
    v0: &Vector3<f32>,
    v1: &Vector3<f32>,
    v2: &Vector3<f32>,
) -> Option<(Vector3<f32>, Vector3<f32>)> {
    let ab = v1 - v0;
    let ac = v2 - v0;
    let ap = center - v0;
    let d1 = ab.dot(&ap);
    let d2 = ac.dot(&ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return finalize_sphere_tri(center, *v0, radius);
    }
    let bp = center - v1;
    let d3 = ab.dot(&bp);
    let d4 = ac.dot(&bp);
    if d3 >= 0.0 && d4 <= d3 { return finalize_sphere_tri(center, *v1, radius); }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return finalize_sphere_tri(center, v0 + ab * v, radius);
    }
    let cp = center - v2;
    let d5 = ab.dot(&cp);
    let d6 = ac.dot(&cp);
    if d6 >= 0.0 && d5 <= d6 { return finalize_sphere_tri(center, *v2, radius); }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return finalize_sphere_tri(center, v0 + ac * w, radius);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return finalize_sphere_tri(center, v1 + (v2 - v1) * w, radius);
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    finalize_sphere_tri(center, v0 + ab * v + ac * w, radius)
}

fn finalize_sphere_tri(
    center: &Vector3<f32>,
    closest: Vector3<f32>,
    radius: f32,
) -> Option<(Vector3<f32>, Vector3<f32>)> {
    let d = center - closest;
    let dist = d.norm();
    if dist <= radius {
        let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0, 1.0, 0.0) };
        Some((closest, normal))
    } else {
        None
    }
}

/// World-space AABB of a collider. Returns `None` for unbounded shapes (Plane, Heightfield).
fn shape_aabb(shape: &Collider) -> Option<BoundingBox> {
    use Collider::*;
    let pad = |c: &Vector3<f32>, r: f32| BoundingBox::new(
        [c.x - r, c.y - r, c.z - r],
        [c.x + r, c.y + r, c.z + r],
    );
    match shape {
        Aabb(bb) => Some(*bb),
        Sphere { center, radius } => Some(pad(center, *radius)),
        Capsule { a, b, radius } => {
            let min = a.inf(b) - Vector3::new(*radius, *radius, *radius);
            let max = a.sup(b) + Vector3::new(*radius, *radius, *radius);
            Some(BoundingBox::new([min.x, min.y, min.z], [max.x, max.y, max.z]))
        }
        Obb { center, half_extents, rotation } => {
            // Conservative: project the OBB's rotated axes onto world axes.
            let a = obb_axes(rotation);
            let r = Vector3::new(
                half_extents.x * a[0].x.abs() + half_extents.y * a[1].x.abs() + half_extents.z * a[2].x.abs(),
                half_extents.x * a[0].y.abs() + half_extents.y * a[1].y.abs() + half_extents.z * a[2].y.abs(),
                half_extents.x * a[0].z.abs() + half_extents.y * a[1].z.abs() + half_extents.z * a[2].z.abs(),
            );
            Some(BoundingBox::new(
                [center.x - r.x, center.y - r.y, center.z - r.z],
                [center.x + r.x, center.y + r.y, center.z + r.z],
            ))
        }
        Mesh(m) => {
            let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
            let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
            for tri in &m.triangles {
                for v in tri.iter() {
                    min = min.inf(v);
                    max = max.sup(v);
                }
            }
            if min.x.is_finite() {
                Some(BoundingBox::new([min.x, min.y, min.z], [max.x, max.y, max.z]))
            } else {
                None
            }
        }
        Plane { .. } | Heightfield => None,
    }
}

/// Overlap a shape against a terrain. The shape's world-space AABB is used to fetch
/// candidate triangles from the terrain, then each triangle is tested against the shape.
/// Returns the first overlap found.
fn overlap_shape_terrain(shape: &Collider, terrain: &crate::terrain::Terrain) -> Option<Hit> {
    use Collider::*;
    let aabb = shape_aabb(shape)?;
    let tris = terrain.triangles_in_aabb(&aabb);
    match shape {
        Sphere { center, radius } => {
            for tri in &tris {
                if let Some((point, normal)) =
                    sphere_overlap_triangle(center, *radius, &tri[0], &tri[1], &tri[2])
                {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
        }
        Capsule { a, b, radius } => {
            // Sample the capsule segment at 9 points, sphere-vs-triangle each.
            for i in 0..=8 {
                let t = i as f32 / 8.0;
                let p = a + (b - a) * t;
                for tri in &tris {
                    if let Some((point, normal)) =
                        sphere_overlap_triangle(&p, *radius, &tri[0], &tri[1], &tri[2])
                    {
                        return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                    }
                }
            }
            None
        }
        Aabb(_) | Obb { .. } => {
            // v1: treat AABB/OBB as a bounding sphere for terrain queries. Less accurate
            // than full box-vs-triangle, but consistent with the "simple code" v1 scope.
            let min = aabb.min_point();
            let max = aabb.max_point();
            let center = (min + max) * 0.5;
            let radius = (max - min).norm() * 0.5;
            for tri in &tris {
                if let Some((point, normal)) =
                    sphere_overlap_triangle(&center, radius, &tri[0], &tri[1], &tri[2])
                {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
        }
        // Plane / Mesh / Heightfield: static-vs-static, not meaningful as a query shape against terrain.
        _ => None,
    }
}

/// Find all objects whose collider (in world space) overlaps the given shape (in world space).
/// Iterates `app_state.objects`. No motion — pure overlap test.
pub struct ShapeOverlap {
    shape: Collider,
}

impl ShapeOverlap {
    pub fn new(shape: Collider) -> Self {
        Self { shape }
    }

    pub fn query(&self, app_state: &mut AppState) -> Vec<Hit> {
        let mut hits = Vec::new();
        for obj in app_state.objects.iter_mut() {
            if !*obj.get_collision() { continue; }
            let world_collider = match obj.get_collider() {
                Some(c) => c.to_world_space(&obj.transform),
                None => Collider::Aabb(obj.get_bounding_box()),
            };
            if let Some(mut hit) = overlap_pair(&self.shape, &world_collider) {
                hit.object_id = obj.get_unique_id();
                hits.push(hit);
            }
        }
        // Terrain — handled separately because `overlap_pair` can't resolve `Heightfield` to
        // a concrete `Terrain` (it only sees `Collider`s).
        if let Some(terrain) = app_state.get_terrain() {
            if let Some(hit) = overlap_shape_terrain(&self.shape, terrain) {
                hits.push(hit);
            }
        }
        hits
    }
}

/// Shifts the positions of a world-space shape by `delta`. Used by `ShapeCast` to step the
/// query shape through space without rebuilding from a transform.
///
/// Plane / Mesh / Heightfield can't be the swept shape (they're static-by-convention) —
/// passing one of those returns a clone, treat that as caller error.
fn translate_world(shape: &Collider, delta: Vector3<f32>) -> Collider {
    use Collider::*;
    match shape {
        Aabb(bb) => {
            let min = Vector3::from(bb.min_point()) + delta;
            let max = Vector3::from(bb.max_point()) + delta;
            Aabb(BoundingBox::new([min.x, min.y, min.z], [max.x, max.y, max.z]))
        }
        Sphere { center, radius } => Sphere { center: center + delta, radius: *radius },
        Capsule { a, b, radius } => Capsule { a: a + delta, b: b + delta, radius: *radius },
        Obb { center, half_extents, rotation } => Obb {
            center: center + delta,
            half_extents: *half_extents,
            rotation: *rotation,
        },
        // Static-by-convention shapes — can't be swept. Return a clone; caller error if reached.
        other => other.clone(),
    }
}

/// Sweep a shape from its starting world-space position along `direction * distance`,
/// returning the first hit found. Implemented as an iterative step-and-overlap on top
/// of `ShapeOverlap`. TOI is rounded to the nearest step (1/`steps` of total distance).
pub struct ShapeCast {
    shape: Collider,
    direction: Vector3<f32>,
    distance: f32,
    steps: usize,
}

impl ShapeCast {
    /// Construct a new sweep. `direction` is normalized internally; if it's near-zero,
    /// `cast` will immediately return `None`.
    pub fn new(shape: Collider, direction: Vector3<f32>, distance: f32) -> Self {
        let dir = if direction.norm() > 1e-6 {
            direction.normalize()
        } else {
            Vector3::zeros()
        };
        Self { shape, direction: dir, distance, steps: 16 }
    }

    /// Override the default step count (16). Clamped to at least 1.
    pub fn with_steps(mut self, n: usize) -> Self {
        self.steps = n.max(1);
        self
    }

    /// Run the sweep. Returns the first hit found, with `toi` set to the fraction of
    /// `distance` at which the hit occurred (rounded to the step grid). Returns `None`
    /// if direction is zero, distance is non-positive, or no overlap is found through
    /// the full sweep.
    pub fn cast(&self, app_state: &mut AppState) -> Option<Hit> {
        if self.direction.norm() < 1e-6 || self.distance <= 0.0 {
            return None;
        }
        let step_size = self.distance / self.steps as f32;
        for i in 1..=self.steps {
            let t = i as f32 / self.steps as f32;
            let swept = translate_world(&self.shape, self.direction * (i as f32 * step_size));
            let overlap = ShapeOverlap::new(swept);
            let hits = overlap.query(app_state);
            if let Some(mut h) = hits.into_iter().next() {
                h.toi = t;
                return Some(h);
            }
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

    #[test]
    fn shape_overlap_finds_objects_in_radius() {
        use crate::AppState;
        use crate::object::Object;

        let mut state = AppState::new();
        let mut o = Object::new(Some("a".into()));
        o.transform.set_position([0.0, 0.0, 0.0]);
        o.set_collider(Collider::Sphere { center: Vector3::zeros(), radius: 1.0 });
        let oid = o.get_unique_id();
        state.add_object(o);

        let overlap = ShapeOverlap::new(Collider::Sphere {
            center: Vector3::new(0.5, 0.0, 0.0),
            radius: 1.0,
        });
        let hits = overlap.query(&mut state);
        assert!(hits.iter().any(|h| h.object_id == oid));
    }

    #[test]
    fn shape_overlap_misses_when_far() {
        use crate::AppState;
        use crate::object::Object;

        let mut state = AppState::new();
        let mut o = Object::new(Some("a".into()));
        o.transform.set_position([0.0, 0.0, 0.0]);
        o.set_collider(Collider::Sphere { center: Vector3::zeros(), radius: 1.0 });
        state.add_object(o);

        let overlap = ShapeOverlap::new(Collider::Sphere {
            center: Vector3::new(100.0, 0.0, 0.0),
            radius: 1.0,
        });
        let hits = overlap.query(&mut state);
        assert!(hits.is_empty());
    }

    #[test]
    fn ray_triangle_hits_when_pointing_at_triangle() {
        // Triangle in the y=0 plane, ray pointing straight down from (0.25, 1, 0.25).
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 0.0, 1.0);
        let origin = Vector3::new(0.25, 1.0, 0.25);
        let dir = Vector3::new(0.0, -1.0, 0.0);
        let t = ray_triangle(&origin, &dir, &v0, &v1, &v2).expect("should hit");
        assert!((t - 1.0).abs() < 1e-4, "expected t=1.0, got {t}");
    }

    #[test]
    fn ray_triangle_misses_when_pointing_away() {
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 0.0, 1.0);
        // Ray points up — away from the triangle.
        let origin = Vector3::new(0.25, 1.0, 0.25);
        let dir = Vector3::new(0.0, 1.0, 0.0);
        assert!(ray_triangle(&origin, &dir, &v0, &v1, &v2).is_none());
    }

    #[test]
    fn ray_triangle_misses_outside_triangle() {
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 0.0, 1.0);
        // Origin is above (5, _, 5) — well outside the triangle's XZ extent.
        let origin = Vector3::new(5.0, 1.0, 5.0);
        let dir = Vector3::new(0.0, -1.0, 0.0);
        assert!(ray_triangle(&origin, &dir, &v0, &v1, &v2).is_none());
    }

    #[test]
    fn sphere_overlap_triangle_hits_when_close() {
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(2.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 0.0, 2.0);
        // Sphere center just above the triangle's interior, radius reaches down to it.
        let center = Vector3::new(0.5, 0.3, 0.5);
        let radius = 0.5;
        let result = sphere_overlap_triangle(&center, radius, &v0, &v1, &v2);
        assert!(result.is_some(), "expected overlap");
        let (point, normal) = result.unwrap();
        // The closest point is directly below the sphere center on the triangle plane.
        assert!((point.y).abs() < 1e-4);
        // Normal should roughly point up.
        assert!(normal.y > 0.5);
    }

    #[test]
    fn sphere_overlap_triangle_misses_when_far() {
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 0.0, 1.0);
        let center = Vector3::new(0.25, 10.0, 0.25);
        let radius = 0.5;
        assert!(sphere_overlap_triangle(&center, radius, &v0, &v1, &v2).is_none());
    }

    #[test]
    fn sphere_overlap_triangle_hits_vertex_region() {
        // Closest point on the triangle is vertex v0; sphere overlaps it.
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(10.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 0.0, 10.0);
        let center = Vector3::new(-0.3, 0.0, -0.3);
        let radius = 0.6;
        let result = sphere_overlap_triangle(&center, radius, &v0, &v1, &v2);
        assert!(result.is_some());
        let (point, _) = result.unwrap();
        assert!((point - v0).norm() < 1e-4, "closest point should be v0, got {point:?}");
    }

    #[test]
    fn shape_aabb_sphere_pads_radius() {
        let s = Collider::Sphere { center: Vector3::new(1.0, 2.0, 3.0), radius: 0.5 };
        let bb = shape_aabb(&s).expect("sphere has aabb");
        let min = bb.min_point();
        let max = bb.max_point();
        assert!((min - Vector3::new(0.5, 1.5, 2.5)).norm() < 1e-5);
        assert!((max - Vector3::new(1.5, 2.5, 3.5)).norm() < 1e-5);
    }

    #[test]
    fn shape_aabb_capsule_bounds_segment_plus_radius() {
        let c = Collider::Capsule {
            a: Vector3::new(0.0, 0.0, 0.0),
            b: Vector3::new(0.0, 2.0, 0.0),
            radius: 0.5,
        };
        let bb = shape_aabb(&c).expect("capsule has aabb");
        let min = bb.min_point();
        let max = bb.max_point();
        assert!((min - Vector3::new(-0.5, -0.5, -0.5)).norm() < 1e-5);
        assert!((max - Vector3::new(0.5, 2.5, 0.5)).norm() < 1e-5);
    }

    #[test]
    fn shape_aabb_plane_returns_none() {
        let p = Collider::Plane { normal: Vector3::new(0.0, 1.0, 0.0), offset: 0.0 };
        assert!(shape_aabb(&p).is_none());
    }

    #[test]
    fn shape_aabb_heightfield_returns_none() {
        let h = Collider::Heightfield;
        assert!(shape_aabb(&h).is_none());
    }

    #[test]
    fn shape_cast_finds_first_hit_along_sweep() {
        use crate::AppState;
        use crate::object::Object;
        let mut state = AppState::new();
        let mut o = Object::new(Some("wall".into()));
        o.transform.set_position([5.0, 0.0, 0.0]);
        o.set_collider(Collider::Aabb(BoundingBox::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])));
        state.objects.push(o);

        let cast = ShapeCast::new(
            Collider::Sphere { center: Vector3::new(0.0, 0.0, 0.0), radius: 0.5 },
            Vector3::new(1.0, 0.0, 0.0),
            10.0,
        );
        let hit = cast.cast(&mut state);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!(h.toi > 0.0 && h.toi < 1.0);
        // Sphere of radius 0.5 starting at origin, moving +X, hits wall (left face at x=4.0) at travel=3.5
        // expected TOI ≈ 3.5 / 10.0 = 0.35
        assert!((h.toi - 0.35).abs() < 0.1, "toi was {}", h.toi);
    }

    /// Helper: build a `Vertex` filled with sensible defaults. `Vertex` doesn't impl
    /// `Default`, so tests need to spell every field. Centralised here to avoid noise.
    fn test_vertex(x: f32, y: f32, z: f32) -> crate::geometry::Vertex {
        crate::geometry::Vertex {
            position: [x, y, z],
            texcoord: [0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            bone_indices: [0, 0, 0, 0],
            bone_weights: [0.0, 0.0, 0.0, 0.0],
        }
    }

    #[test]
    fn mesh_collider_from_object_mesh_collects_triangles() {
        use crate::object::{Object, Shape};
        let mut obj = Object::new(Some("box".into()));
        let mut shape = Shape::new();
        shape.vertices = vec![
            test_vertex(0.0, 0.0, 0.0),
            test_vertex(1.0, 0.0, 0.0),
            test_vertex(0.0, 1.0, 0.0),
        ];
        shape.indices = vec![0, 1, 2];
        obj.add_shape(shape);

        let mesh = MeshCollider::from_object_mesh(&obj);
        assert_eq!(mesh.triangles.len(), 1);
        assert!((mesh.triangles[0][0] - Vector3::new(0.0, 0.0, 0.0)).norm() < 1e-5);
        assert!((mesh.triangles[0][1] - Vector3::new(1.0, 0.0, 0.0)).norm() < 1e-5);
        assert!((mesh.triangles[0][2] - Vector3::new(0.0, 1.0, 0.0)).norm() < 1e-5);
    }

    #[test]
    fn sphere_mesh_overlap() {
        let tri = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 2.0),
        ];
        let mesh = Collider::Mesh(MeshCollider::from_triangles(vec![tri]));
        let s_hit = Collider::Sphere { center: Vector3::new(0.5, 0.3, 0.5), radius: 0.5 };
        let s_miss = Collider::Sphere { center: Vector3::new(5.0, 5.0, 5.0), radius: 0.5 };
        assert!(overlap_pair(&mesh, &s_hit).is_some());
        assert!(overlap_pair(&mesh, &s_miss).is_none());
        // Symmetric ordering must also hit.
        assert!(overlap_pair(&s_hit, &mesh).is_some());
    }

    #[test]
    fn capsule_mesh_overlap() {
        let tri = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 2.0),
        ];
        let mesh = Collider::Mesh(MeshCollider::from_triangles(vec![tri]));
        let cap_hit = Collider::Capsule {
            a: Vector3::new(0.5, 0.3, 0.5),
            b: Vector3::new(0.5, 2.3, 0.5),
            radius: 0.4,
        };
        assert!(overlap_pair(&mesh, &cap_hit).is_some());
    }

    #[test]
    fn aabb_mesh_overlap_approximates_via_bounding_sphere() {
        let tri = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 2.0),
        ];
        let mesh = Collider::Mesh(MeshCollider::from_triangles(vec![tri]));
        let bb = Collider::Aabb(BoundingBox::new([0.3, -0.3, 0.3], [0.7, 0.3, 0.7]));
        assert!(overlap_pair(&mesh, &bb).is_some());
    }

    #[test]
    fn collider_to_world_space_transforms_mesh() {
        use crate::object::Transform;
        let local = Collider::Mesh(MeshCollider::from_triangles(vec![[
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ]]));
        let mut t = Transform::new();
        t.set_position([10.0, 0.0, 0.0]);
        let world = local.to_world_space(&t);
        if let Collider::Mesh(m) = world {
            assert_eq!(m.triangles.len(), 1);
            assert!((m.triangles[0][0] - Vector3::new(10.0, 0.0, 0.0)).norm() < 1e-5);
            assert!((m.triangles[0][1] - Vector3::new(11.0, 0.0, 0.0)).norm() < 1e-5);
            assert!((m.triangles[0][2] - Vector3::new(10.0, 1.0, 0.0)).norm() < 1e-5);
        } else {
            panic!("expected Mesh");
        }
    }
}