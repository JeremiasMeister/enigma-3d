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

    /// Replace the rotation portion of the transform (preserving translation).
    /// Euler angles in radians; rotation order Z * Y * X (yaw-pitch-roll).
    /// Call after `set_position`, before `set_scale`.
    pub fn set_rotation(&mut self, euler_radians: [f32; 3]) {
        let pos = [self.transform[3][0], self.transform[3][1], self.transform[3][2]];
        let (cx, sx) = (euler_radians[0].cos(), euler_radians[0].sin());
        let (cy, sy) = (euler_radians[1].cos(), euler_radians[1].sin());
        let (cz, sz) = (euler_radians[2].cos(), euler_radians[2].sin());
        // R = Rz * Ry * Rx
        let r00 = cy * cz;
        let r01 = -cy * sz;
        let r02 = sy;
        let r10 = cx * sz + sx * sy * cz;
        let r11 = cx * cz - sx * sy * sz;
        let r12 = -sx * cy;
        let r20 = sx * sz - cx * sy * cz;
        let r21 = sx * cz + cx * sy * sz;
        let r22 = cx * cy;
        self.transform = [
            [r00, r01, r02, 0.0],
            [r10, r11, r12, 0.0],
            [r20, r21, r22, 0.0],
            [pos[0], pos[1], pos[2], 1.0],
        ];
    }

    /// Scale the 3×3 rotation/scale block of the transform, preserving translation.
    /// Call after `set_rotation`.
    pub fn set_scale(&mut self, scale: [f32; 3]) {
        for i in 0..3 {
            self.transform[0][i] *= scale[0];
            self.transform[1][i] *= scale[1];
            self.transform[2][i] *= scale[2];
        }
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

    // Hero light extraction
    pub fn extract_hero_lights(&self) -> Vec<crate::light::Light> {
        let cfg = match &self.config.light_emit { Some(c) => c, None => return Vec::new() };
        let max = cfg.max_lights as usize;
        if max == 0 || self.live_count() == 0 { return Vec::new(); }

        // Score: most-energetic particles (middle of life with current alpha)
        let mut scored: Vec<(usize, f32)> = (0..self.live_count())
            .map(|i| {
                let t = (self.buffer.age[i] / self.buffer.lifetime[i]).clamp(0.0, 1.0);
                let mid = 1.0 - (t - 0.5).abs() * 2.0;
                let score = self.buffer.color[i][3] * mid;
                (i, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(max);

        scored.into_iter().map(|(i, _)| {
            let pos = self.buffer.position[i];
            let color = [
                self.buffer.color[i][0] * cfg.color_multiplier[0],
                self.buffer.color[i][1] * cfg.color_multiplier[1],
                self.buffer.color[i][2] * cfg.color_multiplier[2],
            ];
            crate::light::Light::new(pos, color, cfg.intensity, None, false)
        }).collect()
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

    /// Advance simulation by `dt`. `terrain_height_at`, if provided, is queried for collision.
    /// Returns death and collision events for the caller to dispatch sub-emitters.
    pub fn tick(&mut self, dt: f32, terrain_height_at: Option<&dyn Fn(f32, f32) -> Option<f32>>) -> TickResult {
        let mut result = TickResult::default();
        if self.state.stopped {
            return result;
        }

        // 1. Advance time
        let prev_elapsed = self.state.elapsed;
        self.state.elapsed += dt;

        let within_emission = self.config.looped || self.state.elapsed < self.config.duration;
        if within_emission {
            self.state.spawn_accumulator += self.config.emission_rate * dt;
        }

        // 2. Spawn (before aging so new particles get aged this tick too)
        let mut to_spawn: usize = 0;
        if within_emission {
            let n = self.state.spawn_accumulator.floor();
            to_spawn += n as usize;
            self.state.spawn_accumulator -= n;
        }
        // bursts within (prev_elapsed, elapsed]
        for b in &self.config.bursts {
            if b.time > prev_elapsed && b.time <= self.state.elapsed {
                to_spawn += b.count as usize;
            }
        }
        // edge case: burst at time 0.0 should fire on first tick
        if prev_elapsed == 0.0 {
            for b in &self.config.bursts {
                if b.time == 0.0 {
                    to_spawn += b.count as usize;
                }
            }
        }

        let origin = [self.transform[3][0], self.transform[3][1], self.transform[3][2]];

        // Borrow split: snapshot config fields needed for spawning, rng stays mutable
        let max_particles = self.config.max_particles;
        for _ in 0..to_spawn {
            if self.buffer.is_full() { break; }
            let local_pos = self.config.emitter_shape.sample_position(&mut self.rng);
            let world_pos = [origin[0]+local_pos[0], origin[1]+local_pos[1], origin[2]+local_pos[2]];
            let vel = self.config.initial_velocity.sample(&local_pos, &mut self.rng);
            let life = self.config.initial_lifetime.sample(&mut self.rng);
            let size = self.config.initial_size.sample(&mut self.rng);
            let color = self.config.initial_color.sample(&mut self.rng);
            let rotation = self.config.initial_rotation.sample(&mut self.rng);
            let seed: f32 = rand::Rng::gen(&mut self.rng);
            let _ = max_particles; // used to ensure borrow is local
            self.buffer.push(ParticleSpawn {
                position: world_pos, velocity: vel, color, size, rotation, lifetime: life, seed,
            });
            // Per-particle starting frame offset for desynchronized flipbooks.
            if let RenderStyle::Sprite { flipbook: Some(fb), .. } = &self.config.render {
                if fb.randomize_start_frame && fb.frame_count > 0 {
                    let last = self.buffer.count() - 1;
                    self.buffer.flipbook_frame[last] = rand::Rng::gen::<f32>(&mut self.rng) * fb.frame_count as f32;
                }
            }
        }

        // 3. Age all particles (including newly spawned) & kill
        for i in 0..self.buffer.count() {
            self.buffer.age[i] += dt;
        }
        let need_death_events = self.config.on_death.is_some();
        let mut death_events: Vec<DeathEvent> = Vec::new();
        self.buffer.compact_dead(|pos, vel, color| {
            if need_death_events {
                death_events.push(DeathEvent { position: pos, velocity: vel, color });
            }
        });
        result.death_events = death_events;

        // 4-5. Forces + integrate
        let time = self.state.elapsed;
        for i in 0..self.buffer.count() {
            let mut accel = [0.0_f32; 3];
            if let Some(g) = self.config.gravity {
                accel[0] += g[0]; accel[1] += g[1]; accel[2] += g[2];
            }
            if let Some(curl) = self.config.curl_noise {
                let f = curl.force(&self.buffer.position[i], time);
                accel[0] += f[0]; accel[1] += f[1]; accel[2] += f[2];
            }
            if let Some(vortex) = self.config.vortex {
                let f = vortex.force(&self.buffer.position[i]);
                accel[0] += f[0]; accel[1] += f[1]; accel[2] += f[2];
            }
            if let Some(attractor) = self.config.attractor {
                let f = attractor.force(&self.buffer.position[i]);
                accel[0] += f[0]; accel[1] += f[1]; accel[2] += f[2];
            }
            self.buffer.velocity[i][0] += accel[0] * dt;
            self.buffer.velocity[i][1] += accel[1] * dt;
            self.buffer.velocity[i][2] += accel[2] * dt;
            if let Some(d) = self.config.drag {
                crate::particle::forces::apply_drag(&mut self.buffer.velocity[i], d, dt);
            }
            // Integrate position
            self.buffer.position[i][0] += self.buffer.velocity[i][0] * dt;
            self.buffer.position[i][1] += self.buffer.velocity[i][1] * dt;
            self.buffer.position[i][2] += self.buffer.velocity[i][2] * dt;

            // Flipbook advance
            if let RenderStyle::Sprite { flipbook: Some(ref fb), .. } = self.config.render {
                self.buffer.flipbook_frame[i] += fb.fps * dt;
            }
        }

        // 6. Collide
        if let Some(coll) = self.config.collision.clone() {
            for i in 0..self.buffer.count() {
                let mut hit = coll.test_and_resolve_ground(&mut self.buffer.position[i], &mut self.buffer.velocity[i]);
                if !hit {
                    if let Some(query) = terrain_height_at {
                        hit = coll.test_and_resolve_terrain(&mut self.buffer.position[i], &mut self.buffer.velocity[i], query);
                    }
                }
                if hit && self.config.on_collision.is_some() {
                    result.collision_events.push(CollisionEvent {
                        position: self.buffer.position[i],
                        velocity: self.buffer.velocity[i],
                        normal: [0.0, 1.0, 0.0],
                    });
                }
            }
        }

        // 7. Sample curves (color/size/rotation over life) + integrate angular velocity
        let needs_curve_sample = self.config.color_over_life.is_some()
            || self.config.size_over_life.is_some()
            || self.config.rotation_over_life.is_some();
        if needs_curve_sample {
            for i in 0..self.buffer.count() {
                let t = (self.buffer.age[i] / self.buffer.lifetime[i]).clamp(0.0, 1.0);
                if let Some(ref g) = self.config.color_over_life.clone() {
                    self.buffer.color[i] = g.eval(t);
                }
                if let Some(ref c) = self.config.size_over_life.clone() {
                    self.buffer.size[i] = c.eval(t);
                }
                if let Some(ref c) = self.config.rotation_over_life.clone() {
                    self.buffer.rotation[i] = c.eval(t);
                }
            }
        }
        if let Some(av) = self.config.angular_velocity {
            // Each particle gets a deterministic spin rate from its seed,
            // mapped from [0,1] to [av.min, av.max].
            for i in 0..self.buffer.count() {
                let spin = av.min + (av.max - av.min) * self.buffer.seed[i];
                self.buffer.rotation[i] += spin * dt;
            }
        }

        // 8. Push trail history
        if let Some(ref mut trails) = self.buffer.trail {
            let max_points = match self.config.render {
                RenderStyle::Ribbon { max_trail_points, .. } => max_trail_points,
                _ => 0,
            };
            for i in 0..trails.len() {
                trails[i].push_front(self.buffer.position[i]);
                while trails[i].len() > max_points {
                    trails[i].pop_back();
                }
            }
        }

        // 10. Mark finished
        if !self.config.looped && self.state.elapsed > self.config.duration && self.buffer.count() == 0 {
            self.state.finished = true;
        }

        result
    }
}

#[derive(Default, Debug)]
pub struct TickResult {
    pub death_events: Vec<DeathEvent>,
    pub collision_events: Vec<CollisionEvent>,
}

#[derive(Clone, Copy, Debug)]
pub struct DeathEvent {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub color:    [f32; 4],
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

    #[test]
    fn one_particle_dies_after_lifetime() {
        let mut s = ParticleSystem::new("t");
        s.set_max_particles(4);
        s.set_initial_lifetime(1.0, 1.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0, 0.0, 0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.config.bursts.push(BurstSpec { time: 0.0, count: 1 });
        s.set_looped(false);
        s.set_duration(2.0);

        s.tick(0.5, None);
        assert_eq!(s.live_count(), 1);
        s.tick(0.6, None);
        assert_eq!(s.live_count(), 0);
    }

    #[test]
    fn looped_steady_state() {
        let mut s = ParticleSystem::new("loop");
        s.set_max_particles(100);
        s.set_initial_lifetime(1.0, 1.0);
        s.set_emission_rate(10.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0,0.0,0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.set_looped(true);
        for _ in 0..120 { s.tick(1.0/60.0, None); }
        assert!(s.live_count() >= 5);
        assert!(s.live_count() <= 15);
    }

    #[test]
    fn gravity_accelerates() {
        let mut s = ParticleSystem::new("g");
        s.set_max_particles(1);
        s.set_initial_lifetime(10.0, 10.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0,0.0,0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.set_gravity([0.0, -10.0, 0.0]);
        s.config.bursts.push(BurstSpec { time: 0.0, count: 1 });
        s.tick(0.01, None);
        s.tick(1.0, None);
        assert!(s.buffer.velocity[0][1] < -5.0);
    }

    #[test]
    fn hero_lights_empty_without_config() {
        let mut s = ParticleSystem::new("nolights");
        s.set_max_particles(4);
        s.set_initial_lifetime(0.5, 1.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.config.bursts.push(BurstSpec { time: 0.0, count: 2 });
        s.tick(0.01, None);
        assert_eq!(s.extract_hero_lights().len(), 0);
    }

    #[test]
    fn hero_lights_capped_at_max() {
        let mut s = ParticleSystem::new("lights");
        s.set_max_particles(8);
        s.set_initial_lifetime(1.0, 1.0);
        s.set_initial_velocity(InitialVelocity::Direction { direction: [0.0, 1.0, 0.0], speed: Range::new(0.0, 0.0) });
        s.set_initial_size(0.1, 0.1);
        s.set_initial_color(ColorRange::Single([1.0,1.0,1.0,1.0]));
        s.config.bursts.push(BurstSpec { time: 0.0, count: 8 });
        s.set_light_emit(LightEmitConfig { max_lights: 2, color_multiplier: [1.0, 1.0, 1.0], intensity: 1.0, falloff: 1.0 });
        s.tick(0.5, None);
        let lights = s.extract_hero_lights();
        assert_eq!(lights.len(), 2);
    }
}
