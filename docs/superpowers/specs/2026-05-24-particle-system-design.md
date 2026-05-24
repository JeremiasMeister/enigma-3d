# Particle System Design

Status: design — pending implementation
Date: 2026-05-24
Owner: enigma-3d

## Goal

Add a high-quality, fast, CPU-simulated particle system to enigma-3d that supports the full set of effects needed for fantasy magic (A) and impact/juice (C) gameplay VFX. The runtime is data-driven and fully authorable in code; JSON serialization makes it editor-ready, with a separate editor application planned for later. Out of scope for v1: GPU simulation, mesh particles, decals, OIT, motion blur.

## Design decisions (resolved in brainstorming)

| Decision | Choice | Reasoning |
|---|---|---|
| Authoring | Code API + JSON serde, editor app later | Runtime first; editor as separate application communicates via JSON |
| Target effects | Magic/spells (A) + Impact/juice (C) | Game VFX use cases; modest particle counts |
| Simulation | CPU with rayon parallelism, ~10k particles on screen | A+C don't need GPU sim; CPU gives full flexibility for sub-emitters and per-particle logic |
| Render styles | Sprite (billboard) + Ribbon (trail) for v1 | Mesh particles deferred to v2 |
| Magic features | Soft particles, hero per-particle lights, HDR/bloom integration, curl noise + vortex + attractor forces, ground + terrain collision, sub-emitters on death and collision, flipbook, velocity stretch | Realistic ceiling for one focused implementation |
| Engine integration | Separate `AppState.particle_systems`, own render pass between transparent and post | Clean separation from `Object` model; depth-buffer access free |
| Composition model | Shuriken-style fixed pipeline with optional config sections, SoA particle buffer | Faster, cache-friendly, JSON-trivial; vtable overhead unacceptable in 10k-particle hot loop |
| API style | Engine convention: `add_X`, `set_X`, name-based lookup, material UUID refs | Mirrors `Object`/`Material`/`Light` patterns already in the engine |

## Architecture

### File layout

```
src/particle/
  mod.rs          — public re-exports
  config.rs       — ParticleSystemConfig + all sub-configs (serde-derived)
  buffer.rs       — ParticleBuffer (SoA, dense, capacity-fixed)
  system.rs       — ParticleSystem (runtime instance: config + buffer + state)
  emitter.rs      — EmitterShape + spawn position/velocity sampling
  forces.rs       — Gravity, Drag, CurlNoise, Vortex, Attractor
  curves.rs       — Gradient<T> + Curve<f32> keyframe types
  collision.rs    — GroundPlane + TerrainHeightfield queries
  subemit.rs      — sub-emitter event collection and dispatch
  render/
    mod.rs        — ParticleRenderer orchestrator
    sprite.rs     — billboard pass
    ribbon.rs     — ribbon pass
  res/shader/     — particle_sprite_*.glsl, particle_ribbon_*.glsl
```

Touch points elsewhere:

- `lib.rs::AppState`: add `particle_systems: Vec<ParticleSystem>`, `add_particle_system`, `add_particle_system_template`, `emit_particle_system`, lookup methods, and `tick_particles(dt)` invocation in the update step.
- `lib.rs::EventLoop::run`: insert tick + render between the transparent pass and the post-processing chain.
- `light.rs`: read-only; hero particle lights are appended to the per-frame light list.
- `material.rs`: add `Material::particle_sprite(display)` and `Material::particle_ribbon(display)` preset factories.
- `examples/particles.rs`: dedicated example program demonstrating the A+C archetypes.

### Data layout

#### `ParticleBuffer` — SoA hot data

Dense, capacity-fixed at system creation from `config.max_particles`. Dead particles compact via swap-remove. Trail data allocated only if ribbon rendering is enabled. Calling `ParticleSystem::set_max_particles` after creation reallocates the buffer (authoring-time operation; not safe to call during a tick).

```rust
pub struct ParticleBuffer {
    position:       Vec<Vec3>,
    velocity:       Vec<Vec3>,
    color:          Vec<Vec4>,   // current HDR rgba (post-curve)
    size:           Vec<f32>,    // current size (post-curve)
    rotation:       Vec<f32>,    // 2D rotation around camera axis, radians
    age:            Vec<f32>,
    lifetime:       Vec<f32>,
    seed:           Vec<f32>,    // per-particle random [0,1] for deterministic variance
    flipbook_frame: Vec<f32>,    // float frame index for smooth interpolation
    trail:          Option<Vec<VecDeque<Vec3>>>,  // ribbon only, sized by render.max_trail_points
    count:          usize,
    capacity:       usize,
}
```

