pub struct Camera {
    position: [f32; 3],
    rotation: [f32; 3],
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(position: Option<[f32; 3]>, rotation: Option<[f32; 3]>, fov: Option<f32>, aspect: Option<f32>, near: Option<f32>, far: Option<f32>) -> Self {
        Self {
            position: position.unwrap_or_else(|| [0.0, 0.0, 0.0]),
            rotation: rotation.unwrap_or_else(|| [0.0, 0.0, 0.0]),
            fov: fov.unwrap_or_else(|| 90.0),
            aspect: aspect.unwrap_or_else(|| 1.0),
            near: near.unwrap_or_else(|| 0.1),
            far: far.unwrap_or_else(|| 100.0),
            view: [[0.0; 4]; 4],
            projection: [[0.0; 4]; 4],
        }
    }

    pub fn get_position(&self) -> [f32; 3] {
        self.position
    }

    pub fn get_rotation(&self) -> [f32; 3] {
        self.rotation
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn get_aspect(&self) -> f32 {
        self.aspect
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn get_view(&self) -> [[f32; 4]; 4] {
        self.view
    }

    pub fn get_projection(&self) -> [[f32; 4]; 4] {
        self.projection
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: [f32; 3]) {
        self.rotation = rotation;
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }
}