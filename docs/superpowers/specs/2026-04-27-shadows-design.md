# Shadow Mapping Design — enigma-3d

**Date:** 2026-04-27  
**Scope:** Hard shadow maps for up to 4 source lights (directional + point)

---

## Overview

Add a shadow pass before the main render that produces one depth texture per shadow-casting light, then sample those textures in the PBR fragment shader to attenuate each light's contribution. No soft shadows (PCF) in this iteration.

---

## Data Structures

### `ShadowResolution` enum — `src/light.rs`

```rust
pub enum ShadowResolution {
    Low,         // 512
    Medium,      // 1024
    High,        // 2048
    Ultra,       // 4096
    Custom(u32),
}

impl ShadowResolution {
    pub fn value(&self) -> u32 { ... }
}
```

### `ShadowMaps` struct — `src/shadow.rs` (new file)

```rust
pub struct ShadowMaps {
    pub directional_maps: [Option<DepthTexture2d>; 4],
    pub point_maps: [Option<DepthCubemap>; 4],
    pub light_space_matrices: [[[f32; 4]; 4]; 4],
    pub point_far_planes: [f32; 4],
    pub resolution: u32,
}
```

Allocated once in `EventLoop::run` alongside the main `texture`/`depth_texture`. Re-created when `shadow_resolution` changes.

### `AppState` additions — `src/lib.rs`

New fields:
```rust
shadow_resolution: u32,   // internal; set via ShadowResolution enum
shadow_distance: f32,     // orthographic half-extent for directional lights
```

New methods:
```rust
pub fn set_shadow_resolution(&mut self, resolution: ShadowResolution)
pub fn get_shadow_resolution(&self) -> u32
pub fn set_shadow_distance(&mut self, distance: f32)
pub fn get_shadow_distance(&self) -> f32
```

Defaults: `shadow_resolution = 1024`, `shadow_distance = 50.0`.

---

## Shadow Pass (render loop, `src/lib.rs`)

Runs before the main opaque pass each frame. Iterates over `light[0..4]`; for each where `cast_shadow == true`:

### Directional lights (`light.is_directional() == true`)

1. Build orthographic projection centered at camera position, looking along `light.direction`, half-extent = `shadow_distance`.
2. Render all opaque objects into `directional_maps[i]` using the depth-only shader.
3. Store the view-projection matrix in `light_space_matrices[i]`.

### Point lights (`light.is_directional() == false`)

1. Six render passes, one per cube face (`+X −X +Y −Y +Z −Z`).
2. Each pass uses a 90° perspective frustum (aspect 1:1) from `light.position`.
3. Renders into `point_maps[i]` (a `DepthCubemap`).
4. Fragment shader writes `length(frag_pos - light_pos) / far_plane` to `gl_FragDepth` (linear depth).
5. Stores `far_plane` in `point_far_planes[i]`.

---

## Depth-Only Shaders (new)

### `src/res/shader/shadow_depth_vert.glsl`

Minimal vertex shader: applies model matrix + light view-projection. Includes the same skeletal skinning path as the main vertex shader (reads `has_skeleton`, `BoneTransforms`, `bone_indices`, `bone_weights`).

### `src/res/shader/shadow_depth_dir_frag.glsl`

Empty `main()` — GL writes depth automatically for directional passes.

### `src/res/shader/shadow_depth_point_frag.glsl`

Writes `length(frag_world_pos - light_pos) / far_plane` to `gl_FragDepth` (linear depth in [0,1]).

Both passes share the same vert shader; the frag shader variant is selected per light type.

---

## Fragment Shader Changes — `enigma_fragment_shader.glsl`

### New uniforms

```glsl
uniform sampler2D  shadow_map_0, shadow_map_1, shadow_map_2, shadow_map_3;
uniform samplerCube shadow_cube_0, shadow_cube_1, shadow_cube_2, shadow_cube_3;
uniform mat4  shadow_light_space_0, shadow_light_space_1,
              shadow_light_space_2, shadow_light_space_3;
uniform vec4  shadow_far_planes;  // component i = far plane for point light slot i
uniform vec4  light_cast_shadow;  // component i = 1.0 if light i casts shadows
```

Unused slots are bound to a dummy 1×1 depth texture with depth=1.0 (maximum depth = never in shadow) on the Rust side.

### Shadow factor in PBR loop

After computing `reflection` for light `i`:

```glsl
float shadow = compute_shadow(i, world_position, light_position[i].xyz);
result += reflection * shadow;
```

### `compute_shadow` helper

Unrolled if-ladder (required by GLSL 330 — no dynamic sampler indexing):

```glsl
float compute_shadow(int i, vec3 world_pos, vec3 light_pos) {
    if (light_cast_shadow[i] < 0.5) return 1.0;
    bool is_dir = (light_direction[i].w == 1.0);
    if (i == 0) return is_dir
        ? dir_shadow(shadow_map_0, shadow_light_space_0, world_pos)
        : point_shadow(shadow_cube_0, world_pos, light_pos);
    if (i == 1) return is_dir
        ? dir_shadow(shadow_map_1, shadow_light_space_1, world_pos)
        : point_shadow(shadow_cube_1, world_pos, light_pos);
    if (i == 2) return is_dir
        ? dir_shadow(shadow_map_2, shadow_light_space_2, world_pos)
        : point_shadow(shadow_cube_2, world_pos, light_pos);
    if (i == 3) return is_dir
        ? dir_shadow(shadow_map_3, shadow_light_space_3, world_pos)
        : point_shadow(shadow_cube_3, world_pos, light_pos);
    return 1.0;
}
```

`dir_shadow`: transforms `world_pos` to light clip space, perspective divide → NDC, samples `shadow_map`, returns 0.0 if depth comparison fails (in shadow), 1.0 otherwise. Adds a small bias (`0.005`) to avoid shadow acne.

`point_shadow`: computes `length(world_pos - light_pos)`, samples `shadow_cube` using the `world_pos - light_pos` direction vector, compares `stored_depth × shadow_far_planes[i]` against computed distance.

---

## `material.rs` Changes

`get_uniforms()` receives an additional `shadow_maps: &ShadowMaps` parameter and binds all 8 samplers, 4 light-space matrices, and `shadow_far_planes` (vec4) into the uniform block.

---

## `resources.rs` Changes

Add `include_str!` accessors for the three new embedded shader files:
- `shadow_depth_vert_shader()`
- `shadow_depth_dir_frag_shader()`
- `shadow_depth_point_frag_shader()`

---

## Wiring in `EventLoop::run`

1. Allocate `ShadowMaps` after the main `texture`/`depth_texture` allocation.
2. Each `RedrawRequested`: run shadow pass → update `ShadowMaps` → run main render pass with updated shadow maps passed through `get_uniforms()`.
3. If `app_state.shadow_resolution` changes between frames, re-create `ShadowMaps` at the new resolution.

---

## What Is Not In Scope

- PCF / soft shadows
- Cascaded shadow maps (CSM) for directional lights
- Shadow bias exposed as a configurable parameter
- Transparent object shadows
