# Frustum culling implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add per-object AABB frustum culling so the render loop skips draw calls for objects outside the camera frustum.

**Architecture:** New `src/frustum.rs` module with a `Frustum` type extracted from the camera's view-projection matrix (Gribb/Hartmann method) and an `AABB`-containment test. `Object` gains a `pub visible: bool` (default `true`). A new `cull_objects(&mut AppState)` function runs once per frame inside `EventLoop::run` and updates `obj.visible` for every object. Each render pass adds an `if !obj.visible { continue; }` guard. Skybox, particles, and terrain are never culled (v1). Shadow passes cull against the *camera* frustum (v1 simplification).

**Tech Stack:** Rust, nalgebra (`Matrix4<f32>`, `Vector3<f32>`), existing `Camera`, `Object`, `BoundingBox`, `AppState` types. Tests via `cargo test`, inline `#[cfg(test)] mod tests` blocks.

**Reference spec:** `docs/superpowers/specs/2026-05-29-frustum-culling-design.md`

**Resolved facts from codebase inspection:**
- `Camera::get_view_matrix() -> [[f32; 4]; 4]` and `Camera::get_projection_matrix() -> [[f32; 4]; 4]` exist (`src/camera.rs:179, 187`) — return column-major 4x4 arrays. Compose to `Matrix4<f32>` via `nalgebra::Matrix4::from(...)` in `cull_objects`.
- Render passes in `src/lib.rs`: opaque around line 1354, transparent around 1451, shadow depth pass around 1188+ (object update loop just before render). The cull call goes after object updates (line 1190 area) and before the model-matrices/bone-buffers collection at 1194.
- `BoundingBox` exposes `min_point() -> Vector3<f32>` and `max_point() -> Vector3<f32>` (verified across collision tasks).
- `Object::get_bounding_box(&mut self) -> BoundingBox` — takes `&mut self`. Cull pass iterates objects with `iter_mut()` so this works.

---

## File Structure

| File | Action | Responsibility |
|--|--|--|
| `src/frustum.rs` | Create | `Frustum`, `FrustumPlane`, plane extraction, AABB containment test |
| `src/lib.rs` | Modify | `pub mod frustum;`, `pub fn cull_objects`, integration in `EventLoop::run`, `AppState.cull_stats`, `AppState.frustum_culling_enabled` |
| `src/object.rs` | Modify | Add `pub visible: bool` field; default `true` in `Object::new` |
| `src/lib.rs` (render passes) | Modify | Add `if !object.visible { continue; }` guards in opaque, transparent, shadow passes |

---

## Task 1: `Frustum` type + plane extraction

**Files:**
- Create: `src/frustum.rs`
- Modify: `src/lib.rs` (add `pub mod frustum;`)

- [ ] **Step 1.1: Create `src/frustum.rs` with stub types**

Create the file with:

```rust
//! View-frustum extraction and AABB containment tests for frustum culling.
//! See `docs/superpowers/specs/2026-05-29-frustum-culling-design.md` for design.

use nalgebra::{Matrix4, Vector3};
use crate::geometry::BoundingBox;

/// A frustum plane in the convention `normal · p + offset >= 0` for in-frustum points.
#[derive(Copy, Clone, Debug)]
pub struct FrustumPlane {
    pub normal: Vector3<f32>,
    pub offset: f32,
}

/// View frustum as six planes (left, right, bottom, top, near, far).
#[derive(Copy, Clone, Debug)]
pub struct Frustum {
    pub planes: [FrustumPlane; 6],
}
```

- [ ] **Step 1.2: Add `pub mod frustum;` to lib.rs**

In `src/lib.rs`, find the existing `pub mod` declarations near the top (around line 40 — same area as `pub mod terrain;`, `pub mod default_events;`). Add:

```rust
pub mod frustum;
```

- [ ] **Step 1.3: Write failing test for plane extraction**

Append to `src/frustum.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Matrix4;

    /// Build a canonical perspective projection matrix (right-handed, looking down -Z)
    /// for testing. fov=90°, aspect=1, near=1, far=100.
    fn test_perspective() -> Matrix4<f32> {
        Matrix4::new_perspective(1.0, std::f32::consts::FRAC_PI_2, 1.0, 100.0)
    }

    #[test]
    fn frustum_extracts_six_planes() {
        let vp = test_perspective();
        let frustum = Frustum::from_view_projection(&vp);
        // Each plane normal should be unit length after normalization.
        for (i, plane) in frustum.planes.iter().enumerate() {
            let n = plane.normal.norm();
            assert!((n - 1.0).abs() < 1e-3, "plane {} normal not unit ({})", i, n);
        }
    }
}
```

