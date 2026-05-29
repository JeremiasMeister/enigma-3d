# Collision system extensions — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `collision_world` with shape primitives, swept queries, mesh + terrain colliders, and a kinematic character controller in `default_events`, so games can hook up player movement and spatial queries with a few lines.

**Architecture:** Add a `Collider` enum to `collision_world.rs` with seven variants (AABB, Sphere, Capsule, OBB, Plane, Mesh, Heightfield). Add three query types (`RayCast` (upgrade), `ShapeCast`, `ShapeOverlap`) that iterate `AppState.objects` plus the optional terrain, transform each object's collider into world space, and dispatch to per-pair math routines via a `match` on shape pairs. Swept queries use an **iterative sweep** (split motion into N small steps, overlap-test each) rather than analytic TOI per pair — much simpler, accuracy bounded by step size. `Object` gains one new field `collider: Option<Collider>`. Character controller lives in `default_events` as a cluster of `fn(&mut AppState)` functions matching the existing `camera_*` pattern.

**Tech Stack:** Rust, nalgebra (`Vector3<f32>`, `UnitQuaternion<f32>`), existing collision_world / object / terrain / default_events modules. Tests via `cargo test`, inline `#[cfg(test)] mod tests` blocks following the existing `terrain.rs` and `light.rs` patterns.

**Reference spec:** `docs/superpowers/specs/2026-05-29-collision-extensions-design.md`

**Resolved open questions from the spec:**
- `AppState.terrain` is `Option<terrain::Terrain>` (single terrain, not a Vec). `Collider::Heightfield` is therefore a **unit variant**, not `Heightfield(usize)`.
- `state_data` API: `add_state_data(name, Box<dyn Any>)`, `get_state_data_value::<T>(name) -> Option<&T>`, `get_state_data_value_mut::<T>(name) -> Option<&mut T>`.
- Per-frame time: `app_state.delta_time` (f32).
- Tests: `#[cfg(test)] mod tests { use super::*; #[test] fn ... }` at the bottom of each source file.

---

## File Structure

| File | Action | Responsibility |
|--|--|--|
| `src/collision_world.rs` | Modify (heavy) | All shape types, hit info, query structs, per-pair math |
| `src/object.rs` | Modify (light) | Add `collider: Option<Collider>` field + accessors |
| `src/terrain.rs` | Modify (light) | Add `height_at`, `normal_at`, `triangles_in_aabb` |
| `src/default_events.rs` | Modify | Add `character_*` event functions |
| `src/lib.rs` | Modify (trivial) | Re-export new public types if needed |
| `examples/character_controller.rs` | Create | Demo of the new path |

`collision_world.rs` will grow substantially (~600–900 lines). That is acceptable because all the math is one topic and lives together; a future BVH/broadphase extraction can split it.

---

## Task 1: Foundation — Collider enum, Hit struct, Object integration

**Files:**
- Modify: `src/collision_world.rs:1-22` (add imports + new types after existing imports)
- Modify: `src/object.rs:43-56` (add `collider` field)
- Modify: `src/object.rs` (add accessors near `get_collision` at line 250)

- [ ] **Step 1.1: Write failing test for `Collider` enum construction**

In `src/collision_world.rs`, append at the bottom of the file:

```rust
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
}
```

- [ ] **Step 1.2: Run test to verify it fails**

