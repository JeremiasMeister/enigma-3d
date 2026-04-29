use std::any::{Any, TypeId};
use std::collections::HashMap;
use glium::implement_uniform_block;
use serde::{Deserialize, Serialize};

pub enum LightEmissionType {
    Source,
    Ambient,
}

pub enum ShadowResolution {
    Low,
    Medium,
    High,
    Ultra,
    Custom(u32),
}

impl ShadowResolution {
    pub fn value(&self) -> u32 {
        match self {
            ShadowResolution::Low => 512,
            ShadowResolution::Medium => 1024,
            ShadowResolution::High => 2048,
            ShadowResolution::Ultra => 4096,
            ShadowResolution::Custom(v) => *v,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LightSerializer {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub direction: [f32; 3],
    pub cast_shadow: bool,
}

pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub direction: [f32; 3],
    pub cast_shadow: bool,
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl Clone for Light {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            color: self.color,
            intensity: self.intensity,
            direction: self.direction,
            cast_shadow: self.cast_shadow,
            components: HashMap::new(),
        }
    }
}

impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32, direction: Option<[f32;3]>, cast_shadow: bool) -> Self {
        Self {
            position,
            color,
            intensity,
            direction : direction.unwrap_or_else(|| [0.0, 0.0, 0.0]),
            cast_shadow,
            components: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1.0, None, true)
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
            components: HashMap::new(),
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

    /// Stores `component`, replacing any existing component of the same type.
    pub fn set_component<T: Any + 'static>(&mut self, component: T) {
        self.components.insert(TypeId::of::<T>(), Box::new(component));
    }

    pub fn get_component<T: Any + 'static>(&self) -> Option<&T> {
        self.components.get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }

    pub fn get_component_mut<T: Any + 'static>(&mut self) -> Option<&mut T> {
        self.components.get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T>())
    }

    pub fn has_component<T: Any + 'static>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn remove_component<T: Any + 'static>(&mut self) -> Option<T> {
        self.components
            .remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_resolution_values() {
        assert_eq!(ShadowResolution::Low.value(), 512);
        assert_eq!(ShadowResolution::Medium.value(), 1024);
        assert_eq!(ShadowResolution::High.value(), 2048);
        assert_eq!(ShadowResolution::Ultra.value(), 4096);
        assert_eq!(ShadowResolution::Custom(333).value(), 333);
    }

    fn test_light() -> Light {
        Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1.0, None, false)
    }

    #[test]
    fn light_component_set_and_get() {
        let mut l = test_light();
        l.set_component(42u32);
        assert_eq!(l.get_component::<u32>(), Some(&42u32));
    }

    #[test]
    fn light_component_remove_returns_value() {
        let mut l = test_light();
        l.set_component(7u32);
        assert_eq!(l.remove_component::<u32>(), Some(7u32));
        assert_eq!(l.get_component::<u32>(), None);
    }

    #[test]
    fn light_component_clone_isolation() {
        let mut l = test_light();
        l.set_component(99u32);
        let cloned = l.clone();
        assert_eq!(cloned.get_component::<u32>(), None);
        assert_eq!(l.get_component::<u32>(), Some(&99u32));
    }
}