use glium::implement_uniform_block;

#[derive(Copy, Clone)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

glium::implement_uniform_block!(Light, position, color, intensity);
