use std::fs;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use uuid::Uuid;
use crate::particle::*;
use crate::particle::config::*;
use crate::logging::EnigmaError;

pub struct EmissionState {
    pub elapsed: f32,
    pub spawn_accumulator: f32,
    pub pending_collision_events: Vec<CollisionEvent>,
    pub finished: bool,
    pub stopped: bool,
}

impl Default for EmissionState {
    fn default() -> Self {
        EmissionState {
            elapsed: 0.0,
            spawn_accumulator: 0.0,
            pending_collision_events: Vec::new(),
            finished: false,
            stopped: false,
        }
    }
}

pub struct ParticleSystem {
    pub handle: Uuid,
    pub config: ParticleSystemConfig,
    pub buffer: ParticleBuffer,
    pub transform: [[f32; 4]; 4],
    pub parent_object: Option<Uuid>,
    pub local_offset: [[f32; 4]; 4],
    pub material_id: Option<Uuid>,
    pub state: EmissionState,
    pub rng: SmallRng,
}

fn identity4() -> [[f32; 4]; 4] {
    [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]]
}

fn translate4(t: [f32; 3]) -> [[f32; 4]; 4] {
    [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[t[0],t[1],t[2],1.0]]
}

fn default_config(name: &str) -> ParticleSystemConfig {
    ParticleSystemConfig {
        name: name.into(),
        max_particles: 64,
        looped: false,
        duration: 1.0,
        prewarm: false,
        seed: None,
        emission_rate: 0.0,
        bursts: Vec::new(),
        emitter_shape: EmitterShape::Point,
        initial_velocity: InitialVelocity::Direction { direction: [0.0,1.0,0.0], speed: Range::new(1.0, 1.0) },
        initial_lifetime: Range::new(0.5, 1.0),
        initial_size: Range::new(0.1, 0.2),
        initial_color: ColorRange::Single([1.0,1.0,1.0,1.0]),
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

impl ParticleSystem {
    pub fn new(name: &str) -> Self {
        let config = default_config(name);
        Self::from_config_internal(config)
    }

    pub fn from_file(path: &str) -> Result<Self, EnigmaError> {
        let json = fs::read_to_string(path).map_err(|e| EnigmaError::new(Some(&format!("particle from_file '{}': {}", path, e)), false))?;
        Self::from_resource(&json)
    }

    pub fn from_resource(json: &str) -> Result<Self, EnigmaError> {
        let config: ParticleSystemConfig = serde_json::from_str(json)
            .map_err(|e| EnigmaError::new(Some(&format!("particle JSON parse: {}", e)), false))?;
        config.validate()
            .map_err(|e| EnigmaError::new(Some(&format!("particle config invalid: {}", e)), false))?;
        Ok(Self::from_config_internal(config))
    }

    fn from_config_internal(config: ParticleSystemConfig) -> Self {
        let capacity = config.max_particles.max(1);
        let with_trails = matches!(config.render, RenderStyle::Ribbon { .. });
        let seed = config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
        });
        ParticleSystem {
            handle: Uuid::new_v4(),
            config,
            buffer: ParticleBuffer::new(capacity, with_trails),
            transform: identity4(),
            parent_object: None,
            local_offset: identity4(),
            material_id: None,
            state: EmissionState::default(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn validate(&self) -> Result<(), String> { self.config.validate() }

    pub fn to_json(&self) -> Result<String, EnigmaError> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| EnigmaError::new(Some(&format!("particle to_json: {}", e)), false))
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), EnigmaError> {
        let json = self.to_json()?;
        fs::write(path, json).map_err(|e| EnigmaError::new(Some(&format!("particle save_to_file: {}", e)), false))
    }

    // Identity
    pub fn name(&self) -> &str { &self.config.name }
    pub fn get_unique_id(&self) -> Uuid { self.handle }
    pub fn set_name(&mut self, name: &str) { self.config.name = name.into(); }

    // Placement
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.transform = translate4(position);
    }
    pub fn set_local_offset(&mut self, offset: [[f32; 4]; 4]) { self.local_offset = offset; }
    pub fn set_parent_object(&mut self, object_id: Uuid) { self.parent_object = Some(object_id); }
    pub fn clear_parent(&mut self) { self.parent_object = None; }

    // Lifetime / emission
    pub fn set_max_particles(&mut self, max: usize) {
        self.config.max_particles = max;
        let with_trails = matches!(self.config.render, RenderStyle::Ribbon { .. });
        self.buffer = ParticleBuffer::new(max.max(1), with_trails);
    }
    pub fn set_looped(&mut self, looped: bool) { self.config.looped = looped; }
    pub fn set_duration(&mut self, seconds: f32) { self.config.duration = seconds; }
    pub fn set_prewarm(&mut self, prewarm: bool) { self.config.prewarm = prewarm; }
    pub fn set_emission_rate(&mut self, rate: f32) { self.config.emission_rate = rate; }
    pub fn add_burst(&mut self, time: f32, count: u32) { self.config.bursts.push(BurstSpec { time, count }); }
    pub fn clear_bursts(&mut self) { self.config.bursts.clear(); }
    pub fn set_seed(&mut self, seed: u64) {
        self.config.seed = Some(seed);
        self.rng = SmallRng::seed_from_u64(seed);
    }

    // Initial state
    pub fn set_emitter_shape(&mut self, shape: EmitterShape) { self.config.emitter_shape = shape; }
    pub fn set_initial_velocity(&mut self, v: InitialVelocity) { self.config.initial_velocity = v; }
    pub fn set_initial_lifetime(&mut self, min: f32, max: f32) { self.config.initial_lifetime = Range::new(min, max); }
    pub fn set_initial_size(&mut self, min: f32, max: f32) { self.config.initial_size = Range::new(min, max); }
    pub fn set_initial_rotation(&mut self, min: f32, max: f32) { self.config.initial_rotation = Range::new(min, max); }
    pub fn set_initial_color(&mut self, color: ColorRange) { self.config.initial_color = color; }

    // Over-life
    pub fn set_color_over_life(&mut self, g: Gradient<[f32; 4]>) { self.config.color_over_life = Some(g); }
    pub fn set_size_over_life(&mut self, c: Curve<f32>) { self.config.size_over_life = Some(c); }
    pub fn set_rotation_over_life(&mut self, c: Curve<f32>) { self.config.rotation_over_life = Some(c); }
    pub fn set_angular_velocity(&mut self, min: f32, max: f32) { self.config.angular_velocity = Some(Range::new(min, max)); }

    // Forces
    pub fn set_gravity(&mut self, g: [f32; 3]) { self.config.gravity = Some(g); }
    pub fn set_drag(&mut self, d: f32) { self.config.drag = Some(d); }
    pub fn set_curl_noise(&mut self, c: CurlNoiseConfig) { self.config.curl_noise = Some(c); }
    pub fn set_vortex(&mut self, c: VortexConfig) { self.config.vortex = Some(c); }
    pub fn set_attractor(&mut self, c: AttractorConfig) { self.config.attractor = Some(c); }

    // Collision
    pub fn set_collision(&mut self, c: CollisionConfig) { self.config.collision = Some(c); }

    // Sub-emitters
    pub fn set_on_death(&mut self, template_name: &str) { self.config.on_death = Some(template_name.into()); }
    pub fn set_on_collision(&mut self, template_name: &str) { self.config.on_collision = Some(template_name.into()); }

    // Lights
    pub fn set_light_emit(&mut self, c: LightEmitConfig) { self.config.light_emit = Some(c); }

    // Material + render
    pub fn set_material(&mut self, material_id: Uuid) { self.material_id = Some(material_id); }
    pub fn set_render_style(&mut self, style: RenderStyle) {
        let needs_trails = matches!(style, RenderStyle::Ribbon { .. });
        let has_trails = self.buffer.trail.is_some();
        self.config.render = style;
        if needs_trails != has_trails {
            self.buffer = ParticleBuffer::new(self.config.max_particles.max(1), needs_trails);
        }
    }
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        match &mut self.config.render {
            RenderStyle::Sprite { blend_mode, .. } => *blend_mode = mode,
            RenderStyle::Ribbon { blend_mode, .. } => *blend_mode = mode,
        }
    }
    pub fn set_flipbook(&mut self, config: FlipbookConfig) {
        if let RenderStyle::Sprite { flipbook, .. } = &mut self.config.render {
            *flipbook = Some(config);
        }
    }
    pub fn set_soft_particles(&mut self, enabled: bool, fade_distance: f32) {
        if let RenderStyle::Sprite { soft_particles, soft_fade_distance, .. } = &mut self.config.render {
            *soft_particles = enabled;
            *soft_fade_distance = fade_distance;
        }
    }
    pub fn set_velocity_stretch(&mut self, stretch: f32) {
        if let RenderStyle::Sprite { velocity_stretch, .. } = &mut self.config.render {
            *velocity_stretch = stretch;
        }
    }

    // Runtime control
    pub fn play(&mut self) { self.state.stopped = false; }
    pub fn stop(&mut self) { self.state.stopped = true; }
    pub fn restart(&mut self) {
        let capacity = self.config.max_particles.max(1);
        let with_trails = matches!(self.config.render, RenderStyle::Ribbon { .. });
        self.buffer = ParticleBuffer::new(capacity, with_trails);
        self.state = EmissionState::default();
    }
    pub fn is_finished(&self) -> bool { self.state.finished }
    pub fn live_count(&self) -> usize { self.buffer.count() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_minimal_system() {
        let mut s = ParticleSystem::new("test");
        s.set_max_particles(32);
        s.set_initial_lifetime(0.5, 1.0);
        s.set_emitter_shape(EmitterShape::Point);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(1.0, 1.0) });
        s.set_initial_size(0.1, 0.2);
        s.set_initial_color(ColorRange::Single([1.0, 1.0, 1.0, 1.0]));
        s.set_render_style(RenderStyle::Sprite {
            flipbook: None, blend_mode: BlendMode::Alpha,
            soft_particles: false, soft_fade_distance: 0.0, velocity_stretch: 0.0,
        });
        assert_eq!(s.name(), "test");
        assert_eq!(s.config.max_particles, 32);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn from_resource_parses_valid_json() {
        let json = r#"{
            "name": "ex",
            "max_particles": 32,
            "looped": false,
            "duration": 1.0,
            "emission_rate": 10.0,
            "emitter_shape": "Point",
            "initial_velocity": {"Direction": {"direction": [0.0,1.0,0.0], "speed": {"min": 1.0, "max": 2.0}}},
            "initial_lifetime": {"min": 0.5, "max": 1.0},
            "initial_size": {"min": 0.1, "max": 0.2},
            "initial_color": {"Single": [1.0, 1.0, 1.0, 1.0]},
            "render": {"kind": "Sprite", "blend_mode": "Alpha"}
        }"#;
        let s = ParticleSystem::from_resource(json).expect("should parse");
        assert_eq!(s.name(), "ex");
    }

    #[test]
    fn from_resource_rejects_invalid_config() {
        let json = r#"{
            "name": "bad",
            "max_particles": 0,
            "looped": false,
            "duration": 1.0,
            "emission_rate": 10.0,
            "emitter_shape": "Point",
            "initial_velocity": {"Direction": {"direction": [0.0,1.0,0.0], "speed": {"min": 1.0, "max": 2.0}}},
            "initial_lifetime": {"min": 0.5, "max": 1.0},
            "initial_size": {"min": 0.1, "max": 0.2},
            "initial_color": {"Single": [1.0,1.0,1.0,1.0]},
            "render": {"kind": "Sprite", "blend_mode": "Alpha"}
        }"#;
        assert!(ParticleSystem::from_resource(json).is_err());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let mut s = ParticleSystem::new("rt");
        s.set_max_particles(16);
        s.set_initial_lifetime(0.5, 1.0);
        s.set_emitter_shape(EmitterShape::Sphere { radius: 0.5 });
        s.set_initial_velocity(InitialVelocity::Outward { speed: Range::new(1.0, 2.0) });
        s.set_initial_size(0.1, 0.2);
        s.set_initial_color(ColorRange::Single([1.0, 1.0, 1.0, 1.0]));
        s.set_render_style(RenderStyle::Sprite {
            flipbook: None, blend_mode: BlendMode::Additive,
            soft_particles: false, soft_fade_distance: 0.0, velocity_stretch: 0.0,
        });
        let json = s.to_json().unwrap();
        let back = ParticleSystem::from_resource(&json).unwrap();
        assert_eq!(back.name(), "rt");
        assert_eq!(back.config.max_particles, 16);
    }
}