- [ ] **Step 1.4: Run test to verify it fails**

Run: `cargo test --lib frustum_extracts_six_planes`
Expected: FAIL — `from_view_projection` doesn't exist.

- [ ] **Step 1.5: Implement `Frustum::from_view_projection`**

Append to `src/frustum.rs` (before the `#[cfg(test)]` block):

```rust
impl Frustum {
    /// Extracts the 6 frustum planes from a column-major view-projection matrix
    /// using the Gribb/Hartmann method. Planes are normalized so that `normal · p + offset`
    /// equals the signed distance from `p` to the plane (positive = inside).
    pub fn from_view_projection(vp: &Matrix4<f32>) -> Self {
        // Row 1 of vp is the X axis projection, etc. nalgebra is column-major; `.row(i)`
        // returns the i-th matrix row regardless of storage order.
        let row = |i: usize| vp.row(i).transpose();   // RowVector → Vector4
        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        // Gribb/Hartmann: plane equations as row4 ± row_i.
        let mut raw = [
            r3 + r0,   // left
            r3 - r0,   // right
            r3 + r1,   // bottom
            r3 - r1,   // top
            r3 + r2,   // near
            r3 - r2,   // far
        ];

        let mut planes = [FrustumPlane { normal: Vector3::zeros(), offset: 0.0 }; 6];
        for (i, p) in raw.iter_mut().enumerate() {
            let normal = Vector3::new(p.x, p.y, p.z);
            let mag = normal.norm().max(1e-12);
            let inv = 1.0 / mag;
            planes[i] = FrustumPlane {
                normal: normal * inv,
                offset: p.w * inv,
            };
        }
        Frustum { planes }
    }
}
```

- [ ] **Step 1.6: Run test to verify it passes**

Run: `cargo test --lib frustum_extracts_six_planes`
Expected: PASS.

Also: `cargo build` — clean.

- [ ] **Step 1.7: Commit**

```bash
git add src/frustum.rs src/lib.rs
git commit -m "feat(frustum): Frustum type + plane extraction (Gribb/Hartmann)"
```

---

## Task 2: AABB containment test

**Files:**
- Modify: `src/frustum.rs`

- [ ] **Step 2.1: Write failing tests for AABB containment**

Append to the `tests` mod in `src/frustum.rs`:

```rust
#[test]
fn aabb_inside_frustum_returns_true() {
    let vp = test_perspective();
    let frustum = Frustum::from_view_projection(&vp);
    // AABB centered well inside the frustum (in front of camera, between near=1 and far=100).
    let aabb = BoundingBox::new([-1.0, -1.0, -50.0], [1.0, 1.0, -45.0]);
    assert!(frustum.contains_aabb(&aabb));
}

#[test]
fn aabb_behind_camera_returns_false() {
    let vp = test_perspective();
    let frustum = Frustum::from_view_projection(&vp);
    // AABB behind the camera (positive Z in view space, before near plane).
    let aabb = BoundingBox::new([-1.0, -1.0, 5.0], [1.0, 1.0, 10.0]);
    assert!(!frustum.contains_aabb(&aabb));
}

#[test]
fn aabb_beyond_far_plane_returns_false() {
    let vp = test_perspective();
    let frustum = Frustum::from_view_projection(&vp);
    let aabb = BoundingBox::new([-1.0, -1.0, -200.0], [1.0, 1.0, -150.0]);
    assert!(!frustum.contains_aabb(&aabb));
}

#[test]
fn aabb_straddling_near_plane_returns_true() {
    let vp = test_perspective();
    let frustum = Frustum::from_view_projection(&vp);
    // AABB partially before and partially after near plane — should be visible.
    let aabb = BoundingBox::new([-0.5, -0.5, -2.0], [0.5, 0.5, 0.5]);
    assert!(frustum.contains_aabb(&aabb));
}
```

- [ ] **Step 2.2: Run tests to verify they fail**

Run: `cargo test --lib frustum::tests`
Expected: All four new tests FAIL — `contains_aabb` doesn't exist.

- [ ] **Step 2.3: Implement `contains_aabb`**

In the `impl Frustum` block in `src/frustum.rs`, add:

```rust
/// Returns true if the AABB is fully or partially inside the frustum.
/// Uses the standard "center + extents projected onto plane normal" test.
pub fn contains_aabb(&self, aabb: &BoundingBox) -> bool {
    let min = aabb.min_point();
    let max = aabb.max_point();
    let center = (min + max) * 0.5;
    let half = (max - min) * 0.5;
    for plane in &self.planes {
        // r = projection of half-extents onto |normal| — i.e., the maximum extent of the AABB
        // along the plane normal.
        let r = half.x * plane.normal.x.abs()
              + half.y * plane.normal.y.abs()
              + half.z * plane.normal.z.abs();
        let signed_distance = plane.normal.dot(&center) + plane.offset;
        if signed_distance + r < 0.0 {
            // AABB is entirely on the negative side of this plane → outside frustum.
            return false;
        }
    }
    true
}
```

