# Collision system extensions — design

**Date:** 2026-05-29
**Status:** Approved for implementation planning
**Scope:** Extend `collision_world` with shape primitives, swept queries, mesh + terrain colliders. Build a kinematic character controller on top, living in `default_events`.

## Motivation

`collision_world` today supports two operations: a ray cast against object AABBs, and an `AABB↔AABB` overlap test. That is enough for object picking and crude trigger volumes. It is not enough to build a game.

Two specific gaps drive this work:

1. The world is described by visual meshes and a heightfield terrain, but neither can be queried against — only their AABBs can. A player walking on terrain falls through it.
2. There is no shape-vs-world swept query, so any movement-against-the-world (character walk, projectile trace, physics-like response) has no foundation.

This spec extends `collision_world` with the spatial primitives needed to ask the world geometric questions, and then layers a *kinematic character controller* on top using `default_events`, so games can hook up player movement with a few `inject_event` calls.

## Non-goals

The following are explicitly out of scope for v1. Each can be added later behind the same public API.

- **Mesh-vs-mesh contact, mesh-vs-heightfield, heightfield-vs-heightfield.** Hard math (GJK / SAT / contact manifold extraction). Avoided by the static-world convention: meshes and heightfields are *queried against* but do not move and collide with each other. Dynamic things are primitives.
- **Push events / callbacks** (`on_enter`, `on_exit`, `on_stay`). Pull-only v1 — game code calls `ShapeOverlap::query` each frame when it wants to know who is inside a volume.
- **Continuous collision detection.** `ShapeCast` finds the first hit along a sweep, but does not rewind dynamic movers to their TOI. Each mover is responsible for not tunneling.
- **Broadphase acceleration.** Linear scan over all objects. Honest for current scene sizes. When it stops being honest, a BVH slots in behind the same query API.
- **Compound colliders.** One collider per object in v1.
- **Rigid-body dynamics.** Forces, mass, angular momentum, restitution, friction. Not happening.
- **Collision layers / masks.** Trivial to add later as a `u32` bitmask on `Collider`; not in v1.

## Data model

### `Collider` enum

A new public type in `collision_world.rs`:

```rust
pub enum Collider {
    Aabb(BoundingBox),
    Sphere   { center: Vector3<f32>, radius: f32 },
    Capsule  { a: Vector3<f32>, b: Vector3<f32>, radius: f32 },
    Obb      { center: Vector3<f32>, half_extents: Vector3<f32>, rotation: UnitQuaternion<f32> },
    Plane    { normal: Vector3<f32>, offset: f32 },           // n·p = offset
    Mesh(MeshCollider),                                        // static triangle list
    Heightfield(usize),                                        // index into AppState.terrains
}

pub struct MeshCollider {
    pub triangles: Vec<[Vector3<f32>; 3]>,
}

impl MeshCollider {
    pub fn from_object_mesh(object: &Object) -> Self;
    pub fn from_triangles(tris: Vec<[Vector3<f32>; 3]>) -> Self;
}
```

### Space convention

This is the single most important convention in the design, and it is easy to get subtly wrong.

- **Colliders stored on `Object`** are in *local space* (object-relative). At query time, `collision_world` transforms them by the object's current transform.
- **Standalone shapes passed to a query function** are in *world space*. They are not attached to any object.

The pragmatic effect: a moving object's collider follows the object automatically — no manual sync per frame.

### `Object` integration

`Object` gains one field:

```rust
pub collider: Option<Collider>,
```

