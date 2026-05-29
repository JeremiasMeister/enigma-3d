# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build the library
cargo build

# Run examples (requires feature flag due to size)
cargo run --example=engine --features=examples
cargo run --example=chessboard --features=examples
cargo run --example=skinned-mesh --features=examples

# Check without building
cargo check

# Run tests
cargo test

# Publish to crates.io
cargo publish
```

There is no separate lint step — `cargo check` and `cargo build` surface all warnings via compiler output.

## Architecture

enigma-3d is a Rust 3D rendering library built on **glium** (OpenGL) and **winit**. The two central types are `EventLoop` and `AppState`, defined in `src/lib.rs`.

### EventLoop + AppState pattern

`EventLoop` owns the window, display, and event loop. `AppState` owns all scene data — objects, materials, lights, camera, audio, post-processing effects, and arbitrary user state. The user constructs both, populates `AppState`, then calls `event_loop.run(app_state.convert_to_arc_mutex())`.

Inside `run`, the loop processes winit events and dispatches them to injected functions. The rendering sequence per frame is:
1. Opaque objects (back-to-front sorted for transparency correctness)
2. Skybox
3. Transparent objects
4. Post-processing effects (each gets the framebuffer + auxiliary textures)
5. Final blit to screen + egui GUI pass

### Injection model

Behavior is added by injecting functions — not by subclassing. Three injection points exist:
- `inject_event(characteristic, fn, modifiers)` — maps key/mouse events to `fn(&mut AppState)`
- `inject_update_function(fn)` — called every frame before redraw
- `inject_gui(fn)` — called with `(egui_context, &mut AppState)` for UI

All injected functions have signature `fn(&mut AppState)`. They can read/write scene data and the typed state bag (`app_state.add_state_data` / `get_state_data_value`).

### Key modules

| Module | Responsibility |
|---|---|
| `object.rs` | Scene objects with transform, shapes (submeshes), material refs, skeleton; GPU instancing via `ObjectInstance` |
| `material.rs` | Shader program + texture slots (albedo, normal, roughness, metallic, emissive, AO); `Material::lit_pbr()` and `Material::unlit()` are the main constructors |
| `shader.rs` | GLSL shader loading from embedded resources or file |
| `geometry.rs` | Vertex/index buffer types, `InstanceAttribute` for GPU instancing, `BoneTransforms` uniform buffer |
| `postprocessing/` | Trait `PostProcessingEffect`; implementations: bloom, edge, grayscale, lens_dirt, vignette, depth_fog |
| `animation.rs` | Skeletal animation clips and playback state on `Object` |
| `collision_world.rs` | Pull-query collision: `Collider`, `Hit`, `RayCast`, `ShapeCast`, `ShapeOverlap` (see Collision system section) |
| `audio.rs` | `AudioEngine` (rodio-backed) + `AudioClip`; supports one-shot and looping playback |
| `resources.rs` | `include_bytes!`/`include_str!` embedded engine assets (skybox, icon, lens dirt) |
| `example_resources.rs` | Same pattern for example-only assets; gated behind `features=examples` |
| `logging/` | `EnigmaError`, `EnigmaWarning`, `EnigmaMessage` — colored terminal output |
| `particle/` | CPU-simulated particle system (sprite + ribbon, JSON-authorable); update tick runs between transparent and post-processing passes |

### Serialization

`AppState` can be serialized to/from JSON via `AppStateSerializer`. Only scene data is serialized (objects, camera, lights, materials, skybox) — injected functions and audio are not. Use `app_state.to_serializer()` and `inject_serializer()` to save/load scene state at runtime.

### GPU instancing

Objects that share the same `instance_id` (set by cloning an object) are batched into a single draw call. The `setup_instances` method in `AppState` collects all model matrices per instance group into a `VertexBuffer<InstanceAttribute>` each frame.

### Render resources

Shaders live in `src/res/shader/` as `.glsl` files embedded at compile time via `build.rs`. Textures and models for the engine itself (skybox, icon) are in `src/res/`. The `build.rs` script generates `resources.rs` with `include_bytes!` accessors for all embedded assets.

### Skeletal animation

Fully implemented. Key details:
- `Skeleton.root_transform` — world-space transform of the armature node (parent of root joints). Applied to root bones instead of identity so IBMs cancel correctly. Handles Blender's Z-up→Y-up correction (90° X + 0.01 scale) automatically.
- `Bone.node_index` — GLTF node index used to match animation channels. Distinct from `bone.id` (joint array index). `bone.parent_id` is also a joint array index.
- `BoneTransforms` in `geometry.rs` — `implement_uniform_block!` crashes in debug mode (glium-0.33 null pointer bug). Uses a manual `UniformBlock` impl with `matches` returning `Ok(())`.
- GLSL uniform block requires `layout(std140)` for glium's `UniformBuffer` binding to work.
- `get_uniforms` in `material.rs` must pass both `BoneTransforms` buffer and `has_skeleton` to the shader — easy to accidentally leave disabled.

### Shadow system

Hard shadows for up to 4 source lights (directional + point), implemented in `src/shadow.rs`. Key details:
- **Point light maps** use an R32F `Texture2d` 2×3 atlas instead of `DepthCubemap` — glium 0.33 cannot render to individual cubemap faces as depth attachments.
- Atlas layout: face order +X/−X/+Y/−Y/+Z/−Z; col = face%2, row = face/2; `glium::Rect.bottom = row * res` (OpenGL origin at bottom).
- **Shadow depth vertex shader** (`shadow_depth_vert.glsl`) uses `in mat4 model_matrix` — a per-instance vertex attribute, NOT a uniform. Matches the main vertex shader's instancing pattern.
- **Bias** in `enigma_fragment_shader.glsl` is slope-scale: `max(0.005 * (1.0 - n·l), 0.0002)`. A constant bias causes distance-dependent floating artifacts because it ignores surface slope.
- `get_uniforms` in `material.rs` accepts `shadow_maps: &ShadowMaps` as last parameter; 15 shadow uniforms bound; unused slots fall back to `shadow_maps.dummy` (1×1 white texture).

### Particle system

CPU-simulated, Shuriken-style fixed pipeline. Systems are JSON-authorable (`ParticleSystem::from_file` / `from_resource`) and registered on `AppState` via `add_particle_system` / `add_particle_system_template`. Sub-emitters reference templates by name. Materials assigned via UUID, with `Material::particle_sprite(display)` and `Material::particle_ribbon(display)` as preset factories. See `docs/superpowers/specs/2026-05-24-particle-system-design.md` for the full design and `examples/particles.rs` for usage.

Key files:
- `src/particle/system.rs` — `ParticleSystem` with builder API + tick pipeline
- `src/particle/config.rs` — `ParticleSystemConfig` (serde-derived JSON shape)
- `src/particle/buffer.rs` — SoA particle buffer with `compact_dead`
- `src/particle/render/` — sprite + ribbon GPU passes
- `src/res/shader/particle_*.glsl` — embedded particle shaders

Known v1 limitations (deferred to follow-up):
- Hero per-particle lights are extracted via `ParticleSystem::extract_hero_lights()` but not yet injected into the engine's 4-light cap (would require a scratch-lights list in `AppState`).
- Terrain heightfield collision uses a stub `Terrain::heightfield_query_fn` that returns `None`; ground-plane collision works fully.
- Soft particles require the scene depth texture passed in to `ParticleRenderer::render`; the current `EventLoop::run` integration does pass it, so this works.
- Click-to-spawn in the example uses a hardcoded position; a real ray-pick from cursor is a future enhancement.
- No example textures: particle materials in `examples/particles.rs` have no albedo, so sprites/ribbons render as silent skips. Add a small white circle PNG to see particles visually.
- No `AppStateSerializer` integration for particle systems — save/load only persists scene objects today.
- Cycle detection on sub-emitter templates is not implemented (e.g., A.on_death → B.on_death → A would create infinite spawns); rely on author discipline for now.

### Collision system

Pull-query collision in `src/collision_world.rs`. Public types: `Collider` (7 variants: `Sphere`, `Aabb`, `Obb`, `Capsule`, `Plane`, `MeshCollider`, `Heightfield`), `Hit { point, normal, penetration, object_id }`, and the three query entry points `RayCast`, `ShapeCast`, `ShapeOverlap`. Each object carries `Object.collider: Option<Collider>`; when `None`, queries fall back to the mesh AABB so existing scenes keep working.

- **Object-space colliders.** `Collider::to_world_space(&Transform)` applies position/rotation/scale once at query time; authors define colliders in local space.
- **Static-vs-static convention.** Meshes, planes, and heightfields are queried *against* but never used as the moving shape in `ShapeCast`/`ShapeOverlap` — those arms log a warning and return `None`. Use spheres/AABBs/OBBs/capsules as the moving shape.
- **Iterative sweep.** `ShapeCast` substeps the motion (default 16 steps) and returns the first overlap, since v1 has no analytic swept TOI per primitive pair.
- **Terrain integration.** `Terrain::height_at`, `normal_at`, and `triangles_in_aabb` back the `Heightfield` arm and ray-vs-terrain in `RayCast`.
- **Character controller.** `default_events::{CharacterState, character_move_forward/back/left/right, character_jump, character_update}` — kinematic capsule with slide-on-hit, gravity, and ground-snap. See `examples/character_controller.rs`.

Known v1 non-goals (deferred): no broadphase (linear scan), no mesh-vs-mesh / mesh-vs-heightfield / heightfield-vs-heightfield (return `None` + warn), no analytic capsule ray, no rigid-body dynamics, no collision layers/masks, no compound colliders, no push events (pull queries only).
