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
        assert!((vel[1] - 1.0).abs() < 1e-5);
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
