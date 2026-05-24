use serde::{Deserialize, Serialize};
use crate::particle::Range;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EmitterShape {
    Point,
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    Cone { angle: f32, height: f32 },
    Disk { radius: f32 },
}

impl EmitterShape {
    pub fn sample_position(&self, rng: &mut impl rand::Rng) -> [f32; 3] {
        match *self {
            EmitterShape::Point => [0.0, 0.0, 0.0],
            EmitterShape::Sphere { radius } => {
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
    let cos_a = angle.cos();
    let z: f32 = rng.gen_range(cos_a..=1.0);
    let phi: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
    let r = (1.0 - z * z).sqrt();
    let local = [r * phi.cos(), r * phi.sin(), z];
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
                let dir = normalize(*spawn_pos);
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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
        let s = EmitterShape::Point;
        let mut r = rng();
        let p = s.sample_position(&mut r);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn outward_velocity_zero_at_origin() {
        let v = InitialVelocity::Outward { speed: Range::new(1.0, 1.0) };
        let mut r = rng();
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
