use serde::{Deserialize, Serialize};

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
        // Evolve the field isotropically by offsetting all three sample axes with
        // distinct phase factors. Using the same offset on every axis would just
        // translate the field along the diagonal; distinct factors break that
        // degeneracy and give the visual impression of an unfolding 4D field.
        let off = self.evolution_speed * time;
        let p = [
            pos[0] * s + off,
            pos[1] * s + off * 1.31,
            pos[2] * s + off * 0.73,
        ];
        let eps = 0.1_f32;
        let p_x_plus  = potential([p[0]+eps, p[1], p[2]]);
        let p_x_minus = potential([p[0]-eps, p[1], p[2]]);
        let p_y_plus  = potential([p[0], p[1]+eps, p[2]]);
        let p_y_minus = potential([p[0], p[1]-eps, p[2]]);
        let p_z_plus  = potential([p[0], p[1], p[2]+eps]);
        let p_z_minus = potential([p[0], p[1], p[2]-eps]);
        // curl P = (dPz/dy - dPy/dz, dPx/dz - dPz/dx, dPy/dx - dPx/dy); divergence-free by construction.
        let dpz_dy = (p_y_plus[2] - p_y_minus[2]) / (2.0 * eps);  // dPz/dy
        let dpy_dz = (p_z_plus[1] - p_z_minus[1]) / (2.0 * eps);  // dPy/dz
        let dpx_dz = (p_z_plus[0] - p_z_minus[0]) / (2.0 * eps);  // dPx/dz
        let dpz_dx = (p_x_plus[2] - p_x_minus[2]) / (2.0 * eps);  // dPz/dx
        let dpy_dx = (p_x_plus[1] - p_x_minus[1]) / (2.0 * eps);  // dPy/dx
        let dpx_dy = (p_y_plus[0] - p_y_minus[0]) / (2.0 * eps);  // dPx/dy
        [
            (dpz_dy - dpy_dz) * self.strength,
            (dpx_dz - dpz_dx) * self.strength,
            (dpy_dx - dpx_dy) * self.strength,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_decays_exponentially() {
        let mut v = [10.0, 0.0, 0.0];
        let drag = 2.0;
        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            apply_drag(&mut v, drag, dt);
        }
        let expected = 10.0 * (-2.0_f32).exp();
        assert!((v[0] - expected).abs() < 0.01, "drag decay: got {} expected {}", v[0], expected);
    }

    #[test]
    fn vortex_creates_perpendicular_force() {
        let v = VortexConfig { center: [0.0, 0.0, 0.0], axis: [0.0, 1.0, 0.0], strength: 5.0, falloff: 1.0 };
        let f = v.force(&[1.0, 0.0, 0.0]);
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
