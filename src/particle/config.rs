use serde::{Deserialize, Serialize};
use crate::particle::{Range, BurstSpec, Curve, Gradient, EmitterShape, InitialVelocity, ColorRange, VortexConfig, AttractorConfig, CurlNoiseConfig, CollisionConfig};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum BlendMode { Additive, Alpha, PremultipliedAlpha }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlipbookConfig {
    pub cols: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub fps: f32,
    #[serde(default)]
    pub blend: bool,
    /// If true, each spawned particle starts on a random frame instead of frame 0.
    /// Useful for desynchronizing many particles sharing an animated atlas (embers, smoke).
    #[serde(default)]
    pub randomize_start_frame: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LightEmitConfig {
    pub max_lights: u32,
    pub color_multiplier: [f32; 3],
    pub intensity: f32,
    pub falloff: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum RenderStyle {
    Sprite {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        flipbook: Option<FlipbookConfig>,
        blend_mode: BlendMode,
        #[serde(default)]
        soft_particles: bool,
        #[serde(default)]
        soft_fade_distance: f32,
        #[serde(default)]
        velocity_stretch: f32,
    },
    Ribbon {
        max_trail_points: usize,
        width_over_life: Curve<f32>,
        blend_mode: BlendMode,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParticleSystemConfig {
    pub name: String,
    pub max_particles: usize,
    pub looped: bool,
    pub duration: f32,
    #[serde(default)]
    pub prewarm: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<u64>,

    pub emission_rate: f32,
    #[serde(default)]
    pub bursts: Vec<BurstSpec>,

    pub emitter_shape: EmitterShape,
    pub initial_velocity: InitialVelocity,
    pub initial_lifetime: Range<f32>,
    pub initial_size: Range<f32>,
    pub initial_color: ColorRange,
    #[serde(default = "default_rotation_range")]
    pub initial_rotation: Range<f32>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color_over_life: Option<Gradient<[f32; 4]>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size_over_life: Option<Curve<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotation_over_life: Option<Curve<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub angular_velocity: Option<Range<f32>>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gravity: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub drag: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub curl_noise: Option<CurlNoiseConfig>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vortex: Option<VortexConfig>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub attractor: Option<AttractorConfig>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub collision: Option<CollisionConfig>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub on_death: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub on_collision: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub light_emit: Option<LightEmitConfig>,

    pub render: RenderStyle,
}

fn default_rotation_range() -> Range<f32> { Range::new(0.0, 0.0) }

impl ParticleSystemConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_particles == 0 { return Err("max_particles must be > 0".into()); }
        if self.duration < 0.0 { return Err("duration must be >= 0".into()); }
        if self.initial_lifetime.min <= 0.0 {
            return Err("initial_lifetime.min must be > 0".into());
        }
        if self.initial_lifetime.min > self.initial_lifetime.max {
            return Err("initial_lifetime: min > max".into());
        }
        if self.initial_size.min > self.initial_size.max {
            return Err("initial_size: min > max".into());
        }
        if let Some(g) = &self.color_over_life {
            crate::particle::curves::validate_keyframes(&g.keys).map_err(|e| format!("color_over_life: {e}"))?;
        }
        if let Some(c) = &self.size_over_life {
            crate::particle::curves::validate_keyframes(&c.keys).map_err(|e| format!("size_over_life: {e}"))?;
        }
        if let Some(c) = &self.rotation_over_life {
            crate::particle::curves::validate_keyframes(&c.keys).map_err(|e| format!("rotation_over_life: {e}"))?;
        }
        if let RenderStyle::Ribbon { ref width_over_life, .. } = self.render {
            crate::particle::curves::validate_keyframes(&width_over_life.keys).map_err(|e| format!("width_over_life: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particle::*;

    #[test]
    fn config_minimal_roundtrip() {
        let c = ParticleSystemConfig {
            name: "test".into(),
            max_particles: 64,
            looped: true,
            duration: 1.0,
            prewarm: false,
            seed: None,
            emission_rate: 10.0,
            bursts: vec![],
            emitter_shape: EmitterShape::Point,
            initial_velocity: InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(1.0, 2.0) },
            initial_lifetime: Range::new(0.5, 1.0),
            initial_size: Range::new(0.1, 0.2),
            initial_color: ColorRange::Single([1.0, 1.0, 1.0, 1.0]),
            initial_rotation: Range::new(0.0, 0.0),
            color_over_life: None,
            size_over_life: None,
            rotation_over_life: None,
            angular_velocity: None,
            gravity: None,
            drag: None,
            curl_noise: None,
            vortex: None,
            attractor: None,
            collision: None,
            on_death: None,
            on_collision: None,
            light_emit: None,
            render: RenderStyle::Sprite {
                flipbook: None,
                blend_mode: BlendMode::Additive,
                soft_particles: false,
                soft_fade_distance: 0.0,
                velocity_stretch: 0.0,
            },
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: ParticleSystemConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
        assert_eq!(back.max_particles, 64);
    }

    #[test]
    fn config_validate_rejects_zero_capacity() {
        let mut c = minimal_config();
        c.max_particles = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_validate_rejects_negative_lifetime() {
        let mut c = minimal_config();
        c.initial_lifetime = Range::new(0.0, 1.0);
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_validate_accepts_good_config() {
        assert!(minimal_config().validate().is_ok());
    }

    fn minimal_config() -> ParticleSystemConfig {
        ParticleSystemConfig {
            name: "t".into(),
            max_particles: 8,
            looped: false,
            duration: 1.0,
            prewarm: false,
            seed: None,
            emission_rate: 0.0,
            bursts: vec![BurstSpec { time: 0.0, count: 4 }],
            emitter_shape: EmitterShape::Point,
            initial_velocity: InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(1.0, 1.0) },
            initial_lifetime: Range::new(0.5, 1.0),
            initial_size: Range::new(0.1, 0.2),
            initial_color: ColorRange::Single([1.0, 1.0, 1.0, 1.0]),
            initial_rotation: Range::new(0.0, 0.0),
            color_over_life: None,
            size_over_life: None,
            rotation_over_life: None,
            angular_velocity: None,
            gravity: None,
            drag: None,
            curl_noise: None,
            vortex: None,
            attractor: None,
            collision: None,
            on_death: None,
            on_collision: None,
            light_emit: None,
            render: RenderStyle::Sprite {
                flipbook: None,
                blend_mode: BlendMode::Alpha,
                soft_particles: false,
                soft_fade_distance: 0.0,
                velocity_stretch: 0.0,
            },
        }
    }
}