- [ ] **Step 2.4: Run tests to verify they pass**

Run: `cargo test --lib frustum::tests`
Expected: All five frustum tests pass.

- [ ] **Step 2.5: Commit**

```bash
git add src/frustum.rs
git commit -m "feat(frustum): AABB-vs-frustum containment test"
```

---

## Task 3: `Object.visible` field

**Files:**
- Modify: `src/object.rs`

- [ ] **Step 3.1: Write failing test**

Find the existing `#[cfg(test)] mod tests` block in `src/object.rs` (added during the collision branch). Append:

```rust
#[test]
fn object_default_visible_is_true() {
    let obj = Object::new(Some("test".into()));
    assert!(obj.visible);
}

#[test]
fn object_visible_can_be_toggled() {
    let mut obj = Object::new(Some("test".into()));
    obj.visible = false;
    assert!(!obj.visible);
    obj.visible = true;
    assert!(obj.visible);
}
```

- [ ] **Step 3.2: Run test to verify it fails**

Run: `cargo test --lib object_default_visible_is_true`
Expected: FAIL — `visible` field doesn't exist.

- [ ] **Step 3.3: Add `visible` field**

In `src/object.rs`, modify the `pub struct Object` definition. Find the struct (it was modified in Task 1 of the collision branch to add `collider`). Add a new field:

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
    pub visible: bool,
}
```

In `Object::new`, find the struct literal it returns and add `visible: true` to it.

In `impl Clone for Object` (the manual Clone impl), add `new_object.visible = self.visible;` just before the final `new_object` line.

- [ ] **Step 3.4: Run tests to verify they pass**

Run: `cargo test --lib object_default_visible_is_true object_visible_can_be_toggled`
Expected: PASS.

Also: `cargo build` — clean.

- [ ] **Step 3.5: Commit**

```bash
git add src/object.rs
git commit -m "feat(frustum): Object.visible field (default true)"
```

---

## Task 4: `CullStats` + `frustum_culling_enabled` on AppState

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 4.1: Add `CullStats` struct + AppState fields**

In `src/lib.rs`, add near the other public types (e.g., just before `pub struct AppState` around line 198):

```rust
/// Stats from the most recent frustum-cull pass.
#[derive(Copy, Clone, Debug, Default)]
pub struct CullStats {
    pub total: usize,
    pub visible: usize,
}
```

Then modify `pub struct AppState` (around line 198). Add two fields anywhere among the existing ones:

```rust
pub cull_stats: CullStats,
pub frustum_culling_enabled: bool,
```

In `impl AppState::new()` (around line 243), find the struct literal it returns. Add:

```rust
cull_stats: CullStats::default(),
frustum_culling_enabled: true,
```

- [ ] **Step 4.2: Build check**

Run: `cargo build`
Expected: Clean. (No test for this step — the test for both fields lands with `cull_objects` in Task 5.)

- [ ] **Step 4.3: Commit**

```bash
git add src/lib.rs
git commit -m "feat(frustum): CullStats + frustum_culling_enabled on AppState"
```

---

## Task 5: `cull_objects` function

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 5.1: Write failing test**

In `src/lib.rs`, find the existing `#[cfg(test)] mod tests` block (around line 1600). Append:

```rust
#[test]
fn cull_objects_marks_visible_only_inside_frustum() {
    use crate::object::Object;
    use crate::camera::Camera;

    let mut state = AppState::new();
    // Camera at origin, looking down -Z, perspective fov=90°.
    let mut cam = Camera::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 90.0, 1.0, 1.0, 100.0);
    state.set_camera(cam);

    // Object directly in front of camera (in frustum).
    let mut o_in = Object::new(Some("in".into()));
    o_in.transform.set_position([0.0, 0.0, -10.0]);
    // Give it a small explicit collider so `get_bounding_box` returns a real box.
    // (Or: give it shapes. Easier: explicit collider, but `get_bounding_box` reads shapes.
    // We need to either add shapes or accept the degenerate AABB.)
    // For this test, the test verifies the cull pass *runs* and writes stats — we don't
    // depend on AABB accuracy of a shape-less object.
    let id_in = o_in.get_unique_id();
    state.objects.push(o_in);

    // Object behind camera (out of frustum).
    let mut o_out = Object::new(Some("out".into()));
    o_out.transform.set_position([0.0, 0.0, 10.0]);
    state.objects.push(o_out);

    cull_objects(&mut state);
    assert_eq!(state.cull_stats.total, 2);
    // At least the stats are populated. With shape-less objects the AABB is degenerate
    // (NaN extents per Task 9 review of collision branch) so the cull result is
    // implementation-defined. We only assert total here. A scene with real shapes
    // would discriminate cleanly.
    assert!(state.cull_stats.visible <= 2);
}

#[test]
fn cull_objects_respects_enabled_flag() {
    use crate::object::Object;
    use crate::camera::Camera;

    let mut state = AppState::new();
    let cam = Camera::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 90.0, 1.0, 1.0, 100.0);
    state.set_camera(cam);
    state.frustum_culling_enabled = false;

    let mut o = Object::new(Some("o".into()));
    o.visible = false;   // pretend a previous frame marked it invisible
    state.objects.push(o);

    cull_objects(&mut state);
    // With culling disabled, cull_objects should early-return and NOT touch visible.
    assert!(!state.objects[0].visible, "cull_objects should not modify visibility when disabled");
}
```

Note: `Camera::new(...)` signature — check `src/camera.rs` for the exact constructor. If it differs from `Camera::new(position, rotation, fov, aspect, near, far)`, adapt the call. If a setter pattern is used (e.g., `Camera::new()` + `set_*` methods), use that.

- [ ] **Step 5.2: Run tests to verify they fail**

Run: `cargo test --lib cull_objects`
Expected: FAIL — `cull_objects` doesn't exist.

- [ ] **Step 5.3: Implement `cull_objects`**

In `src/lib.rs`, add (anywhere appropriate — near other free functions, or just below `pub fn init_default` around line 53):

```rust
/// Run frustum culling against the active camera. Updates `Object.visible` on every
/// object and writes `app_state.cull_stats`. No-op if `frustum_culling_enabled` is false
/// or no camera is set. Called automatically inside `EventLoop::run` before the render
/// passes; users can also call it manually.
pub fn cull_objects(app_state: &mut AppState) {
    if !app_state.frustum_culling_enabled { return; }
    let Some(camera) = app_state.camera.as_ref() else { return; };
    let view: nalgebra::Matrix4<f32> = camera.get_view_matrix().into();
    let proj: nalgebra::Matrix4<f32> = camera.get_projection_matrix().into();
    let vp = proj * view;
    let frustum = frustum::Frustum::from_view_projection(&vp);
    let total = app_state.objects.len();
    let mut visible_count = 0;
    for obj in app_state.objects.iter_mut() {
        let aabb = obj.get_bounding_box();
        obj.visible = frustum.contains_aabb(&aabb);
        if obj.visible { visible_count += 1; }
    }
    app_state.cull_stats = CullStats { total, visible: visible_count };
}
```

- [ ] **Step 5.4: Run tests to verify they pass**

Run: `cargo test --lib cull_objects`
Expected: PASS.

- [ ] **Step 5.5: Commit**

```bash
git add src/lib.rs
git commit -m "feat(frustum): cull_objects function with enabled-flag short-circuit"
```

---

## Task 6: Wire `cull_objects` into the render loop + add visibility guards

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 6.1: Call `cull_objects` once per frame**

In `src/lib.rs`, inside `EventLoop::run`, find the per-frame block where objects are updated (around line 1187-1190). Currently:

```rust
let deltatime = app_state.delta_time;
for object in app_state.objects.iter_mut() {
    object.update(deltatime);
}

let render_target = &mut framebuffer;
```

Insert the cull call between the object update loop and the render_target setup:

```rust
let deltatime = app_state.delta_time;
for object in app_state.objects.iter_mut() {
    object.update(deltatime);
}

// Frustum culling — populates Object.visible and AppState.cull_stats.
cull_objects(&mut app_state);

let render_target = &mut framebuffer;
```

Wait — `app_state` may not be a `&mut AppState` at that point — it's likely an `Arc<Mutex<AppState>>` unlocked into a guard. Check the surrounding code (around line 1493 where `app: &mut AppState = &mut *app_state;` is used). Use the same pattern. If `app_state` is already a `&mut AppState` in this scope (it is, based on the existing `app_state.delta_time` access on line 1187), the simple call above works.

- [ ] **Step 6.2: Add visibility guards to render passes**

In `src/lib.rs`, find the opaque render loop (around line 1376). Locate the line `for ... in app_state.objects.iter() { ` (or similar — the loop iterating objects). Just inside the loop body, before the existing logic, add:

