use crate::object::{Transform, TransformSerializer};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CameraSerializer {
    transform: TransformSerializer,
    fov: f32,
    width: f32,
    height: f32,
    near: f32,
    far: f32,
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}


impl Camera {
    pub fn new(
        position: Option<[f32; 3]>,
        rotation: Option<[f32; 3]>, // Expected in degrees, will be converted to radians
        fov: Option<f32>, // Expected in degrees, will be converted to radians
        aspect: Option<f32>,
        near: Option<f32>,
        far: Option<f32>
    ) -> Self {
        let mut c = Self {
            transform: {
                let mut t = Transform::new();
                t.set_position(position.unwrap_or_else(|| [0.0, 0.0, 0.0]));
                t.set_rotation(rotation.unwrap_or_else(|| [0.0, 0.0, 0.0]));
                t
            },
            fov: fov.unwrap_or_else(|| 90.0).to_radians(),
            width: aspect.unwrap_or_else(|| 1920.0),
            height: aspect.unwrap_or_else(|| 1080.0),
            near: near.unwrap_or_else(|| 0.1),
            far: far.unwrap_or_else(|| 1024.0),
            view: [[0.0; 4]; 4],
            projection: [[0.0; 4]; 4],
        };
        c.update_matrices();
        c
    }

    pub fn from_serializer(serializer: CameraSerializer) -> Self {
        Self {
            transform: Transform::from_serializer(serializer.transform),
            fov: serializer.fov,
            width: serializer.width,
            height: serializer.height,
            near: serializer.near,
            far: serializer.far,
            view: serializer.view,
            projection: serializer.projection,
        }
    }

    pub fn to_serializer(&self) -> CameraSerializer {
        CameraSerializer {
            transform: self.transform.to_serializer(),
            fov: self.fov,
            width: self.width,
            height: self.height,
            near: self.near,
            far: self.far,
            view: self.view,
            projection: self.projection,
        }
    }

    pub fn update_matrices(&mut self) {
        self.view = Camera::view_matrix(
            &self.transform.get_position().into(),
            &self.calculate_direction_vector(),
            &[0.0, 1.0, 0.0],
        );
        self.projection = Camera::projection_matrix(
            self.fov,
            self.width / self.height,
            self.near,
            self.far,
        );
    }

    pub fn calculate_direction_vector(&self) -> [f32; 3] {
        let pitch = self.transform.rotation[0]; // Rotation around X-axis
        let yaw = self.transform.rotation[1];   // Rotation around Y-axis

        let x = yaw.sin() * pitch.cos();
        let y = pitch.sin();
        let z = yaw.cos() * pitch.cos();

        [-x, y, -z] // Pointing down negative Z-axis
    }


    fn projection_matrix(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
        let f = 1.0 / (fov / 2.0).tan();
        [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), -1.0],
            [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
        ]
    }

    fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
        let f = {
            let len = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
            [-direction[0] / len, -direction[1] / len, -direction[2] / len] // Negate direction for a right-handed system
        };

        let s = [
            up[1] * f[2] - up[2] * f[1],
            up[2] * f[0] - up[0] * f[2],
            up[0] * f[1] - up[1] * f[0],
        ];
        let s_norm = {
            let len = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
            [s[0] / len, s[1] / len, s[2] / len]
        };

        let u = [
            f[1] * s_norm[2] - f[2] * s_norm[1],
            f[2] * s_norm[0] - f[0] * s_norm[2],
            f[0] * s_norm[1] - f[1] * s_norm[0],
        ];

        let p = [
            -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
            -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
            -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
        ];

        [
            [s_norm[0], u[0], f[0], 0.0],
            [s_norm[1], u[1], f[1], 0.0],
            [s_norm[2], u[2], f[2], 0.0],
            [p[0], p[1], p[2], 1.0],
        ]
    }

    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        Camera::view_matrix(
            &self.transform.get_position().into(),
            &self.calculate_direction_vector(),
            &[0.0, 1.0, 0.0],
        )
    }

    pub fn get_projection_matrix(&self) -> [[f32; 4]; 4] {
        Camera::projection_matrix(
            self.fov,
            self.width / self.height,
            self.near,
            self.far,
        )
    }

    pub fn get_position(&self) -> [f32; 3] {
        self.transform.get_position().into()
    }

    pub fn get_rotation(&self) -> [f32; 3] {
        self.transform.get_rotation().into()
    }

    pub fn get_fov(&self) -> f32 {
        self.fov.clone()
    }

    pub fn get_aspect(&self) -> (f32, f32) {
        (self.width.clone(), self.height.clone())
    }

    pub fn get_near(&self) -> f32 {
        self.near.clone()
    }

    pub fn get_far(&self) -> f32 {
        self.far.clone()
    }

    pub fn get_view(&self) -> [[f32; 4]; 4] {
        self.view.clone()
    }

    pub fn get_projection(&self) -> [[f32; 4]; 4] {
        self.projection.clone()
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.transform.set_position(position);
        self.update_matrices();
    }

    pub fn set_rotation(&mut self, rotation: [f32; 3]) {
        self.transform.set_rotation(rotation);
        self.update_matrices();
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.update_matrices();
    }

    pub fn set_aspect(&mut self, width: f32, heigth: f32) {
        self.width = width;
        self.height = heigth;
        self.update_matrices();
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
        self.update_matrices();
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
        self.update_matrices();
    }
}