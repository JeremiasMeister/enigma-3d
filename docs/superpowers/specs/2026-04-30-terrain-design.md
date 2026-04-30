# Terrain System Design

**Date:** 2026-04-30
**Status:** Approved

## Overview

A `Terrain` struct that generates a procedural heightmap mesh from fBm noise, colors vertices by height and slope, and exposes a `get_height(x, z) -> f32` method for player grounding. Rendered as an independent pass alongside opaque objects. Internally tiled for future streaming readiness.

## Constraints

- Single terrain per scene (stored as `AppState.terrain: Option<Terrain>`)
- No changes to `Object`, `Material`, or `Geometry`
- No new crate dependencies — gradient noise implemented in-library
- Vertex colors baked at build time; no per-frame CPU cost
- `get_height()` must be O(1), safe to call every frame

---

## Data Structures

### `TerrainConfig` (public)

| Field | Type | Default | Description |
|---|---|---|---|
| `width` | `f32` | `100.0` | World-space width (X axis) |
| `depth` | `f32` | `100.0` | World-space depth (Z axis) |
| `max_height` | `f32` | `15.0` | Maximum vertex Y displacement |
| `resolution` | `u32` | `128` | Quads per side across all tiles (vertices per side = resolution + 1) |
| `tile_count` | `u32` | `1` | Tiles per side (1 = single chunk, 2 = 2×2 grid) |
| `noise_scale` | `f32` | `0.03` | Base frequency of noise |
| `noise_amplitude` | `f32` | `1.0` | Multiplier on fBm output before applying `max_height` |
| `noise_octaves` | `u32` | `4` | Number of fBm layers |
| `noise_persistence` | `f32` | `0.5` | Amplitude decay per octave |
| `custom_noise` | `Option<Box<dyn Fn(f32, f32) -> f32 + Send + Sync>>` | `None` | If set, overrides fBm entirely |
| `color_flat_low` | `[f32; 3]` | `[0.22, 0.55, 0.12]` | Vertex color at low flat areas (grass) |
| `color_flat_high` | `[f32; 3]` | `[0.85, 0.85, 0.80]` | Vertex color at high flat areas (snow) |
| `color_slope` | `[f32; 3]` | `[0.40, 0.35, 0.28]` | Vertex color on steep faces (rock/dirt) |
| `slope_threshold` | `f32` | `0.75` | `normal.y` below this = full slope color (0.0–1.0) |
| `height_mid` | `f32` | `0.5` | Normalized height (0–1) where low→high flat color blend starts |

`TerrainConfig` implements `Default` with the values above.

### Internal types

```rust
// Vertex format uploaded to GPU
#[derive(Copy, Clone)]
struct TerrainVertex {
    position: [f32; 3],
    normal:   [f32; 3],
    color:    [f32; 3],
}

// One renderable chunk
struct TerrainTile {
    vertex_buffer: glium::VertexBuffer<TerrainVertex>,
    index_buffer:  glium::IndexBuffer<u32>,
    tile_x: i32,   // tile column index
    tile_z: i32,   // tile row index
}
```

### `Terrain` (public)

```rust
pub struct Terrain {
    tiles:     Vec<TerrainTile>,   // internal tile grid
    heightmap: Vec<f32>,           // flattened row-major, resolution² entries
    config:    TerrainConfig,
    program:   glium::Program,     // compiled terrain shader
    position:  [f32; 3],           // world-space origin (default [0,0,0])
}
```

---

## Public API

```rust
impl Terrain {
    pub fn new(display: &Display<WindowSurface>, config: TerrainConfig) -> Self
    pub fn get_height(&self, x: f32, z: f32) -> f32
    pub fn get_position(&self) -> [f32; 3]
    pub fn set_position(&mut self, pos: [f32; 3])
}

// AppState additions
pub fn set_terrain(&mut self, terrain: Terrain)
pub fn get_terrain(&self) -> Option<&Terrain>
pub fn get_terrain_mut(&mut self) -> Option<&mut Terrain>
```

