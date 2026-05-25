use serde::{Deserialize, Serialize};

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
    /// Returns `Some(normal)` on hit (ground plane is always +Y), `None` otherwise.
    pub fn test_and_resolve_ground(&self, pos: &mut [f32; 3], vel: &mut [f32; 3]) -> Option<[f32; 3]> {
        if let Some(y) = self.ground_plane_y {
            if pos[1] < y {
                pos[1] = y;
                if vel[1] < 0.0 { vel[1] = -vel[1] * self.bounce; }
                vel[0] *= 1.0 - self.friction;
                vel[2] *= 1.0 - self.friction;
                return Some([0.0, 1.0, 0.0]);
            }
        }
        None
    }

    /// Returns `Some(normal)` on hit, derived from a finite-difference gradient of the
    /// height field; `None` otherwise. Step size is chosen small enough for smooth
    /// terrains but is the same regardless of mesh density.
    pub fn test_and_resolve_terrain(
        &self,
        pos: &mut [f32; 3],
        vel: &mut [f32; 3],
        height_at: impl Fn(f32, f32) -> Option<f32>,
    ) -> Option<[f32; 3]> {
        if !self.terrain { return None; }
        let h = height_at(pos[0], pos[2])?;
        if pos[1] >= h { return None; }
        pos[1] = h;
        if vel[1] < 0.0 { vel[1] = -vel[1] * self.bounce; }
        vel[0] *= 1.0 - self.friction;
        vel[2] *= 1.0 - self.friction;
        // Central difference: dh/dx and dh/dz. Surface normal in world space is
        // (-dh/dx, 1, -dh/dz), normalized. Fall back to +Y if neighbours are off-map.
        let eps = 0.1_f32;
        let dx = match (height_at(pos[0] + eps, pos[2]), height_at(pos[0] - eps, pos[2])) {
            (Some(hp), Some(hm)) => (hp - hm) / (2.0 * eps),
            _ => 0.0,
        };
        let dz = match (height_at(pos[0], pos[2] + eps), height_at(pos[0], pos[2] - eps)) {
            (Some(hp), Some(hm)) => (hp - hm) / (2.0 * eps),
            _ => 0.0,
        };
        let n = [-dx, 1.0, -dz];
        let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
        if len < 1e-6 { Some([0.0, 1.0, 0.0]) } else { Some([n[0]/len, n[1]/len, n[2]/len]) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_plane_reflects_below() {
        let c = CollisionConfig { ground_plane_y: Some(0.0), terrain: false, bounce: 0.5, friction: 0.0 };
        let mut pos = [0.0, -0.1, 0.0];
        let mut vel = [1.0, -2.0, 0.0];
        let hit = c.test_and_resolve_ground(&mut pos, &mut vel);
        assert_eq!(hit, Some([0.0, 1.0, 0.0]));
        assert!(pos[1] >= 0.0);
        assert!(vel[1] > 0.0);
        assert!((vel[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ground_plane_passes_above() {
        let c = CollisionConfig { ground_plane_y: Some(0.0), terrain: false, bounce: 0.5, friction: 0.0 };
        let mut pos = [0.0, 1.0, 0.0];
        let mut vel = [1.0, -2.0, 0.0];
        let hit = c.test_and_resolve_ground(&mut pos, &mut vel);
        assert!(hit.is_none());
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

    #[test]
    fn terrain_slope_returns_tilted_normal() {
        // h(x,z) = 0.5 * x  → constant slope of 0.5 along +X, surface normal tilts toward -X.
        let c = CollisionConfig { ground_plane_y: None, terrain: true, bounce: 0.5, friction: 0.0 };
        let height_at = |x: f32, _z: f32| Some(0.5 * x);
        let mut pos = [2.0, 0.5, 0.0];   // below h(2)=1.0 → hit
        let mut vel = [0.0, -1.0, 0.0];
        let n = c.test_and_resolve_terrain(&mut pos, &mut vel, height_at).expect("hit");
        // Expected normal (after normalization) of plane y = 0.5x is (-0.5, 1, 0)/sqrt(1.25).
        let expected = {
            let l = (0.25_f32 + 1.0).sqrt();
            [-0.5 / l, 1.0 / l, 0.0]
        };
        assert!((n[0] - expected[0]).abs() < 1e-3, "nx: got {}, expected {}", n[0], expected[0]);
        assert!((n[1] - expected[1]).abs() < 1e-3, "ny: got {}, expected {}", n[1], expected[1]);
        assert!(n[2].abs() < 1e-3, "nz: got {}, expected 0", n[2]);
    }
}
