# Particle System

A CPU-simulated, JSON-authorable particle system for enigma-3d. Shuriken-style fixed pipeline, structure-of-arrays particle buffer, two render styles (sprite + ribbon), sub-emitters by template name, optional hero per-particle lights.

The runtime is engine code; the authoring format is JSON loaded via `ParticleSystem::from_file` / `from_resource`. Every JSON field is also reachable through a Rust setter on `ParticleSystem`, so you can author entirely in code, entirely in JSON, or mix.

---

## Contents

1. [Mental model](#mental-model)
2. [Quick start](#quick-start)
3. [Engine API](#engine-api)
4. [JSON reference](#json-reference)
5. [Render styles](#render-styles)
6. [Sub-emitters](#sub-emitters)
7. [Materials and textures](#materials-and-textures)
8. [Hero lights](#hero-lights)
9. [Determinism](#determinism)
10. [Update pipeline](#update-pipeline)
11. [Performance budget](#performance-budget)
12. [Known v1 limitations](#known-v1-limitations)

---

## Mental model

A `ParticleSystem` is a self-contained emitter. It holds:

- A **config** (`ParticleSystemConfig`) — the JSON-shaped description.
- A **buffer** (`ParticleBuffer`) — a dense SoA array of live particle state.
- A **state** (`EmissionState`) — tick counters, spawn accumulator, finished flag.
- A **transform** — world position, rotation, scale (where new particles spawn).
- A **material reference** by `Uuid` (the same `Material` system used by `Object`).
- An optional **parent object** (not yet wired — see limitations).

Two render styles are available:

- **Sprite**: each particle is a camera-facing billboard quad. Supports flipbook atlases, soft particles, velocity stretching.
- **Ribbon**: each particle leaves a trail. CPU rebuilds a triangle-strip mesh from the trail history every frame.

Every particle ages from 0 to its `lifetime` and is removed when it expires. Configurable forces (gravity, drag, curl noise, vortex, attractor) modify velocity per tick. Optional curves modulate color, size, and rotation over the normalized lifetime `t = age / lifetime`. Optional collision against a ground plane or terrain heightfield reflects velocity and can trigger a sub-emitter.

The render pass runs in `EventLoop::run` between the transparent object pass and post-processing, so HDR-bright particle colors get picked up by the bloom post-process for free.

---

## Quick start

```rust
use std::sync::Arc;
use enigma_3d::{AppState, EventLoop, init_default};
use enigma_3d::material::{Material, TextureType};
use enigma_3d::particle::ParticleSystem;
use enigma_3d::resources;

let event_loop = EventLoop::new("Demo", 1280, 720);
let mut app = AppState::new();
init_default(&mut app);
let display = event_loop.get_display_clone();

// 1) Material: a particle-sprite preset with an albedo texture.
let mut mat = Material::particle_sprite(display.clone());
mat.set_name("fire");
mat.set_texture_from_resource(resources::vfx_sprite_sheet(), TextureType::Albedo);
app.add_material(mat);
let mat_id = app.materials.iter().find(|m| m.name == "fire").unwrap().uuid;

// 2) System: load from embedded JSON, attach the material, place it, register.
let mut fireball = ParticleSystem::from_resource(resources::vfx_fireball()).unwrap();
fireball.set_material(mat_id);
fireball.set_position([0.0, 2.0, 0.0]);
app.add_particle_system(fireball);

event_loop.run(app.convert_to_arc_mutex());
```

Equivalent JSON (one of the bundled assets, `src/res/vfx/fireball.json`):

```json
{
  "name": "fireball",
  "max_particles": 256,
  "looped": true,
  "duration": 1.0,
  "emission_rate": 120.0,
  "bursts": [],
  "emitter_shape": { "Sphere": { "radius": 0.15 } },
  "initial_velocity": { "Outward": { "speed": { "min": 0.5, "max": 1.5 } } },
  "initial_lifetime": { "min": 0.6, "max": 1.0 },
  "initial_size":     { "min": 0.3, "max": 0.55 },
  "initial_rotation": { "min": 0.0, "max": 6.283 },
  "initial_color": { "Hdr": { "rgb": [1.0, 0.55, 0.15], "intensity": 25.0, "alpha": 1.0 } },
  "color_over_life": { "keys": [[0.0, [1.0, 0.6, 0.2, 1.0]], [1.0, [0.2, 0.1, 0.05, 0.0]]] },
  "size_over_life": { "keys": [[0.0, 0.4], [1.0, 0.0]] },
  "drag": 1.8,
  "curl_noise": { "scale": 1.5, "strength": 2.0, "evolution_speed": 0.5 },
  "light_emit": { "max_lights": 1, "color_multiplier": [1.0, 0.5, 0.15], "intensity": 4.0, "falloff": 6.0 },
  "on_death": "fireball_smoke",
  "render": { "kind": "Sprite", "blend_mode": "Additive", "soft_particles": true, "soft_fade_distance": 0.3, "velocity_stretch": 0.0 }
}
```

---

## Engine API

### Constructing a `ParticleSystem`

```rust
ParticleSystem::new(name: &str) -> Self
ParticleSystem::from_file(path: &str) -> Result<Self, EnigmaError>
ParticleSystem::from_resource(json: &str) -> Result<Self, EnigmaError>
```

`new` creates a minimal default system. `from_file` reads JSON from disk. `from_resource` parses an embedded `include_str!` string. Both parsing constructors run `validate()` and return an error on malformed configs.

### Registering with `AppState`

```rust
app.add_particle_system(system);
app.add_particle_system_template(system);   // registers but does not play
app.emit_particle_system(template_name: &str, transform: [[f32; 4]; 4]) -> Option<Uuid>;

app.get_particle_system_by_name(name) -> Option<&ParticleSystem>
app.get_particle_system_by_name_mut(name) -> Option<&mut ParticleSystem>
app.get_particle_system(uuid) -> Option<&ParticleSystem>
app.get_particle_system_mut(uuid) -> Option<&mut ParticleSystem>

app.remove_particle_system(uuid);
app.remove_particle_systems_by_name(name);
```

`add_particle_system` adds a live system that starts emitting immediately. `add_particle_system_template` adds a system in stopped state — it is only used as a source for `emit_particle_system` or as a sub-emitter target referenced by name.

`emit_particle_system(name, transform)` clones the named template's config, applies the transform, and adds a live instance. Returns the new instance's UUID. Returns `None` if the name does not resolve.

`tick_particles(dt)` is called automatically inside `EventLoop::run`. You do not call it yourself.

### Builder-style setters

Every config field has a setter. Common ones:

```rust
// Identity & lookup
sys.set_name(name)
sys.name() -> &str
sys.get_unique_id() -> Uuid

// Placement (apply in this order: position, then rotation, then scale)
sys.set_position([x, y, z])
sys.set_rotation([rx, ry, rz])     // euler radians, ZYX order
sys.set_scale([sx, sy, sz])
sys.set_local_offset(mat4)
sys.set_parent_object(uuid)        // currently a no-op; see limitations

// Lifetime & emission
sys.set_max_particles(n)           // reallocates the buffer
sys.set_looped(bool)
sys.set_duration(seconds)
sys.set_prewarm(bool)
sys.set_emission_rate(particles_per_second)
sys.add_burst(time, count)
sys.clear_bursts()
sys.set_seed(u64)                  // deterministic RNG

// Initial state
sys.set_emitter_shape(EmitterShape::Sphere { radius: 0.5 })
sys.set_initial_velocity(InitialVelocity::Outward { speed: Range::new(1.0, 2.0) })
sys.set_initial_lifetime(min, max)
sys.set_initial_size(min, max)
sys.set_initial_rotation(min, max)
sys.set_initial_color(ColorRange::Hdr { rgb: [1.0, 0.5, 0.1], intensity: 20.0, alpha: 1.0 })

// Over-life modulators
sys.set_color_over_life(gradient)
sys.set_size_over_life(curve)
sys.set_rotation_over_life(curve)
sys.set_angular_velocity(min, max)

// Forces
sys.set_gravity([0.0, -9.81, 0.0])
sys.set_drag(1.5)
sys.set_curl_noise(CurlNoiseConfig { scale, strength, evolution_speed })
sys.set_vortex(VortexConfig { center, axis, strength, falloff })
sys.set_attractor(AttractorConfig { center, strength, falloff })

// Collision
sys.set_collision(CollisionConfig { ground_plane_y, terrain, bounce, friction })

// Sub-emitters (referenced by template name)
sys.set_on_death(template_name)
sys.set_on_collision(template_name)

// Lights
sys.set_light_emit(LightEmitConfig { max_lights, color_multiplier, intensity, falloff })

// Material & rendering
sys.set_material(material_uuid)
sys.set_render_style(RenderStyle::Sprite { .. } | RenderStyle::Ribbon { .. })
sys.set_blend_mode(BlendMode::Additive | Alpha | PremultipliedAlpha)
sys.set_flipbook(FlipbookConfig { .. })
sys.set_soft_particles(enabled, fade_distance)
sys.set_velocity_stretch(stretch)

// Runtime control
sys.play()
sys.stop()
sys.restart()                      // clears buffer and state
sys.is_finished() -> bool
sys.live_count() -> usize

// Persistence
sys.to_json() -> Result<String, EnigmaError>
sys.save_to_file(path) -> Result<(), EnigmaError>
```

### `Material` factories

```rust
Material::particle_sprite(display) -> Material
Material::particle_ribbon(display) -> Material
```

These set up the appropriate vertex/fragment shader pair and enable transparency. Add an albedo texture with `set_texture_from_file` or `set_texture_from_resource`. If you skip the albedo, the system falls back to a 1×1 white texture so particles still render as their HDR instance color.

---

## JSON reference

All field types derive `serde::{Serialize, Deserialize}`. `Option<>` fields are skipped from output when `None`, so authored files stay lean.

### Top-level: `ParticleSystemConfig`

```json
{
  "name": "string",
  "max_particles": 256,
  "looped": true,
  "duration": 1.0,
  "prewarm": false,
  "seed": 12345,

  "emission_rate": 120.0,
  "bursts": [{ "time": 0.0, "count": 8 }],

  "emitter_shape": { ... },
  "initial_velocity": { ... },
  "initial_lifetime": { "min": 0.6, "max": 1.0 },
  "initial_size":     { "min": 0.3, "max": 0.55 },
  "initial_color":    { ... },
  "initial_rotation": { "min": 0.0, "max": 6.283 },

  "color_over_life":    { "keys": [...] },
  "size_over_life":     { "keys": [...] },
  "rotation_over_life": { "keys": [...] },
  "angular_velocity":   { "min": -1.0, "max": 1.0 },

  "gravity":    [0.0, -9.81, 0.0],
  "drag":       1.5,
  "curl_noise": { ... },
  "vortex":     { ... },
  "attractor":  { ... },

  "collision":    { ... },
  "on_death":     "template_name",
  "on_collision": "template_name",
  "light_emit":   { ... },

  "render": { ... }
}
```

#### Required fields

| Field | Type | Description |
|---|---|---|
| `name` | string | System name. Used for `get_*_by_name` lookups and as the target of sub-emitter references. |
| `max_particles` | usize | Hard cap on simultaneously alive particles. Buffer is preallocated at this capacity. Must be > 0. |
| `looped` | bool | If true, emission continues forever (ignores `duration`). If false, emission ends at `duration` and the system is reaped once all live particles die. |
| `duration` | f32 | Seconds of emission when not looped. Must be >= 0. (Zero is fine — emits only the bursts.) |
| `emission_rate` | f32 | Continuous emission rate, particles per second. Fractional rates accumulate across ticks. |
| `emitter_shape` | enum | Where new particles spawn. See [EmitterShape](#emittershape). |
| `initial_velocity` | enum | Initial velocity model. See [InitialVelocity](#initialvelocity). |
| `initial_lifetime` | Range<f32> | Per-particle lifetime, uniformly sampled. Min must be > 0. |
| `initial_size` | Range<f32> | Per-particle size at spawn. |
| `initial_color` | enum | Per-particle color at spawn. See [ColorRange](#colorrange). |
| `render` | enum | Render style. See [RenderStyle](#render-styles). |

#### Optional fields

| Field | Type | Default | Description |
|---|---|---|---|
| `prewarm` | bool | false | (Reserved; not yet implemented as a full warm-up tick.) |
| `seed` | u64 | wall-clock | Per-system RNG seed. Setting this makes the simulation deterministic. |
| `bursts` | `[BurstSpec]` | `[]` | Burst events: `{ "time": f32, "count": u32 }`. Fired once when `elapsed` passes `time`. A burst at `time: 0.0` fires on the first tick. |
| `initial_rotation` | Range<f32> | `{0,0}` | Initial 2D rotation (radians, around the camera axis for sprites). |
| `color_over_life` | Gradient<[f32;4]> | none | RGBA gradient evaluated at `t = age / lifetime`. Overwrites color. |
| `size_over_life` | Curve<f32> | none | Size multiplier evaluated at `t`. Overwrites size. |
| `rotation_over_life` | Curve<f32> | none | Rotation curve evaluated at `t`. Overwrites rotation. |
| `angular_velocity` | Range<f32> | none | Per-particle constant spin rate (radians/sec). Integrated each frame. |
| `gravity` | [f32; 3] | none | Constant acceleration applied to all particles. |
| `drag` | f32 | none | Exponential velocity damping. Velocity is multiplied by `exp(-drag * dt)` each tick. |
| `curl_noise` | CurlNoiseConfig | none | Divergence-free noise force. See [CurlNoiseConfig](#curlnoiseconfig). |
| `vortex` | VortexConfig | none | Tangential swirl force. See [VortexConfig](#vortexconfig). |
| `attractor` | AttractorConfig | none | Radial pull/push force. See [AttractorConfig](#attractorconfig). |
| `collision` | CollisionConfig | none | Collision response. See [CollisionConfig](#collisionconfig). |
| `on_death` | string | none | Template name; spawned at each particle's death position when it expires naturally. |
| `on_collision` | string | none | Template name; spawned at each particle's collision position when it hits ground or terrain. |
| `light_emit` | LightEmitConfig | none | Hero per-particle point lights. See [LightEmitConfig](#lightemitconfig). |

### `Range<T>`

```json
{ "min": 0.5, "max": 1.5 }
```

Inclusive on both sides. Sampling is uniform.

### `BurstSpec`

```json
{ "time": 0.0, "count": 8 }
```

- `time`: seconds since system start at which to fire.
- `count`: number of particles to spawn.

### `Curve<f32>` and `Gradient<[f32; 4]>`

Both are keyframe lists, `t` in [0, 1]:

```json
{ "keys": [[0.0, 1.0], [0.5, 2.0], [1.0, 0.0]] }
{ "keys": [[0.0, [1.0, 0.5, 0.0, 1.0]], [1.0, [0.0, 0.0, 0.0, 0.0]]] }
```

Linear interpolation between keys. Out-of-range `t` clamps to the first/last keyframe. Empty curves evaluate to `0.0` (Curve) or `[0,0,0,0]` (Gradient).

Validation: keyframe times must be sorted ascending and within [0, 1].

### `EmitterShape`

Where new particles spawn (local to the system's transform). Encoded as an externally-tagged enum:

```json
"Point"                                                    // single point at origin
{ "Sphere": { "radius": 0.5 } }                            // uniformly inside sphere
{ "Box":    { "half_extents": [1.0, 2.0, 3.0] } }          // uniformly inside box
{ "Cone":   { "angle": 0.5, "height": 1.0 } }              // inside upward cone (angle in radians)
{ "Disk":   { "radius": 0.5 } }                            // uniformly on XZ disk
```

Notes:

- `Sphere` uses rejection sampling — uniform inside the volume.
- `Cone` samples height uniformly along the axis; density is therefore higher near the apex.
- `Disk` is XZ-flat; rotate the system if you need it elsewhere.

### `InitialVelocity`

```json
{ "Outward":    { "speed": { "min": 0.5, "max": 1.5 } } }
{ "Direction":  { "direction": [0.0, 1.0, 0.0], "speed": { "min": 1.0, "max": 2.0 } } }
{ "Cone":       { "direction": [0.0, 1.0, 0.0], "angle": 0.6, "speed": { "min": 0.4, "max": 1.0 } } }
{ "Hemisphere": { "normal":    [0.0, 1.0, 0.0], "speed": { "min": 4.0, "max": 9.0 } } }
```

- `Outward`: from the system's spawn point outward along the per-particle local position. Falls back to `[0, 1, 0]` at the origin.
- `Direction`: fixed direction. The direction vector is normalized internally.
- `Cone`: random direction within `angle` radians of the axis (uniform over the spherical cap).
- `Hemisphere`: random direction in the half-space defined by `normal`.

### `ColorRange`

Per-particle spawn color.

```json
{ "Single":   [1.0, 1.0, 1.0, 1.0] }
{ "Range":    { "min": [0.5, 0.5, 0.5, 1.0], "max": [1.0, 1.0, 1.0, 1.0] } }
{ "Hdr":      { "rgb": [1.0, 0.5, 0.1], "intensity": 25.0, "alpha": 1.0 } }
{ "HdrRange": {
    "rgb_min": [1.0, 0.4, 0.1], "rgb_max": [1.0, 0.7, 0.3],
    "intensity": { "min": 15.0, "max": 30.0 },
    "alpha":     { "min": 0.8, "max": 1.0 }
  }
}
```

`Hdr` and `HdrRange` produce colors above [0, 1]. The render path writes into the engine's HDR framebuffer, so values above 1.0 propagate to the bloom post-process for a natural glow.

### `VortexConfig`

```json
{ "center": [0.0, 0.0, 0.0], "axis": [0.0, 1.0, 0.0], "strength": 5.0, "falloff": 1.0 }
```

Tangential force perpendicular to both `axis` and the vector from `center` to the particle. Magnitude is `strength / (1 + falloff * distance)`.

### `AttractorConfig`

```json
{ "center": [0.0, 0.0, 0.0], "strength": 3.0, "falloff": 1.0 }
```

Radial force toward `center`. Positive `strength` pulls inward; negative pushes outward. Magnitude is `strength / (1 + falloff * distance)`.

### `CurlNoiseConfig`

```json
{ "scale": 1.5, "strength": 2.0, "evolution_speed": 0.5 }
```

Divergence-free pseudorandom velocity field (the swirly, non-compressing kind you want for fire and smoke).

- `scale`: spatial frequency — bigger means smaller swirls.
- `strength`: force magnitude multiplier.
- `evolution_speed`: how fast the field changes over time (seconds).

### `CollisionConfig`

```json
{ "ground_plane_y": 0.0, "terrain": true, "bounce": 0.4, "friction": 0.3 }
```

- `ground_plane_y`: if set, an infinite horizontal plane at this Y. Particles below it are clamped to it and reflected.
- `terrain`: if true, queries the engine's terrain heightfield. **Currently a no-op stub returning None — see limitations.**
- `bounce`: vertical-velocity multiplier on collision. `0.0` = no bounce, `1.0` = elastic.
- `friction`: horizontal-velocity damping on collision. `0.0` = frictionless, `1.0` = full stop.

### `LightEmitConfig`

```json
{ "max_lights": 1, "color_multiplier": [1.0, 0.5, 0.15], "intensity": 4.0, "falloff": 6.0 }
```

- `max_lights`: how many of the system's particles emit light.
- `color_multiplier`: multiplied into the particle color when constructing the light.
- `intensity`: light intensity (matches `Light::new`'s intensity scale).
- `falloff`: reserved for future distance attenuation work.

**Note: extracted but not yet injected into the engine's light array. The data is available via `ParticleSystem::extract_hero_lights()` but no automatic wiring exists. Treat as v2.**

### `FlipbookConfig` (sprite render style only)

```json
{
  "cols": 4,
  "rows": 4,
  "frame_count": 16,
  "fps": 24.0,
  "blend": false,
  "randomize_start_frame": false,
  "fixed_frame": 7
}
```

- `cols`, `rows`: atlas grid layout.
- `frame_count`: number of valid frames in the atlas (top-to-bottom, left-to-right reading order; `<= cols * rows`).
- `fps`: animation speed. Set to `0.0` to freeze the frame.
- `blend`: cross-fade between frames. **Currently parsed but not yet implemented in the fragment shader.**
- `randomize_start_frame`: if true, each spawned particle starts on a random frame instead of frame 0. Combine with `fps > 0` for desynchronized loops, or with `fps = 0` for per-particle variation.
- `fixed_frame`: if set, every particle in the system renders that specific atlas cell, ignoring per-particle frame state. Use to lock a whole system to one sprite of a variation atlas.

Three common combinations:

| Goal | Config |
|---|---|
| Animated atlas, all particles synced | `fps: 24.0, randomize_start_frame: false` |
| Animated atlas, desynchronized | `fps: 24.0, randomize_start_frame: true` |
| Static random-cell-per-particle | `fps: 0.0, randomize_start_frame: true` |
| Static one-cell-per-system | `fps: 0.0, fixed_frame: Some(n)` |

---

## Render styles

### `Sprite`

```json
{
  "kind": "Sprite",
  "blend_mode": "Additive",
  "soft_particles": true,
  "soft_fade_distance": 0.3,
  "velocity_stretch": 0.0,
  "flipbook": { ... }
}
```

- `kind`: discriminator, must be `"Sprite"`.
- `blend_mode`: `"Additive"`, `"Alpha"`, or `"PremultipliedAlpha"`. Additive is the typical choice for glowing magic and impact effects; Alpha is the typical choice for smoke and dust.
- `soft_particles`: when true, the fragment shader fades alpha as the particle approaches scene geometry — prevents the hard "sticker on a wall" look.
- `soft_fade_distance`: world-space distance over which soft-particle alpha fades from 0 to 1. Typical: 0.2 to 0.5 meters.
- `velocity_stretch`: amount of motion-aligned elongation. `0.0` = always camera-aligned billboard; higher values stretch the quad along projected velocity proportional to speed. Good for sparks.
- `flipbook`: optional. See [FlipbookConfig](#flipbookconfig-sprite-render-style-only).

Each sprite particle is rendered as a single instanced quad. The vertex shader builds a camera-facing billboard from the view matrix; the fragment shader samples the material's albedo (or a 1×1 white fallback if none).

### `Ribbon`

```json
{
  "kind": "Ribbon",
  "max_trail_points": 12,
  "width_over_life": { "keys": [[0.0, 1.0], [1.0, 0.0]] },
  "blend_mode": "Additive"
}
```

- `kind`: discriminator, must be `"Ribbon"`.
- `max_trail_points`: how many past positions to remember per particle. More points = smoother curves but more vertex work each frame.
- `width_over_life`: ribbon width along the trail. `t = 0` is the particle's current position; `t = 1` is the oldest stored point.
- `blend_mode`: same options as sprite.

Each frame the CPU walks every alive particle's trail history, extrudes a strip of camera-facing quads from it, and uploads to a dynamic vertex buffer. Drawn as `TrianglesList`. Texture is sampled from the assigned `Material::particle_ribbon` material.

Ribbons cost about 4 vertices and 6 indices per trail segment per particle per frame. A scene with 100 active ribbon particles × 12 trail points = ~5KB/frame, trivial.

---

## Sub-emitters

A particle system can spawn other registered templates when its particles die or collide:

```json
"on_death": "fireball_smoke",
"on_collision": "spark_dust"
```

The name must match the `name` field of a registered template. Register templates with `add_particle_system_template`:

```rust
let mut smoke = ParticleSystem::from_resource(resources::vfx_fireball_smoke())?;
smoke.set_material(mat_id);
app.add_particle_system_template(smoke);
```

When a parent particle expires or collides, the engine builds a fresh instance from the template, places it at the event position, and adds it to the system list. The new instance ticks normally from the next frame.

Sub-emitters can nest arbitrarily (a parent's sub-emitter can have its own `on_death`). **Cycle detection is not yet implemented** — avoid A → B → A configurations.

The example program demonstrates the pattern: `fireball` references `fireball_smoke` via `on_death`, and `hit_sparks` references `spark_dust` via `on_collision`.

---

## Materials and textures

Particle systems do not own their own textures — they reference a `Material` by UUID, exactly like `Object` does. Two material preset factories are provided:

```rust
Material::particle_sprite(display)    // attaches the particle sprite shaders
Material::particle_ribbon(display)    // attaches the particle ribbon shaders
```

Both are pre-configured for transparency. Add an albedo texture and you're done:

```rust
let mut mat = Material::particle_sprite(display.clone());
mat.set_name("fire");
mat.set_texture_from_resource(resources::vfx_sprite_sheet(), TextureType::Albedo);
// or: mat.set_texture_from_file("path/to/atlas.png", TextureType::Albedo);
app.add_material(mat);

let mat_id = app.materials.iter().find(|m| m.name == "fire").unwrap().uuid;
my_system.set_material(mat_id);
```

If a system's material has no albedo, the renderer falls back to a 1×1 white texture (`ParticleRenderer::fallback_albedo`). Particles still render — as flat-shaded quads colored by their HDR instance color — but you lose any shape, softness, or atlas variation.

The render style on the system **must match** the material's shader: a Sprite-style system needs a `particle_sprite` material; a Ribbon-style system needs a `particle_ribbon` material. Mixing them produces nothing visible (and potentially a GL warning).

---

## Hero lights

Each system can flag a few of its particles to emit point lights:

```json
"light_emit": { "max_lights": 1, "color_multiplier": [1.0, 0.5, 0.15], "intensity": 4.0, "falloff": 6.0 }
```

`ParticleSystem::extract_hero_lights() -> Vec<Light>` selects the `max_lights` particles with the highest "energy" score (alpha × mid-life curve), multiplies their color by `color_multiplier`, and returns them as `Light` values.

**Status:** the extraction works correctly, but the engine does not yet automatically inject these into the per-frame light array (the engine has a hard cap of 4 lights total). You can call `extract_hero_lights()` yourself and add them via `app.add_light(...)` before the render pass if you want them today. Proper integration via a scratch-lights list is on the v2 list.

---

## Determinism

Each system holds its own `SmallRng`. If `seed` is set on the config (or via `set_seed`), the simulation is fully deterministic: same seed + same `dt` sequence → identical particle trajectories. Useful for tests, deterministic replays, or reproducible visual debugging.

If `seed` is `None`, the system seeds from wall-clock nanoseconds at construction time.

Sub-emitters get their own fresh seed each time they spawn — they do not inherit the parent's RNG.

---

## Update pipeline

Per frame, per system, `tick(dt)` runs these steps in order:

1. **Advance time** — increment `elapsed`, accumulate the continuous spawn budget.
2. **Spawn** — fire bursts that crossed their trigger time; spawn integer particles from the accumulator. Initial state is sampled from the config (position, velocity, color, lifetime, size, rotation, seed). If a flipbook with `randomize_start_frame` is configured, set the new particle's frame to a random value.
3. **Age** — increment every particle's age by `dt` (including the newly spawned ones, so a `time: 0.0` burst's particles correctly die after `lifetime` seconds).
4. **Kill** — particles past their lifetime are removed via swap-remove from the SoA buffer; their position/velocity/color is captured for `on_death` sub-emit.
5. **Forces** — gravity + curl + vortex + attractor add to acceleration; velocity is updated by `accel * dt`; drag applies `velocity *= exp(-drag * dt)`.
6. **Integrate** — `position += velocity * dt`. Flipbook frame index advances by `fps * dt`.
7. **Collide** — test ground plane and terrain heightfield. On hit: reflect velocity, apply bounce + friction, queue an `on_collision` event.
8. **Curves** — evaluate color/size/rotation gradients at `t = age / lifetime` and write the results into the buffer. Integrate angular velocity (each particle's rate derived from its stable per-particle seed).
9. **Trails** — for ribbon systems, push the current position onto each particle's trail deque, truncating to `max_trail_points`.
10. **Finished check** — non-looped systems with `elapsed > duration` and zero live particles are flagged `finished` and reaped next frame.

Then `AppState::tick_particles` collects all queued death and collision events and dispatches `emit_particle_system` for each, and finally retains only non-finished systems.

---

## Performance budget

- Designed for **~10,000 simultaneously alive particles total** across all systems on screen.
- All simulation is **CPU-side, single-threaded** in v1. The hot loop is a straight pass over the SoA buffer; no virtual dispatch.
- Per particle memory: ~64 bytes (sprite) or ~256 bytes (ribbon with 12 trail points).
- 5–10 active systems × 1k particles each fits comfortably under 15 MB total.
- Rendering: one instanced draw call per sprite system; one CPU-built vertex buffer per ribbon system per frame.

The current implementation has not been benchmarked under load. Rayon parallelism over the force/integrate/curve passes is a known v2 follow-up.

---

## Known v1 limitations

These are explicit, documented gaps. Most have a one-day fix when needed:

- **`set_parent_object` is a no-op.** The field exists; `tick` does not read it. To attach a particle system to a moving object today, write an `inject_update_function` that calls `sys.set_position(...)` each frame.
- **Terrain heightfield collision is stubbed.** `Terrain::heightfield_query_fn` returns `None`. Ground-plane collision works fully; terrain-shaped collision will work once the stub is replaced.
- **Hero lights are extracted, not injected.** `extract_hero_lights()` produces correct `Light` values, but they are not yet added to the engine's light list automatically. The engine has a hard cap of 4 lights total.
- **No `AppStateSerializer` integration.** Scene save/load preserves objects, materials, lights, camera, skybox — but drops particle systems silently.
- **No sub-emitter cycle detection.** Authoring A.on_death → B.on_death → A will spawn an unbounded tree. Currently relies on author discipline.
- **`prewarm: true` is parsed but does not yet warm up the simulation** (it's a no-op).
- **`flipbook.blend: true` is parsed but the fragment shader does not cross-fade frames.** Frames snap.
- **No rayon parallelism in the tick.** Single-threaded inner loop.
- **No GPU simulation.** Mesh particles, GPU compute / transform feedback paths are explicit v2+ items.
- **No motion blur, no decals, no order-independent transparency.** Out of scope for v1.

---

## Related files

- Design spec: `docs/superpowers/specs/2026-05-24-particle-system-design.md`
- Implementation plan: `docs/superpowers/plans/2026-05-24-particle-system.md`
- Engine integration in `CLAUDE.md` (Particle system section)
- Example program: `examples/particles.rs`
- Bundled VFX assets: `src/res/vfx/*.json`
- Bundled particle textures: `src/res/textures/particles/*.png`
- Module source: `src/particle/`
