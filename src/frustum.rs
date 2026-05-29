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
}
