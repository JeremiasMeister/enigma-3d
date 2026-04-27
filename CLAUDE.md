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
| `collision_world.rs` | Screen-to-world ray casting and object selection |
| `audio.rs` | `AudioEngine` (rodio-backed) + `AudioClip`; supports one-shot and looping playback |
| `resources.rs` | `include_bytes!`/`include_str!` embedded engine assets (skybox, icon, lens dirt) |
| `example_resources.rs` | Same pattern for example-only assets; gated behind `features=examples` |
| `logging/` | `EnigmaError`, `EnigmaWarning`, `EnigmaMessage` — colored terminal output |

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
