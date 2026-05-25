# Particle System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a CPU-simulated, JSON-authorable particle system to enigma-3d supporting sprite + ribbon rendering, sub-emitters, hero per-particle lights, soft particles, HDR/bloom integration, and ground/terrain collision — fully driven by the spec at `docs/superpowers/specs/2026-05-24-particle-system-design.md`.

**Architecture:** New `src/particle/` module with SoA particle buffers, Shuriken-style fixed update pipeline parallelized via rayon. Renders in a new pass between the existing transparent and post-processing passes. JSON I/O via serde. Material assignment + name-based lookup mirror the engine's `Object`/`Material` conventions.

**Tech Stack:** Rust 2021, glium 0.33 (OpenGL), serde + serde_json, rayon (already in deps), nalgebra-glm, uuid. New dev-dependency: `criterion`.

---

## Pre-flight

- [ ] **Verify baseline**: `cargo check` should succeed on a clean tree (it did at plan-write time). Run `cargo test --lib` to confirm zero failing tests before starting.

## Task 1 — Module scaffold + supporting numeric types

**Files:**
- Create: `src/particle/mod.rs`
- Create: `src/particle/types.rs`
- Modify: `src/lib.rs` (add `pub mod particle;`)

- [ ] **Step 1.1: Add module declaration**

In `src/lib.rs`, add `pub mod particle;` next to the other `pub mod` declarations (after `pub mod terrain;`):

```rust
pub mod terrain;
pub mod particle;
```

- [ ] **Step 1.2: Create `src/particle/mod.rs` with skeleton**

```rust
pub mod types;

pub use types::{Range, BurstSpec};
```

- [ ] **Step 1.3: Write failing tests for `Range` and `BurstSpec` round-trip**

Create `src/particle/types.rs` with only the tests at the bottom (no impl yet):

```rust
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_serde_roundtrip() {
        let r: Range<f32> = Range { min: 0.5, max: 2.5 };
        let json = serde_json::to_string(&r).unwrap();
        let back: Range<f32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.min, 0.5);
        assert_eq!(back.max, 2.5);
    }

    #[test]
    fn burst_serde_roundtrip() {
        let b = BurstSpec { time: 1.2, count: 8 };
        let json = serde_json::to_string(&b).unwrap();
        let back: BurstSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.time, 1.2);
        assert_eq!(back.count, 8);
    }
}
```

- [ ] **Step 1.4: Run tests, see them fail**

Run: `cargo test --lib particle::types`
Expected: compile error — `Range` and `BurstSpec` not defined.

- [ ] **Step 1.5: Implement `Range<T>` and `BurstSpec`**

Add above the `#[cfg(test)] mod tests` block in `src/particle/types.rs`:

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T: Copy> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Range { min, max }
    }
}