Memory budget: ~64 bytes/particle always-on, ~256 bytes/particle with trails. 10k particles = <1 MB without trails, <3 MB with. Comfortable.

#### `ParticleSystem` — runtime instance

```rust
pub struct ParticleSystem {
    handle:    Uuid,
    config:    ParticleSystemConfig,
    buffer:    ParticleBuffer,
    transform: Mat4,
    parent:    Option<Uuid>,             // optional Object parent
    state:     EmissionState,
    rng:       SmallRng,
}

pub struct EmissionState {
    elapsed:           f32,
    spawn_accumulator: f32,
    pending_bursts:    Vec<BurstSpec>,
    pending_collision_events: Vec<CollisionEvent>,  // queued for next-frame sub-emit
    finished:          bool,
}
```

A particle system whose `parent` Object is despawned keeps simulating in its last-known transform until it finishes — the outlives-owner case typical of game VFX.

#### `ParticleSystemConfig` — authored shape

Flat struct with optional sub-sections. `#[serde(skip_serializing_if = "Option::is_none")]` keeps JSON files readable.

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleSystemConfig {
    pub name:           String,
    pub max_particles:  usize,
    pub looped:         bool,
    pub duration:       f32,
    pub prewarm:        bool,
    pub seed:           Option<u64>,

    pub emission_rate:  f32,
    pub bursts:         Vec<BurstSpec>,

    pub emitter_shape:   EmitterShape,
    pub initial_velocity: InitialVelocity,
    pub initial_lifetime: Range<f32>,
    pub initial_size:    Range<f32>,
    pub initial_color:   ColorRange,
    pub initial_rotation: Range<f32>,

    pub color_over_life:    Option<Gradient<[f32; 4]>>,
    pub size_over_life:     Option<Curve<f32>>,
    pub rotation_over_life: Option<Curve<f32>>,
    pub angular_velocity:   Option<Range<f32>>,

    pub gravity:    Option<[f32; 3]>,
    pub drag:       Option<f32>,
    pub curl_noise: Option<CurlNoiseConfig>,
    pub vortex:     Option<VortexConfig>,
    pub attractor:  Option<AttractorConfig>,

    pub collision: Option<CollisionConfig>,

    pub on_death:     Option<String>,   // template name reference
    pub on_collision: Option<String>,

    pub light_emit: Option<LightEmitConfig>,

    pub render: RenderConfig,           // Sprite { ... } | Ribbon { ... }
}
```

Sub-emitters reference templates by name rather than nesting configs, so a "smoke puff" can be the on-death of three different effects without duplication.

### Supporting types

```rust
// Custom range with serde support — std::ops::Range<f32> doesn't derive Serialize.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Range<T> { pub min: T, pub max: T }

#[derive(Serialize, Deserialize, Clone)]
pub struct BurstSpec { pub time: f32, pub count: u32 }

