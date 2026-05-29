# Frustum culling — design

**Date:** 2026-05-29
**Status:** Approved for implementation planning
**Scope:** Add per-object AABB frustum culling to the main render loop. Each frame, compute the camera's view frustum and skip rendering objects fully outside it.

## Motivation

The render loop currently draws every object in `app_state.objects` every frame regardless of whether the camera can see it. As scenes grow — especially with the new collision system enabling proper levels with many props — most objects are off-screen at any given moment. Frustum culling skips those draw calls. Big perf win for the cost of one AABB-vs-frustum test per object.

## Non-goals

Each can be added later behind the same API.

- **Hierarchical culling (BVH / octree).** Only matters once scenes have thousands of objects. v1 linear walk is fine.
- **Per-light shadow frustums.** A directional light's frustum differs from the camera's; for full correctness shadow passes should cull against the light's frustum. v1 culls the shadow pass against the *camera* frustum — less accurate (shadow casters outside the camera frustum get skipped, potentially missing shadows from off-screen geometry) but vastly simpler. Acceptable for v1 since the typical scene's shadow casters are also in-camera.
- **Per-submesh culling.** Single AABB per `Object`. Shapes within an object are not individually tested.
- **Particle culling.** Particle systems' bounds are fuzzy and change per frame. Always render.
- **Terrain per-tile culling.** Most scenes have ≤1 terrain. Always render the whole terrain.
- **Occlusion culling.** Not happening in v1.

## Data model

### `Frustum`

New module `src/frustum.rs`:

```rust
use nalgebra::Vector3;

pub struct Frustum {
    pub planes: [FrustumPlane; 6],
}

/// A frustum plane in the convention `normal · p + offset >= 0` for in-frustum points.
/// (Note: this is the same form as `Collider::Plane` but uses a different convention —
/// `normal · p >= -offset` rather than `normal · p == offset`. Documented inline.)
pub struct FrustumPlane {
    pub normal: Vector3<f32>,
    pub offset: f32,
}

impl Frustum {
    /// Extracts the 6 frustum planes from a view-projection matrix using Gribb/Hartmann.
    /// Order: left, right, bottom, top, near, far.
    pub fn from_view_projection(vp: &nalgebra::Matrix4<f32>) -> Self;

    /// Returns `true` if the AABB is fully or partially inside the frustum.
    /// Uses the standard "center + extents" projection test against each plane.
    pub fn contains_aabb(&self, aabb: &crate::geometry::BoundingBox) -> bool;
}
```

Plane extraction (Gribb/Hartmann):

```
left   = row4 + row1
right  = row4 - row1
bottom = row4 + row2
top    = row4 - row2
near   = row4 + row3
far    = row4 - row3
```

Each row is normalized: divide by `normal.norm()`. After normalization, the plane equation is `n · p + offset = signed_distance`; a point is inside if `signed_distance >= 0` for all six planes.

AABB test: for each plane, compute the "positive vertex" (the AABB corner most along the plane normal) by projecting half-extents onto `normal.abs()`. If `n · center + offset + radius < 0`, the AABB is fully outside that plane → return `false`. Otherwise (in or straddling) move on. If no plane rejects, return `true`.

### `Object.visible`

`Object` gains:

```rust
pub visible: bool,
```

Defaults to `true` in `Object::new` (so objects render until the cull pass updates them, which is correct on the first frame before the pass runs).

The field is `pub` so the cull pass can write it directly without an accessor — keeps the cull-pass code small. Users who don't run the cull pass (e.g., not calling `cull_objects`) still see all objects render because `visible` defaults to `true`.

### `CullStats`

```rust
pub struct CullStats {
    pub total: usize,
    pub visible: usize,
}
```

Stored on `AppState` as `pub cull_stats: CullStats`. Reset at the start of each cull pass.

## Cull pass

New function in `src/lib.rs` (or in `frustum.rs` if cleaner — the engineer picks during implementation):

```rust
/// Run frustum culling against the active camera. Updates `obj.visible` on every object
/// and writes counts to `app_state.cull_stats`. Called automatically once per frame inside
/// `EventLoop::run` if a camera is set.
pub fn cull_objects(app_state: &mut AppState) {
    let Some(camera) = app_state.camera.as_ref() else { return; };
    let vp = camera.get_view_projection_matrix();
    let frustum = Frustum::from_view_projection(&vp);
    let mut visible_count = 0;
    let total = app_state.objects.len();
    for obj in app_state.objects.iter_mut() {
        let aabb = obj.get_bounding_box();
        obj.visible = frustum.contains_aabb(&aabb);
        if obj.visible { visible_count += 1; }
    }
    app_state.cull_stats = CullStats { total, visible: visible_count };
}
```

`Camera::get_view_projection_matrix()` already exists or is trivially composed from `get_view_matrix()` and `get_projection_matrix()` — the engineer confirms during implementation.

## Render-loop integration

Inside `EventLoop::run`, **before** the opaque/transparent/shadow passes:

```rust
cull_objects(&mut app_state);
```

Each render pass that iterates objects adds an early-continue:

```rust
for object in app_state.objects.iter() {
    if !object.visible { continue; }
    // ... existing draw code
}
```

Passes affected:
- **Opaque pass** — cull
- **Transparent pass** — cull
- **Shadow depth pass (per-light)** — cull against camera frustum (v1 simplification, see non-goals)
- **Skybox** — never cull (always renders, centered on camera)
- **Particle pass** — never cull (v1)
- **Terrain** — never cull (v1)

## Backward compatibility

- `Object.visible` is a new public field with `true` default. Existing examples that don't touch it continue to render all objects.
- `cull_objects` is called automatically inside `EventLoop::run`. Users who want to opt out can set `app_state.frustum_culling_enabled = false` (new `pub` field on `AppState`, default `true`). If disabled, `cull_objects` returns early without touching `visible`.
- `CullStats` is purely informational; ignoring it has no effect.
- No existing test breaks. New tests cover plane extraction math + AABB-in/AABB-out cases.

## Implementation order

Suggested for the planning phase:

1. `Frustum` type + `from_view_projection` + tests against canonical projection matrices.
2. `contains_aabb` + tests for inside / outside / straddling cases.
3. `Object.visible` field + `Object::new` default.
4. `AppState.cull_stats` + `frustum_culling_enabled` toggle.
5. `cull_objects` function.
6. Wire into `EventLoop::run` before render passes.
7. Add `if !object.visible { continue; }` guards in opaque, transparent, and shadow passes.
8. Manual verification: stats counter shows reduced visible count when camera looks away from objects.

## Open questions for implementation

- Does `Camera` expose a `get_view_projection_matrix()`? If not, compose `proj * view` in `cull_objects`.
- Where exactly in `EventLoop::run` does the cull pass go? Before the opaque sort, presumably — confirm during implementation by reading the run loop.
- Should the shadow depth pass check `visible` per-light or use the same camera-culled set? v1: same camera-culled set, accepting the inaccuracy (see non-goals).