- `None` → fall back to the auto-derived mesh AABB (today's behavior, unchanged).
- `Some(...)` → use the explicit collider.

The existing `get_collision()` bool gate is preserved — it still toggles whether an object participates in collision at all.

### `Hit`

A single return type for all queries:

```rust
pub struct Hit {
    pub object_id: Uuid,
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub toi: f32,    // 0..1 for swept queries, 0 for overlaps
}
```

For terrain hits, `object_id` is `Uuid::nil()`; the caller can check for the nil UUID before treating the hit as an object hit.

## Query API

Three query types, all matching the existing `RayCast` struct-builder pattern.

### `RayCast` (existing, kept)

Already implemented against AABBs. Will be expanded internally to dispatch against the full collider set when an object has an explicit collider, but the public API does not change:

```rust
let mut ray = RayCast::new(origin, direction, length);
ray.cast(&app_state);
for (uuid, point) in ray.get_intersection_map() { ... }
```

### `ShapeCast` (new)

Sweep a shape along a direction. Returns the first hit, if any.

```rust
let cast = ShapeCast::new(
    Collider::Sphere { center: pos, radius: 0.5 },
    direction,
    distance,
);
if let Some(hit) = cast.cast(&app_state) {
    // hit.point, hit.normal, hit.toi, hit.object_id
}
```

### `ShapeOverlap` (new)

Find everything overlapping a shape at a single instant. No motion.

```rust
let overlap = ShapeOverlap::new(
    Collider::Sphere { center: pos, radius: 2.0 },
);
let hits: Vec<Hit> = overlap.query(&app_state);
```

### Pair-routine dispatch

Each query iterates `app_state.objects` (transforming each object's collider into world space) plus terrain. For each pair, it dispatches to a per-pair routine via a `match` on `(shape_a, shape_b)`. Coverage:

| | Aabb | Sphere | Capsule | Obb | Plane | Mesh | HField |
|--|--|--|--|--|--|--|--|
| Aabb | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Sphere | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Capsule | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Obb | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Plane | ✓ | ✓ | ✓ | ✓ | — | — | — |
| Mesh | ✓ | ✓ | ✓ | ✓ | — | ✗ | ✗ |
| HField | ✓ | ✓ | ✓ | ✓ | — | ✗ | ✗ |

`—` means "static-vs-static, no useful query" — plane, mesh, and heightfield are all "the world"; querying one against another rarely yields a usable answer. The caller should always have at least one dynamic primitive (sphere/capsule/AABB/OBB) as one side of the pair.
`✗` means "explicitly out of scope" — mesh-vs-mesh family. Hitting one of these returns `None` with an `EnigmaWarning::log` call so it's audible during development.

## Terrain integration

`Terrain` (in `terrain.rs`) gains the methods that the heightfield query routines need:

```rust
impl Terrain {
    pub fn height_at(&self, x: f32, z: f32) -> Option<f32>;
    pub fn normal_at(&self, x: f32, z: f32) -> Option<Vector3<f32>>;
    pub fn triangles_in_aabb(&self, aabb: &BoundingBox) -> Vec<[Vector3<f32>; 3]>;
}
```

This fills the `heightfield_query_fn` stub noted in `CLAUDE.md`'s particle system entry.

`AppState` already owns terrain(s). The exact field name will be determined during implementation — if a single `Option<Terrain>` exists today, the `Heightfield(usize)` collider variant becomes `Heightfield` (no index), and we revisit if a `terrains: Vec<Terrain>` is added later.

## Character controller (`default_events`)

A small set of injectable functions following the existing `camera_fly_*` pattern. State lives in `app_state.state_data` keyed by a known string so multiple controllers can exist.

```rust
pub struct CharacterState {
    pub capsule: Capsule,           // local collider, world position derived from transform
    pub velocity: Vector3<f32>,
    pub grounded: bool,
    pub move_speed: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
}

pub fn character_move_forward(app_state: &mut AppState);
pub fn character_move_backward(app_state: &mut AppState);
pub fn character_move_left(app_state: &mut AppState);
pub fn character_move_right(app_state: &mut AppState);
pub fn character_jump(app_state: &mut AppState);
pub fn character_update(app_state: &mut AppState);   // inject_update_function
```

`character_update` per frame:
1. Apply gravity to `velocity.y`.
2. `ShapeCast` the capsule along `velocity * dt`.
3. If hit, advance to `toi`, project remaining motion along the contact tangent (`v - (v·n) * n`), recast. Cap at 3 iterations to avoid infinite loops in corners.
4. Update `grounded` from whether a small downward `ShapeCast` hits with normal ≈ up.
5. Write the new world position back to the character `Object`'s transform.

The user wires this up the same way as `camera_fly_forward` today — a handful of `inject_event` calls plus one `inject_update_function`.

## Backwards compatibility

- `RayCast` public API unchanged.
- `is_colliding(aabb1, aabb2)` free function unchanged (still useful for trivial AABB checks).
- `MouseState` unchanged.
- `Object` gains a new field with a sane default (`None`). Existing code that doesn't set a collider continues to work via the mesh-AABB fallback.
- No example breaks. New example (`character_controller` or extension of `engine`) demonstrates the new path.

## Implementation order

Suggested order for the implementation plan (decided in the planning phase, not here — included as orientation):

1. `Collider` enum + shape math (no integration yet).
2. Per-pair overlap + cast routines, table-driven.
3. `ShapeOverlap` query.
4. `ShapeCast` query.
5. `Object.collider` integration; `RayCast` upgraded to dispatch against full collider set.
6. `Terrain` query methods; heightfield pair routines.
7. `MeshCollider::from_object_mesh`; mesh pair routines.
8. Character controller in `default_events`.
9. Example demonstrating the path.

## Open questions for implementation

- Does `AppState` currently hold a single terrain or multiple? `Heightfield` variant signature depends on this.
- The exact shape of `state_data` access (typed bag vs untyped) — the character controller's `CharacterState` lookup needs to match what's already there.
- Whether `Plane` colliders are reachable from any object's `collider` field, or only used as standalone query shapes. (Probably both, but worth confirming during impl — an infinite plane attached to a moving object is geometrically valid but probably never useful.)
