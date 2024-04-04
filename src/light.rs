use glium::implement_uniform_block;
use serde::{Deserialize, Serialize};

pub enum LightEmissionType {
    Source,
    Ambient,
}

#[derive(Serialize, Deserialize)]
pub struct LightSerializer {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub direction: [f32; 3],
    pub cast_shadow: bool,
}

#[derive(Copy, Clone)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub direction: [f32; 3],
    pub cast_shadow: bool
}

impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32, direction: Option<[f32;3]>, cast_shadow: bool) -> Self {
        Self {
            position,
            color,
            intensity,
            direction : direction.unwrap_or_else(|| [0.0, 0.0, 0.0]),
            cast_shadow,
        }
    }

    pub fn is_directional(&self) -> bool {
        self.direction != [0.0, 0.0, 0.0]
    }

    pub fn from_serializer(serializer: LightSerializer) -> Self {
        Self {
            position: serializer.position,
            color: serializer.color,
            intensity: serializer.intensity,
            direction: serializer.direction,
            cast_shadow: serializer.cast_shadow,
        }
    }

    pub fn to_serializer(&self) -> LightSerializer {
        LightSerializer {
            position: self.position,
            color: self.color,
            intensity: self.intensity,
            direction: self.direction,
            cast_shadow: self.cast_shadow,
        }
    }
}

pub struct LightBlock {
    pub position: [[f32; 4]; 4],
    pub directions: [[f32; 4]; 4],
    pub color: [[f32; 4]; 4],
    pub intensity: [f32; 4],
    pub cast_shadow: [i32; 4],
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

glium::implement_uniform_block!(Light, position, color, intensity, direction, cast_shadow);
glium::implement_uniform_block!(LightBlock, position, directions, cast_shadow, color, intensity, amount, ambient_color, ambient_intensity);