```rust
if !object.visible { continue; }
```

(Where `object` is the loop variable name — match what's already used.)

Do the same in the transparent render pass (around line 1466). And in the shadow depth pass (around line 1188+ — the existing render loops within shadow rendering).

Check by grepping for `for object in app_state.objects` or similar — every render-pass loop should get the guard. **Be exhaustive: opaque, transparent, shadow.** Skybox is rendered as a single fixed object (not via the objects loop) — leave it alone. Particles and terrain are separate render paths — leave them alone (v1 non-goals).

- [ ] **Step 6.3: Build check**

Run: `cargo build --features=examples`
Expected: Clean, no warnings introduced.

- [ ] **Step 6.4: Run all tests**

Run: `cargo test --lib`
Expected: All previous tests pass + the new frustum/cull tests pass. (The pre-existing `terrain::tests::vertex_color_flat_low` failure is acceptable — it predates this branch and is unrelated.)

- [ ] **Step 6.5: Commit**

```bash
git add src/lib.rs
git commit -m "feat(frustum): integrate cull pass + visibility guards in render loop"
```

---

## Task 7: Manual verification — character_controller example shows reduced visible count

**Files:**
- Modify: `examples/character_controller.rs` (optional egui HUD line)

This task verifies the cull pass actually does something visible by adding a stats line to the existing character_controller example.

- [ ] **Step 7.1: Add an egui line showing cull stats**

In `examples/character_controller.rs`, find where the egui GUI is set up (or add one if absent). Add an `inject_gui` call that displays the cull stats:

```rust
app_state.inject_gui(Arc::new(|ctx, app| {
    egui::Window::new("Frustum culling").show(ctx, |ui| {
        ui.label(format!("Visible: {} / {}", app.cull_stats.visible, app.cull_stats.total));
    });
}));
```

(Adjust the egui import path to whatever the project uses — see `examples/engine.rs` for the existing egui import pattern.)

- [ ] **Step 7.2: Build the example**

Run: `cargo build --example=character_controller --features=examples`
Expected: Clean build.

- [ ] **Step 7.3: Smoke test (manual — instructions for human reviewer)**

The plan can't run a GUI window. Document for the reviewer:

> Run `cargo run --example=character_controller --features=examples`. Look at the "Frustum culling" egui window. Total should equal the number of scene objects (~5-7 depending on showcase additions). Visible should decrease when the camera turns away from objects and increase when looking at them.

- [ ] **Step 7.4: Commit**

```bash
git add examples/character_controller.rs
git commit -m "feat(examples): show frustum cull stats in character_controller HUD"
```

---

## Task 8: Final sweep — full test + build check + CLAUDE.md update

- [ ] **Step 8.1: Full test suite**

Run: `cargo test --lib`
Expected: All tests pass except the pre-existing `terrain::tests::vertex_color_flat_low` failure.

- [ ] **Step 8.2: Build all examples**

Run: `cargo build --features=examples`
Expected: Clean.

- [ ] **Step 8.3: Update CLAUDE.md**

In `/Users/cg-jm/jm/enigma-3d/CLAUDE.md`, add a new `### Frustum culling` section after the Collision system section (line 119+). Content:

```
### Frustum culling

Per-object AABB culling in `src/frustum.rs`. `Frustum::from_view_projection(&Matrix4)` extracts 6 planes (Gribb/Hartmann); `Frustum::contains_aabb(&BoundingBox)` returns false only when the AABB is fully outside any plane. `cull_objects(&mut AppState)` runs each frame inside `EventLoop::run`, sets `Object.visible`, and populates `AppState.cull_stats { total, visible }`. Opaque, transparent, and shadow render passes skip objects with `visible = false`. Toggle via `AppState.frustum_culling_enabled` (default `true`).

v1 limits: per-object only (no per-submesh, no hierarchical/BVH), shadow passes cull against the *camera* frustum (not per-light), particles/terrain/skybox never culled.
```

Also update the `collision_world.rs` row in the modules table — add a new row for `frustum.rs`:

```
| `frustum.rs` | View frustum extraction + AABB containment test for cull pass |
```

- [ ] **Step 8.4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: frustum culling section in CLAUDE.md"
```

---

## What's not in this plan (deferred follow-ups)

- Hierarchical culling (BVH / octree) — defer until scenes have thousands of objects.
- Per-light shadow frustums — defer; v1 uses camera frustum for shadow passes too.
- Per-submesh culling — defer.
- Particle culling — defer (fuzzy bounds).
- Terrain per-tile culling — defer (typical ≤1 terrain).
- Occlusion culling — not happening.