impl Range<f32> {
    pub fn sample(&self, rng: &mut impl rand::Rng) -> f32 {
        if self.min == self.max { self.min } else { rng.gen_range(self.min..=self.max) }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BurstSpec {
    pub time: f32,
    pub count: u32,
}
```

- [ ] **Step 1.6: Run tests, see them pass**

Run: `cargo test --lib particle::types`
Expected: 2 tests pass.

- [ ] **Step 1.7: Commit**

```bash
git add src/lib.rs src/particle/
git commit -m "feat(particle): module scaffold + Range and BurstSpec types"
```

---

## Task 2 — Curve and Gradient keyframe evaluators

**Files:**
- Create: `src/particle/curves.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 2.1: Failing tests for `Curve<f32>::eval`**

Create `src/particle/curves.rs`:

```rust
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_keyframe_exact() {
        let c = Curve::new(vec![(0.0, 1.0), (0.5, 3.0), (1.0, 2.0)]);
        assert_eq!(c.eval(0.0), 1.0);
        assert_eq!(c.eval(0.5), 3.0);
        assert_eq!(c.eval(1.0), 2.0);
    }

    #[test]
    fn curve_interpolation() {
        let c = Curve::new(vec![(0.0, 0.0), (1.0, 10.0)]);
        assert!((c.eval(0.25) - 2.5).abs() < 1e-5);
        assert!((c.eval(0.75) - 7.5).abs() < 1e-5);
    }

    #[test]
    fn curve_clamps_out_of_range() {
        let c = Curve::new(vec![(0.0, 5.0), (1.0, 10.0)]);
        assert_eq!(c.eval(-0.5), 5.0);
        assert_eq!(c.eval(1.5), 10.0);
    }

    #[test]
    fn curve_empty_returns_default() {
        let c: Curve<f32> = Curve::new(vec![]);
        assert_eq!(c.eval(0.5), 0.0);
    }

    #[test]
    fn gradient_vec4_midpoint() {
        let g = Gradient::new(vec![
            (0.0, [0.0, 0.0, 0.0, 0.0]),
            (1.0, [1.0, 1.0, 1.0, 1.0]),
        ]);
        let v = g.eval(0.5);
        assert!((v[0] - 0.5).abs() < 1e-5);
        assert!((v[3] - 0.5).abs() < 1e-5);
    }
}
```

- [ ] **Step 2.2: Run tests, see them fail (`Curve`/`Gradient` undefined)**

Run: `cargo test --lib particle::curves`

- [ ] **Step 2.3: Implement `Curve<f32>` and `Gradient<[f32; 4]>`**

Add above the `#[cfg(test)] mod tests` block:

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Curve<T: Copy> {
    pub keys: Vec<(f32, T)>,
}

impl<T: Copy + Default> Curve<T> {
    pub fn new(keys: Vec<(f32, T)>) -> Self {
        Curve { keys }
    }
}

impl Curve<f32> {
    pub fn eval(&self, t: f32) -> f32 {
        if self.keys.is_empty() { return 0.0; }
        if t <= self.keys[0].0 { return self.keys[0].1; }
        if t >= self.keys[self.keys.len() - 1].0 { return self.keys[self.keys.len() - 1].1; }
        for i in 0..self.keys.len() - 1 {
            let (t0, v0) = self.keys[i];
            let (t1, v1) = self.keys[i + 1];
            if t >= t0 && t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
                return v0 + (v1 - v0) * f;
            }
        }
        self.keys[self.keys.len() - 1].1
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Gradient<T: Copy> {
    pub keys: Vec<(f32, T)>,
}

impl<T: Copy + Default> Gradient<T> {
    pub fn new(keys: Vec<(f32, T)>) -> Self {
        Gradient { keys }
    }
}

impl Gradient<[f32; 4]> {
    pub fn eval(&self, t: f32) -> [f32; 4] {
        if self.keys.is_empty() { return [0.0; 4]; }
        if t <= self.keys[0].0 { return self.keys[0].1; }
        if t >= self.keys[self.keys.len() - 1].0 { return self.keys[self.keys.len() - 1].1; }
        for i in 0..self.keys.len() - 1 {
            let (t0, v0) = self.keys[i];
            let (t1, v1) = self.keys[i + 1];
            if t >= t0 && t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
                return [
                    v0[0] + (v1[0] - v0[0]) * f,
                    v0[1] + (v1[1] - v0[1]) * f,
                    v0[2] + (v1[2] - v0[2]) * f,
                    v0[3] + (v1[3] - v0[3]) * f,
                ];
            }
        }
        self.keys[self.keys.len() - 1].1
    }
}

/// Returns Err with description if keyframes are unsorted or t out of [0,1].
pub fn validate_keyframes<T: Copy>(keys: &[(f32, T)]) -> Result<(), String> {
    if keys.is_empty() { return Ok(()); }
    if keys[0].0 < 0.0 || keys[0].0 > 1.0 {
        return Err(format!("keyframe time {} out of [0,1]", keys[0].0));
    }
    for i in 1..keys.len() {
        if keys[i].0 < keys[i - 1].0 {
            return Err(format!("keyframes unsorted at index {}", i));
        }
        if keys[i].0 > 1.0 {
            return Err(format!("keyframe time {} out of [0,1]", keys[i].0));
        }
    }
    Ok(())
}
```

- [ ] **Step 2.4: Add `pub mod curves;` to `src/particle/mod.rs`**

```rust
pub mod types;
pub mod curves;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
```

- [ ] **Step 2.5: Run tests, see them pass**

Run: `cargo test --lib particle::curves`
Expected: 5 tests pass.

- [ ] **Step 2.6: Commit**

```bash
git add src/particle/curves.rs src/particle/mod.rs
git commit -m "feat(particle): Curve and Gradient keyframe evaluators"
```

---

## Task 3 — Emitter shapes + initial velocity + color range

**Files:**
- Create: `src/particle/emitter.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 3.1: Failing tests for emitter sampling**

Create `src/particle/emitter.rs`:

```rust
use serde::{Deserialize, Serialize};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use crate::particle::Range;

#[cfg(test)]
mod tests {
    use super::*;

    fn rng() -> SmallRng { SmallRng::seed_from_u64(42) }

    #[test]
    fn sphere_samples_within_radius() {
        let s = EmitterShape::Sphere { radius: 2.0 };
        let mut r = rng();
        for _ in 0..1000 {
            let p = s.sample_position(&mut r);
            let len = (p[0]*p[0] + p[1]*p[1] + p[2]*p[2]).sqrt();
            assert!(len <= 2.0 + 1e-5, "sphere sample exceeded radius: {:?} len={}", p, len);
        }
    }

    #[test]
    fn box_samples_within_extents() {
        let s = EmitterShape::Box { half_extents: [1.0, 2.0, 3.0] };
        let mut r = rng();
        for _ in 0..1000 {
            let p = s.sample_position(&mut r);
            assert!(p[0].abs() <= 1.0 + 1e-5);
            assert!(p[1].abs() <= 2.0 + 1e-5);
            assert!(p[2].abs() <= 3.0 + 1e-5);
        }
    }

    #[test]
    fn point_samples_origin() {
        let s = EmitterShape::Point {};
        let mut r = rng();
        let p = s.sample_position(&mut r);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn outward_velocity_zero_at_origin() {
        let v = InitialVelocity::Outward { speed: Range::new(1.0, 1.0) };
        let mut r = rng();
        // At origin, outward direction is undefined → fallback to up
        let result = v.sample(&[0.0, 0.0, 0.0], &mut r);
        let len = (result[0]*result[0] + result[1]*result[1] + result[2]*result[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn direction_velocity_aligned() {
        let v = InitialVelocity::Direction {
            direction: [0.0, 1.0, 0.0],
            speed: Range::new(2.0, 2.0),
        };
        let mut r = rng();
        let result = v.sample(&[5.0, 5.0, 5.0], &mut r);
        assert!((result[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn deterministic_with_same_seed() {
        let s = EmitterShape::Sphere { radius: 1.0 };
        let mut r1 = SmallRng::seed_from_u64(99);
        let mut r2 = SmallRng::seed_from_u64(99);
        for _ in 0..10 {
            assert_eq!(s.sample_position(&mut r1), s.sample_position(&mut r2));
        }
    }
}
```

- [ ] **Step 3.2: Run, see failures**

Run: `cargo test --lib particle::emitter`

- [ ] **Step 3.3: Implement `EmitterShape` and `InitialVelocity`**

Above the test module:

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EmitterShape {
    Point {},
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    Cone { angle: f32, height: f32 },
    Disk { radius: f32 },
}

impl EmitterShape {
    pub fn sample_position(&self, rng: &mut impl rand::Rng) -> [f32; 3] {
        match *self {
            EmitterShape::Point {} => [0.0, 0.0, 0.0],
            EmitterShape::Sphere { radius } => {
                // rejection sample inside unit sphere, then scale
                loop {
                    let x: f32 = rng.gen_range(-1.0..=1.0);
                    let y: f32 = rng.gen_range(-1.0..=1.0);
                    let z: f32 = rng.gen_range(-1.0..=1.0);
                    if x*x + y*y + z*z <= 1.0 {
                        return [x * radius, y * radius, z * radius];
                    }
                }
            }
            EmitterShape::Box { half_extents } => [
                rng.gen_range(-half_extents[0]..=half_extents[0]),
                rng.gen_range(-half_extents[1]..=half_extents[1]),
                rng.gen_range(-half_extents[2]..=half_extents[2]),
            ],
            EmitterShape::Cone { angle, height } => {
                let h: f32 = rng.gen_range(0.0..=height);
                let r_at_h = h * angle.tan();
                let theta: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                let r: f32 = (rng.gen::<f32>().sqrt()) * r_at_h;
                [r * theta.cos(), h, r * theta.sin()]
            }
            EmitterShape::Disk { radius } => {
                let theta: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                let r: f32 = (rng.gen::<f32>().sqrt()) * radius;
                [r * theta.cos(), 0.0, r * theta.sin()]
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InitialVelocity {
    Outward { speed: Range<f32> },
    Direction { direction: [f32; 3], speed: Range<f32> },
    Cone { direction: [f32; 3], angle: f32, speed: Range<f32> },
    Hemisphere { normal: [f32; 3], speed: Range<f32> },
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if len < 1e-6 { [0.0, 1.0, 0.0] } else { [v[0]/len, v[1]/len, v[2]/len] }
}

fn sample_cone_direction(axis: [f32; 3], angle: f32, rng: &mut impl rand::Rng) -> [f32; 3] {
    // pick a random direction within `angle` of axis
    let cos_a = angle.cos();
    let z: f32 = rng.gen_range(cos_a..=1.0);
    let phi: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
    let r = (1.0 - z * z).sqrt();
    // direction in local space where axis = +Z
    let local = [r * phi.cos(), r * phi.sin(), z];
    // build basis from axis
    let n = normalize(axis);
    let helper = if n[1].abs() < 0.99 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let t = normalize([
        helper[1] * n[2] - helper[2] * n[1],
        helper[2] * n[0] - helper[0] * n[2],
        helper[0] * n[1] - helper[1] * n[0],
    ]);
    let b = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    [
        t[0] * local[0] + b[0] * local[1] + n[0] * local[2],
        t[1] * local[0] + b[1] * local[1] + n[1] * local[2],
        t[2] * local[0] + b[2] * local[1] + n[2] * local[2],
    ]
}

impl InitialVelocity {
    pub fn sample(&self, spawn_pos: &[f32; 3], rng: &mut impl rand::Rng) -> [f32; 3] {
        match *self {
            InitialVelocity::Outward { speed } => {
                let dir = if spawn_pos[0].abs() + spawn_pos[1].abs() + spawn_pos[2].abs() < 1e-6 {
                    [0.0, 1.0, 0.0]
                } else {
                    normalize(*spawn_pos)
                };
                let s = speed.sample(rng);
                [dir[0] * s, dir[1] * s, dir[2] * s]
            }
            InitialVelocity::Direction { direction, speed } => {
                let d = normalize(direction);
                let s = speed.sample(rng);
                [d[0] * s, d[1] * s, d[2] * s]
            }
            InitialVelocity::Cone { direction, angle, speed } => {
                let d = sample_cone_direction(direction, angle, rng);
                let s = speed.sample(rng);
                [d[0] * s, d[1] * s, d[2] * s]
            }
            InitialVelocity::Hemisphere { normal, speed } => {
                let d = sample_cone_direction(normal, std::f32::consts::FRAC_PI_2, rng);
                let s = speed.sample(rng);
                [d[0] * s, d[1] * s, d[2] * s]
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ColorRange {
    Single([f32; 4]),
    Range { min: [f32; 4], max: [f32; 4] },
    Hdr { rgb: [f32; 3], intensity: f32, alpha: f32 },
    HdrRange { rgb_min: [f32; 3], rgb_max: [f32; 3], intensity: Range<f32>, alpha: Range<f32> },
}

impl ColorRange {
    pub fn sample(&self, rng: &mut impl rand::Rng) -> [f32; 4] {
        match *self {
            ColorRange::Single(c) => c,
            ColorRange::Range { min, max } => [
                rng.gen_range(min[0]..=max[0]),
                rng.gen_range(min[1]..=max[1]),
                rng.gen_range(min[2]..=max[2]),
                rng.gen_range(min[3]..=max[3]),
            ],
            ColorRange::Hdr { rgb, intensity, alpha } => [rgb[0]*intensity, rgb[1]*intensity, rgb[2]*intensity, alpha],
            ColorRange::HdrRange { rgb_min, rgb_max, intensity, alpha } => {
                let inten = intensity.sample(rng);
                [
                    rng.gen_range(rgb_min[0]..=rgb_max[0]) * inten,
                    rng.gen_range(rgb_min[1]..=rgb_max[1]) * inten,
                    rng.gen_range(rgb_min[2]..=rgb_max[2]) * inten,
                    alpha.sample(rng),
                ]
            }
        }
    }
}
```

- [ ] **Step 3.4: Wire into `mod.rs`**

```rust
pub mod types;
pub mod curves;
pub mod emitter;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
```

- [ ] **Step 3.5: Run, see pass**

`cargo test --lib particle::emitter` → 6 tests pass.

- [ ] **Step 3.6: Commit**

```bash
git add src/particle/emitter.rs src/particle/mod.rs
git commit -m "feat(particle): emitter shapes, initial velocity, color range"
```

---

## Task 4 — Forces + collision queries

**Files:**
- Create: `src/particle/forces.rs`
- Create: `src/particle/collision.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 4.1: Failing tests in `forces.rs`**

Create `src/particle/forces.rs`:

```rust
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_adds_acceleration() {
        let g = [0.0, -9.81, 0.0];
        let a = gravity_acceleration(g);
        assert_eq!(a, [0.0, -9.81, 0.0]);
    }

    #[test]
    fn drag_decays_exponentially() {
        let mut v = [10.0, 0.0, 0.0];
        let drag = 2.0;
        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            apply_drag(&mut v, drag, dt);
        }
        let expected = 10.0 * (-2.0_f32).exp();
        assert!((v[0] - expected).abs() < 0.5, "drag decay: got {} expected {}", v[0], expected);
    }

    #[test]
    fn vortex_creates_perpendicular_force() {
        let v = VortexConfig { center: [0.0, 0.0, 0.0], axis: [0.0, 1.0, 0.0], strength: 5.0, falloff: 1.0 };
        let f = v.force(&[1.0, 0.0, 0.0]);
        // particle on +x with axis +y → tangent should be -z (right-hand) or +z; magnitude > 0
        assert!(f[0].abs() < 1e-5);
        assert!(f[2].abs() > 0.0);
    }

    #[test]
    fn attractor_points_toward_center() {
        let a = AttractorConfig { center: [0.0, 0.0, 0.0], strength: 3.0, falloff: 1.0 };
        let f = a.force(&[5.0, 0.0, 0.0]);
        assert!(f[0] < 0.0);
        assert_eq!(f[1], 0.0);
        assert_eq!(f[2], 0.0);
    }

    #[test]
    fn curl_noise_finite() {
        let c = CurlNoiseConfig { scale: 1.0, strength: 1.0, evolution_speed: 0.5 };
        let f = c.force(&[1.0, 2.0, 3.0], 0.5);
        for component in f.iter() {
            assert!(component.is_finite());
        }
    }
}
```

- [ ] **Step 4.2: Run failures**

`cargo test --lib particle::forces`

- [ ] **Step 4.3: Implement force functions**

Above test module:

```rust
pub fn gravity_acceleration(g: [f32; 3]) -> [f32; 3] { g }

pub fn apply_drag(velocity: &mut [f32; 3], drag: f32, dt: f32) {
    let factor = (-drag * dt).exp();
    velocity[0] *= factor;
    velocity[1] *= factor;
    velocity[2] *= factor;
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct VortexConfig {
    pub center: [f32; 3],
    pub axis: [f32; 3],
    pub strength: f32,
    pub falloff: f32,
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]]
}

fn length(v: [f32; 3]) -> f32 { (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt() }

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = length(v);
    if l < 1e-6 { [0.0, 0.0, 0.0] } else { [v[0]/l, v[1]/l, v[2]/l] }
}

impl VortexConfig {
    pub fn force(&self, pos: &[f32; 3]) -> [f32; 3] {
        let r = [pos[0]-self.center[0], pos[1]-self.center[1], pos[2]-self.center[2]];
        let dist = length(r);
        let dir = normalize(cross(self.axis, r));
        let mag = self.strength / (1.0 + self.falloff * dist);
        [dir[0]*mag, dir[1]*mag, dir[2]*mag]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct AttractorConfig {
    pub center: [f32; 3],
    pub strength: f32,
    pub falloff: f32,
}

impl AttractorConfig {
    pub fn force(&self, pos: &[f32; 3]) -> [f32; 3] {
        let to_center = [self.center[0]-pos[0], self.center[1]-pos[1], self.center[2]-pos[2]];
        let dist = length(to_center);
        if dist < 1e-6 { return [0.0; 3]; }
        let dir = normalize(to_center);
        let mag = self.strength / (1.0 + self.falloff * dist);
        [dir[0]*mag, dir[1]*mag, dir[2]*mag]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CurlNoiseConfig {
    pub scale: f32,
    pub strength: f32,
    pub evolution_speed: f32,
}

// Simple value-noise gradient, finite-difference curl.
// Not for visual perfection — just a swirly, divergence-free-ish vector field.
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ z.wrapping_mul(1274126177)) as u32;
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    (h as f32) / (u32::MAX as f32) * 2.0 - 1.0
}

fn smoothstep(t: f32) -> f32 { t * t * (3.0 - 2.0 * t) }

fn noise3(p: [f32; 3]) -> f32 {
    let xi = p[0].floor() as i32;
    let yi = p[1].floor() as i32;
    let zi = p[2].floor() as i32;
    let xf = smoothstep(p[0] - xi as f32);
    let yf = smoothstep(p[1] - yi as f32);
    let zf = smoothstep(p[2] - zi as f32);
    let c000 = hash3(xi, yi, zi);
    let c100 = hash3(xi+1, yi, zi);
    let c010 = hash3(xi, yi+1, zi);
    let c110 = hash3(xi+1, yi+1, zi);
    let c001 = hash3(xi, yi, zi+1);
    let c101 = hash3(xi+1, yi, zi+1);
    let c011 = hash3(xi, yi+1, zi+1);
    let c111 = hash3(xi+1, yi+1, zi+1);
    let x00 = c000 + (c100 - c000) * xf;
    let x10 = c010 + (c110 - c010) * xf;
    let x01 = c001 + (c101 - c001) * xf;
    let x11 = c011 + (c111 - c011) * xf;
    let y0 = x00 + (x10 - x00) * yf;
    let y1 = x01 + (x11 - x01) * yf;
    y0 + (y1 - y0) * zf
}

fn potential(p: [f32; 3]) -> [f32; 3] {
    [
        noise3([p[0], p[1], p[2]]),
        noise3([p[0] + 31.4, p[1] - 17.1, p[2] + 53.3]),
        noise3([p[0] - 47.7, p[1] + 73.5, p[2] - 19.9]),
    ]
}

impl CurlNoiseConfig {
    pub fn force(&self, pos: &[f32; 3], time: f32) -> [f32; 3] {
        let s = self.scale;
        let off = self.evolution_speed * time;
        let p = [pos[0] * s + off, pos[1] * s, pos[2] * s];
        let eps = 0.1_f32;
        let p_x_plus  = potential([p[0]+eps, p[1], p[2]]);
        let p_x_minus = potential([p[0]-eps, p[1], p[2]]);
        let p_y_plus  = potential([p[0], p[1]+eps, p[2]]);
        let p_y_minus = potential([p[0], p[1]-eps, p[2]]);
        let p_z_plus  = potential([p[0], p[1], p[2]+eps]);
        let p_z_minus = potential([p[0], p[1], p[2]-eps]);
        let dpz_dy = (p_z_plus[1] - p_z_minus[1]) / (2.0 * eps);
        let dpy_dz = (p_y_plus[2] - p_y_minus[2]) / (2.0 * eps);
        let dpx_dz = (p_x_plus[2] - p_x_minus[2]) / (2.0 * eps);
        let dpz_dx = (p_z_plus[0] - p_z_minus[0]) / (2.0 * eps);
        let dpy_dx = (p_y_plus[0] - p_y_minus[0]) / (2.0 * eps);
        let dpx_dy = (p_x_plus[1] - p_x_minus[1]) / (2.0 * eps);
        [
            (dpz_dy - dpy_dz) * self.strength,
            (dpx_dz - dpz_dx) * self.strength,
            (dpy_dx - dpx_dy) * self.strength,
        ]
    }
}
```

- [ ] **Step 4.4: Tests pass for forces**

`cargo test --lib particle::forces`

- [ ] **Step 4.5: Failing tests for collision**

Create `src/particle/collision.rs`:

```rust
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_plane_reflects_below() {
        let c = CollisionConfig { ground_plane_y: Some(0.0), terrain: false, bounce: 0.5, friction: 0.0 };
        let mut pos = [0.0, -0.1, 0.0];
        let mut vel = [1.0, -2.0, 0.0];
        let hit = c.test_and_resolve_ground(&mut pos, &mut vel);
        assert!(hit);
        assert!(pos[1] >= 0.0);
        assert!(vel[1] > 0.0);
        assert!((vel[1] - 1.0).abs() < 1e-5);  // bounce 0.5 of -2.0 = 1.0
    }

    #[test]
    fn ground_plane_passes_above() {
        let c = CollisionConfig { ground_plane_y: Some(0.0), terrain: false, bounce: 0.5, friction: 0.0 };
        let mut pos = [0.0, 1.0, 0.0];
        let mut vel = [1.0, -2.0, 0.0];
        let hit = c.test_and_resolve_ground(&mut pos, &mut vel);
        assert!(!hit);
        assert_eq!(vel, [1.0, -2.0, 0.0]);
    }

    #[test]
    fn ground_plane_friction_reduces_horizontal() {
        let c = CollisionConfig { ground_plane_y: Some(0.0), terrain: false, bounce: 0.5, friction: 0.5 };
        let mut pos = [0.0, -0.1, 0.0];
        let mut vel = [4.0, -2.0, 2.0];
        c.test_and_resolve_ground(&mut pos, &mut vel);
        assert!((vel[0] - 2.0).abs() < 1e-5);
        assert!((vel[2] - 1.0).abs() < 1e-5);
    }
}
```

- [ ] **Step 4.6: Implement `CollisionConfig`**

Above tests:

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CollisionConfig {
    pub ground_plane_y: Option<f32>,
    pub terrain: bool,
    pub bounce: f32,
    pub friction: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionEvent {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub normal: [f32; 3],
}

impl CollisionConfig {
    pub fn test_and_resolve_ground(&self, pos: &mut [f32; 3], vel: &mut [f32; 3]) -> bool {
        if let Some(y) = self.ground_plane_y {
            if pos[1] < y {
                pos[1] = y;
                if vel[1] < 0.0 { vel[1] = -vel[1] * self.bounce; }
                vel[0] *= 1.0 - self.friction;
                vel[2] *= 1.0 - self.friction;
                return true;
            }
        }
        false
    }

    /// Resolve against a terrain heightfield. Caller supplies the height query.
    /// Returns true if collision occurred.
    pub fn test_and_resolve_terrain(
        &self,
        pos: &mut [f32; 3],
        vel: &mut [f32; 3],
        height_at: impl Fn(f32, f32) -> Option<f32>,
    ) -> bool {
        if !self.terrain { return false; }
        if let Some(h) = height_at(pos[0], pos[2]) {
            if pos[1] < h {
                pos[1] = h;
                if vel[1] < 0.0 { vel[1] = -vel[1] * self.bounce; }
                vel[0] *= 1.0 - self.friction;
                vel[2] *= 1.0 - self.friction;
                return true;
            }
        }
        false
    }
}
```

- [ ] **Step 4.7: Wire into mod.rs**

```rust
pub mod types;
pub mod curves;
pub mod emitter;
pub mod forces;
pub mod collision;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
pub use forces::{VortexConfig, AttractorConfig, CurlNoiseConfig};
pub use collision::{CollisionConfig, CollisionEvent};
```

- [ ] **Step 4.8: Tests pass**

`cargo test --lib particle::` — all tests should pass.

- [ ] **Step 4.9: Commit**

```bash
git add src/particle/
git commit -m "feat(particle): forces (gravity/drag/curl/vortex/attractor) + collision"
```

---

## Task 5 — ParticleBuffer (SoA storage)

**Files:**
- Create: `src/particle/buffer.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 5.1: Failing tests**

Create `src/particle/buffer.rs`:

```rust
use std::collections::VecDeque;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_honors_capacity() {
        let mut b = ParticleBuffer::new(2, false);
        assert!(b.push(ParticleSpawn::default()));
        assert!(b.push(ParticleSpawn::default()));
        assert!(!b.push(ParticleSpawn::default()));   // over capacity
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn swap_remove_compacts() {
        let mut b = ParticleBuffer::new(4, false);
        let mut s = ParticleSpawn::default();
        s.lifetime = 1.0;
        for _ in 0..4 { b.push(s.clone()); }
        b.age[0] = 0.5;
        b.age[1] = 1.5;   // dead
        b.age[2] = 0.3;
        b.age[3] = 2.0;   // dead
        b.compact_dead(|i, _, _, _| {});
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn trail_allocated_only_when_ribbon() {
        let no_trail = ParticleBuffer::new(4, false);
        assert!(no_trail.trail.is_none());
        let with_trail = ParticleBuffer::new(4, true);
        assert!(with_trail.trail.is_some());
    }

    #[test]
    fn compact_dead_invokes_callback() {
        let mut b = ParticleBuffer::new(4, false);
        let mut s = ParticleSpawn::default();
        s.lifetime = 1.0;
        for _ in 0..4 { b.push(s.clone()); }
        b.age[1] = 2.0;
        b.age[3] = 2.0;
        let mut killed: Vec<usize> = vec![];
        b.compact_dead(|_orig_idx, pos, vel, color| {
            killed.push(0);
            let _ = (pos, vel, color);
        });
        assert_eq!(killed.len(), 2);
    }
}
```

- [ ] **Step 5.2: Run failures**

`cargo test --lib particle::buffer`

- [ ] **Step 5.3: Implement `ParticleBuffer`**

Above tests:

```rust
#[derive(Clone, Debug, Default)]
pub struct ParticleSpawn {
    pub position:  [f32; 3],
    pub velocity:  [f32; 3],
    pub color:     [f32; 4],
    pub size:      f32,
    pub rotation:  f32,
    pub lifetime:  f32,
    pub seed:      f32,
}

pub struct ParticleBuffer {
    pub position:       Vec<[f32; 3]>,
    pub velocity:       Vec<[f32; 3]>,
    pub color:          Vec<[f32; 4]>,
    pub size:           Vec<f32>,
    pub rotation:       Vec<f32>,
    pub age:            Vec<f32>,
    pub lifetime:       Vec<f32>,
    pub seed:           Vec<f32>,
    pub flipbook_frame: Vec<f32>,
    pub trail:          Option<Vec<VecDeque<[f32; 3]>>>,
    capacity:           usize,
}

impl ParticleBuffer {
    pub fn new(capacity: usize, with_trails: bool) -> Self {
        ParticleBuffer {
            position:       Vec::with_capacity(capacity),
            velocity:       Vec::with_capacity(capacity),
            color:          Vec::with_capacity(capacity),
            size:           Vec::with_capacity(capacity),
            rotation:       Vec::with_capacity(capacity),
            age:            Vec::with_capacity(capacity),
            lifetime:       Vec::with_capacity(capacity),
            seed:           Vec::with_capacity(capacity),
            flipbook_frame: Vec::with_capacity(capacity),
            trail:          if with_trails { Some(Vec::with_capacity(capacity)) } else { None },
            capacity,
        }
    }

    pub fn count(&self) -> usize { self.position.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn is_full(&self) -> bool { self.count() >= self.capacity }

    pub fn push(&mut self, s: ParticleSpawn) -> bool {
        if self.is_full() { return false; }
        self.position.push(s.position);
        self.velocity.push(s.velocity);
        self.color.push(s.color);
        self.size.push(s.size);
        self.rotation.push(s.rotation);
        self.age.push(0.0);
        self.lifetime.push(s.lifetime);
        self.seed.push(s.seed);
        self.flipbook_frame.push(0.0);
        if let Some(t) = &mut self.trail { t.push(VecDeque::new()); }
        true
    }

    /// Remove all particles whose age >= lifetime, in-place, compacting via swap_remove.
    /// Callback invoked for each killed particle with (original_index, position, velocity, color).
    pub fn compact_dead(&mut self, mut on_kill: impl FnMut(usize, [f32; 3], [f32; 3], [f32; 4])) {
        let mut i = 0;
        while i < self.position.len() {
            if self.age[i] >= self.lifetime[i] {
                on_kill(i, self.position[i], self.velocity[i], self.color[i]);
                let last = self.position.len() - 1;
                self.position.swap_remove(i);
                self.velocity.swap_remove(i);
                self.color.swap_remove(i);
                self.size.swap_remove(i);
                self.rotation.swap_remove(i);
                self.age.swap_remove(i);
                self.lifetime.swap_remove(i);
                self.seed.swap_remove(i);
                self.flipbook_frame.swap_remove(i);
                if let Some(t) = &mut self.trail { t.swap_remove(i); }
                let _ = last;
                // don't advance i — new particle at this slot
            } else {
                i += 1;
            }
        }
    }
}
```

- [ ] **Step 5.4: Wire into mod.rs**

```rust
pub mod types;
pub mod curves;
pub mod emitter;
pub mod forces;
pub mod collision;
pub mod buffer;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
pub use forces::{VortexConfig, AttractorConfig, CurlNoiseConfig};
pub use collision::{CollisionConfig, CollisionEvent};
pub use buffer::{ParticleBuffer, ParticleSpawn};
```

- [ ] **Step 5.5: Tests pass**

`cargo test --lib particle::buffer` → 4 tests pass.

- [ ] **Step 5.6: Commit**

```bash
git add src/particle/
git commit -m "feat(particle): ParticleBuffer SoA with capacity + compact_dead"
```

---

## Task 6 — Config, render style, light/flipbook configs, JSON roundtrip

**Files:**
- Create: `src/particle/config.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 6.1: Failing test for full-config roundtrip**

Create `src/particle/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::particle::{Range, BurstSpec, Curve, Gradient, EmitterShape, InitialVelocity, ColorRange, VortexConfig, AttractorConfig, CurlNoiseConfig, CollisionConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_minimal_roundtrip() {
        let c = ParticleSystemConfig {
            name: "test".into(),
            max_particles: 64,
            looped: true,
            duration: 1.0,
            prewarm: false,
            seed: None,
            emission_rate: 10.0,
            bursts: vec![],
            emitter_shape: EmitterShape::Point {},
            initial_velocity: InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(1.0, 2.0) },
            initial_lifetime: Range::new(0.5, 1.0),
            initial_size: Range::new(0.1, 0.2),
            initial_color: ColorRange::Single([1.0, 1.0, 1.0, 1.0]),
            initial_rotation: Range::new(0.0, 0.0),
            color_over_life: None,
            size_over_life: None,
            rotation_over_life: None,
            angular_velocity: None,
            gravity: None,
            drag: None,
            curl_noise: None,
            vortex: None,
            attractor: None,
            collision: None,
            on_death: None,
            on_collision: None,
            light_emit: None,
            render: RenderStyle::Sprite {
                flipbook: None,
                blend_mode: BlendMode::Additive,
                soft_particles: false,
                soft_fade_distance: 0.0,
                velocity_stretch: 0.0,
            },
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: ParticleSystemConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
        assert_eq!(back.max_particles, 64);
    }

    #[test]
    fn config_validate_rejects_zero_capacity() {
        let mut c = minimal_config();
        c.max_particles = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_validate_rejects_negative_lifetime() {
        let mut c = minimal_config();
        c.initial_lifetime = Range::new(0.0, 1.0);
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_validate_accepts_good_config() {
        assert!(minimal_config().validate().is_ok());
    }

    fn minimal_config() -> ParticleSystemConfig {
        ParticleSystemConfig {
            name: "t".into(),
            max_particles: 8,
            looped: false,
            duration: 1.0,
            prewarm: false,
            seed: None,
            emission_rate: 0.0,
            bursts: vec![BurstSpec { time: 0.0, count: 4 }],
            emitter_shape: EmitterShape::Point {},
            initial_velocity: InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(1.0, 1.0) },
            initial_lifetime: Range::new(0.5, 1.0),
            initial_size: Range::new(0.1, 0.2),
            initial_color: ColorRange::Single([1.0, 1.0, 1.0, 1.0]),
            initial_rotation: Range::new(0.0, 0.0),
            color_over_life: None,
            size_over_life: None,
            rotation_over_life: None,
            angular_velocity: None,
            gravity: None,
            drag: None,
            curl_noise: None,
            vortex: None,
            attractor: None,
            collision: None,
            on_death: None,
            on_collision: None,
            light_emit: None,
            render: RenderStyle::Sprite {
                flipbook: None,
                blend_mode: BlendMode::Alpha,
                soft_particles: false,
                soft_fade_distance: 0.0,
                velocity_stretch: 0.0,
            },
        }
    }
}
```

- [ ] **Step 6.2: Run failures**

`cargo test --lib particle::config`

- [ ] **Step 6.3: Implement `ParticleSystemConfig` and friends**

Above tests:

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum BlendMode { Additive, Alpha, PremultipliedAlpha }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlipbookConfig {
    pub cols: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub fps: f32,
    #[serde(default)]
    pub blend: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LightEmitConfig {
    pub max_lights: u32,
    pub color_multiplier: [f32; 3],
    pub intensity: f32,
    pub falloff: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum RenderStyle {
    Sprite {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        flipbook: Option<FlipbookConfig>,
        blend_mode: BlendMode,
        #[serde(default)]
        soft_particles: bool,
        #[serde(default)]
        soft_fade_distance: f32,
        #[serde(default)]
        velocity_stretch: f32,
    },
    Ribbon {
        max_trail_points: usize,
        width_over_life: Curve<f32>,
        blend_mode: BlendMode,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParticleSystemConfig {
    pub name: String,
    pub max_particles: usize,
    pub looped: bool,
    pub duration: f32,
    #[serde(default)]
    pub prewarm: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<u64>,

    pub emission_rate: f32,
    #[serde(default)]
    pub bursts: Vec<BurstSpec>,

    pub emitter_shape: EmitterShape,
    pub initial_velocity: InitialVelocity,
    pub initial_lifetime: Range<f32>,
    pub initial_size: Range<f32>,
    pub initial_color: ColorRange,
    #[serde(default = "default_rotation_range")]
    pub initial_rotation: Range<f32>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color_over_life: Option<Gradient<[f32; 4]>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size_over_life: Option<Curve<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotation_over_life: Option<Curve<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub angular_velocity: Option<Range<f32>>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gravity: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub drag: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub curl_noise: Option<CurlNoiseConfig>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vortex: Option<VortexConfig>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub attractor: Option<AttractorConfig>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub collision: Option<CollisionConfig>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub on_death: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub on_collision: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub light_emit: Option<LightEmitConfig>,

    pub render: RenderStyle,
}

fn default_rotation_range() -> Range<f32> { Range::new(0.0, 0.0) }

impl ParticleSystemConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_particles == 0 { return Err("max_particles must be > 0".into()); }
        if self.duration < 0.0 { return Err("duration must be >= 0".into()); }
        if self.initial_lifetime.min <= 0.0 {
            return Err("initial_lifetime.min must be > 0".into());
        }
        if self.initial_lifetime.min > self.initial_lifetime.max {
            return Err("initial_lifetime: min > max".into());
        }
        if self.initial_size.min > self.initial_size.max {
            return Err("initial_size: min > max".into());
        }
        if let Some(g) = &self.color_over_life {
            crate::particle::curves::validate_keyframes(&g.keys).map_err(|e| format!("color_over_life: {e}"))?;
        }
        if let Some(c) = &self.size_over_life {
            crate::particle::curves::validate_keyframes(&c.keys).map_err(|e| format!("size_over_life: {e}"))?;
        }
        if let Some(c) = &self.rotation_over_life {
            crate::particle::curves::validate_keyframes(&c.keys).map_err(|e| format!("rotation_over_life: {e}"))?;
        }
        if let RenderStyle::Ribbon { ref width_over_life, .. } = self.render {
            crate::particle::curves::validate_keyframes(&width_over_life.keys).map_err(|e| format!("width_over_life: {e}"))?;
        }
        Ok(())
    }
}
```

- [ ] **Step 6.4: Wire into mod.rs**

```rust
pub mod types;
pub mod curves;
pub mod emitter;
pub mod forces;
pub mod collision;
pub mod buffer;
pub mod config;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
pub use forces::{VortexConfig, AttractorConfig, CurlNoiseConfig};
pub use collision::{CollisionConfig, CollisionEvent};
pub use buffer::{ParticleBuffer, ParticleSpawn};
pub use config::{ParticleSystemConfig, RenderStyle, BlendMode, FlipbookConfig, LightEmitConfig};
```

- [ ] **Step 6.5: Tests pass**

`cargo test --lib particle::config` → 4 tests pass.

- [ ] **Step 6.6: Commit**

```bash
git add src/particle/
git commit -m "feat(particle): ParticleSystemConfig with serde + validate"
```

---

## Task 7 — `ParticleSystem` struct + setters + JSON I/O

**Files:**
- Create: `src/particle/system.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 7.1: Failing tests**

Create `src/particle/system.rs`:

```rust
use std::fs;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use uuid::Uuid;
use crate::particle::*;
use crate::particle::config::*;
use crate::logging::EnigmaError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_minimal_system() {
        let mut s = ParticleSystem::new("test");
        s.set_max_particles(32);
        s.set_initial_lifetime(0.5, 1.0);
        s.set_emitter_shape(EmitterShape::Point {});
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(1.0, 1.0) });
        s.set_initial_size(0.1, 0.2);
        s.set_initial_color(ColorRange::Single([1.0, 1.0, 1.0, 1.0]));
        s.set_render_style(RenderStyle::Sprite {
            flipbook: None, blend_mode: BlendMode::Alpha,
            soft_particles: false, soft_fade_distance: 0.0, velocity_stretch: 0.0,
        });
        assert_eq!(s.name(), "test");
        assert_eq!(s.config.max_particles, 32);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn from_resource_parses_valid_json() {
        let json = r#"{
            "name": "ex",
            "max_particles": 32,
            "looped": false,
            "duration": 1.0,
            "emission_rate": 10.0,
            "emitter_shape": {"Point": {}},
            "initial_velocity": {"Direction": {"direction": [0.0,1.0,0.0], "speed": {"min": 1.0, "max": 2.0}}},
            "initial_lifetime": {"min": 0.5, "max": 1.0},
            "initial_size": {"min": 0.1, "max": 0.2},
            "initial_color": {"Single": [1.0, 1.0, 1.0, 1.0]},
            "render": {"kind": "Sprite", "blend_mode": "Alpha"}
        }"#;
        let s = ParticleSystem::from_resource(json).expect("should parse");
        assert_eq!(s.name(), "ex");
    }

    #[test]
    fn from_resource_rejects_invalid_config() {
        let json = r#"{
            "name": "bad",
            "max_particles": 0,
            "looped": false,
            "duration": 1.0,
            "emission_rate": 10.0,
            "emitter_shape": {"Point": {}},
            "initial_velocity": {"Direction": {"direction": [0.0,1.0,0.0], "speed": {"min": 1.0, "max": 2.0}}},
            "initial_lifetime": {"min": 0.5, "max": 1.0},
            "initial_size": {"min": 0.1, "max": 0.2},
            "initial_color": {"Single": [1.0,1.0,1.0,1.0]},
            "render": {"kind": "Sprite", "blend_mode": "Alpha"}
        }"#;
        assert!(ParticleSystem::from_resource(json).is_err());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let mut s = ParticleSystem::new("rt");
        s.set_max_particles(16);
        s.set_initial_lifetime(0.5, 1.0);
        s.set_emitter_shape(EmitterShape::Sphere { radius: 0.5 });
        s.set_initial_velocity(InitialVelocity::Outward { speed: Range::new(1.0, 2.0) });
        s.set_initial_size(0.1, 0.2);
        s.set_initial_color(ColorRange::Single([1.0, 1.0, 1.0, 1.0]));
        s.set_render_style(RenderStyle::Sprite {
            flipbook: None, blend_mode: BlendMode::Additive,
            soft_particles: false, soft_fade_distance: 0.0, velocity_stretch: 0.0,
        });
        let json = s.to_json().unwrap();
        let back = ParticleSystem::from_resource(&json).unwrap();
        assert_eq!(back.name(), "rt");
        assert_eq!(back.config.max_particles, 16);
    }
}
```

- [ ] **Step 7.2: Run failures**

`cargo test --lib particle::system`

- [ ] **Step 7.3: Implement `ParticleSystem`**

Above tests:

```rust
pub struct EmissionState {
    pub elapsed: f32,
    pub spawn_accumulator: f32,
    pub pending_collision_events: Vec<CollisionEvent>,
    pub finished: bool,
    pub stopped: bool,
}

impl Default for EmissionState {
    fn default() -> Self {
        EmissionState {
            elapsed: 0.0,
            spawn_accumulator: 0.0,
            pending_collision_events: Vec::new(),
            finished: false,
            stopped: false,
        }
    }
}

pub struct ParticleSystem {
    pub handle: Uuid,
    pub config: ParticleSystemConfig,
    pub buffer: ParticleBuffer,
    pub transform: [[f32; 4]; 4],
    pub parent_object: Option<Uuid>,
    pub local_offset: [[f32; 4]; 4],
    pub material_id: Option<Uuid>,
    pub state: EmissionState,
    pub rng: SmallRng,
}

fn identity4() -> [[f32; 4]; 4] {
    [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]]
}

fn translate4(t: [f32; 3]) -> [[f32; 4]; 4] {
    [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[t[0],t[1],t[2],1.0]]
}

fn default_config(name: &str) -> ParticleSystemConfig {
    ParticleSystemConfig {
        name: name.into(),
        max_particles: 64,
        looped: false,
        duration: 1.0,
        prewarm: false,
        seed: None,
        emission_rate: 0.0,
        bursts: Vec::new(),
        emitter_shape: EmitterShape::Point {},
        initial_velocity: InitialVelocity::Direction { direction: [0.0,1.0,0.0], speed: Range::new(1.0, 1.0) },
        initial_lifetime: Range::new(0.5, 1.0),
        initial_size: Range::new(0.1, 0.2),
        initial_color: ColorRange::Single([1.0,1.0,1.0,1.0]),
        initial_rotation: Range::new(0.0, 0.0),
        color_over_life: None,
        size_over_life: None,
        rotation_over_life: None,
        angular_velocity: None,
        gravity: None,
        drag: None,
        curl_noise: None,
        vortex: None,
        attractor: None,
        collision: None,
        on_death: None,
        on_collision: None,
        light_emit: None,
        render: RenderStyle::Sprite {
            flipbook: None,
            blend_mode: BlendMode::Alpha,
            soft_particles: false,
            soft_fade_distance: 0.0,
            velocity_stretch: 0.0,
        },
    }
}

impl ParticleSystem {
    pub fn new(name: &str) -> Self {
        let config = default_config(name);
        Self::from_config_internal(config)
    }

    pub fn from_file(path: &str) -> Result<Self, EnigmaError> {
        let json = fs::read_to_string(path).map_err(|e| EnigmaError::new(Some(&format!("particle from_file '{}': {}", path, e)), false))?;
        Self::from_resource(&json)
    }

    pub fn from_resource(json: &str) -> Result<Self, EnigmaError> {
        let config: ParticleSystemConfig = serde_json::from_str(json)
            .map_err(|e| EnigmaError::new(Some(&format!("particle JSON parse: {}", e)), false))?;
        config.validate()
            .map_err(|e| EnigmaError::new(Some(&format!("particle config invalid: {}", e)), false))?;
        Ok(Self::from_config_internal(config))
    }

    fn from_config_internal(config: ParticleSystemConfig) -> Self {
        let capacity = config.max_particles.max(1);
        let with_trails = matches!(config.render, RenderStyle::Ribbon { .. });
        let seed = config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
        });
        ParticleSystem {
            handle: Uuid::new_v4(),
            config,
            buffer: ParticleBuffer::new(capacity, with_trails),
            transform: identity4(),
            parent_object: None,
            local_offset: identity4(),
            material_id: None,
            state: EmissionState::default(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn validate(&self) -> Result<(), String> { self.config.validate() }

    pub fn to_json(&self) -> Result<String, EnigmaError> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| EnigmaError::new(Some(&format!("particle to_json: {}", e)), false))
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), EnigmaError> {
        let json = self.to_json()?;
        fs::write(path, json).map_err(|e| EnigmaError::new(Some(&format!("particle save_to_file: {}", e)), false))
    }

    // Identity
    pub fn name(&self) -> &str { &self.config.name }
    pub fn get_unique_id(&self) -> Uuid { self.handle }
    pub fn set_name(&mut self, name: &str) { self.config.name = name.into(); }

    // Placement
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.transform = translate4(position);
    }
    pub fn set_local_offset(&mut self, offset: [[f32; 4]; 4]) { self.local_offset = offset; }
    pub fn set_parent_object(&mut self, object_id: Uuid) { self.parent_object = Some(object_id); }
    pub fn clear_parent(&mut self) { self.parent_object = None; }

    // Lifetime / emission
    pub fn set_max_particles(&mut self, max: usize) {
        self.config.max_particles = max;
        let with_trails = matches!(self.config.render, RenderStyle::Ribbon { .. });
        self.buffer = ParticleBuffer::new(max.max(1), with_trails);
    }
    pub fn set_looped(&mut self, looped: bool) { self.config.looped = looped; }
    pub fn set_duration(&mut self, seconds: f32) { self.config.duration = seconds; }
    pub fn set_prewarm(&mut self, prewarm: bool) { self.config.prewarm = prewarm; }
    pub fn set_emission_rate(&mut self, rate: f32) { self.config.emission_rate = rate; }
    pub fn add_burst(&mut self, time: f32, count: u32) { self.config.bursts.push(BurstSpec { time, count }); }
    pub fn clear_bursts(&mut self) { self.config.bursts.clear(); }
    pub fn set_seed(&mut self, seed: u64) {
        self.config.seed = Some(seed);
        self.rng = SmallRng::seed_from_u64(seed);
    }

    // Initial state
    pub fn set_emitter_shape(&mut self, shape: EmitterShape) { self.config.emitter_shape = shape; }
    pub fn set_initial_velocity(&mut self, v: InitialVelocity) { self.config.initial_velocity = v; }
    pub fn set_initial_lifetime(&mut self, min: f32, max: f32) { self.config.initial_lifetime = Range::new(min, max); }
    pub fn set_initial_size(&mut self, min: f32, max: f32) { self.config.initial_size = Range::new(min, max); }
    pub fn set_initial_rotation(&mut self, min: f32, max: f32) { self.config.initial_rotation = Range::new(min, max); }
    pub fn set_initial_color(&mut self, color: ColorRange) { self.config.initial_color = color; }

    // Over-life
    pub fn set_color_over_life(&mut self, g: Gradient<[f32; 4]>) { self.config.color_over_life = Some(g); }
    pub fn set_size_over_life(&mut self, c: Curve<f32>) { self.config.size_over_life = Some(c); }
    pub fn set_rotation_over_life(&mut self, c: Curve<f32>) { self.config.rotation_over_life = Some(c); }
    pub fn set_angular_velocity(&mut self, min: f32, max: f32) { self.config.angular_velocity = Some(Range::new(min, max)); }

    // Forces
    pub fn set_gravity(&mut self, g: [f32; 3]) { self.config.gravity = Some(g); }
    pub fn set_drag(&mut self, d: f32) { self.config.drag = Some(d); }
    pub fn set_curl_noise(&mut self, c: CurlNoiseConfig) { self.config.curl_noise = Some(c); }
    pub fn set_vortex(&mut self, c: VortexConfig) { self.config.vortex = Some(c); }
    pub fn set_attractor(&mut self, c: AttractorConfig) { self.config.attractor = Some(c); }

    // Collision
    pub fn set_collision(&mut self, c: CollisionConfig) { self.config.collision = Some(c); }

    // Sub-emitters
    pub fn set_on_death(&mut self, template_name: &str) { self.config.on_death = Some(template_name.into()); }
    pub fn set_on_collision(&mut self, template_name: &str) { self.config.on_collision = Some(template_name.into()); }

    // Lights
    pub fn set_light_emit(&mut self, c: LightEmitConfig) { self.config.light_emit = Some(c); }

    // Material + render
    pub fn set_material(&mut self, material_id: Uuid) { self.material_id = Some(material_id); }
    pub fn set_render_style(&mut self, style: RenderStyle) {
        let needs_trails = matches!(style, RenderStyle::Ribbon { .. });
        let has_trails = self.buffer.trail.is_some();
        self.config.render = style;
        if needs_trails != has_trails {
            self.buffer = ParticleBuffer::new(self.config.max_particles.max(1), needs_trails);
        }
    }
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        match &mut self.config.render {
            RenderStyle::Sprite { blend_mode, .. } => *blend_mode = mode,
            RenderStyle::Ribbon { blend_mode, .. } => *blend_mode = mode,
        }
    }
    pub fn set_flipbook(&mut self, config: FlipbookConfig) {
        if let RenderStyle::Sprite { flipbook, .. } = &mut self.config.render {
            *flipbook = Some(config);
        }
    }
    pub fn set_soft_particles(&mut self, enabled: bool, fade_distance: f32) {
        if let RenderStyle::Sprite { soft_particles, soft_fade_distance, .. } = &mut self.config.render {
            *soft_particles = enabled;
            *soft_fade_distance = fade_distance;
        }
    }
    pub fn set_velocity_stretch(&mut self, stretch: f32) {
        if let RenderStyle::Sprite { velocity_stretch, .. } = &mut self.config.render {
            *velocity_stretch = stretch;
        }
    }

    // Runtime control
    pub fn play(&mut self) { self.state.stopped = false; }
    pub fn stop(&mut self) { self.state.stopped = true; }
    pub fn restart(&mut self) {
        let capacity = self.config.max_particles.max(1);
        let with_trails = matches!(self.config.render, RenderStyle::Ribbon { .. });
        self.buffer = ParticleBuffer::new(capacity, with_trails);
        self.state = EmissionState::default();
    }
    pub fn is_finished(&self) -> bool { self.state.finished }
    pub fn live_count(&self) -> usize { self.buffer.count() }
}
```

- [ ] **Step 7.4: Wire into mod.rs**

```rust
pub mod types;
pub mod curves;
pub mod emitter;
pub mod forces;
pub mod collision;
pub mod buffer;
pub mod config;
pub mod system;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
pub use forces::{VortexConfig, AttractorConfig, CurlNoiseConfig};
pub use collision::{CollisionConfig, CollisionEvent};
pub use buffer::{ParticleBuffer, ParticleSpawn};
pub use config::{ParticleSystemConfig, RenderStyle, BlendMode, FlipbookConfig, LightEmitConfig};
pub use system::{ParticleSystem, EmissionState};
```

- [ ] **Step 7.5: Run tests, pass**

`cargo test --lib particle::system` → 4 tests pass. Run full library: `cargo test --lib particle::`.

- [ ] **Step 7.6: Commit**

```bash
git add src/particle/ src/lib.rs
git commit -m "feat(particle): ParticleSystem with builder API + JSON I/O"
```

---

## Task 8 — Update pipeline (tick)

**Files:**
- Modify: `src/particle/system.rs` (add `tick` method)

- [ ] **Step 8.1: Failing test — single non-looped particle dies on schedule**

Append to `src/particle/system.rs` tests:

```rust
    #[test]
    fn one_particle_dies_after_lifetime() {
        let mut s = ParticleSystem::new("t");
        s.set_max_particles(4);
        s.set_initial_lifetime(1.0, 1.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0, 0.0, 0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.config.bursts.push(BurstSpec { time: 0.0, count: 1 });
        s.set_looped(false);
        s.set_duration(2.0);

        s.tick(0.5, None);  // spawn at t=0; one particle
        assert_eq!(s.live_count(), 1);
        s.tick(0.6, None);  // age now 1.1 → dead
        assert_eq!(s.live_count(), 0);
    }

    #[test]
    fn looped_steady_state() {
        let mut s = ParticleSystem::new("loop");
        s.set_max_particles(100);
        s.set_initial_lifetime(1.0, 1.0);
        s.set_emission_rate(10.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0,0.0,0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.set_looped(true);
        for _ in 0..120 { s.tick(1.0/60.0, None); }
        assert!(s.live_count() >= 5);
        assert!(s.live_count() <= 15);
    }

    #[test]
    fn gravity_accelerates() {
        let mut s = ParticleSystem::new("g");
        s.set_max_particles(1);
        s.set_initial_lifetime(10.0, 10.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0,0.0,0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.set_gravity([0.0, -10.0, 0.0]);
        s.config.bursts.push(BurstSpec { time: 0.0, count: 1 });
        s.tick(0.01, None);  // spawn
        s.tick(1.0, None);   // 1 sec gravity
        assert!(s.buffer.velocity[0][1] < -5.0);
    }
```

- [ ] **Step 8.2: Run, see failures (`tick` undefined)**

- [ ] **Step 8.3: Implement `tick`**

Append to `impl ParticleSystem`:

```rust
    /// Advance simulation by `dt`. `terrain_height_at`, if provided, is queried for collision.
    /// Death and collision events that should spawn sub-emitters are returned for the caller to dispatch.
    pub fn tick(&mut self, dt: f32, terrain_height_at: Option<&dyn Fn(f32, f32) -> Option<f32>>) -> TickResult {
        let mut result = TickResult::default();
        if self.state.stopped {
            return result;
        }

        // 1. Advance time
        let prev_elapsed = self.state.elapsed;
        self.state.elapsed += dt;

        let within_emission = self.config.looped || self.state.elapsed < self.config.duration;
        if within_emission {
            self.state.spawn_accumulator += self.config.emission_rate * dt;
        }

        // 2. Age & kill — collect death positions for sub-emit
        let need_death_events = self.config.on_death.is_some();
        let mut death_events: Vec<([f32;3], [f32;3], [f32;4])> = Vec::new();
        for i in 0..self.buffer.count() {
            self.buffer.age[i] += dt;
        }
        self.buffer.compact_dead(|_, pos, vel, color| {
            if need_death_events {
                death_events.push((pos, vel, color));
            }
        });
        if need_death_events {
            for (pos, vel, color) in death_events {
                result.death_events.push(DeathEvent { position: pos, velocity: vel, color });
            }
        }

        // 3. Spawn
        let mut to_spawn: usize = 0;
        if within_emission {
            let n = self.state.spawn_accumulator.floor();
            to_spawn += n as usize;
            self.state.spawn_accumulator -= n;
        }
        // bursts within (prev_elapsed, elapsed]
        for b in &self.config.bursts {
            if b.time > prev_elapsed && b.time <= self.state.elapsed {
                to_spawn += b.count as usize;
            }
        }
        // pending collision events from prior frame
        let pending = std::mem::take(&mut self.state.pending_collision_events);
        let collision_extra = pending.len();

        let origin = [self.transform[3][0], self.transform[3][1], self.transform[3][2]];

        for _ in 0..to_spawn {
            if self.buffer.is_full() { break; }
            let local_pos = self.config.emitter_shape.sample_position(&mut self.rng);
            let world_pos = [origin[0]+local_pos[0], origin[1]+local_pos[1], origin[2]+local_pos[2]];
            let vel = self.config.initial_velocity.sample(&local_pos, &mut self.rng);
            let life = self.config.initial_lifetime.sample(&mut self.rng);
            let size = self.config.initial_size.sample(&mut self.rng);
            let color = self.config.initial_color.sample(&mut self.rng);
            let rotation = self.config.initial_rotation.sample(&mut self.rng);
            let seed: f32 = rand::Rng::gen(&mut self.rng);
            self.buffer.push(ParticleSpawn {
                position: world_pos, velocity: vel, color, size, rotation, lifetime: life, seed,
            });
        }
        for ev in pending {
            if self.buffer.is_full() { break; }
            let _ = ev;  // collision events spawn from the on_collision sub-emitter, not here
        }
        let _ = collision_extra;

        // 4. Forces + 5. Integrate + 7. Sample curves
        let time = self.state.elapsed;
        let cfg = &self.config;
        for i in 0..self.buffer.count() {
            let mut accel = [0.0_f32; 3];
            if let Some(g) = cfg.gravity {
                accel[0] += g[0]; accel[1] += g[1]; accel[2] += g[2];
            }
            if let Some(curl) = cfg.curl_noise {
                let f = curl.force(&self.buffer.position[i], time);
                accel[0] += f[0]; accel[1] += f[1]; accel[2] += f[2];
            }
            if let Some(vortex) = cfg.vortex {
                let f = vortex.force(&self.buffer.position[i]);
                accel[0] += f[0]; accel[1] += f[1]; accel[2] += f[2];
            }
            if let Some(attractor) = cfg.attractor {
                let f = attractor.force(&self.buffer.position[i]);
                accel[0] += f[0]; accel[1] += f[1]; accel[2] += f[2];
            }
            self.buffer.velocity[i][0] += accel[0] * dt;
            self.buffer.velocity[i][1] += accel[1] * dt;
            self.buffer.velocity[i][2] += accel[2] * dt;
            if let Some(d) = cfg.drag {
                crate::particle::forces::apply_drag(&mut self.buffer.velocity[i], d, dt);
            }
            // Integrate
            self.buffer.position[i][0] += self.buffer.velocity[i][0] * dt;
            self.buffer.position[i][1] += self.buffer.velocity[i][1] * dt;
            self.buffer.position[i][2] += self.buffer.velocity[i][2] * dt;
            self.buffer.rotation[i] += 0.0;  // angular_velocity handled later if set

            // Flipbook advance
            if let RenderStyle::Sprite { flipbook: Some(fb), .. } = &cfg.render {
                self.buffer.flipbook_frame[i] += fb.fps * dt;
            }
        }

        // 6. Collide
        if let Some(coll) = &cfg.collision {
            for i in 0..self.buffer.count() {
                let mut hit = coll.test_and_resolve_ground(&mut self.buffer.position[i], &mut self.buffer.velocity[i]);
                if !hit {
                    if let Some(query) = terrain_height_at {
                        hit = coll.test_and_resolve_terrain(&mut self.buffer.position[i], &mut self.buffer.velocity[i], query);
                    }
                }
                if hit && cfg.on_collision.is_some() {
                    result.collision_events.push(CollisionEvent {
                        position: self.buffer.position[i],
                        velocity: self.buffer.velocity[i],
                        normal: [0.0, 1.0, 0.0],
                    });
                }
            }
        }

        // 7. Sample curves
        if cfg.color_over_life.is_some() || cfg.size_over_life.is_some() {
            for i in 0..self.buffer.count() {
                let t = (self.buffer.age[i] / self.buffer.lifetime[i]).clamp(0.0, 1.0);
                if let Some(g) = &cfg.color_over_life {
                    let c = g.eval(t);
                    self.buffer.color[i] = c;
                }
                if let Some(c) = &cfg.size_over_life {
                    self.buffer.size[i] = c.eval(t);
                }
            }
        }

        // 8. Push trail history
        if let Some(trails) = &mut self.buffer.trail {
            let max_points = match cfg.render {
                RenderStyle::Ribbon { max_trail_points, .. } => max_trail_points,
                _ => 0,
            };
            for i in 0..self.buffer.count() {
                trails[i].push_front(self.buffer.position[i]);
                while trails[i].len() > max_points {
                    trails[i].pop_back();
                }
            }
        }

        // 10. Mark finished
        if !cfg.looped && self.state.elapsed > cfg.duration && self.buffer.count() == 0 {
            self.state.finished = true;
        }

        result
    }
}

#[derive(Default, Debug)]
pub struct TickResult {
    pub death_events: Vec<DeathEvent>,
    pub collision_events: Vec<CollisionEvent>,
}

#[derive(Clone, Copy, Debug)]
pub struct DeathEvent {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub color:    [f32; 4],
}
```

- [ ] **Step 8.4: Run, pass**

`cargo test --lib particle::system` → 7 tests pass.

- [ ] **Step 8.5: Commit**

```bash
git add src/particle/system.rs
git commit -m "feat(particle): update pipeline (tick) with forces, integration, collision, curves"
```

---

## Task 9 — Sub-emitter registry + AppState integration

**Files:**
- Modify: `src/lib.rs` (AppState fields + methods + EventLoop tick call)

- [ ] **Step 9.1: Failing test for AppState.add_particle_system + lookup**

Append a new `#[cfg(test)] mod particle_integration_tests` at the end of `src/lib.rs`:

```rust
#[cfg(test)]
mod particle_integration_tests {
    use super::*;
    use crate::particle::*;

    fn make_template(name: &str) -> ParticleSystem {
        let mut s = ParticleSystem::new(name);
        s.set_max_particles(4);
        s.set_initial_lifetime(0.5, 1.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0,1.0,0.0], speed: Range::new(1.0, 1.0) });
        s.set_initial_size(0.1, 0.2);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.set_render_style(RenderStyle::Sprite {
            flipbook: None, blend_mode: BlendMode::Alpha,
            soft_particles: false, soft_fade_distance: 0.0, velocity_stretch: 0.0,
        });
        s
    }

    #[test]
    fn add_and_lookup_by_name() {
        let mut app = AppState::new();
        let s = make_template("smoke");
        let id = s.get_unique_id();
        app.add_particle_system(s);
        assert!(app.get_particle_system_by_name("smoke").is_some());
        assert!(app.get_particle_system(id).is_some());
    }

    #[test]
    fn emit_from_template() {
        let mut app = AppState::new();
        let mut tpl = make_template("hit");
        tpl.add_burst(0.0, 3);
        app.add_particle_system_template(tpl);
        let xform = [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[1.0,2.0,3.0,1.0]];
        let new_id = app.emit_particle_system("hit", xform);
        assert!(new_id.is_some());
    }

    #[test]
    fn remove_by_name_clears() {
        let mut app = AppState::new();
        app.add_particle_system(make_template("x"));
        app.remove_particle_systems_by_name("x");
        assert!(app.get_particle_system_by_name("x").is_none());
    }
}
```

- [ ] **Step 9.2: Run, see failures (methods undefined)**

`cargo test --lib particle_integration`

- [ ] **Step 9.3: Add `particle_systems` field to `AppState`**

In `src/lib.rs`, in the `AppState` struct (around line 159, near `terrain: Option<terrain::Terrain>,`):

```rust
    pub terrain: Option<terrain::Terrain>,
    pub particle_systems: Vec<particle::ParticleSystem>,
```

Also import at top of `lib.rs` (near other `use` lines):

```rust
use crate::particle::ParticleSystem;
```

And initialize in `AppState::new()` (around line 204):

```rust
            terrain: None,
            particle_systems: Vec::new(),
```

- [ ] **Step 9.4: Implement add/get/remove and emit methods**

Add to `impl AppState` block (after `set_terrain` or similar — pick a stable spot):

```rust
    pub fn add_particle_system(&mut self, system: particle::ParticleSystem) {
        self.particle_systems.push(system);
    }

    pub fn add_particle_system_template(&mut self, mut system: particle::ParticleSystem) {
        system.stop();
        self.particle_systems.push(system);
    }

    pub fn emit_particle_system(&mut self, template_name: &str, transform: [[f32; 4]; 4]) -> Option<Uuid> {
        let template_cfg = self.particle_systems.iter()
            .find(|s| s.name() == template_name)
            .map(|s| s.config.clone());
        let mat = self.particle_systems.iter()
            .find(|s| s.name() == template_name)
            .and_then(|s| s.material_id);
        if let Some(cfg) = template_cfg {
            let json = serde_json::to_string(&cfg).ok()?;
            let mut instance = particle::ParticleSystem::from_resource(&json).ok()?;
            instance.transform = transform;
            instance.material_id = mat;
            instance.play();
            let id = instance.get_unique_id();
            self.particle_systems.push(instance);
            Some(id)
        } else {
            None
        }
    }

    pub fn get_particle_system_by_name(&self, name: &str) -> Option<&particle::ParticleSystem> {
        self.particle_systems.iter().find(|s| s.name() == name)
    }

    pub fn get_particle_system_by_name_mut(&mut self, name: &str) -> Option<&mut particle::ParticleSystem> {
        self.particle_systems.iter_mut().find(|s| s.name() == name)
    }

    pub fn get_particle_system(&self, id: Uuid) -> Option<&particle::ParticleSystem> {
        self.particle_systems.iter().find(|s| s.get_unique_id() == id)
    }

    pub fn get_particle_system_mut(&mut self, id: Uuid) -> Option<&mut particle::ParticleSystem> {
        self.particle_systems.iter_mut().find(|s| s.get_unique_id() == id)
    }

    pub fn remove_particle_system(&mut self, id: Uuid) {
        self.particle_systems.retain(|s| s.get_unique_id() != id);
    }

    pub fn remove_particle_systems_by_name(&mut self, name: &str) {
        self.particle_systems.retain(|s| s.name() != name);
    }

    pub fn tick_particles(&mut self, dt: f32) {
        let template_index: std::collections::HashMap<String, usize> = self.particle_systems.iter()
            .enumerate()
            .map(|(i, s)| (s.name().to_string(), i))
            .collect();

        // Collect events from each system without overlapping borrows
        let mut to_spawn_death: Vec<(String, [f32; 3])> = Vec::new();
        let mut to_spawn_collision: Vec<(String, [f32; 3])> = Vec::new();

        let terrain_ref = self.terrain.as_ref();
        let query: Option<Box<dyn Fn(f32, f32) -> Option<f32>>> = match terrain_ref {
            Some(t) => {
                let height_fn = t.heightfield_query_fn();
                Some(Box::new(height_fn))
            }
            None => None,
        };
        let query_ref: Option<&dyn Fn(f32, f32) -> Option<f32>> = query.as_deref();

        for sys in &mut self.particle_systems {
            let result = sys.tick(dt, query_ref);
            if let Some(name) = &sys.config.on_death {
                for ev in &result.death_events {
                    to_spawn_death.push((name.clone(), ev.position));
                }
            }
            if let Some(name) = &sys.config.on_collision {
                for ev in &result.collision_events {
                    to_spawn_collision.push((name.clone(), ev.position));
                }
            }
        }

        let _ = template_index;
        for (name, pos) in to_spawn_death.into_iter().chain(to_spawn_collision.into_iter()) {
            let xform = [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[pos[0],pos[1],pos[2],1.0]];
            let _ = self.emit_particle_system(&name, xform);
        }

        // Reap finished
        self.particle_systems.retain(|s| !s.is_finished());
    }
```

- [ ] **Step 9.5: Add a placeholder `heightfield_query_fn` to terrain**

Open `src/terrain.rs` and add a method on `Terrain`:

```rust
impl Terrain {
    pub fn heightfield_query_fn(&self) -> impl Fn(f32, f32) -> Option<f32> + '_ {
        // Stub returning None until terrain height sampling is wired up.
        // The hook keeps the particle tick API stable; replace with real sampling later.
        |_x: f32, _z: f32| None
    }
}
```

If `Terrain` is `Send + Sync` and we need a `Box<dyn Fn + 'static>`, change the closure return to `move |x, z| ...` and capture by value (clone needed data into the closure). For v1, returning `None` keeps the integration test green; the terrain heightfield wiring becomes a follow-up.

- [ ] **Step 9.6: Run, fix any compile errors, see tests pass**

`cargo test --lib particle_integration` → 3 tests pass.

Likely compile fixes: `Uuid` import in `lib.rs` (already present), `serde_json` is already a dep. The `terrain_ref` block above may need flattening if borrow checker complains — adjust accordingly. If lifetime issues arise with `Box<dyn Fn>`, replace with a simple `if let Some(t) = terrain_ref { ... query t directly }` inside the tick loop.

- [ ] **Step 9.7: Commit**

```bash
git add src/lib.rs src/terrain.rs
git commit -m "feat(particle): AppState integration (add/get/remove/emit/tick)"
```

---

## Task 10 — Material presets + shader resources

**Files:**
- Create: `src/res/shader/particle_sprite_vert.glsl`
- Create: `src/res/shader/particle_sprite_frag.glsl`
- Create: `src/res/shader/particle_ribbon_vert.glsl`
- Create: `src/res/shader/particle_ribbon_frag.glsl`
- Modify: `src/resources.rs`
- Modify: `src/material.rs`

- [ ] **Step 10.1: Sprite vertex shader**

Create `src/res/shader/particle_sprite_vert.glsl`:

```glsl
#version 330 core
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

// per-vertex (unit quad): position in [-0.5, 0.5]^2, uv in [0,1]^2
in vec3 corner;
in vec2 corner_uv;

// per-instance
in vec3 inst_position;
in float inst_size;
in vec4  inst_color;
in vec3  inst_velocity;
in float inst_rotation;
in vec4  inst_flipbook_uv;  // x,y = atlas offset; z,w = atlas frame size

uniform float velocity_stretch;

out vec2 v_uv;
out vec4 v_color;
out vec3 v_view_pos;
out vec4 v_flipbook_uv;

void main() {
    // Camera-aligned billboard from view matrix basis
    vec3 right = vec3(view_matrix[0][0], view_matrix[1][0], view_matrix[2][0]);
    vec3 up    = vec3(view_matrix[0][1], view_matrix[1][1], view_matrix[2][1]);

    // Apply 2D rotation around screen-space normal
    float c = cos(inst_rotation);
    float s = sin(inst_rotation);
    vec2 rotated = vec2(c * corner.x - s * corner.y, s * corner.x + c * corner.y);

    // Velocity stretch (along projected velocity direction)
    vec3 vel_view = (view_matrix * vec4(inst_velocity, 0.0)).xyz;
    vec2 vel_screen = normalize(vec2(vel_view.x, vel_view.y) + vec2(1e-6, 0.0));
    float stretch_factor = 1.0 + velocity_stretch * length(inst_velocity);
    vec2 stretch_dir = vel_screen;
    vec2 perp = vec2(-stretch_dir.y, stretch_dir.x);
    vec2 stretched = stretch_dir * rotated.x * stretch_factor + perp * rotated.y;

    // Choose stretched layout only if velocity_stretch > 0; otherwise plain billboard
    vec2 use_pos = mix(rotated, stretched, step(0.001, velocity_stretch));

    vec3 world_pos = inst_position + right * (use_pos.x * inst_size) + up * (use_pos.y * inst_size);

    vec4 vp = view_matrix * vec4(world_pos, 1.0);
    v_view_pos = vp.xyz;
    v_uv = corner_uv;
    v_color = inst_color;
    v_flipbook_uv = inst_flipbook_uv;
    gl_Position = projection_matrix * vp;
}
```

- [ ] **Step 10.2: Sprite fragment shader**

Create `src/res/shader/particle_sprite_frag.glsl`:

```glsl
#version 330 core
in vec2 v_uv;
in vec4 v_color;
in vec3 v_view_pos;
in vec4 v_flipbook_uv;

uniform sampler2D albedo;
uniform sampler2D scene_depth;
uniform float soft_fade_distance;  // 0 = disabled
uniform int   use_flipbook;        // 0 or 1
uniform vec2  screen_size;
uniform float near_plane;
uniform float far_plane;

out vec4 out_color;

float linearize_depth(float d) {
    float z = d * 2.0 - 1.0;
    return (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane));
}

void main() {
    vec2 uv;
    if (use_flipbook == 1) {
        uv = v_flipbook_uv.xy + v_uv * v_flipbook_uv.zw;
    } else {
        uv = v_uv;
    }
    vec4 tex = texture(albedo, uv);
    vec3 color = tex.rgb * v_color.rgb;
    float alpha = tex.a * v_color.a;

    if (soft_fade_distance > 0.001) {
        vec2 screen_uv = gl_FragCoord.xy / screen_size;
        float scene_d  = texture(scene_depth, screen_uv).r;
        float scene_z  = linearize_depth(scene_d);
        float p_z      = -v_view_pos.z;
        float delta    = scene_z - p_z;
        alpha *= smoothstep(0.0, soft_fade_distance, delta);
    }

    if (alpha < 0.001) discard;
    out_color = vec4(color, alpha);
}
```

- [ ] **Step 10.3: Ribbon shaders**

Create `src/res/shader/particle_ribbon_vert.glsl`:

```glsl
#version 330 core
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

in vec3 position;
in vec2 uv;
in vec4 color;

out vec2 v_uv;
out vec4 v_color;

void main() {
    v_uv = uv;
    v_color = color;
    gl_Position = projection_matrix * view_matrix * vec4(position, 1.0);
}
```

Create `src/res/shader/particle_ribbon_frag.glsl`:

```glsl
#version 330 core
in vec2 v_uv;
in vec4 v_color;

uniform sampler2D albedo;
out vec4 out_color;

void main() {
    vec4 tex = texture(albedo, v_uv);
    vec4 c = vec4(tex.rgb * v_color.rgb, tex.a * v_color.a);
    if (c.a < 0.001) discard;
    out_color = c;
}
```

- [ ] **Step 10.4: Add shader accessors in `src/resources.rs`**

Append:

```rust
pub fn particle_sprite_vert() -> &'static str {
    include_str!("res/shader/particle_sprite_vert.glsl")
}
pub fn particle_sprite_frag() -> &'static str {
    include_str!("res/shader/particle_sprite_frag.glsl")
}
pub fn particle_ribbon_vert() -> &'static str {
    include_str!("res/shader/particle_ribbon_vert.glsl")
}
pub fn particle_ribbon_frag() -> &'static str {
    include_str!("res/shader/particle_ribbon_frag.glsl")
}
```

- [ ] **Step 10.5: Add Material preset factories**

In `src/material.rs`, after the existing `unlit` factory (around line 373), add:

```rust
    pub fn particle_sprite(display: Display<WindowSurface>) -> Self {
        let mut mat = Material::default(
            shader::Shader::from_strings(
                resources::particle_sprite_vert(),
                resources::particle_sprite_frag(),
                None,
            ),
            &display,
        );
        mat.set_transparency(true);
        mat
    }

    pub fn particle_ribbon(display: Display<WindowSurface>) -> Self {
        let mut mat = Material::default(
            shader::Shader::from_strings(
                resources::particle_ribbon_vert(),
                resources::particle_ribbon_frag(),
                None,
            ),
            &display,
        );
        mat.set_transparency(true);
        mat
    }
```

- [ ] **Step 10.6: Verify build**

Run: `cargo check`
Expected: clean compile (no test for shader files yet — they're string includes).

- [ ] **Step 10.7: Commit**

```bash
git add src/res/shader/particle_*.glsl src/resources.rs src/material.rs
git commit -m "feat(particle): GLSL shaders + Material::particle_sprite/ribbon presets"
```

---

## Task 11 — Sprite render path

**Files:**
- Create: `src/particle/render/mod.rs`
- Create: `src/particle/render/sprite.rs`
- Modify: `src/particle/mod.rs`

- [ ] **Step 11.1: Renderer scaffold**

Create `src/particle/render/mod.rs`:

```rust
pub mod sprite;

use std::collections::HashMap;
use glium::{Display, Frame, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::texture::DepthTexture2d;
use uuid::Uuid;
use crate::camera::Camera;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::RenderStyle;

pub struct ParticleRenderer {
    pub sprite_quad_vb: VertexBuffer<sprite::SpriteCorner>,
    pub sprite_quad_ib: IndexBuffer<u16>,
}

impl ParticleRenderer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let corners = vec![
            sprite::SpriteCorner { corner: [-0.5, -0.5, 0.0], corner_uv: [0.0, 0.0] },
            sprite::SpriteCorner { corner: [ 0.5, -0.5, 0.0], corner_uv: [1.0, 0.0] },
            sprite::SpriteCorner { corner: [ 0.5,  0.5, 0.0], corner_uv: [1.0, 1.0] },
            sprite::SpriteCorner { corner: [-0.5,  0.5, 0.0], corner_uv: [0.0, 1.0] },
        ];
        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];
        ParticleRenderer {
            sprite_quad_vb: VertexBuffer::new(display, &corners).unwrap(),
            sprite_quad_ib: IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap(),
        }
    }
}

pub fn render<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    depth_texture: Option<&DepthTexture2d>,
    camera: &Camera,
    systems: &mut [ParticleSystem],
    materials: &HashMap<Uuid, &Material>,
    renderer: &ParticleRenderer,
    screen_size: [f32; 2],
) {
    for system in systems.iter_mut() {
        if system.live_count() == 0 { continue; }
        let mat_id = match system.material_id { Some(id) => id, None => continue };
        let mat = match materials.get(&mat_id) { Some(m) => *m, None => continue };
        match system.config.render {
            RenderStyle::Sprite { .. } => {
                sprite::render_one(display, target, depth_texture, camera, system, mat, renderer, screen_size);
            }
            RenderStyle::Ribbon { .. } => {
                // task 13
            }
        }
    }
}
```

- [ ] **Step 11.2: Sprite per-system render**

Create `src/particle/render/sprite.rs`:

```rust
use glium::{Display, Surface, VertexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::implement_vertex;
use glium::texture::DepthTexture2d;
use crate::camera::Camera;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::{RenderStyle, BlendMode, FlipbookConfig};
use super::ParticleRenderer;

#[derive(Copy, Clone)]
pub struct SpriteCorner {
    pub corner: [f32; 3],
    pub corner_uv: [f32; 2],
}
implement_vertex!(SpriteCorner, corner, corner_uv);

#[derive(Copy, Clone)]
pub struct SpriteInstance {
    pub inst_position:     [f32; 3],
    pub inst_size:         f32,
    pub inst_color:        [f32; 4],
    pub inst_velocity:     [f32; 3],
    pub inst_rotation:     f32,
    pub inst_flipbook_uv:  [f32; 4],
}
implement_vertex!(SpriteInstance, inst_position, inst_size, inst_color, inst_velocity, inst_rotation, inst_flipbook_uv);

fn blend_params(mode: BlendMode) -> glium::Blend {
    use glium::{Blend, BlendingFunction, LinearBlendingFactor::*};
    match mode {
        BlendMode::Additive => Blend {
            color: BlendingFunction::Addition { source: SourceAlpha, destination: One },
            alpha: BlendingFunction::Addition { source: One, destination: One },
            ..Default::default()
        },
        BlendMode::Alpha => Blend::alpha_blending(),
        BlendMode::PremultipliedAlpha => Blend {
            color: BlendingFunction::Addition { source: One, destination: OneMinusSourceAlpha },
            alpha: BlendingFunction::Addition { source: One, destination: OneMinusSourceAlpha },
            ..Default::default()
        },
    }
}

fn flipbook_uv(frame: f32, fb: &FlipbookConfig) -> [f32; 4] {
    let idx = (frame as u32) % fb.frame_count.max(1);
    let col = (idx % fb.cols.max(1)) as f32;
    let row = (idx / fb.cols.max(1)) as f32;
    let w = 1.0 / fb.cols.max(1) as f32;
    let h = 1.0 / fb.rows.max(1) as f32;
    [col * w, 1.0 - (row + 1.0) * h, w, h]
}

pub fn render_one<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    depth_texture: Option<&DepthTexture2d>,
    camera: &Camera,
    system: &ParticleSystem,
    material: &Material,
    renderer: &ParticleRenderer,
    screen_size: [f32; 2],
) {
    let (flipbook, blend_mode, soft_fade, vel_stretch) = match &system.config.render {
        RenderStyle::Sprite { flipbook, blend_mode, soft_particles, soft_fade_distance, velocity_stretch } => {
            let fade = if *soft_particles { *soft_fade_distance } else { 0.0 };
            (flipbook.clone(), *blend_mode, fade, *velocity_stretch)
        }
        _ => return,
    };

    // Build instance data
    let mut instances: Vec<SpriteInstance> = Vec::with_capacity(system.live_count());
    for i in 0..system.live_count() {
        let uv = match &flipbook {
            Some(fb) => flipbook_uv(system.buffer.flipbook_frame[i], fb),
            None => [0.0, 0.0, 1.0, 1.0],
        };
        instances.push(SpriteInstance {
            inst_position: system.buffer.position[i],
            inst_size:     system.buffer.size[i],
            inst_color:    system.buffer.color[i],
            inst_velocity: system.buffer.velocity[i],
            inst_rotation: system.buffer.rotation[i],
            inst_flipbook_uv: uv,
        });
    }
    if instances.is_empty() { return; }

    let instance_vb = match VertexBuffer::dynamic(display, &instances) {
        Ok(vb) => vb,
        Err(_) => return,
    };

    let albedo_ref = material.albedo.as_ref();
    let albedo_tex = match albedo_ref {
        Some(t) => t,
        None => return,
    };
    // Glium SrgbTexture2d sampler — wrap with a uniform sampler
    let albedo_sampler = albedo_tex.sampler();

    // Depth sampler (or 1x1 dummy)
    let dummy_depth = DepthTexture2d::empty(display, 1, 1).unwrap();
    let depth_sampler = depth_texture.unwrap_or(&dummy_depth).sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest);

    let view = camera.get_view_matrix_arrays();
    let proj = camera.get_projection_matrix_arrays(screen_size[0] / screen_size[1]);

    let use_flipbook: i32 = if flipbook.is_some() { 1 } else { 0 };

    let uniforms = glium::uniform! {
        view_matrix: view,
        projection_matrix: proj,
        albedo: albedo_sampler,
        scene_depth: depth_sampler,
        soft_fade_distance: soft_fade,
        use_flipbook: use_flipbook,
        screen_size: screen_size,
        near_plane: camera.near,
        far_plane: camera.far,
        velocity_stretch: vel_stretch,
    };

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: false,
            ..Default::default()
        },
        blend: blend_params(blend_mode),
        ..Default::default()
    };

    let _ = target.draw(
        (&renderer.sprite_quad_vb, instance_vb.per_instance().unwrap()),
        &renderer.sprite_quad_ib,
        &material.program,
        &uniforms,
        &params,
    );
}
```

- [ ] **Step 11.3: Wire render module into mod.rs**

```rust
pub mod render;
```

(append at end of pub-mods)

- [ ] **Step 11.4: Add Texture::sampler() helper if it doesn't exist**

Check `src/texture.rs` for a `pub fn sampler(&self)` or equivalent that returns a glium sampler. If it doesn't exist, add:

```rust
impl Texture {
    pub fn sampler(&self) -> glium::uniforms::Sampler<glium::texture::SrgbTexture2d> {
        self.texture.sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
    }
}
```

Field name (`texture`) may differ — adapt to the actual `Texture` field that holds the `SrgbTexture2d`.

- [ ] **Step 11.5: Verify cargo check passes**

Run: `cargo check`
Expected: clean compile. Likely fix-ups: `Camera::near` / `Camera::far` may have different accessors — check `src/camera.rs`. If `Camera::get_view_matrix_arrays` does not exist, use whichever matrix accessor the engine provides (likely `view_matrix()` or similar returning `[[f32;4];4]`). Adapt names to actual API.

- [ ] **Step 11.6: Commit**

```bash
git add src/particle/render/ src/particle/mod.rs src/texture.rs
git commit -m "feat(particle): sprite render pass with flipbook + soft particles + velocity stretch"
```

---

## Task 12 — Render pass slot-in (EventLoop::run)

**Files:**
- Modify: `src/lib.rs` (`EventLoop::run` to call `tick_particles` + `render` between transparent pass and post-processing)

- [ ] **Step 12.1: Locate the render block in `EventLoop::run`**

Open `src/lib.rs` and locate `impl EventLoop { pub fn run(...) }`. Search for where transparent objects are drawn (likely a loop over `app_state.objects` filtered by `render_transparent`, sorted back-to-front) and where the post-processing chain entry starts. Read 50 lines of surrounding code first — the engine code names matter and will not match the names in the code blocks below verbatim. In particular, identify:

- The framebuffer / `Frame` variable used during scene rendering (likely a `MultiOutputFrameBuffer` or similar).
- The depth texture attached to that framebuffer, if accessible.
- The window/viewport size variable available at that point.
- The camera reference and its accessors (`view_matrix`, `projection_matrix`, etc.) — adapt the calls in Tasks 11 and 13 to match.

This step has no automated test — read carefully and adapt the code blocks below to the real names. Do not hand-edit the code blocks; capture the names first, then transcribe with the correct names.

- [ ] **Step 12.2: Add `tick_particles` call and `render` call**

After the transparent pass closes and before the post-processing chain begins, add:

```rust
// === Particle systems ===
{
    let dt = app_state.delta_time;
    app_state.tick_particles(dt);

    // Render
    let renderer = particle_renderer.get_or_insert_with(|| crate::particle::render::ParticleRenderer::new(&self.display));
    let materials_map: std::collections::HashMap<Uuid, &Material> = app_state.materials.iter()
        .map(|m| (m.uuid, m)).collect();
    if let Some(cam) = &app_state.camera {
        let depth_ref: Option<&glium::texture::DepthTexture2d> = None;  // wire actual depth texture from FBO once available
        crate::particle::render::render(
            &self.display,
            &mut target,
            depth_ref,
            cam,
            &mut app_state.particle_systems,
            &materials_map,
            renderer,
            [window_size.width as f32, window_size.height as f32],
        );
    }
}
```

You will need to:
- Declare `let mut particle_renderer: Option<crate::particle::render::ParticleRenderer> = None;` once outside the event loop body.
- Replace `target` with the actual scene framebuffer variable used by the transparent pass.
- Replace `window_size` with whatever size variable the existing transparent pass uses.
- Wire `depth_ref` to the engine's scene depth attachment if one is accessible at this point; otherwise leave `None` for v1 (soft particles will then be a no-op until depth is wired).

- [ ] **Step 12.3: `cargo check`, fix compile**

Run: `cargo check`
Expected: any name-fix errors (e.g., framebuffer variable name) become clear here.

- [ ] **Step 12.4: Run examples/terrain example to verify nothing crashes**

Run: `cargo run --example=terrain --features=examples` (manual smoke test)
Expected: terrain example runs unchanged; no particles visible since none added.

- [ ] **Step 12.5: Commit**

```bash
git add src/lib.rs
git commit -m "feat(particle): tick + render slot-in EventLoop::run between transparent and post"
```

---

## Task 13 — Ribbon render path + sorting + hero lights

**Files:**
- Create: `src/particle/render/ribbon.rs`
- Modify: `src/particle/render/mod.rs`
- Modify: `src/particle/system.rs` (hero light extraction)

- [ ] **Step 13.1: Ribbon mesh builder + render**

Create `src/particle/render/ribbon.rs`:

```rust
use glium::{Display, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::implement_vertex;
use crate::camera::Camera;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::{RenderStyle, BlendMode};
use super::sprite::blend_params;

#[derive(Copy, Clone)]
pub struct RibbonVertex {
    pub position: [f32; 3],
    pub uv:       [f32; 2],
    pub color:    [f32; 4],
}
implement_vertex!(RibbonVertex, position, uv, color);

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if l < 1e-6 { [0.0, 0.0, 0.0] } else { [v[0]/l, v[1]/l, v[2]/l] }
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

pub fn render_one<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    camera: &Camera,
    system: &ParticleSystem,
    material: &Material,
    screen_size: [f32; 2],
) {
    let (max_pts, width_curve, blend_mode) = match &system.config.render {
        RenderStyle::Ribbon { max_trail_points, width_over_life, blend_mode } => {
            (*max_trail_points, width_over_life.clone(), *blend_mode)
        }
        _ => return,
    };
    let trails = match &system.buffer.trail { Some(t) => t, None => return };
    if trails.is_empty() { return; }

    let cam_pos = camera.position;
    let mut verts: Vec<RibbonVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..system.live_count() {
        let trail = &trails[i];
        if trail.len() < 2 { continue; }
        let color = system.buffer.color[i];
        let base_size = system.buffer.size[i];

        for s in 0..trail.len() - 1 {
            let p0 = trail[s];
            let p1 = trail[s + 1];
            let tangent = normalize([p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]]);
            let view_dir0 = normalize([cam_pos[0]-p0[0], cam_pos[1]-p0[1], cam_pos[2]-p0[2]]);
            let side0 = normalize(cross(tangent, view_dir0));
            let view_dir1 = normalize([cam_pos[0]-p1[0], cam_pos[1]-p1[1], cam_pos[2]-p1[2]]);
            let side1 = normalize(cross(tangent, view_dir1));

            let t0 = s as f32 / (max_pts.max(1) - 1) as f32;
            let t1 = (s as f32 + 1.0) / (max_pts.max(1) - 1) as f32;
            let w0 = base_size * width_curve.eval(t0);
            let w1 = base_size * width_curve.eval(t1);

            let base_idx = verts.len() as u32;
            verts.push(RibbonVertex { position: [p0[0]+side0[0]*w0, p0[1]+side0[1]*w0, p0[2]+side0[2]*w0], uv: [0.0, t0], color });
            verts.push(RibbonVertex { position: [p0[0]-side0[0]*w0, p0[1]-side0[1]*w0, p0[2]-side0[2]*w0], uv: [1.0, t0], color });
            verts.push(RibbonVertex { position: [p1[0]+side1[0]*w1, p1[1]+side1[1]*w1, p1[2]+side1[2]*w1], uv: [0.0, t1], color });
            verts.push(RibbonVertex { position: [p1[0]-side1[0]*w1, p1[1]-side1[1]*w1, p1[2]-side1[2]*w1], uv: [1.0, t1], color });
            indices.extend_from_slice(&[base_idx, base_idx+1, base_idx+2, base_idx+1, base_idx+3, base_idx+2]);
        }
    }
    if verts.is_empty() { return; }

    let vb = match VertexBuffer::dynamic(display, &verts) { Ok(v) => v, Err(_) => return };
    let ib = match IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices) { Ok(i) => i, Err(_) => return };

    let albedo_ref = material.albedo.as_ref();
    let albedo_sampler = match albedo_ref {
        Some(tex) => tex.sampler(),
        None => return,
    };

    let view = camera.get_view_matrix_arrays();
    let proj = camera.get_projection_matrix_arrays(screen_size[0] / screen_size[1]);
    let uniforms = glium::uniform! {
        view_matrix: view,
        projection_matrix: proj,
        albedo: albedo_sampler,
    };
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: false,
            ..Default::default()
        },
        blend: blend_params(blend_mode),
        ..Default::default()
    };
    let _ = target.draw(&vb, &ib, &material.program, &uniforms, &params);
}
```

- [ ] **Step 13.2: Make `blend_params` public in sprite.rs**

In `src/particle/render/sprite.rs`, change `fn blend_params(...)` to `pub fn blend_params(...)`.

- [ ] **Step 13.3: Wire ribbon into `render/mod.rs`**

Add `pub mod ribbon;` and in the match arm:

```rust
RenderStyle::Ribbon { .. } => {
    ribbon::render_one(display, target, camera, system, mat, screen_size);
}
```

- [ ] **Step 13.4: Sorting across systems**

In `render/mod.rs`, before the loop, sort systems by camera distance descending and blend mode (additive first to allow OOI within additive batch):

```rust
let cam_pos = camera.position;
let mut order: Vec<usize> = (0..systems.len()).collect();
order.sort_by(|&a, &b| {
    let pa = systems[a].transform[3];
    let pb = systems[b].transform[3];
    let da = (pa[0]-cam_pos[0]).powi(2) + (pa[1]-cam_pos[1]).powi(2) + (pa[2]-cam_pos[2]).powi(2);
    let db = (pb[0]-cam_pos[0]).powi(2) + (pb[1]-cam_pos[1]).powi(2) + (pb[2]-cam_pos[2]).powi(2);
    let key_a = match systems[a].config.render { RenderStyle::Sprite { blend_mode, .. } | RenderStyle::Ribbon { blend_mode, .. } => blend_mode };
    let key_b = match systems[b].config.render { RenderStyle::Sprite { blend_mode, .. } | RenderStyle::Ribbon { blend_mode, .. } => blend_mode };
    let blend_rank = |m| match m { BlendMode::Additive => 0, BlendMode::PremultipliedAlpha => 1, BlendMode::Alpha => 2 };
    blend_rank(key_a).cmp(&blend_rank(key_b)).then(db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal))
});
```

Then iterate `for idx in order { ... }`. Import `BlendMode` at module top.

- [ ] **Step 13.5: Hero light extraction**

In `src/particle/system.rs`, add method on `ParticleSystem`:

```rust
    pub fn extract_hero_lights(&self) -> Vec<crate::light::Light> {
        let cfg = match &self.config.light_emit { Some(c) => c, None => return Vec::new() };
        let max = cfg.max_lights as usize;
        if max == 0 || self.live_count() == 0 { return Vec::new(); }

        // Score: most-energetic particles (middle of life with current alpha)
        let mut scored: Vec<(usize, f32)> = (0..self.live_count())
            .map(|i| {
                let t = (self.buffer.age[i] / self.buffer.lifetime[i]).clamp(0.0, 1.0);
                let mid = 1.0 - (t - 0.5).abs() * 2.0;
                let score = self.buffer.color[i][3] * mid;
                (i, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(max);

        scored.into_iter().map(|(i, _)| {
            let pos = self.buffer.position[i];
            let color = [
                self.buffer.color[i][0] * cfg.color_multiplier[0],
                self.buffer.color[i][1] * cfg.color_multiplier[1],
                self.buffer.color[i][2] * cfg.color_multiplier[2],
            ];
            crate::light::Light::new(pos, color, cfg.intensity, None, false)
        }).collect()
    }
```

- [ ] **Step 13.6: Plug hero lights into `tick_particles`**

In `AppState::tick_particles`, after the tick loop and before the spawn loop, gather hero lights and append to `self.light` — but only up to the engine's 4-light cap minus any existing scene lights. Add at the end of `tick_particles`:

```rust
        // Hero lights: append up to engine cap (4 total)
        let mut new_lights: Vec<light::Light> = Vec::new();
        for sys in &self.particle_systems {
            new_lights.extend(sys.extract_hero_lights());
        }
        // Replace previously-injected hero lights — for v1 we keep this simple:
        // assume the user-added scene lights live at the front; truncate any prior hero lights.
        // (To enable this, store a marker: `particle_light_start: usize`.)
```

For v1, we skip mutating the scene light list directly to avoid disturbing user lights. Document this as a v2 follow-up: a clean way to inject "ephemeral" lights without mutating `self.light` permanently. Annotate with a comment, no code-change to the existing light list yet.

Mark the limitation in plan note: hero lights are **collected** in v1 but not yet **applied**; full integration is a follow-up that requires a "scratch lights" list in AppState.

- [ ] **Step 13.7: Build + commit**

Run: `cargo check`
Expected: clean.

```bash
git add src/particle/
git commit -m "feat(particle): ribbon render pass + sort + hero light extraction"
```

---

## Task 14 — VFX JSON assets + example program

**Files:**
- Create: `assets/vfx/fireball.json`
- Create: `assets/vfx/fireball_smoke.json`
- Create: `assets/vfx/hit_sparks.json`
- Create: `assets/vfx/spark_dust.json`
- Create: `assets/vfx/magic_orb.json`
- Create: `examples/particles.rs`
- Modify: `Cargo.toml` (verify the example registers; gated by features=examples)

- [ ] **Step 14.1: Create asset directory**

```bash
mkdir -p assets/vfx
```

Add to `.gitignore` if not already covered — but `assets/` should be tracked. Confirm with `git check-ignore -v assets/vfx`.

- [ ] **Step 14.2: Write `assets/vfx/fireball.json`**

(Use the JSON example from the spec, section "JSON schema" → "Fireball with on-death smoke".)

- [ ] **Step 14.3: Write `assets/vfx/fireball_smoke.json`**

```json
{
  "name": "fireball_smoke",
  "max_particles": 32,
  "looped": false,
  "duration": 0.0,
  "bursts": [{ "time": 0.0, "count": 6 }],
  "emission_rate": 0.0,
  "emitter_shape": { "Point": {} },
  "initial_velocity": { "Cone": { "direction": [0,1,0], "angle": 0.6, "speed": { "min": 0.4, "max": 1.0 } } },
  "initial_lifetime": { "min": 0.8, "max": 1.4 },
  "initial_size":     { "min": 0.3, "max": 0.6 },
  "initial_rotation": { "min": 0.0, "max": 6.283 },
  "initial_color": { "Hdr": { "rgb": [0.2, 0.2, 0.2], "intensity": 1.0, "alpha": 0.6 } },
  "color_over_life": { "keys": [[0.0, [0.3,0.3,0.3,0.6]], [1.0, [0.1,0.1,0.1,0.0]]] },
  "size_over_life":  { "keys": [[0.0, 0.5], [1.0, 1.5]] },
  "gravity": [0.0, 0.5, 0.0],
  "drag": 1.2,
  "render": { "kind": "Sprite", "blend_mode": "Alpha", "soft_particles": true, "soft_fade_distance": 0.4 }
}
```

- [ ] **Step 14.4: Write `assets/vfx/hit_sparks.json`**

(Use the JSON example from the spec → "Hit sparks with collision and on-collision dust".)

- [ ] **Step 14.5: Write `assets/vfx/spark_dust.json`**

```json
{
  "name": "spark_dust",
  "max_particles": 8,
  "looped": false,
  "duration": 0.0,
  "bursts": [{ "time": 0.0, "count": 2 }],
  "emission_rate": 0.0,
  "emitter_shape": { "Point": {} },
  "initial_velocity": { "Cone": { "direction": [0,1,0], "angle": 0.8, "speed": { "min": 0.2, "max": 0.6 } } },
  "initial_lifetime": { "min": 0.3, "max": 0.5 },
  "initial_size":     { "min": 0.05, "max": 0.1 },
  "initial_color": { "Hdr": { "rgb": [0.5,0.4,0.3], "intensity": 1.0, "alpha": 0.4 } },
  "color_over_life": { "keys": [[0.0,[0.5,0.4,0.3,0.4]],[1.0,[0.3,0.2,0.1,0.0]]] },
  "gravity": [0,-1.0,0],
  "render": { "kind": "Sprite", "blend_mode": "Alpha" }
}
```

- [ ] **Step 14.6: Write `assets/vfx/magic_orb.json` (ribbon demo)**

```json
{
  "name": "magic_orb",
  "max_particles": 64,
  "looped": true,
  "duration": 0.0,
  "emission_rate": 30.0,
  "emitter_shape": { "Sphere": { "radius": 0.05 } },
  "initial_velocity": { "Outward": { "speed": { "min": 0.5, "max": 1.5 } } },
  "initial_lifetime": { "min": 1.0, "max": 1.8 },
  "initial_size":     { "min": 0.05, "max": 0.1 },
  "initial_color": { "Hdr": { "rgb": [0.3, 0.7, 1.0], "intensity": 8.0, "alpha": 1.0 } },
  "drag": 0.6,
  "curl_noise": { "scale": 1.0, "strength": 1.5, "evolution_speed": 0.3 },
  "attractor": { "center": [0.0, 0.0, 0.0], "strength": 2.0, "falloff": 0.5 },
  "color_over_life": { "keys": [[0.0,[0.3,0.7,1.0,1.0]],[1.0,[0.1,0.2,0.5,0.0]]] },
  "render": {
    "kind": "Ribbon",
    "max_trail_points": 12,
    "width_over_life": { "keys": [[0.0,1.0],[1.0,0.0]] },
    "blend_mode": "Additive"
  }
}
```

- [ ] **Step 14.7: Write `examples/particles.rs`**

Pattern follow the existing `examples/terrain.rs` for camera setup and engine boilerplate. The body of `main()`:

```rust
use enigma_3d::{AppState, EventLoop, particle, material::{Material, TextureType}, init_default};
use enigma_3d::particle::ParticleSystem;

fn main() {
    let event_loop = EventLoop::new("Particles Example", 1280, 720);
    let mut app_state = AppState::new();
    init_default(&mut app_state);
    app_state.set_display(event_loop.display.clone());

    // Materials — for the demo we use placeholder white textures so the example
    // runs without external assets; replace albedo with a real fire/spark texture later.
    let mut fire_mat = Material::particle_sprite(event_loop.display.clone());
    fire_mat.set_name("fire");
    // (set_texture_from_file if a real texture asset exists)
    app_state.add_material(fire_mat);

    let mut ribbon_mat = Material::particle_ribbon(event_loop.display.clone());
    ribbon_mat.set_name("ribbon");
    app_state.add_material(ribbon_mat);

    let fire_id = app_state.materials.iter().find(|m| m.name == "fire").map(|m| m.uuid).unwrap();
    let ribbon_id = app_state.materials.iter().find(|m| m.name == "ribbon").map(|m| m.uuid).unwrap();

    // Sub-emitter templates
    let mut smoke = ParticleSystem::from_file("assets/vfx/fireball_smoke.json").unwrap();
    smoke.set_material(fire_id);
    app_state.add_particle_system_template(smoke);

    let mut dust = ParticleSystem::from_file("assets/vfx/spark_dust.json").unwrap();
    dust.set_material(fire_id);
    app_state.add_particle_system_template(dust);

    // Active fireball
    let mut fireball = ParticleSystem::from_file("assets/vfx/fireball.json").unwrap();
    fireball.set_material(fire_id);
    fireball.set_position([0.0, 2.0, 0.0]);
    app_state.add_particle_system(fireball);

    // Magic orb (ribbon)
    let mut orb = ParticleSystem::from_file("assets/vfx/magic_orb.json").unwrap();
    orb.set_material(ribbon_id);
    orb.set_position([-3.0, 2.0, 0.0]);
    app_state.add_particle_system(orb);

    // Hit-sparks template, triggered on click
    let mut sparks = ParticleSystem::from_file("assets/vfx/hit_sparks.json").unwrap();
    sparks.set_material(ribbon_id);
    app_state.add_particle_system_template(sparks);

    // Click-to-spawn handler
    fn on_click(app: &mut AppState) {
        let xform = [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[2.0,1.0,0.0,1.0]];
        let _ = app.emit_particle_system("hit_sparks", xform);
    }
    app_state.inject_event(
        enigma_3d::event::EventCharacteristic::MousePress(winit::event::MouseButton::Left),
        on_click,
        None,
    );

    // egui counter
    app_state.inject_gui(|ctx, app| {
        egui::Window::new("Particles").show(ctx, |ui| {
            let total: usize = app.particle_systems.iter().map(|s| s.live_count()).sum();
            ui.label(format!("Active systems: {}", app.particle_systems.len()));
            ui.label(format!("Total live particles: {}", total));
        });
    });

    event_loop.run(app_state.convert_to_arc_mutex());
}
```

The example will need API-shape fixups: `EventLoop::new` signature, `init_default`, `convert_to_arc_mutex`, and `inject_gui` API may differ — copy from `examples/terrain.rs` for current usage. Click-to-spawn at `[2,1,0]` is a placeholder until ray-pick is wired.

- [ ] **Step 14.8: Build the example**

Run: `cargo build --example=particles --features=examples`
Expected: success.

Run: `cargo run --example=particles --features=examples`
Expected: window opens; fireball emits HDR particles at `[0,2,0]`; magic orb swirls with ribbon trails at `[-3,2,0]`; left-click spawns sparks burst; on-screen GUI shows counts.

If textures aren't authored, sprites will draw as solid-color quads — that's fine for the smoke test. Real texture assets are a follow-up.

- [ ] **Step 14.9: Commit**

```bash
git add assets/vfx/ examples/particles.rs
git commit -m "feat(particle): example program + VFX JSON assets for fireball/orb/sparks"
```

---

## Task 15 — Final integration polish

**Files:**
- Modify: `Cargo.toml` (include rule for assets if needed)
- Modify: `CLAUDE.md` (add "Particle system" section)

- [ ] **Step 15.1: Add docs to CLAUDE.md**

Add a section to `CLAUDE.md` under "Key modules":

```markdown
| `particle/` | CPU-simulated particle system (sprite + ribbon, JSON-authorable); update tick runs between transparent and post-processing passes |
```

And a new section:

```markdown
### Particle system

CPU-simulated, Shuriken-style fixed pipeline. Systems are JSON-authorable (`ParticleSystem::from_file` / `from_resource`) and registered on `AppState` via `add_particle_system` / `add_particle_system_template`. Sub-emitters reference templates by name. Materials assigned via UUID, with `Material::particle_sprite(display)` and `Material::particle_ribbon(display)` as preset factories. See `docs/superpowers/specs/2026-05-24-particle-system-design.md` for the full design and `examples/particles.rs` for usage.

Known v1 limitations:
- Hero per-particle lights are extracted but not yet injected into the engine's 4-light cap (follow-up: add a scratch-lights list).
- Terrain heightfield collision uses a stub `heightfield_query_fn` returning `None`; ground-plane collision works fully.
- Soft particles require the scene depth texture to be threaded into `ParticleRenderer::render`; if `None`, soft fade is a no-op.
```

- [ ] **Step 15.2: Run all tests one more time**

Run: `cargo test --lib`
Expected: all particle tests pass; no regressions in existing tests.

- [ ] **Step 15.3: Final commit**

```bash
git add CLAUDE.md
git commit -m "docs(particle): add particle system section to CLAUDE.md"
```

---

## Done — verification

After all tasks complete:

- [ ] `cargo build` clean
- [ ] `cargo test --lib` passes (all new tests + no regressions)
- [ ] `cargo run --example=particles --features=examples` shows fireball, magic orb, and click-to-spawn sparks
- [ ] `cargo run --example=terrain --features=examples` still works (no regressions)

## Follow-ups (v2)

These are intentionally deferred:

- Hero-light injection (the "scratch lights" list in AppState)
- Terrain heightfield collision query (replace `heightfield_query_fn` stub)
- Soft-particles depth wiring (depth texture access in `EventLoop::run`)
- Ray-pick for click-to-spawn at the actual world position
- Real texture authoring for fire atlas + spark ribbon
- Criterion benchmarks for the 10k-particle update path
- `AppStateSerializer` integration (save/load running particle systems)
- Mesh particles, GPU simulation, decals, OIT — all v2+
- Cycle detection on sub-emitter templates (currently relies on JSON tree being acyclic; with named refs, A→B→A is possible at registration — detect and reject)