---

## Mesh Generation

### Grid layout

Each tile covers `(width / tile_count) × (depth / tile_count)` world units. Each tile has `(verts_per_tile)²` vertices where `verts_per_tile = resolution / tile_count + 1`. Tiles are stitched seamlessly — shared edges sample the same heightmap coordinates.

Vertex XZ positions span `[-width/2, width/2]` × `[-depth/2, depth/2]` relative to `terrain.position`. Y is zero at the base, positive upward.

### Height sampling

For each vertex at world position `(vx, vz)`:

```
height = custom_noise(vx, vz)               // if custom_noise is Some
       OR fbm(vx * noise_scale, vz * noise_scale) * max_height * noise_amplitude
```

fBm implementation (no external crate):

```
fbm(x, z):
    value = 0, amplitude = 1, frequency = 1, max_value = 0
    for _ in 0..octaves:
        value     += gradient_noise(x * frequency, z * frequency) * amplitude
        max_value += amplitude
        amplitude *= persistence
        frequency *= 2
    return value / max_value   // normalized to roughly [-1, 1]
```

Gradient noise: permutation-table based, ~30 lines, sufficient quality for terrain.

### Normal calculation

Per vertex: average the cross products of all surrounding quad face normals. Computed from final Y positions, not from the noise gradient. Gives smooth shading across the whole mesh.

### Vertex color assignment

```
slope_factor  = 1.0 - clamp(normal.y / slope_threshold, 0.0, 1.0)
height_t      = clamp((height / max_height - height_mid) / (1.0 - height_mid), 0.0, 1.0)
flat_color    = mix(color_flat_low, color_flat_high, height_t)
vertex_color  = mix(flat_color, color_slope, slope_factor)
```

Colors are baked into the vertex buffer at build time. No per-frame CPU work.

### CPU heightmap

The flat `Vec<f32>` (`heightmap`) stores one entry per vertex, row-major in Z-then-X order, length `(resolution + 1)²`. The tile meshes are built from this same array — single source of truth.

---

## `get_height(x, z)`

1. Translate to terrain-local space: subtract `terrain.position.x/z`, add half-width/depth to shift origin to corner
2. Divide by cell size to get floating-point grid coordinates `(gx, gz)`
3. Clamp to `[0, resolution-1]` — positions outside terrain return edge height
4. Bilinear interpolation between four surrounding heightmap entries

O(1), no allocation. Safe to call every frame.

---

## Shader

Two new embedded GLSL files: `src/res/shader/terrain_vert.glsl`, `terrain_frag.glsl`.

**Vertex shader inputs:**
- `in vec3 position`, `in vec3 normal`, `in vec3 color`
- Uniforms: `model`, `view`, `projection`

**Fragment shader:**
- Diffuse lighting: `dot(normalize(normal), normalize(light_dir))` summed over up to 4 lights
- Plus ambient light contribution
- Base color is the interpolated vertex color
- No PBR — designed to be upgraded later without touching the mesh/data pipeline

The `glium::Program` is compiled once in `Terrain::new()` and stored on the struct.

---

## Rendering Integration (`lib.rs`)

`AppState` gains:
```rust
pub terrain: Option<terrain::Terrain>
```

In the render loop, a terrain pass runs **after opaque objects, before skybox**:

```
DrawParameters {
    depth: IfLess, write: true,
    backface_culling: CullClockwise,
}
```

Iterates `terrain.tiles`, draws each with the terrain program and scene lights/camera uniforms.

---

## File Changelist

| File | Change |
|---|---|
| `src/terrain.rs` | New — all terrain logic |
| `src/res/shader/terrain_vert.glsl` | New |
| `src/res/shader/terrain_frag.glsl` | New |
| `src/resources.rs` | Add `terrain_vert_shader()` and `terrain_frag_shader()` accessors |
| `build.rs` | Embed the two new shader files |
| `src/lib.rs` | Add `terrain` field, accessors, render pass |