#[derive(Serialize, Deserialize, Clone)]
pub enum ColorRange {
    Single([f32; 4]),                       // fixed color
    Range { min: [f32; 4], max: [f32; 4] }, // uniform random between two LDR colors
    Hdr { rgb: [f32; 3], intensity: f32, alpha: f32 },  // single HDR color
    HdrRange { rgb_min: [f32; 3], rgb_max: [f32; 3], intensity: Range<f32>, alpha: Range<f32> },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EmitterShape {
    Point {},
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    Cone { angle: f32, height: f32 },
    Disk { radius: f32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum InitialVelocity {
    Outward { speed: Range<f32> },                                  // from emitter origin
    Direction { direction: [f32; 3], speed: Range<f32> },           // fixed direction
    Cone { direction: [f32; 3], angle: f32, speed: Range<f32> },    // within cone of axis
    Hemisphere { normal: [f32; 3], speed: Range<f32> },             // dome around normal
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RenderStyle {
    Sprite {
        flipbook: Option<FlipbookConfig>,
        blend_mode: BlendMode,
        soft_particles: bool,
        soft_fade_distance: f32,
        velocity_stretch: f32,
    },
    Ribbon {
        max_trail_points: usize,
        width_over_life: Curve<f32>,
        blend_mode: BlendMode,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum BlendMode { Additive, Alpha, PremultipliedAlpha }
```

`Curve<T>` and `Gradient<T>` are `Vec<(f32, T)>` keyframe lists with `t ∈ [0, 1]`, evaluated by binary search + linear interpolation.

### Update pipeline (per-frame, per-system)

Strict order, each step depends on the previous. `AppState::tick_particles(dt)` runs this for every active system.

1. **Advance time**: increment `elapsed`, accumulate `spawn_accumulator`, sync transform from parent if `parent` resolves.
2. **Age & kill**: increment `age`; particles past lifetime are queued for sub-emit and compacted via swap-remove.
3. **Spawn**: continuous (rate) + bursts (within duration) + sub-emit-on-collision queued from previous frame.
4. **Forces** (parallel, rayon): accumulate gravity, curl noise, vortex, attractor into accel; integrate velocity; apply drag analytically (`v *= exp(-drag * dt)`); apply angular velocity.
5. **Integrate**: `position += velocity * dt`, `rotation += angular_velocity * dt`.
6. **Collide**: ground plane test (if `collision.ground_plane_y` set); terrain heightfield query (if `collision.terrain` set). On hit: reflect velocity with `bounce` factor, apply friction, queue collision event for next-frame sub-emit.
7. **Sample curves** (parallel): evaluate color/size/rotation gradients and curves at normalized age `t = age / lifetime`.
8. **Push trail history** (ribbon only): push current position to each particle's trail deque, truncate to `max_trail_points`.
9. **Sub-emit on death**: spawn child instances from registered templates at each death position.
10. **Mark finished**: if non-looped, past duration, and zero live particles, flag for reap.

After all systems tick, the renderer extracts hero lights (top N particles by `light_score`) and appends them to the per-frame light list.

### Render path

Render order in `EventLoop::run`:
1. Opaque pass
2. Skybox
3. Transparent pass
4. **`tick_particles(dt)`** — NEW
5. **`render_particles(...)`** — NEW
6. Post-processing chain (bloom picks up HDR particles)
7. Final blit + egui

Particles render into the existing HDR multi-output framebuffer. Depth read on, depth write off. The opaque-pass depth texture is bound as a sampler for soft particles.

**Sorting**:
- Across systems: by `(blend_mode, distance_to_camera_desc)`. Additive systems first (order-independent under additive), then alpha-blended back-to-front.
- Within a system: alpha-blended sprites only — index sort by depth each frame. Additive systems and ribbons skip.

**Sprite path**: one instanced draw call per (system, material). Per-instance `SpriteInstance { position, size, color, velocity, rotation, flipbook_uv }`. Vertex shader builds a camera-facing quad with optional velocity stretch. Fragment shader samples the material's albedo and applies soft-particle alpha falloff against the scene depth.

**Ribbon path**: CPU rebuilds a single dynamic `VertexBuffer<RibbonVertex>` each frame from each particle's trail history. For each segment of each trail, emit a quad oriented so its short side is `cross(tangent, camera_forward)`. Width tapers along trail age via `width_over_life`. Cost: <100 KB/frame at typical scene scale.

**HDR & bloom**: HDR-bright particle colors (`color * 30.0` for fire) automatically pick up the existing bloom post-process. Authored via `ColorRange::Hdr { rgb, intensity, alpha }`.

**Hero lights**: each system optionally emits up to `max_lights` unshadowed point lights, computed per frame from particle position + lifetime + intensity curve. Capped globally (default 8) across all systems. Plugs into the engine's existing light array — no shader changes.

**Blend modes**: `Additive` (One, One), `Alpha` (SrcAlpha, OneMinusSrcAlpha), `PremultipliedAlpha` (One, OneMinusSrcAlpha).

### Engine API

#### `ParticleSystem` construction and authoring

```rust
impl ParticleSystem {
    pub fn new(name: &str) -> Self;
    pub fn from_file(path: &str) -> Result<Self, EnigmaError>;
    pub fn from_resource(json: &str) -> Result<Self, EnigmaError>;

    pub fn set_name(&mut self, name: &str);
    pub fn name(&self) -> &str;
    pub fn get_unique_id(&self) -> Uuid;

    pub fn set_position(&mut self, position: [f32; 3]);
    pub fn set_rotation(&mut self, rotation: [f32; 3]);
    pub fn set_scale(&mut self, scale: [f32; 3]);
    pub fn set_parent_object(&mut self, object_id: Uuid);
    pub fn clear_parent(&mut self);
    pub fn set_local_offset(&mut self, offset: Mat4);

    pub fn set_max_particles(&mut self, max: usize);
    pub fn set_looped(&mut self, looped: bool);
    pub fn set_duration(&mut self, seconds: f32);
    pub fn set_prewarm(&mut self, prewarm: bool);
    pub fn set_emission_rate(&mut self, rate: f32);
    pub fn add_burst(&mut self, time: f32, count: u32);
    pub fn clear_bursts(&mut self);
    pub fn set_seed(&mut self, seed: u64);

    pub fn set_emitter_shape(&mut self, shape: EmitterShape);
    pub fn set_initial_velocity(&mut self, velocity: InitialVelocity);
    pub fn set_initial_lifetime(&mut self, min: f32, max: f32);
    pub fn set_initial_size(&mut self, min: f32, max: f32);
    pub fn set_initial_rotation(&mut self, min: f32, max: f32);
    pub fn set_initial_color(&mut self, color: ColorRange);

    pub fn set_color_over_life(&mut self, gradient: Gradient<[f32; 4]>);
    pub fn set_size_over_life(&mut self, curve: Curve<f32>);
    pub fn set_rotation_over_life(&mut self, curve: Curve<f32>);
    pub fn set_angular_velocity(&mut self, min: f32, max: f32);

    pub fn set_gravity(&mut self, gravity: [f32; 3]);
    pub fn set_drag(&mut self, drag: f32);
    pub fn set_curl_noise(&mut self, config: CurlNoiseConfig);
    pub fn set_vortex(&mut self, config: VortexConfig);
    pub fn set_attractor(&mut self, config: AttractorConfig);

    pub fn set_collision(&mut self, config: CollisionConfig);

    pub fn set_on_death(&mut self, child_template_name: &str);
    pub fn set_on_collision(&mut self, child_template_name: &str);

    pub fn set_light_emit(&mut self, config: LightEmitConfig);

    pub fn set_material(&mut self, material_id: Uuid);
    pub fn set_render_style(&mut self, style: RenderStyle);
    pub fn set_blend_mode(&mut self, mode: BlendMode);
    pub fn set_flipbook(&mut self, config: FlipbookConfig);
    pub fn set_soft_particles(&mut self, enabled: bool, fade_distance: f32);
    pub fn set_velocity_stretch(&mut self, stretch: f32);

    pub fn play(&mut self);
    pub fn stop(&mut self);
    pub fn restart(&mut self);
    pub fn is_finished(&self) -> bool;
    pub fn live_count(&self) -> usize;

    pub fn to_json(&self) -> Result<String, EnigmaError>;
    pub fn save_to_file(&self, path: &str) -> Result<(), EnigmaError>;
}
```

#### `AppState` surface

```rust
impl AppState {
    /// Register a system as a template AND spawn an active instance if emission is configured.
    pub fn add_particle_system(&mut self, system: ParticleSystem);

    /// Register as template only — no live instance is created on add.
    /// Used for sub-emitter sources or click-to-spawn templates.
    pub fn add_particle_system_template(&mut self, system: ParticleSystem);

    /// Spawn a live instance from a registered template at a world transform.
    pub fn emit_particle_system(
        &mut self,
        template_name: &str,
        transform: Mat4,
    ) -> Option<Uuid>;

    pub fn get_particle_system_by_name(&self, name: &str) -> Option<&ParticleSystem>;
    pub fn get_particle_system_by_name_mut(&mut self, name: &str) -> Option<&mut ParticleSystem>;
    pub fn get_particle_system(&self, id: Uuid) -> Option<&ParticleSystem>;
    pub fn get_particle_system_mut(&mut self, id: Uuid) -> Option<&mut ParticleSystem>;

    pub fn remove_particle_system(&mut self, id: Uuid);
    pub fn remove_particle_systems_by_name(&mut self, name: &str);
}
```

#### Material assignment

```rust
impl Material {
    pub fn particle_sprite(display: Display<WindowSurface>) -> Self;
    pub fn particle_ribbon(display: Display<WindowSurface>) -> Self;
}
```

These wire up the particle vertex/fragment shaders, expose an albedo slot, and accept the standard `set_albedo`, `set_color`, `set_emissive_strength` setters. Particle systems reference materials by `Uuid` exactly like `Object` does.

### JSON schema

Every config type derives `Serialize, Deserialize`. Two reference examples follow; both round-trip cleanly through `from_resource` / `to_json`.

**Fireball with on-death smoke** (`assets/vfx/fireball.json`):

```json
{
  "name": "fireball",
  "max_particles": 256,
  "looped": false,
  "duration": 2.0,
  "emission_rate": 120.0,
  "bursts": [],
  "emitter_shape": { "Sphere": { "radius": 0.15 } },
  "initial_velocity": { "kind": "Outward", "speed": { "min": 0.5, "max": 1.5 } },
  "initial_lifetime": { "min": 0.6, "max": 1.0 },
  "initial_size":     { "min": 0.3, "max": 0.55 },
  "initial_rotation": { "min": 0.0, "max": 6.283 },
  "initial_color": { "kind": "Hdr", "rgb": [1.0, 0.55, 0.15], "intensity": 25.0, "alpha": 1.0 },
  "color_over_life": [
    [0.0, [1.0, 0.6, 0.2, 1.0]],
    [0.5, [1.0, 0.3, 0.05, 0.8]],
    [1.0, [0.2, 0.1, 0.05, 0.0]]
  ],
  "size_over_life": [[0.0, 0.4], [0.6, 1.0], [1.0, 0.0]],
  "drag": 1.8,
  "curl_noise": { "scale": 1.5, "strength": 2.0, "evolution_speed": 0.5 },
  "light_emit": { "max_lights": 1, "color_multiplier": [1.0, 0.5, 0.15], "intensity": 4.0, "falloff": 6.0 },
  "on_death": "fireball_smoke",
  "render": {
    "kind": "Sprite",
    "flipbook": { "cols": 4, "rows": 4, "frame_count": 16, "fps": 30.0, "blend": true },
    "blend_mode": "Additive",
    "soft_particles": true,
    "soft_fade_distance": 0.3,
    "velocity_stretch": 0.0
  }
}
```

**Hit sparks with collision and on-collision dust** (`assets/vfx/hit_sparks.json`):

```json
{
  "name": "hit_sparks",
  "max_particles": 64,
  "looped": false,
  "duration": 0.0,
  "bursts": [{ "time": 0.0, "count": 24 }],
  "emission_rate": 0.0,
  "emitter_shape": { "Point": {} },
  "initial_velocity": { "kind": "Hemisphere", "normal": [0, 1, 0], "speed": { "min": 4.0, "max": 9.0 } },
  "initial_lifetime": { "min": 0.3, "max": 0.7 },
  "initial_size":     { "min": 0.04, "max": 0.08 },
  "initial_color": { "kind": "Hdr", "rgb": [1.0, 0.85, 0.5], "intensity": 12.0, "alpha": 1.0 },
  "gravity": [0, -18.0, 0],
  "drag": 0.4,
  "color_over_life": [[0.0,[1.0,0.85,0.5,1.0]],[1.0,[0.6,0.2,0.05,0.0]]],
  "size_over_life":  [[0.0,1.0],[1.0,0.3]],
  "collision": { "ground_plane_y": 0.0, "terrain": true, "bounce": 0.4, "friction": 0.3 },
  "on_collision": "spark_dust",
  "render": {
    "kind": "Ribbon",
    "max_trail_points": 8,
    "width_over_life": [[0.0, 1.0], [1.0, 0.0]],
    "blend_mode": "Additive"
  }
}
```

Texture and shader references live in the assigned `Material`, not in the particle JSON — separation of concerns matches the engine's existing model.

### Validation

Validation runs in three phases:

**1. Structural — at `from_file` / `from_resource`** (returns `EnigmaError` on failure):

- `max_particles > 0`
- `duration >= 0`
- `initial_lifetime.min > 0` and `min <= max`
- Curve and gradient keyframes are sorted by time and span `[0, 1]`
- All `Range<T>` fields satisfy `min <= max`

**2. Link-time — when material is assigned via `set_material`**:

- Render style matches material kind (Sprite material with `RenderStyle::Sprite`, Ribbon material with `RenderStyle::Ribbon`). Mismatch returns `EnigmaError`.

**3. Graph-time — at `add_particle_system` / `add_particle_system_template`**:

- `on_death` / `on_collision` template names: if the referenced template is already registered, validate now; if not, defer to first spawn (log `EnigmaWarning` and skip silently if still unresolved at spawn time — this allows registration in any order during scene setup).
- Sub-emitter cycle detection: walk the on_death/on_collision graph from the newly-registered template. If a cycle through already-registered templates would close, reject with `EnigmaError`.

## Example program

`examples/particles.rs` (gated behind `features=examples`) demonstrates the A+C archetypes side by side:

1. **Persistent magic fireball** at fixed position. On-death smoke. Hero light visible on a wall behind.
2. **Click-to-spawn hit sparks** with terrain collision and on-collision dust puff. Demonstrates raycast-from-mouse + `emit_particle_system`.
3. **Slow swirling magic orb** with sparkle ribbon trails. Demonstrates ribbon render path with curl noise + attractor force.
4. **On-screen counter** via `inject_gui`: active particle count, system count, frame time.

Acceptance: all three effects render visibly, the fireball blooms, smoke fades softly against geometry, sparks bounce off terrain, ribbons render without tears or z-fighting, and frame time stays under 16 ms on a development machine.

## Testing

### Unit tests — pure data, no GPU

Run in `cargo test` without a display:

- **`config.rs`**: round-trip serialization preserves all fields; `validate()` catches malformed configs; malformed JSON returns `EnigmaError`.
- **`buffer.rs`**: `push` honors capacity; `swap_remove_dead` compacts correctly; trail buffer only allocates when ribbon enabled.
- **`curves.rs`**: keyframe-exact and interpolation correctness; clamping at boundaries.
- **`emitter.rs`**: shape constraints (sphere bounded by radius, cone within angle, box within bounds); deterministic with fixed seed.
- **`forces.rs`**: gravity adds `gravity * dt` to velocity; drag decays exponentially; curl noise is divergence-free; vortex/attractor force directions correct.
- **`collision.rs`**: ground plane reflects + dampens; friction reduces horizontal velocity; terrain heightfield query returns expected height.
- **`subemit.rs`**: death events collected before dispatch (no mid-iteration mutation); unknown template names warn and skip; cycle detection rejects A→B→A.
- **`system.rs`**: end-to-end tick — 1-particle non-looped dies at lifetime; looped system reaches steady state; prewarm matches steady state on creation; sub-emit on death adds child instance.

### Integration test — example program

`examples/particles.rs` is the visual acceptance test (see above).

### Performance budget

- 10k particles, 60 fps target; <2 ms total particle frame cost
- CPU update: <1 ms with rayon
- GPU render: <0.5 ms for sprite + ribbon
- Memory: <16 MB across all active systems

Tracked via `criterion` benchmarks added under `benches/` as a `dev-dependency`. Not hard gates for v1; baseline for regression detection.

## Out of scope (v2+)

- GPU simulation (transform feedback or compute)
- Mesh particles
- Decal-on-impact system
- Order-independent transparency
- Hot reload (deferred to editor app)
- Volumetric / raymarched particles
- Custom per-system shaders beyond the two ship presets
- Motion blur on particles
- Per-particle audio triggers

## Serialization (engine-level)

`AppStateSerializer` gains:

```rust
pub struct SerializedParticleSystem {
    pub config:    ParticleSystemConfig,
    pub transform: Mat4,
    pub parent:    Option<Uuid>,
    pub elapsed:   f32,
}
```

Live particle buffers are NOT serialized — only the system's template and emission state. A reload causes the system to resume from `elapsed`, re-prewarming if configured. This matches the "scene state, not transient" contract of the existing `AppStateSerializer`.

## Dependencies added

- `serde_json` — likely already present transitively; verify
- `rayon` — for parallel update loops
- `arraydeque` or equivalent fixed-capacity deque (or a hand-rolled small ring for trails)
- `criterion` (dev-dependency only) — for performance benchmarks

No new runtime dependencies that affect ship binary size meaningfully.

## Open questions resolved before implementation

None outstanding. Ready for implementation planning.