Run: `cargo test --lib collider_variants_construct`
Expected: FAIL with "cannot find type `Collider` in this scope" (or similar — the types don't exist yet).

- [ ] **Step 1.3: Add `Hit`, `MeshCollider`, and `Collider` types**

In `src/collision_world.rs`, add after the existing `use` block (line 6):

```rust
use nalgebra::UnitQuaternion;
```

Then, after `pub fn is_colliding(...)` (around line 42), add:

```rust
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
```

- [ ] **Step 1.4: Run test to verify it passes**

Run: `cargo test --lib collider_variants_construct`
Expected: PASS.

- [ ] **Step 1.5: Write failing test for `Object.collider`**

In `src/object.rs`, find the existing `#[cfg(test)]` block (if any — there isn't yet, add one at the bottom):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision_world::Collider;
    use nalgebra::Vector3;

    #[test]
    fn object_stores_explicit_collider() {
        let mut obj = Object::new(Some("test".into()));
        assert!(obj.get_collider().is_none());
        obj.set_collider(Collider::Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        });
        assert!(obj.get_collider().is_some());
        obj.clear_collider();
        assert!(obj.get_collider().is_none());
    }
}
```

- [ ] **Step 1.6: Run test to verify it fails**

Run: `cargo test --lib object_stores_explicit_collider`
Expected: FAIL — `get_collider`, `set_collider`, `clear_collider` don't exist; field `collider` doesn't exist.

- [ ] **Step 1.7: Add `collider` field and accessors to `Object`**

In `src/object.rs`, modify the `pub struct Object` definition (line 43-56):

```rust
pub struct Object {
    pub name: String,
    pub transform: Transform,
    collision: bool,
    shapes: Vec<Shape>,
    materials: Vec<Uuid>,
    bounding_box: Option<geometry::BoundingBox>,
    unique_id: Uuid,
    cloned_id: Uuid,
    animations: HashMap<String, animation::Animation>,
    skeleton: Option<animation::Skeleton>,
    current_animation: Option<AnimationState>,
    components: HashMap<TypeId, Box<dyn Any>>,
    collider: Option<crate::collision_world::Collider>,
}
```

Find `Object::new(...)` (search for `pub fn new(`) and add `collider: None` to the struct literal it returns.

Find `impl Clone for Object` (line 58) and add `new_object.collider = self.collider.clone();` just before the final `new_object` line.

Then add accessor methods anywhere in `impl Object`, e.g. near `get_collision`:

```rust
pub fn get_collider(&self) -> Option<&crate::collision_world::Collider> {
    self.collider.as_ref()
}

pub fn set_collider(&mut self, collider: crate::collision_world::Collider) {
    self.collider = Some(collider);
}

pub fn clear_collider(&mut self) {
    self.collider = None;
}
```

- [ ] **Step 1.8: Run test to verify it passes**

Run: `cargo test --lib object_stores_explicit_collider`
Expected: PASS.

Also run `cargo build` to confirm no other code broke (the new field has a `None` default in all constructors).
Expected: Clean build.

- [ ] **Step 1.9: Commit**

```bash
git add src/collision_world.rs src/object.rs
git commit -m "feat(collision): Collider enum + Hit struct + Object.collider field"
```

---

## Task 2: Helper — world-space collider resolution

When a collision query iterates objects, it needs the object's collider *in world space*. Local-space collider + object transform → world-space collider. One helper unifies this.

**Files:**
- Modify: `src/collision_world.rs` (add `to_world_space` method on `Collider`)

- [ ] **Step 2.1: Write failing test**

Append to the `tests` mod in `src/collision_world.rs`:

```rust
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
```

- [ ] **Step 2.2: Run test to verify it fails**

Run: `cargo test --lib collider_to_world_space_translates_sphere`
Expected: FAIL — `to_world_space` doesn't exist.

- [ ] **Step 2.3: Implement `to_world_space`**

In `src/collision_world.rs`, add:

```rust
use crate::object::Transform;

impl Collider {
    /// Transforms a local-space collider into world space using the object's transform.
    /// For Plane and Heightfield, the result equals the input (those are world-space by convention,
    /// regardless of which object owns them).
    pub fn to_world_space(&self, transform: &Transform) -> Collider {
        let pos: Vector3<f32> = transform.get_position().into();
        let scale: Vector3<f32> = transform.get_scale().into();
        let rot = transform.get_rotation_quat();   // UnitQuaternion<f32>
        let uniform_scale = (scale.x + scale.y + scale.z) / 3.0;

        match self {
            Collider::Aabb(bb) => {
                // Transform bbox corners and rebuild AABB. Loses precision on rotation —
                // for rotated objects, prefer Obb.
                let min = Vector3::from(bb.min_point());
                let max = Vector3::from(bb.max_point());
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
```

Note: `Transform::rotation` is a Euler-angle vector (verified by `camera_rotate` in `default_events.rs:171–179` which reads `transform.rotation.x/.y` directly). Build a quaternion explicitly:

```rust
let rot_euler = transform.rotation;   // or transform.get_rotation() — match existing pattern
let rot = UnitQuaternion::from_euler_angles(rot_euler.x, rot_euler.y, rot_euler.z);
```

Replace `transform.get_rotation_quat()` in the code above with this construction.

- [ ] **Step 2.4: Run test to verify it passes**

Run: `cargo test --lib collider_to_world_space_translates_sphere`
Expected: PASS.

- [ ] **Step 2.5: Add tests for capsule and OBB transforms**

```rust
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
    let local = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 3.0 };
    let mut t = Transform::new();
    t.set_position([100.0, 100.0, 100.0]);
    let world = local.to_world_space(&t);
    if let Collider::Plane { normal, offset } = world {
        assert!((normal - Vector3::new(0.0,1.0,0.0)).norm() < 1e-5);
        assert!((offset - 3.0).abs() < 1e-5);
    } else { panic!() }
}
```

Run: `cargo test --lib collider_to_world_space`
Expected: All three pass.

- [ ] **Step 2.6: Commit**

```bash
git add src/collision_world.rs
git commit -m "feat(collision): Collider::to_world_space for object-space → world-space conversion"
```

---

## Task 3: Primitive-vs-primitive overlap math

Implement overlap tests for every pair of dynamic primitives: AABB, Sphere, Capsule, OBB. Symmetric pairs share a routine — each routine takes both shapes by reference and returns `bool` (overlap?) plus an optional contact point and normal.

**Convention:** all overlap routines take *world-space* shapes. The caller is responsible for `to_world_space` conversion.

**Files:**
- Modify: `src/collision_world.rs`

- [ ] **Step 3.1: Add helper — point-to-segment distance**

Most pair routines need this. Append to `src/collision_world.rs` (before the test module):

```rust
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
```

- [ ] **Step 3.2: Write test for sphere-vs-sphere overlap**

Append to `tests` mod:

```rust
#[test]
fn sphere_sphere_overlap() {
    let s1 = Collider::Sphere { center: Vector3::new(0.0,0.0,0.0), radius: 1.0 };
    let s2_hit = Collider::Sphere { center: Vector3::new(1.5,0.0,0.0), radius: 1.0 };
    let s2_miss = Collider::Sphere { center: Vector3::new(3.0,0.0,0.0), radius: 1.0 };
    assert!(overlap_pair(&s1, &s2_hit).is_some());
    assert!(overlap_pair(&s1, &s2_miss).is_none());
}
```

- [ ] **Step 3.3: Run test to verify it fails**

Run: `cargo test --lib sphere_sphere_overlap`
Expected: FAIL — `overlap_pair` doesn't exist.

- [ ] **Step 3.4: Implement `overlap_pair` dispatcher with sphere-vs-sphere arm**

Append to `src/collision_world.rs`:

```rust
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
        _ => None,  // other arms added in later steps
    }
}
```

- [ ] **Step 3.5: Run test to verify it passes**

Run: `cargo test --lib sphere_sphere_overlap`
Expected: PASS.

- [ ] **Step 3.6: Add AABB-vs-AABB arm + test**

```rust
#[test]
fn aabb_aabb_overlap_via_pair() {
    let a = Collider::Aabb(BoundingBox::new([0.0,0.0,0.0], [1.0,1.0,1.0]));
    let b_hit = Collider::Aabb(BoundingBox::new([0.5,0.5,0.5], [2.0,2.0,2.0]));
    let b_miss = Collider::Aabb(BoundingBox::new([2.0,2.0,2.0], [3.0,3.0,3.0]));
    assert!(overlap_pair(&a, &b_hit).is_some());
    assert!(overlap_pair(&a, &b_miss).is_none());
}
```

Add arm to `overlap_pair`:

```rust
(Aabb(b1), Aabb(b2)) => {
    if is_colliding(b1, b2) {
        // crude contact: midpoint of overlap region
        let min = Vector3::from(b1.min_point()).sup(&Vector3::from(b2.min_point()));
        let max = Vector3::from(b1.max_point()).inf(&Vector3::from(b2.max_point()));
        let point = (min + max) * 0.5;
        Some(Hit { object_id: Uuid::nil(), point, normal: Vector3::new(0.0,1.0,0.0), toi: 0.0 })
    } else { None }
}
```

Run: `cargo test --lib aabb_aabb_overlap_via_pair`
Expected: PASS.

- [ ] **Step 3.7: Add sphere-vs-AABB arm + test**

```rust
#[test]
fn sphere_aabb_overlap() {
    let aabb = Collider::Aabb(BoundingBox::new([-1.0,-1.0,-1.0], [1.0,1.0,1.0]));
    let s_hit = Collider::Sphere { center: Vector3::new(1.4,0.0,0.0), radius: 0.5 };
    let s_miss = Collider::Sphere { center: Vector3::new(3.0,0.0,0.0), radius: 0.5 };
    assert!(overlap_pair(&aabb, &s_hit).is_some());
    assert!(overlap_pair(&s_hit, &aabb).is_some(), "symmetry");
    assert!(overlap_pair(&aabb, &s_miss).is_none());
}
```

Add arms (both orderings — the dispatcher must handle both):

```rust
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
```

Run: `cargo test --lib sphere_aabb_overlap`
Expected: PASS.

- [ ] **Step 3.8: Add capsule-vs-sphere arm + test**

```rust
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
```

Add arms:

```rust
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
```

Run: `cargo test --lib capsule_sphere_overlap`
Expected: PASS.

- [ ] **Step 3.9: Add capsule-vs-capsule arm + test**

```rust
#[test]
fn capsule_capsule_overlap() {
    let c1 = Collider::Capsule { a: Vector3::new(0.0,0.0,0.0), b: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
    let c2_hit = Collider::Capsule { a: Vector3::new(0.8,0.5,0.0), b: Vector3::new(0.8,2.5,0.0), radius: 0.5 };
    let c2_miss = Collider::Capsule { a: Vector3::new(5.0,0.0,0.0), b: Vector3::new(5.0,2.0,0.0), radius: 0.5 };
    assert!(overlap_pair(&c1, &c2_hit).is_some());
    assert!(overlap_pair(&c1, &c2_miss).is_none());
}
```

Add arm:

```rust
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
```

Run: `cargo test --lib capsule_capsule_overlap`
Expected: PASS.

- [ ] **Step 3.10: Add capsule-vs-AABB arm + test**

```rust
#[test]
fn capsule_aabb_overlap() {
    let cap = Collider::Capsule { a: Vector3::new(0.0,0.0,0.0), b: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
    let bb_hit = Collider::Aabb(BoundingBox::new([0.3, 0.5, -0.5], [2.0, 1.5, 0.5]));
    let bb_miss = Collider::Aabb(BoundingBox::new([5.0, 0.0, 0.0], [6.0, 1.0, 1.0]));
    assert!(overlap_pair(&cap, &bb_hit).is_some());
    assert!(overlap_pair(&cap, &bb_miss).is_none());
}
```

Capsule-vs-AABB: clamp the capsule segment endpoints to the box, find closest point on the segment to that clamped point, then test as sphere-vs-AABB. Approximation, good enough for v1:

```rust
(Capsule { a, b, radius: cr }, Aabb(bb))
| (Aabb(bb), Capsule { a, b, radius: cr }) => {
    let min = Vector3::from(bb.min_point());
    let max = Vector3::from(bb.max_point());
    // Sample the segment at a few parameters; find min distance to box.
    // 9 samples is enough resolution for this approximation; analytic version is a later upgrade.
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
```

Run: `cargo test --lib capsule_aabb_overlap`
Expected: PASS.

- [ ] **Step 3.11: Add OBB-vs-OBB arm via SAT + test**

```rust
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
```

Add helper + arm. Full SAT (15 axes for OBB-OBB):

```rust
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

fn sat_obb_obb(c1: &Vector3<f32>, a1: &[Vector3<f32>;3], h1: &Vector3<f32>,
               c2: &Vector3<f32>, a2: &[Vector3<f32>;3], h2: &Vector3<f32>) -> bool {
    let mut axes: Vec<Vector3<f32>> = Vec::with_capacity(15);
    axes.extend_from_slice(a1);
    axes.extend_from_slice(a2);
    for i in 0..3 { for j in 0..3 {
        let cross = a1[i].cross(&a2[j]);
        if cross.norm_squared() > 1e-10 { axes.push(cross.normalize()); }
    }}
    for axis in &axes {
        let (min1, max1) = obb_project(c1, a1, h1, axis);
        let (min2, max2) = obb_project(c2, a2, h2, axis);
        if max1 < min2 || max2 < min1 { return false; }
    }
    true
}
```

Then add arm:

```rust
(Obb { center: c1, half_extents: h1, rotation: r1 },
 Obb { center: c2, half_extents: h2, rotation: r2 }) => {
    let a1 = obb_axes(r1);
    let a2 = obb_axes(r2);
    if sat_obb_obb(c1, &a1, h1, c2, &a2, h2) {
        let mid = (c1 + c2) * 0.5;
        let d = c2 - c1;
        let normal = if d.norm() > 1e-6 { d.normalize() } else { Vector3::new(0.0,1.0,0.0) };
        Some(Hit { object_id: Uuid::nil(), point: mid, normal, toi: 0.0 })
    } else { None }
}
```

Run: `cargo test --lib obb_obb_overlap`
Expected: PASS.

- [ ] **Step 3.12: Add OBB-vs-Sphere arm + test**

```rust
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
```

OBB-vs-sphere: transform sphere center into OBB local space, clamp to box, transform back, distance check.

```rust
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
```

Run: `cargo test --lib obb_sphere_overlap`
Expected: PASS.

- [ ] **Step 3.13: Add OBB-vs-AABB arm + test**

```rust
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
```

Treat the AABB as an axis-aligned OBB and use the existing SAT:

```rust
(Obb { center: oc, half_extents: oh, rotation: orot }, Aabb(bb))
| (Aabb(bb), Obb { center: oc, half_extents: oh, rotation: orot }) => {
    let bmin = Vector3::from(bb.min_point());
    let bmax = Vector3::from(bb.max_point());
    let bc = (bmin + bmax) * 0.5;
    let bh = (bmax - bmin) * 0.5;
    let aabb_axes = [Vector3::x(), Vector3::y(), Vector3::z()];
    let obb_a = obb_axes(orot);
    if sat_obb_obb(oc, &obb_a, oh, &bc, &aabb_axes, &bh) {
        let mid = (oc + bc) * 0.5;
        let d = bc - oc;
        let normal = if d.norm() > 1e-6 { d.normalize() } else { Vector3::new(0.0,1.0,0.0) };
        Some(Hit { object_id: Uuid::nil(), point: mid, normal, toi: 0.0 })
    } else { None }
}
```

Run: `cargo test --lib obb_aabb_overlap`
Expected: PASS.

- [ ] **Step 3.14: Add OBB-vs-Capsule arm + test**

```rust
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
```

OBB-vs-Capsule: same 9-sample-along-segment approach as capsule-vs-AABB, but use OBB local-space clamp:

```rust
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
```

Run: `cargo test --lib obb_capsule_overlap`
Expected: PASS.

- [ ] **Step 3.15: Commit**

```bash
git add src/collision_world.rs
git commit -m "feat(collision): primitive-vs-primitive overlap math (sphere/AABB/capsule/OBB)"
```

---

## Task 4: Plane overlap arms

**Files:**
- Modify: `src/collision_world.rs`

- [ ] **Step 4.1: Add sphere-vs-plane + test**

Test:

```rust
#[test]
fn sphere_plane_overlap() {
    let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
    let s_hit = Collider::Sphere { center: Vector3::new(0.0,0.3,0.0), radius: 0.5 };
    let s_miss = Collider::Sphere { center: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
    assert!(overlap_pair(&plane, &s_hit).is_some());
    assert!(overlap_pair(&plane, &s_miss).is_none());
}
```

Add arm:

```rust
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
```

Run: `cargo test --lib sphere_plane_overlap`
Expected: PASS.

- [ ] **Step 4.2: Add capsule-vs-plane + test**

```rust
#[test]
fn capsule_plane_overlap() {
    let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
    let cap_hit = Collider::Capsule { a: Vector3::new(0.0,0.3,0.0), b: Vector3::new(0.0,2.0,0.0), radius: 0.5 };
    let cap_miss = Collider::Capsule { a: Vector3::new(0.0,2.0,0.0), b: Vector3::new(0.0,3.0,0.0), radius: 0.4 };
    assert!(overlap_pair(&plane, &cap_hit).is_some());
    assert!(overlap_pair(&plane, &cap_miss).is_none());
}
```

```rust
(Plane { normal, offset }, Capsule { a, b, radius })
| (Capsule { a, b, radius }, Plane { normal, offset }) => {
    let da = normal.dot(a) - *offset;
    let db = normal.dot(b) - *offset;
    let min_d = da.min(db);
    if min_d <= *radius {
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
```

Run: `cargo test --lib capsule_plane_overlap`
Expected: PASS.

- [ ] **Step 4.3: Add AABB-vs-plane + test**

```rust
#[test]
fn aabb_plane_overlap() {
    let plane = Collider::Plane { normal: Vector3::new(0.0,1.0,0.0), offset: 0.0 };
    let bb_hit = Collider::Aabb(BoundingBox::new([-1.0,-0.5,-1.0], [1.0,0.5,1.0]));
    let bb_miss = Collider::Aabb(BoundingBox::new([-1.0,1.0,-1.0], [1.0,2.0,1.0]));
    assert!(overlap_pair(&plane, &bb_hit).is_some());
    assert!(overlap_pair(&plane, &bb_miss).is_none());
}
```

```rust
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
```

Run: `cargo test --lib aabb_plane_overlap`
Expected: PASS.

- [ ] **Step 4.4: Add OBB-vs-plane + test**

```rust
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
```

```rust
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
```

Run: `cargo test --lib obb_plane_overlap`
Expected: PASS.

- [ ] **Step 4.5: Add fallback arm logging warnings for out-of-scope pairs**

At the end of the `match` in `overlap_pair`, before the closing brace, add:

```rust
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
```

(The `EnigmaWarning` constructor signature must match what's in `src/logging/`. If it differs, adapt — the intent is "log a warning and return None.")

Run: `cargo build`
Expected: Clean.

- [ ] **Step 4.6: Commit**

```bash
git add src/collision_world.rs
git commit -m "feat(collision): plane overlap arms + warning for unsupported static pairs"
```

---

## Task 5: `ShapeOverlap` query type

**Files:**
- Modify: `src/collision_world.rs`

- [ ] **Step 5.1: Write failing test**

Append to `tests` mod:

```rust
#[test]
fn shape_overlap_finds_objects_in_radius() {
    use crate::AppState;
    let mut state = AppState::default();   // see step 5.3 — may need to use ::new or stub helper
    let mut o = Object::new(Some("a".into()));
    o.transform.set_position([0.0, 0.0, 0.0]);
    o.set_collider(Collider::Sphere { center: Vector3::zeros(), radius: 1.0 });
    let oid = o.get_unique_id();
    state.add_object(o);

    let q = ShapeOverlap::new(Collider::Sphere {
        center: Vector3::new(0.5, 0.0, 0.0),
        radius: 1.0,
    });
    let hits = q.query(&mut state);
    assert!(hits.iter().any(|h| h.object_id == oid));
}
```

If `AppState::default()` doesn't exist, use the actual constructor. The exact import path for `Object` is `crate::object::Object`. If `AppState::new` requires a display/event-loop, you may need to gate this test behind `#[cfg(not(test))]` and write an integration test in `tests/` instead — pick whichever path the existing `lib.rs::tests` module uses.

Note: this test may need adaptation because `AppState` construction requires a display. **If the test cannot run inline, move it to `tests/collision_world_integration.rs` and use the headless example pattern.** Inspect `src/lib.rs:1600+` to see how the existing tests handle AppState — match that.

- [ ] **Step 5.2: Run test to verify it fails**

Run: `cargo test --lib shape_overlap_finds_objects_in_radius` (or the integration test path)
Expected: FAIL — `ShapeOverlap` doesn't exist.

- [ ] **Step 5.3: Implement `ShapeOverlap`**

In `src/collision_world.rs`:

```rust
/// Find all objects whose collider (in world space) overlaps the given shape (in world space).
/// Iterates `app_state.objects` and the terrain. No motion — pure overlap test.
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
        // Terrain — handled in Task 7 when heightfield pair routines exist.
        hits
    }
}
```

- [ ] **Step 5.4: Run test to verify it passes**

Run: `cargo test --lib shape_overlap_finds_objects_in_radius`
Expected: PASS.

- [ ] **Step 5.5: Commit**

```bash
git add src/collision_world.rs
git commit -m "feat(collision): ShapeOverlap query type"
```

---

## Task 6: `ShapeCast` — iterative sweep

**Approach:** split the motion into N small steps (default 16 or `max(8, ceil(distance / radius))`), advance the query shape one step at a time, run `ShapeOverlap` at each, return on first hit. TOI accuracy is bounded by step size; for character-scale movement at 60+ Hz, ≤16 steps is fine. Analytic per-pair sweep can replace this later behind the same API.

**Files:**
- Modify: `src/collision_world.rs`

- [ ] **Step 6.1: Write failing test**

```rust
#[test]
fn shape_cast_finds_first_hit_along_sweep() {
    use crate::AppState;
    let mut state = AppState::default();
    let mut o = Object::new(Some("wall".into()));
    o.transform.set_position([5.0, 0.0, 0.0]);
    o.set_collider(Collider::Aabb(BoundingBox::new([-1.0,-1.0,-1.0],[1.0,1.0,1.0])));
    state.add_object(o);

    let cast = ShapeCast::new(
        Collider::Sphere { center: Vector3::new(0.0,0.0,0.0), radius: 0.5 },
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
```

- [ ] **Step 6.2: Run test to verify it fails**

Run: `cargo test --lib shape_cast_finds_first_hit_along_sweep`
Expected: FAIL — `ShapeCast` doesn't exist.

- [ ] **Step 6.3: Implement `ShapeCast`**

Helper — translate a collider in world space (just shift positions):

```rust
fn translate_world(shape: &Collider, delta: Vector3<f32>) -> Collider {
    use Collider::*;
    match shape {
        Aabb(bb) => {
            let min = Vector3::from(bb.min_point()) + delta;
            let max = Vector3::from(bb.max_point()) + delta;
            Aabb(BoundingBox::new([min.x,min.y,min.z], [max.x,max.y,max.z]))
        }
        Sphere { center, radius } => Sphere { center: center + delta, radius: *radius },
        Capsule { a, b, radius } => Capsule { a: a + delta, b: b + delta, radius: *radius },
        Obb { center, half_extents, rotation } => Obb {
            center: center + delta,
            half_extents: *half_extents,
            rotation: *rotation,
        },
        // Planes / meshes / heightfields can't be the swept shape — caller error if reached.
        other => other.clone(),
    }
}

/// Sweep a shape from its starting world-space position along `direction * distance`,
/// returning the first hit found. Internally an iterative step-and-overlap.
pub struct ShapeCast {
    shape: Collider,
    direction: Vector3<f32>,
    distance: f32,
    steps: usize,
}

impl ShapeCast {
    pub fn new(shape: Collider, direction: Vector3<f32>, distance: f32) -> Self {
        let dir = if direction.norm() > 1e-6 { direction.normalize() } else { Vector3::zeros() };
        Self { shape, direction: dir, distance, steps: 16 }
    }

    pub fn with_steps(mut self, n: usize) -> Self {
        self.steps = n.max(1);
        self
    }

    pub fn cast(&self, app_state: &mut AppState) -> Option<Hit> {
        if self.direction.norm() < 1e-6 || self.distance <= 0.0 { return None; }
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
```

- [ ] **Step 6.4: Run test to verify it passes**

Run: `cargo test --lib shape_cast_finds_first_hit_along_sweep`
Expected: PASS.

- [ ] **Step 6.5: Commit**

```bash
git add src/collision_world.rs
git commit -m "feat(collision): ShapeCast via iterative sweep"
```

---

## Task 7: Terrain integration

`Terrain` already has `get_height(x, z) -> f32` (clamped). We need: an `Option`-returning bounds-aware `height_at`, a `normal_at`, a `triangles_in_aabb`, and to fill in `heightfield_query_fn` (currently a stub). Then add heightfield arms to `overlap_pair`. Finally, extend `ShapeOverlap::query` to also check the terrain.

**Files:**
- Modify: `src/terrain.rs`
- Modify: `src/collision_world.rs`

- [ ] **Step 7.1: Write failing test for terrain methods**

Append to the existing `#[cfg(test)] mod tests` in `src/terrain.rs`:

```rust
#[test]
fn height_at_returns_none_outside_bounds() {
    // Construct a Terrain with known bounds — see existing terrain tests for the pattern.
    // ... if existing tests use a fixture builder, use it. Otherwise build a minimal TerrainConfig.
    // The exact constructor lives at terrain.rs:311.
    let cfg = TerrainConfig::default();
    let terrain = make_test_terrain(&cfg);   // helper from existing tests
    assert!(terrain.height_at(9999.0, 9999.0).is_none());
    assert!(terrain.height_at(0.0, 0.0).is_some());
}

#[test]
fn triangles_in_aabb_returns_triangles() {
    let cfg = TerrainConfig::default();
    let terrain = make_test_terrain(&cfg);
    let bb = crate::geometry::BoundingBox::new([-1.0, -100.0, -1.0], [1.0, 100.0, 1.0]);
    let tris = terrain.triangles_in_aabb(&bb);
    assert!(!tris.is_empty());
}
```

Look at the existing terrain tests in `src/terrain.rs:555-678` to see the actual constructor pattern and adapt — `make_test_terrain` may already exist or you need to add a helper that doesn't require a `Display`.

- [ ] **Step 7.2: Run test to verify it fails**

Run: `cargo test --lib height_at_returns_none_outside_bounds`
Expected: FAIL — `height_at` doesn't exist.

- [ ] **Step 7.3: Add the new Terrain methods**

In `src/terrain.rs`, near `get_height` (line 430):

```rust
/// Returns `Some(world_y)` at the given world-space XZ position if it is within the terrain bounds,
/// or `None` if outside.
pub fn height_at(&self, x: f32, z: f32) -> Option<f32> {
    let cfg = &self.config;
    let half_w = cfg.width * 0.5;
    let half_d = cfg.depth * 0.5;
    let lx = x - self.position[0];
    let lz = z - self.position[2];
    if lx < -half_w || lx > half_w || lz < -half_d || lz > half_d {
        return None;
    }
    Some(self.get_height(x, z))
}

/// Returns the world-space upward normal at the given XZ position. `None` if outside bounds.
pub fn normal_at(&self, x: f32, z: f32) -> Option<Vector3<f32>> {
    let h_here = self.height_at(x, z)?;
    let eps = 0.1_f32;
    let h_dx = self.height_at(x + eps, z)?;
    let h_dz = self.height_at(x, z + eps)?;
    let tx = Vector3::new(eps, h_dx - h_here, 0.0);
    let tz = Vector3::new(0.0, h_dz - h_here, eps);
    Some(tz.cross(&tx).normalize())
}

/// Returns the terrain triangles (world-space) whose XZ projection overlaps the given AABB.
/// Used by mesh-style collision queries (sphere/capsule vs terrain).
pub fn triangles_in_aabb(&self, aabb: &crate::geometry::BoundingBox) -> Vec<[Vector3<f32>; 3]> {
    let cfg = &self.config;
    let verts = (cfg.resolution + 1) as usize;
    let cell_x = cfg.width / cfg.resolution as f32;
    let cell_z = cfg.depth / cfg.resolution as f32;
    let half_w = cfg.width * 0.5;
    let half_d = cfg.depth * 0.5;

    let bmin = aabb.min_point();
    let bmax = aabb.max_point();
    let lx_min = ((bmin[0] - self.position[0] + half_w) / cell_x).floor() as isize;
    let lx_max = ((bmax[0] - self.position[0] + half_w) / cell_x).ceil()  as isize;
    let lz_min = ((bmin[2] - self.position[2] + half_d) / cell_z).floor() as isize;
    let lz_max = ((bmax[2] - self.position[2] + half_d) / cell_z).ceil()  as isize;

    let mut out = Vec::new();
    let res = cfg.resolution as isize;
    for iz in lz_min.max(0)..lz_max.min(res) {
        for ix in lx_min.max(0)..lx_max.min(res) {
            let i00 = iz as usize * verts + ix as usize;
            let i10 = i00 + 1;
            let i01 = i00 + verts;
            let i11 = i01 + 1;
            let v = |i: usize, x_off: f32, z_off: f32| -> Vector3<f32> {
                let wx = self.position[0] - half_w + x_off;
                let wz = self.position[2] - half_d + z_off;
                Vector3::new(wx, self.heightmap[i] + self.position[1], wz)
            };
            let x0 = ix as f32 * cell_x;
            let x1 = (ix as f32 + 1.0) * cell_x;
            let z0 = iz as f32 * cell_z;
            let z1 = (iz as f32 + 1.0) * cell_z;
            let p00 = v(i00, x0, z0);
            let p10 = v(i10, x1, z0);
            let p01 = v(i01, x0, z1);
            let p11 = v(i11, x1, z1);
            out.push([p00, p10, p11]);
            out.push([p00, p11, p01]);
        }
    }
    out
}
```

Also replace `heightfield_query_fn` (line 442) with a real implementation:

```rust
pub fn heightfield_query_fn(&self) -> impl Fn(f32, f32) -> Option<f32> + '_ {
    move |x: f32, z: f32| self.height_at(x, z)
}
```

- [ ] **Step 7.4: Run terrain tests**

Run: `cargo test --lib terrain`
Expected: All terrain tests pass, including the new ones.

- [ ] **Step 7.5: Add ray-vs-triangle helper**

Append to `src/collision_world.rs`:

```rust
/// Möller–Trumbore ray-triangle intersection. Returns t along ray if hit.
fn ray_triangle(
    origin: &Vector3<f32>, dir: &Vector3<f32>,
    v0: &Vector3<f32>, v1: &Vector3<f32>, v2: &Vector3<f32>,
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

/// Sphere-vs-triangle overlap: closest point on triangle test.
fn sphere_overlap_triangle(
    center: &Vector3<f32>, radius: f32,
    v0: &Vector3<f32>, v1: &Vector3<f32>, v2: &Vector3<f32>,
) -> Option<(Vector3<f32>, Vector3<f32>)> {
    // Closest point on triangle — Real-Time Collision Detection §5.1.5.
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
    center: &Vector3<f32>, closest: Vector3<f32>, radius: f32,
) -> Option<(Vector3<f32>, Vector3<f32>)> {
    let d = center - closest;
    let dist = d.norm();
    if dist <= radius {
        let normal = if dist > 1e-6 { d / dist } else { Vector3::new(0.0,1.0,0.0) };
        Some((closest, normal))
    } else { None }
}
```

- [ ] **Step 7.6: Add heightfield arms to `overlap_pair`**

```rust
(Heightfield, Sphere { center, radius }) | (Sphere { center, radius }, Heightfield) => {
    // Build a small AABB around the sphere, fetch overlapping triangles from terrain, test each.
    // This requires access to AppState — but overlap_pair has no AppState arg.
    // Resolution: leave heightfield handling to ShapeOverlap::query (which has AppState),
    // not overlap_pair. Return None here as a no-op.
    None
}
(Heightfield, Capsule { .. }) | (Capsule { .. }, Heightfield)
| (Heightfield, Aabb(_)) | (Aabb(_), Heightfield)
| (Heightfield, Obb { .. }) | (Obb { .. }, Heightfield) => None,
```

The cleaner architecture: `overlap_pair` handles object-vs-object only. Terrain is handled separately by `ShapeOverlap::query` because it needs `AppState`.

- [ ] **Step 7.7: Extend `ShapeOverlap::query` to check terrain**

Replace the existing `ShapeOverlap::query` body with:

```rust
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
    // Terrain
    if let Some(terrain) = app_state.get_terrain() {
        if let Some(hit) = overlap_shape_terrain(&self.shape, terrain) {
            hits.push(hit);
        }
    }
    hits
}
```

Add the helper:

```rust
fn overlap_shape_terrain(shape: &Collider, terrain: &crate::terrain::Terrain) -> Option<Hit> {
    use Collider::*;
    // Compute the shape's AABB (in world space) to fetch overlapping triangles.
    let aabb = shape_aabb(shape)?;
    let tris = terrain.triangles_in_aabb(&aabb);
    match shape {
        Sphere { center, radius } => {
            for tri in &tris {
                if let Some((point, normal)) = sphere_overlap_triangle(center, *radius, &tri[0], &tri[1], &tri[2]) {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
        }
        Capsule { a, b, radius } => {
            // 9-sample-along-segment, sphere-vs-triangle each
            for i in 0..=8 {
                let t = i as f32 / 8.0;
                let p = a + (b - a) * t;
                for tri in &tris {
                    if let Some((point, normal)) = sphere_overlap_triangle(&p, *radius, &tri[0], &tri[1], &tri[2]) {
                        return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                    }
                }
            }
            None
        }
        Aabb(_) | Obb { .. } => {
            // For v1, treat AABB/OBB as their bounding sphere for terrain queries.
            // Less accurate but consistent with "simple code".
            let aabb = shape_aabb(shape)?;
            let min = Vector3::from(aabb.min_point());
            let max = Vector3::from(aabb.max_point());
            let center = (min + max) * 0.5;
            let radius = (max - min).norm() * 0.5;
            for tri in &tris {
                if let Some((point, normal)) = sphere_overlap_triangle(&center, radius, &tri[0], &tri[1], &tri[2]) {
                    return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
                }
            }
            None
        }
        _ => None,
    }
}

/// World-space AABB of a collider. None for unbounded shapes (Plane, Heightfield).
fn shape_aabb(shape: &Collider) -> Option<BoundingBox> {
    use Collider::*;
    let pad = |c: &Vector3<f32>, r: f32| BoundingBox::new(
        [c.x - r, c.y - r, c.z - r], [c.x + r, c.y + r, c.z + r],
    );
    match shape {
        Aabb(bb) => Some(bb.clone()),
        Sphere { center, radius } => Some(pad(center, *radius)),
        Capsule { a, b, radius } => {
            let min = a.inf(b) - Vector3::new(*radius, *radius, *radius);
            let max = a.sup(b) + Vector3::new(*radius, *radius, *radius);
            Some(BoundingBox::new([min.x,min.y,min.z],[max.x,max.y,max.z]))
        }
        Obb { center, half_extents, rotation } => {
            // Conservative: rotate axes, sum projections.
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
                for v in tri.iter() { min = min.inf(v); max = max.sup(v); }
            }
            if min.x.is_finite() {
                Some(BoundingBox::new([min.x,min.y,min.z],[max.x,max.y,max.z]))
            } else { None }
        }
        Plane { .. } | Heightfield => None,
    }
}
```

- [ ] **Step 7.8: Write integration test (sphere over terrain hits the ground)**

This needs a real `AppState` with a terrain. Look at how existing tests in `src/lib.rs:1600` or examples instantiate terrain headlessly. If headless terrain construction isn't possible, add this test to `tests/` as an integration test that uses the example pattern.

```rust
#[test]
fn sphere_above_terrain_overlaps_when_dropped() {
    // Construct a flat terrain at y=0.
    let cfg = crate::terrain::TerrainConfig::default();   // adjust as needed
    let terrain = make_test_terrain(&cfg);   // existing helper or new
    let mut state = AppState::default();
    state.set_terrain(terrain);

    // Sphere centered just above the ground.
    let s = Collider::Sphere { center: Vector3::new(0.0, 0.4, 0.0), radius: 0.5 };
    let hits = ShapeOverlap::new(s).query(&mut state);
    assert!(hits.iter().any(|h| h.object_id == Uuid::nil()), "expected terrain hit");
}
```

Run: `cargo test`
Expected: PASS (including terrain test).

- [ ] **Step 7.9: Commit**

```bash
git add src/terrain.rs src/collision_world.rs
git commit -m "feat(collision): terrain height_at / normal_at / triangles_in_aabb + heightfield overlap"
```

---

## Task 8: Mesh collider

Mesh-vs-primitive overlaps (sphere, capsule, AABB, OBB vs static triangle list) + `MeshCollider::from_object_mesh` constructor.

**Files:**
- Modify: `src/collision_world.rs`

- [ ] **Step 8.1: Add `MeshCollider::from_object_mesh` + test**

Test:

```rust
#[test]
fn mesh_collider_from_object_mesh_collects_triangles() {
    use crate::object::{Object, Shape};
    use crate::geometry::Vertex;
    let mut obj = Object::new(Some("box".into()));
    // Add a single-triangle shape.
    let mut shape = Shape::new();
    let mk = |x: f32, y: f32, z: f32| Vertex {
        position: [x, y, z],
        normal: [0.0, 1.0, 0.0],
        ..Default::default()
    };
    shape.vertices = vec![mk(0.0,0.0,0.0), mk(1.0,0.0,0.0), mk(0.0,1.0,0.0)];
    shape.indices  = vec![0, 1, 2];
    obj.add_shape(shape);

    let mesh = MeshCollider::from_object_mesh(&obj);
    assert_eq!(mesh.triangles.len(), 1);
}
```

(Adapt `Vertex` construction to match the actual struct — check `src/geometry.rs` for field names. If `Vertex` doesn't impl `Default`, fill all required fields.)

Implementation:

```rust
impl MeshCollider {
    /// Builds a mesh collider from all triangles in the object's visual shapes.
    /// Triangles are stored in object-local space (the object's transform is applied at query time).
    pub fn from_object_mesh(object: &crate::object::Object) -> Self {
        let mut tris = Vec::new();
        for shape in object.shapes_ref() {
            // shape.indices in groups of 3
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
```

This requires `Object::shapes_ref() -> &[Shape]`. If it doesn't exist, add it:

```rust
// In src/object.rs:
impl Object {
    pub fn shapes_ref(&self) -> &[Shape] { &self.shapes }
}
```

Run: `cargo test --lib mesh_collider_from_object_mesh`
Expected: PASS.

- [ ] **Step 8.2: Add sphere-vs-mesh arm to `overlap_pair` + test**

Test:

```rust
#[test]
fn sphere_mesh_overlap() {
    let tri = [
        Vector3::new(0.0,0.0,0.0),
        Vector3::new(2.0,0.0,0.0),
        Vector3::new(0.0,0.0,2.0),
    ];
    let mesh = Collider::Mesh(MeshCollider::from_triangles(vec![tri]));
    let s_hit  = Collider::Sphere { center: Vector3::new(0.5, 0.3, 0.5), radius: 0.5 };
    let s_miss = Collider::Sphere { center: Vector3::new(5.0, 5.0, 5.0), radius: 0.5 };
    assert!(overlap_pair(&mesh, &s_hit).is_some());
    assert!(overlap_pair(&mesh, &s_miss).is_none());
}
```

Add arm to `overlap_pair`:

```rust
(Mesh(m), Sphere { center, radius }) | (Sphere { center, radius }, Mesh(m)) => {
    for tri in &m.triangles {
        if let Some((point, normal)) = sphere_overlap_triangle(center, *radius, &tri[0], &tri[1], &tri[2]) {
            return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
        }
    }
    None
}
```

Run: `cargo test --lib sphere_mesh_overlap`
Expected: PASS.

- [ ] **Step 8.3: Add capsule-vs-mesh + test**

```rust
#[test]
fn capsule_mesh_overlap() {
    let tri = [
        Vector3::new(0.0,0.0,0.0),
        Vector3::new(2.0,0.0,0.0),
        Vector3::new(0.0,0.0,2.0),
    ];
    let mesh = Collider::Mesh(MeshCollider::from_triangles(vec![tri]));
    let cap_hit  = Collider::Capsule {
        a: Vector3::new(0.5, 0.3, 0.5),
        b: Vector3::new(0.5, 2.3, 0.5),
        radius: 0.4,
    };
    assert!(overlap_pair(&mesh, &cap_hit).is_some());
}
```

```rust
(Mesh(m), Capsule { a, b, radius }) | (Capsule { a, b, radius }, Mesh(m)) => {
    for i in 0..=8 {
        let t = i as f32 / 8.0;
        let p = a + (b - a) * t;
        for tri in &m.triangles {
            if let Some((point, normal)) = sphere_overlap_triangle(&p, *radius, &tri[0], &tri[1], &tri[2]) {
                return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
            }
        }
    }
    None
}
```

Run: `cargo test --lib capsule_mesh_overlap`
Expected: PASS.

- [ ] **Step 8.4: Add AABB-vs-mesh + OBB-vs-mesh arms**

For v1 simplicity: treat the AABB/OBB as its bounding sphere for mesh queries. Less accurate but matches the spec's "simple code" preference. Add to `overlap_pair`:

```rust
(Mesh(m), Aabb(bb)) | (Aabb(bb), Mesh(m)) => {
    let min = Vector3::from(bb.min_point());
    let max = Vector3::from(bb.max_point());
    let center = (min + max) * 0.5;
    let radius = (max - min).norm() * 0.5;
    for tri in &m.triangles {
        if let Some((point, normal)) = sphere_overlap_triangle(&center, radius, &tri[0], &tri[1], &tri[2]) {
            return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
        }
    }
    None
}
(Mesh(m), Obb { center, half_extents, .. }) | (Obb { center, half_extents, .. }, Mesh(m)) => {
    let radius = half_extents.norm();
    for tri in &m.triangles {
        if let Some((point, normal)) = sphere_overlap_triangle(center, radius, &tri[0], &tri[1], &tri[2]) {
            return Some(Hit { object_id: Uuid::nil(), point, normal, toi: 0.0 });
        }
    }
    None
}
```

Test:

```rust
#[test]
fn aabb_mesh_overlap_approximates_via_bounding_sphere() {
    let tri = [
        Vector3::new(0.0,0.0,0.0),
        Vector3::new(2.0,0.0,0.0),
        Vector3::new(0.0,0.0,2.0),
    ];
    let mesh = Collider::Mesh(MeshCollider::from_triangles(vec![tri]));
    let bb = Collider::Aabb(BoundingBox::new([0.3, -0.3, 0.3], [0.7, 0.3, 0.7]));
    assert!(overlap_pair(&mesh, &bb).is_some());
}
```

Run: `cargo test --lib aabb_mesh_overlap_approximates_via_bounding_sphere`
Expected: PASS.

- [ ] **Step 8.5: Commit**

```bash
git add src/collision_world.rs src/object.rs
git commit -m "feat(collision): mesh collider primitives + MeshCollider::from_object_mesh"
```

---

## Task 9: `RayCast` upgrade — dispatch to full collider set

The existing `RayCast::cast` only tests against the auto-derived AABB (`object.get_bounding_box()`). Upgrade it to use the object's explicit collider when present.

**Files:**
- Modify: `src/collision_world.rs`

- [ ] **Step 9.1: Write failing test**

The discriminating signal is the *hit point*. Pre-upgrade, the ray hits the auto-derived AABB and the recorded point lies on the box face. Post-upgrade, the ray hits the explicit sphere surface, which is at a different (smaller-x) point.

```rust
#[test]
fn ray_cast_uses_explicit_sphere_collider() {
    use crate::AppState;
    let mut state = AppState::default();   // see Task 5 note — if AppState cannot be constructed
                                            // headlessly, move this to tests/collision_world_test.rs
                                            // following whatever pattern existing integration tests use.
    let mut o = Object::new(Some("ball".into()));
    o.transform.set_position([5.0, 0.0, 0.0]);
    o.set_collider(Collider::Sphere { center: Vector3::zeros(), radius: 1.0 });
    let oid = o.get_unique_id();
    state.add_object(o);

    let mut ray = RayCast::new(Vector3::zeros(), Vector3::new(1.0, 0.0, 0.0), 100.0);
    ray.cast(&mut state);
    assert!(ray.get_intersection_uuids().contains(&oid));

    // Sphere centered at world (5,0,0) with radius 1.0 → ray hits surface at x=4.0.
    // If RayCast were still using the auto AABB (degenerate for a shape-less object,
    // or 0.5 unit cube around origin), the hit point would be at x≈5 or different.
    // This assertion fails pre-upgrade.
    let hit_point = ray.get_intersection_map().get(&oid).unwrap();
    assert!((hit_point.x - 4.0).abs() < 0.05,
        "expected sphere surface hit at x=4.0, got x={}", hit_point.x);
}
```

- [ ] **Step 9.2: Run test to verify it fails**

Run: `cargo test --lib ray_cast_uses_explicit_sphere_collider`
Expected: FAIL — either the object's auto-derived AABB doesn't match the explicit sphere, or the hit point assertion fails because pre-upgrade still uses the AABB.

- [ ] **Step 9.3: Add ray-vs-shape helper and upgrade `RayCast::cast`**

Add to `src/collision_world.rs`:

```rust
/// Tests a ray against a world-space collider. Returns (t along ray, hit point) if hit.
fn ray_vs_collider(
    origin: &Vector3<f32>, dir: &Vector3<f32>, length: f32,
    shape: &Collider,
) -> Option<(f32, Vector3<f32>)> {
    use Collider::*;
    match shape {
        Aabb(bb) => {
            // Existing slab test — reuse the logic in RayCast::intersects_bounding_box,
            // refactored into this helper.
            ray_vs_aabb(origin, dir, length, bb)
        }
        Sphere { center, radius } => {
            let oc = origin - center;
            let b = oc.dot(dir);
            let c = oc.norm_squared() - radius * radius;
            let disc = b * b - c;
            if disc < 0.0 { return None; }
            let t = -b - disc.sqrt();
            if t > 0.0 && t < length { Some((t, origin + dir * t)) } else { None }
        }
        Plane { normal, offset } => {
            let denom = normal.dot(dir);
            if denom.abs() < 1e-6 { return None; }
            let t = (*offset - normal.dot(origin)) / denom;
            if t > 0.0 && t < length { Some((t, origin + dir * t)) } else { None }
        }
        Capsule { a, b, radius } => {
            // Iterative sweep approximation: sample sphere-vs-segment-point along ray.
            // For v1 simplicity: enclose capsule in AABB and ray-test that. Less accurate.
            let amin = a.inf(b) - Vector3::new(*radius, *radius, *radius);
            let amax = a.sup(b) + Vector3::new(*radius, *radius, *radius);
            let bb = BoundingBox::new([amin.x,amin.y,amin.z],[amax.x,amax.y,amax.z]);
            ray_vs_aabb(origin, dir, length, &bb)
        }
        Obb { center, half_extents, rotation } => {
            // Transform ray into OBB local space, test against axis-aligned half-extents.
            let inv = rotation.inverse();
            let lo = inv * (origin - center);
            let ld = inv * dir;
            let local_bb = BoundingBox::new(
                [-half_extents.x, -half_extents.y, -half_extents.z],
                [half_extents.x, half_extents.y, half_extents.z],
            );
            let hit = ray_vs_aabb(&lo, &ld, length, &local_bb)?;
            // Transform hit point back to world.
            let world_hit = center + rotation * hit.1;
            Some((hit.0, world_hit))
        }
        Mesh(m) => {
            let mut best: Option<(f32, Vector3<f32>)> = None;
            for tri in &m.triangles {
                if let Some(t) = ray_triangle(origin, dir, &tri[0], &tri[1], &tri[2]) {
                    if t < length && best.map_or(true, |(bt, _)| t < bt) {
                        best = Some((t, origin + dir * t));
                    }
                }
            }
            best
        }
        Heightfield => None,  // handled separately by RayCast::cast since needs AppState
    }
}

fn ray_vs_aabb(
    origin: &Vector3<f32>, dir: &Vector3<f32>, length: f32, bb: &BoundingBox,
) -> Option<(f32, Vector3<f32>)> {
    // Lifted from existing RayCast::intersects_bounding_box logic.
    // BoundingBox::min_point() / max_point() return Vector3<f32> (verified at collision_world.rs:153-156
    // where the existing code accesses .x / .y / .z fields directly).
    let inv = Vector3::new(1.0/dir.x, 1.0/dir.y, 1.0/dir.z);
    let sign = [
        (inv.x < 0.0) as usize,
        (inv.y < 0.0) as usize,
        (inv.z < 0.0) as usize,
    ];
    let bbox = [bb.min_point(), bb.max_point()];
    let mut tmin = (bbox[sign[0]].x - origin.x) * inv.x;
    let mut tmax = (bbox[1-sign[0]].x - origin.x) * inv.x;
    let tymin = (bbox[sign[1]].y - origin.y) * inv.y;
    let tymax = (bbox[1-sign[1]].y - origin.y) * inv.y;
    if tmin > tymax || tymin > tmax { return None; }
    if tymin > tmin { tmin = tymin; }
    if tymax < tmax { tmax = tymax; }
    let tzmin = (bbox[sign[2]].z - origin.z) * inv.z;
    let tzmax = (bbox[1-sign[2]].z - origin.z) * inv.z;
    if tmin > tzmax || tzmin > tmax { return None; }
    if tzmin > tmin { tmin = tzmin; }
    if tzmax < tmax { tmax = tzmax; }
    if tmin < length && tmax > 0.0 {
        Some((tmin, origin + dir * tmin))
    } else { None }
}
```

Now modify `RayCast::cast`:

```rust
pub fn cast(&mut self, app_state: &mut AppState) {
    for object in app_state.objects.iter_mut() {
        if !*object.get_collision() { continue; }
        let world_collider = match object.get_collider() {
            Some(c) => c.to_world_space(&object.transform),
            None => Collider::Aabb(object.get_bounding_box()),
        };
        if let Some((_, point)) = ray_vs_collider(&self.origin, &self.direction, self.length, &world_collider) {
            self.intersection_objects.insert(object.get_unique_id(), point);
        }
    }
    // Heightfield ray-cast: walk along the ray sampling terrain heights.
    if let Some(terrain) = app_state.get_terrain() {
        if let Some(point) = ray_vs_terrain(&self.origin, &self.direction, self.length, terrain) {
            self.intersection_objects.insert(Uuid::nil(), point);
        }
    }
}

fn ray_vs_terrain(
    origin: &Vector3<f32>, dir: &Vector3<f32>, length: f32,
    terrain: &crate::terrain::Terrain,
) -> Option<Vector3<f32>> {
    let steps = 64;
    let step_size = length / steps as f32;
    let mut prev_above: Option<bool> = None;
    let mut prev_p = *origin;
    for i in 0..=steps {
        let t = i as f32 * step_size;
        let p = origin + dir * t;
        if let Some(h) = terrain.height_at(p.x, p.z) {
            let above = p.y > h;
            if let Some(pa) = prev_above {
                if pa != above {
                    // Bracketed — linear-interpolate between prev_p and p for the crossing point.
                    let prev_h = terrain.height_at(prev_p.x, prev_p.z).unwrap_or(prev_p.y);
                    let alpha = (prev_p.y - prev_h) / ((prev_p.y - prev_h) - (p.y - h));
                    return Some(prev_p + (p - prev_p) * alpha);
                }
            }
            prev_above = Some(above);
            prev_p = p;
        }
    }
    None
}
```

Remove the old `intersects_bounding_box` method on `RayCast` (it's superseded by `ray_vs_aabb`).

- [ ] **Step 9.4: Run test to verify it passes**

Run: `cargo test --lib ray_cast_uses_explicit_collider_when_present`
Expected: PASS.

Also run all collision_world tests: `cargo test --lib collision_world`
Expected: All pass.

- [ ] **Step 9.5: Commit**

```bash
git add src/collision_world.rs
git commit -m "feat(collision): RayCast dispatches to full collider set + terrain ray-cast"
```

---

## Task 10: Character controller in `default_events`

A capsule-based kinematic character with WASD + jump + gravity + slide-along-surfaces. State held in `app_state.state_data` so multiple controllers can exist.

**Files:**
- Modify: `src/default_events.rs`

- [ ] **Step 10.1: Define `CharacterState` and the constants key**

Append to `src/default_events.rs`:

```rust
use nalgebra::Vector3;
use crate::collision_world::{Collider, ShapeCast, ShapeOverlap};

/// State for a kinematic character controller. Stored in `app_state.state_data`
/// under the key `"character_state"` (or a custom key if multiple characters).
pub struct CharacterState {
    /// Object UUID this controller drives. Position is written to `object.transform`.
    pub object_id: uuid::Uuid,
    /// Capsule shape (local to the object's transform — typically a vertical capsule).
    pub capsule: Collider,
    pub velocity: Vector3<f32>,
    pub grounded: bool,
    pub move_speed: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
}

impl CharacterState {
    pub fn new(object_id: uuid::Uuid) -> Self {
        Self {
            object_id,
            capsule: Collider::Capsule {
                a: Vector3::new(0.0, 0.5, 0.0),
                b: Vector3::new(0.0, 1.5, 0.0),
                radius: 0.4,
            },
            velocity: Vector3::zeros(),
            grounded: false,
            move_speed: 5.0,
            jump_velocity: 5.0,
            gravity: -20.0,
        }
    }
}

pub const CHARACTER_STATE_KEY: &str = "character_state";
```

Note: this won't compile if the `inject_event` callback signature can't hold the closure capture for state_data. Verify the existing `default_events` patterns use `app_state.get_state_data_value::<T>` to retrieve. Match that.

- [ ] **Step 10.2: Add the movement input functions + test sketch**

```rust
fn add_horizontal_velocity(app_state: &mut AppState, world_dir: Vector3<f32>) {
    let dt = app_state.delta_time;
    if let Some(cs) = app_state.get_state_data_value_mut::<CharacterState>(CHARACTER_STATE_KEY) {
        // Add an impulse-like horizontal acceleration. We overwrite XZ velocity each frame
        // for tight responsiveness — feel free to switch to additive if you want sliding inertia.
        let dir = Vector3::new(world_dir.x, 0.0, world_dir.z);
        let n = dir.norm();
        if n > 1e-6 {
            let v = dir / n * cs.move_speed;
            cs.velocity.x = v.x;
            cs.velocity.z = v.z;
        } else {
            cs.velocity.x = 0.0;
            cs.velocity.z = 0.0;
        }
        let _ = dt;
    }
}

pub fn character_move_forward(app_state: &mut AppState) {
    let cam_fwd = app_state.camera.as_ref().map(|c| c.transform.forward());
    if let Some(f) = cam_fwd {
        add_horizontal_velocity(app_state, Vector3::new(-f.x, 0.0, -f.z));
    }
}

pub fn character_move_backward(app_state: &mut AppState) {
    let cam_fwd = app_state.camera.as_ref().map(|c| c.transform.forward());
    if let Some(f) = cam_fwd {
        add_horizontal_velocity(app_state, Vector3::new(f.x, 0.0, f.z));
    }
}

pub fn character_move_left(app_state: &mut AppState) {
    let cam_left = app_state.camera.as_ref().map(|c| c.transform.left());
    if let Some(l) = cam_left {
        add_horizontal_velocity(app_state, Vector3::new(l.x, 0.0, l.z));
    }
}

pub fn character_move_right(app_state: &mut AppState) {
    let cam_left = app_state.camera.as_ref().map(|c| c.transform.left());
    if let Some(l) = cam_left {
        add_horizontal_velocity(app_state, Vector3::new(-l.x, 0.0, -l.z));
    }
}

pub fn character_jump(app_state: &mut AppState) {
    if let Some(cs) = app_state.get_state_data_value_mut::<CharacterState>(CHARACTER_STATE_KEY) {
        if cs.grounded {
            cs.velocity.y = cs.jump_velocity;
            cs.grounded = false;
        }
    }
}
```

The exact accessor names (`camera`, `delta_time`, `get_state_data_value_mut`) should already match what `camera_walk_*` uses (see lines 107–153 of `default_events.rs`). If you find a mismatch, follow the existing pattern.

- [ ] **Step 10.3: Add `character_update` (the per-frame integrator)**

```rust
/// Per-frame: apply gravity, sweep the capsule against the world, slide along contacts,
/// snap to ground, write the new position back to the character object.
/// Register via `app_state.inject_update_function(Arc::new(character_update))`.
pub fn character_update(app_state: &mut AppState) {
    let dt = app_state.delta_time;
    // 1. Read & integrate.
    let (mut velocity, mut grounded, capsule, object_id, gravity) = {
        let Some(cs) = app_state.get_state_data_value::<CharacterState>(CHARACTER_STATE_KEY) else { return; };
        (cs.velocity, cs.grounded, cs.capsule.clone(), cs.object_id, cs.gravity)
    };
    velocity.y += gravity * dt;
    let mut remaining = velocity * dt;

    // 2. Get current object position.
    let mut pos = {
        let obj = app_state.objects.iter().find(|o| o.get_unique_id() == object_id);
        match obj {
            Some(o) => Vector3::from(o.transform.get_position()),
            None => return,
        }
    };

    // 3. Iteratively cast + slide. Max 3 iterations to avoid infinite loops in corners.
    for _ in 0..3 {
        if remaining.norm() < 1e-5 { break; }
        let dir = remaining.normalize();
        let dist = remaining.norm();
        let world_capsule = translate_capsule(&capsule, pos);
        let cast = ShapeCast::new(world_capsule, dir, dist);
        match cast.cast(app_state) {
            Some(hit) if hit.toi < 1.0 => {
                // Advance to just before the hit.
                let advance = (hit.toi * dist - 1e-3).max(0.0);
                pos += dir * advance;
                // Project remaining motion along the contact tangent.
                let consumed = advance / dist;
                let left = remaining * (1.0 - consumed);
                let tangent = left - hit.normal * left.dot(&hit.normal);
                remaining = tangent;
                // Kill velocity into the surface.
                velocity = velocity - hit.normal * velocity.dot(&hit.normal);
            }
            _ => {
                pos += remaining;
                break;
            }
        }
    }

    // 4. Ground check — small downward cast.
    let world_capsule = translate_capsule(&capsule, pos);
    let down_cast = ShapeCast::new(world_capsule, Vector3::new(0.0, -1.0, 0.0), 0.05);
    grounded = down_cast.cast(app_state).map_or(false, |h| h.normal.y > 0.5);

    // 5. Write back.
    if let Some(obj) = app_state.objects.iter_mut().find(|o| o.get_unique_id() == object_id) {
        obj.transform.set_position([pos.x, pos.y, pos.z]);
    }
    if let Some(cs) = app_state.get_state_data_value_mut::<CharacterState>(CHARACTER_STATE_KEY) {
        cs.velocity = velocity;
        cs.grounded = grounded;
    }
}

fn translate_capsule(capsule: &Collider, pos: Vector3<f32>) -> Collider {
    if let Collider::Capsule { a, b, radius } = capsule {
        Collider::Capsule { a: a + pos, b: b + pos, radius: *radius }
    } else {
        capsule.clone()
    }
}
```

- [ ] **Step 10.4: Build and check warnings**

Run: `cargo build`
Expected: Clean build, no warnings related to the new code.

Run: `cargo test --lib`
Expected: All previous tests still pass.

- [ ] **Step 10.5: Commit**

```bash
git add src/default_events.rs
git commit -m "feat(default_events): kinematic character controller (capsule, slide, jump, gravity)"
```

---

## Task 11: Example — `character_controller.rs`

A minimal example showing a player capsule walking on a flat ground plane + a few box obstacles, with WASD + space + mouse-look.

**Files:**
- Create: `examples/character_controller.rs`

- [ ] **Step 11.1: Write the example**

Create `examples/character_controller.rs`:

```rust
//! Kinematic character controller demo.
//!
//! WASD to move, space to jump, mouse to look. The player is a capsule;
//! the world is a ground plane plus a few static box obstacles.
//!
//! Run with: `cargo run --example=character_controller --features=examples`

use std::sync::Arc;
use enigma_3d::{AppState, EventLoop};
use enigma_3d::collision_world::{Collider, MeshCollider};
use enigma_3d::default_events::{
    self, CharacterState, CHARACTER_STATE_KEY,
    character_move_forward, character_move_backward, character_move_left, character_move_right,
    character_jump, character_update,
};
use enigma_3d::event::{EventCharacteristic, EventModifiers, VirtualKeyCode};
use enigma_3d::object::Object;
use enigma_3d::geometry::BoundingBox;
use nalgebra::Vector3;

fn main() {
    let event_loop = EventLoop::new("character_controller", 1280, 720);
    let mut app_state = AppState::new();

    // -- Player object (just a placeholder transform; collider does the work).
    let mut player = Object::new(Some("player".into()));
    player.transform.set_position([0.0, 5.0, 0.0]);
    player.set_collider(Collider::Capsule {
        a: Vector3::new(0.0, 0.5, 0.0),
        b: Vector3::new(0.0, 1.5, 0.0),
        radius: 0.4,
    });
    let player_id = player.get_unique_id();
    app_state.add_object(player);

    // -- Static box obstacles.
    for (i, x) in [-3.0, 0.0, 3.0].iter().enumerate() {
        let mut box_obj = Object::new(Some(format!("box_{}", i)));
        box_obj.transform.set_position([*x, 0.5, 4.0]);
        box_obj.set_collider(Collider::Aabb(
            BoundingBox::new([-1.0, -0.5, -1.0], [1.0, 0.5, 1.0])
        ));
        app_state.add_object(box_obj);
    }

    // -- Ground plane via Plane collider.
    let mut ground = Object::new(Some("ground".into()));
    ground.set_collider(Collider::Plane {
        normal: Vector3::new(0.0, 1.0, 0.0),
        offset: 0.0,
    });
    app_state.add_object(ground);

    // -- Character state.
    let cs = CharacterState::new(player_id);
    app_state.add_state_data(CHARACTER_STATE_KEY, Box::new(cs));

    // -- Camera (existing flycam to start — replace with follow-cam in a real game).
    // (Use the same camera setup pattern as `examples/engine.rs`.)

    // -- Bind events.
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::W),
        Arc::new(character_move_forward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::S),
        Arc::new(character_move_backward), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::A),
        Arc::new(character_move_left), None);
    app_state.inject_event(EventCharacteristic::KeyDown(VirtualKeyCode::D),
        Arc::new(character_move_right), None);
    app_state.inject_event(EventCharacteristic::KeyPress(VirtualKeyCode::Space),
        Arc::new(character_jump), None);

    // -- Per-frame update for physics.
    app_state.inject_update_function(Arc::new(character_update));

    event_loop.run(app_state.convert_to_arc_mutex());
}
```

If `AppState::new()` requires a `Display`, mirror the pattern from `examples/engine.rs` (it has the full bootstrapping). The intent is "minimal viable demo of the new API"; adapt to whatever the existing example bootstrapping requires.

- [ ] **Step 11.2: Build the example**

Run: `cargo build --example=character_controller --features=examples`
Expected: Clean build.

- [ ] **Step 11.3: Manually verify by running**

Run: `cargo run --example=character_controller --features=examples`
Expected: A window with a player capsule that:
- Stands on the ground (gravity + plane collision works).
- Moves with WASD relative to camera direction.
- Jumps with space and falls back down.
- Collides with the box obstacles (slides along them, doesn't tunnel through).

Take a quick spin to confirm. If any of those fail, the test for the relevant subsystem should catch it (re-run `cargo test`), but movement feel is hard to unit-test — manual smoke is the verification.

- [ ] **Step 11.4: Commit**

```bash
git add examples/character_controller.rs
git commit -m "feat(examples): character_controller demo using new collision API"
```

---

## Task 12: Final sweep — full test run, doc check, polish

- [ ] **Step 12.1: Run the full test suite**

Run: `cargo test`
Expected: All tests pass — preexisting and new.

- [ ] **Step 12.2: Run `cargo build` with all features**

Run: `cargo build --features=examples`
Expected: Clean build, no warnings introduced by this work.

- [ ] **Step 12.3: Run all examples briefly**

Run: `cargo run --example=engine --features=examples` (just verify it boots)
Run: `cargo run --example=skinned-mesh --features=examples` (verify still works)
Run: `cargo run --example=character_controller --features=examples`

Expected: No regressions in existing examples (the new `Object.collider: Option<Collider>` field has a `None` default so existing code is untouched).

- [ ] **Step 12.4: Update `CLAUDE.md` with the new collision module section**

Append a `### Collision system` section to `CLAUDE.md` near the existing rendering/skeletal sections. Brief — 8–12 lines summarizing the public types (`Collider`, `Hit`, `RayCast`, `ShapeCast`, `ShapeOverlap`, `MeshCollider`), the static-vs-static convention, the iterative-sweep approach, and the non-goals deferred from the spec.

- [ ] **Step 12.5: Final commit**

```bash
git add CLAUDE.md
git commit -m "docs: collision system section in CLAUDE.md"
```

---

## What's not in this plan (deferred follow-ups)

These were explicit non-goals in the spec but are worth noting here for the engineer:

- Push events / collision callbacks — pull queries only.
- Analytic swept TOI per pair — iterative sweep is the v1 implementation.
- Broadphase acceleration (BVH/octree) — linear scan over all objects.
- Mesh-vs-mesh, mesh-vs-heightfield, heightfield-vs-heightfield — return `None` with warning log.
- Collision layers / masks — not added.
- Compound colliders — one collider per object.
- Rigid-body dynamics — kinematic only.

Each of these slots in behind the v1 API without breaking it. Add when a specific game asks.
