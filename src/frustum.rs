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

    /// Tests whether an AABB is at least partially inside the frustum.
    /// Returns `false` only when the box is fully outside one of the six planes
    /// (conservative — may return `true` for boxes outside the corner region of
    /// two planes, which is acceptable for culling).
    pub fn contains_aabb(&self, aabb: &BoundingBox) -> bool {
        let min = aabb.min_point();
        let max = aabb.max_point();
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;

        for plane in &self.planes {
            // Projected radius of the AABB onto the plane normal.
            let r = half.x * plane.normal.x.abs()
                + half.y * plane.normal.y.abs()
                + half.z * plane.normal.z.abs();
            // Signed distance from center to plane (positive = inside half-space).
            // If center+r is still negative, the entire box is on the outside.
            if plane.normal.dot(&center) + plane.offset + r < 0.0 {
                return false;
            }
        }
        true
    }
}

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
}
