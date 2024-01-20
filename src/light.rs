use glium::implement_uniform_block;

pub enum LightType {
    Point,
    Ambient,
}

#[derive(Copy, Clone)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

pub struct LightBlock {
    pub position: [[f32; 4]; 4],
    pub color: [[f32; 4]; 4],
    pub intensity: [f32; 4],
    pub amount: i32,
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
}

impl std::fmt::Debug for LightBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LightBlock")
            .field("position", &self.position)
            .field("color", &self.color)
            .field("intensity", &self.intensity)
            .field("amount", &self.amount)
            .field("ambient_color", &self.ambient_color)
            .finish()
    }
}

glium::implement_uniform_block!(Light, position, color, intensity);
glium::implement_uniform_block!(LightBlock, position, color, intensity, amount, ambient_color, ambient_intensity);