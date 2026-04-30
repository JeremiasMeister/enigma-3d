use glium::glutin::surface::WindowSurface;
use glium::Display;
use glium::Surface;
use crate::resources;
use crate::camera::Camera;
use crate::light::Light;

// ── Config ────────────────────────────────────────────────────────────────────

pub struct TerrainConfig {
    pub width:             f32,
    pub depth:             f32,
    pub max_height:        f32,
    /// Quads per side across all tiles. Vertices per side = resolution + 1.
    pub resolution:        u32,
    /// Tiles per side (1 = single chunk, 2 = 2×2 grid).
    pub tile_count:        u32,
    pub noise_scale:       f32,
    pub noise_amplitude:   f32,
    pub noise_octaves:     u32,
    pub noise_persistence: f32,
    /// If Some, replaces fBm entirely. Receives world-space (x, z), returns height in [0, max_height].
    pub custom_noise:      Option<Box<dyn Fn(f32, f32) -> f32 + Send + Sync>>,
    /// Vertex color for low flat areas (e.g. grass).
    pub color_flat_low:    [f32; 3],
    /// Vertex color for high flat areas (e.g. snow).
    pub color_flat_high:   [f32; 3],
    /// Vertex color for steep faces (e.g. rock).
    pub color_slope:       [f32; 3],
    /// normal.y below this threshold blends toward color_slope (0.0–1.0).
    pub slope_threshold:   f32,
    /// Normalized height [0–1] at which the low→high flat color blend starts.
    pub height_mid:        f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            width:             100.0,
            depth:             100.0,
            max_height:        15.0,
            resolution:        128,
            tile_count:        1,
            noise_scale:       0.03,
            noise_amplitude:   1.0,
            noise_octaves:     4,
            noise_persistence: 0.5,
            custom_noise:      None,
            color_flat_low:    [0.22, 0.55, 0.12],
            color_flat_high:   [0.85, 0.85, 0.80],
            color_slope:       [0.40, 0.35, 0.28],
            slope_threshold:   0.75,
            height_mid:        0.5,
        }
    }
}

// ── Gradient noise (no external crates) ──────────────────────────────────────

fn hash2(ix: i32, iz: i32) -> u32 {
    let mut h = (ix.wrapping_mul(1619) ^ iz.wrapping_mul(31337)) as u32;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

fn grad2(hash: u32, x: f32, z: f32) -> f32 {
    match hash & 3 {
        0 =>  x + z,
        1 => -x + z,
        2 =>  x - z,
        _ => -x - z,
    }
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub(crate) fn gradient_noise(x: f32, z: f32) -> f32 {
    let ix = x.floor() as i32;
    let iz = z.floor() as i32;
    let fx = x - x.floor();
    let fz = z - z.floor();
    let u = fade(fx);
    let v = fade(fz);

    let g00 = grad2(hash2(ix,     iz    ), fx,       fz      );
    let g10 = grad2(hash2(ix + 1, iz    ), fx - 1.0, fz      );
    let g01 = grad2(hash2(ix,     iz + 1), fx,       fz - 1.0);
    let g11 = grad2(hash2(ix + 1, iz + 1), fx - 1.0, fz - 1.0);

    let x0 = g00 + u * (g10 - g00);
    let x1 = g01 + u * (g11 - g01);
    x0 + v * (x1 - x0)
}

pub(crate) fn fbm(x: f32, z: f32, octaves: u32, persistence: f32) -> f32 {
    let mut value     = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut max_value = 0.0f32;
    for _ in 0..octaves {
        value     += gradient_noise(x * frequency, z * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }
    value / max_value
}

// ── Heightmap generation and normal calculation ────────────────────────────────

/// Generates the flat CPU-side heightmap. Returns (resolution+1)² f32 values,
/// row-major in Z-then-X order. Height values are in [0, max_height].
pub(crate) fn generate_heightmap(cfg: &TerrainConfig) -> Vec<f32> {
    let verts = (cfg.resolution + 1) as usize;
    let cell_x = cfg.width  / cfg.resolution as f32;
    let cell_z = cfg.depth  / cfg.resolution as f32;
    let mut heightmap = Vec::with_capacity(verts * verts);

    for zi in 0..verts {
        for xi in 0..verts {
            let wx = -cfg.width  * 0.5 + xi as f32 * cell_x;
            let wz = -cfg.depth  * 0.5 + zi as f32 * cell_z;

            let h = if let Some(f) = &cfg.custom_noise {
                // User returns world-space Y directly; clamped to [0, max_height]
                f(wx, wz).clamp(0.0, cfg.max_height)
            } else {
                let nx = wx * cfg.noise_scale;
                let nz = wz * cfg.noise_scale;
                // fbm returns [-1,1]; shift to [0,1] then apply amplitude and max_height
                (fbm(nx, nz, cfg.noise_octaves, cfg.noise_persistence) * 0.5 + 0.5)
                    * cfg.noise_amplitude
                    * cfg.max_height
            };

            heightmap.push(h);
        }
    }
    heightmap
}

/// Computes per-vertex normals by averaging surrounding quad face normals.
/// `heightmap` is (resolution+1)² in Z-then-X row-major order.
pub(crate) fn calculate_normals(
    heightmap: &[f32],
    resolution: u32,
    cell_x: f32,
    cell_z: f32,
) -> Vec<[f32; 3]> {
    let verts = (resolution + 1) as usize;
    let mut normals = vec![[0.0f32; 3]; verts * verts];

    let idx = |xi: usize, zi: usize| zi * verts + xi;
    let h   = |xi: usize, zi: usize| heightmap[idx(xi, zi)];

    for zi in 0..verts {
        for xi in 0..verts {
            let mut nx = 0.0f32;
            let mut ny = 0.0f32;
            let mut nz = 0.0f32;
            let mut count = 0u32;

            // Each surrounding quad contributes one face normal.
            // For each quad, use edge vectors to two adjacent corners.
            if xi > 0 && zi > 0 {
                // quad with corners (xi-1,zi-1), (xi,zi-1), (xi,zi), (xi-1,zi)
                // edges from (xi,zi) to (xi-1,zi) and (xi,zi-1)
                let to_left  = [-cell_x, h(xi - 1, zi) - h(xi, zi), 0.0f32];
                let to_back  = [0.0f32, h(xi, zi - 1) - h(xi, zi), -cell_z];
                let cn = cross(to_back, to_left);
                nx += cn[0]; ny += cn[1]; nz += cn[2]; count += 1;
            }
            if xi + 1 < verts && zi > 0 {
                // quad with corners (xi,zi-1), (xi+1,zi-1), (xi+1,zi), (xi,zi)
                // edges from (xi,zi) to (xi+1,zi) and (xi,zi-1)
                let to_right = [cell_x, h(xi + 1, zi) - h(xi, zi), 0.0f32];
                let to_back  = [0.0f32, h(xi, zi - 1) - h(xi, zi), -cell_z];
                let cn = cross(to_right, to_back);
                nx += cn[0]; ny += cn[1]; nz += cn[2]; count += 1;
            }
            if xi > 0 && zi + 1 < verts {
                // quad with corners (xi-1,zi), (xi,zi), (xi,zi+1), (xi-1,zi+1)
                // edges from (xi,zi) to (xi-1,zi) and (xi,zi+1)
                let to_left  = [-cell_x, h(xi - 1, zi) - h(xi, zi), 0.0f32];
                let to_fwd   = [0.0f32, h(xi, zi + 1) - h(xi, zi), cell_z];
                let cn = cross(to_left, to_fwd);
                nx += cn[0]; ny += cn[1]; nz += cn[2]; count += 1;
            }
            if xi + 1 < verts && zi + 1 < verts {
                // quad with corners (xi,zi), (xi+1,zi), (xi+1,zi+1), (xi,zi+1)
                // edges from (xi,zi) to (xi+1,zi) and (xi,zi+1)
                let to_right = [cell_x, h(xi + 1, zi) - h(xi, zi), 0.0f32];
                let to_fwd   = [0.0f32, h(xi, zi + 1) - h(xi, zi), cell_z];
                let cn = cross(to_fwd, to_right);
                nx += cn[0]; ny += cn[1]; nz += cn[2]; count += 1;
            }

            let n = if count > 0 {
                let c = count as f32;
                let len = ((nx/c)*(nx/c) + (ny/c)*(ny/c) + (nz/c)*(nz/c)).sqrt().max(0.0001);
                [nx / c / len, ny / c / len, nz / c / len]
            } else {
                [0.0, 1.0, 0.0]
            };
            normals[idx(xi, zi)] = n;
        }
    }
    normals
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0],
    ]
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Assigns a vertex color given the normalized height and the face normal.
pub(crate) fn vertex_color(normal: [f32; 3], height: f32, cfg: &TerrainConfig) -> [f32; 3] {
    let slope_factor = if cfg.slope_threshold > 0.0 {
        (1.0 - (normal[1] / cfg.slope_threshold)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let height_t = if cfg.max_height > 0.0 {
        ((height / cfg.max_height - cfg.height_mid)
            / (1.0 - cfg.height_mid))
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let flat_color = mix(cfg.color_flat_low, cfg.color_flat_high, height_t);
    mix(flat_color, cfg.color_slope, slope_factor)
}

// ── GPU types ─────────────────────────────────────────────────────────────────

#[derive(Copy, Clone)]
struct TerrainVertex {
    position: [f32; 3],
    normal:   [f32; 3],
    color:    [f32; 3],
}
glium::implement_vertex!(TerrainVertex, position, normal, color);

struct TerrainTile {
    vertex_buffer: glium::VertexBuffer<TerrainVertex>,
    index_buffer:  glium::IndexBuffer<u32>,
    #[allow(dead_code)]
    tile_x: i32,
    #[allow(dead_code)]
    tile_z: i32,
}

pub struct Terrain {
    tiles:     Vec<TerrainTile>,
    heightmap: Vec<f32>,   // private; accessed via get_height()
    config:    TerrainConfig,
    program:   glium::Program,
    position:  [f32; 3],
}

impl Terrain {
    pub fn new(display: &Display<WindowSurface>, config: TerrainConfig) -> Self {
        assert!(config.tile_count > 0, "tile_count must be >= 1");
        assert_eq!(
            config.resolution % config.tile_count, 0,
            "resolution ({}) must be divisible by tile_count ({})",
            config.resolution, config.tile_count
        );

        let program = glium::Program::from_source(
            display,
            resources::terrain_vert_shader(),
            resources::terrain_frag_shader(),
            None,
        ).expect("Failed to compile terrain shader");

        let heightmap = generate_heightmap(&config);
        let verts_per_side = config.resolution + 1;
        let cell_x = config.width  / config.resolution as f32;
        let cell_z = config.depth  / config.resolution as f32;
        let normals = calculate_normals(&heightmap, config.resolution, cell_x, cell_z);

        let tile_quads = config.resolution / config.tile_count;
        let tile_verts = tile_quads + 1;
        let mut tiles = Vec::new();

        for tz in 0..config.tile_count as i32 {
            for tx in 0..config.tile_count as i32 {
                let x0 = tx as u32 * tile_quads;
                let z0 = tz as u32 * tile_quads;

                let mut vertices: Vec<TerrainVertex> = Vec::with_capacity((tile_verts * tile_verts) as usize);
                for lz in 0..tile_verts {
                    for lx in 0..tile_verts {
                        let gx = x0 + lx;
                        let gz = z0 + lz;
                        let gi = (gz * verts_per_side + gx) as usize;
                        let wx = -config.width  * 0.5 + gx as f32 * cell_x;
                        let wz = -config.depth  * 0.5 + gz as f32 * cell_z;
                        let h  = heightmap[gi];
                        let n  = normals[gi];
                        let c  = vertex_color(n, h, &config);
                        vertices.push(TerrainVertex {
                            position: [wx, h, wz],
                            normal:   n,
                            color:    c,
                        });
                    }
                }

                let mut indices: Vec<u32> = Vec::with_capacity((tile_quads * tile_quads * 6) as usize);
                for qz in 0..tile_quads {
                    for qx in 0..tile_quads {
                        let tl = qz * tile_verts + qx;
                        let tr = tl + 1;
                        let bl = tl + tile_verts;
                        let br = bl + 1;
                        indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
                    }
                }

                let vb = glium::VertexBuffer::new(display, &vertices)
                    .expect("Failed to create terrain vertex buffer");
                let ib = glium::IndexBuffer::new(
                    display,
                    glium::index::PrimitiveType::TrianglesList,
                    &indices,
                ).expect("Failed to create terrain index buffer");

                tiles.push(TerrainTile { vertex_buffer: vb, index_buffer: ib, tile_x: tx, tile_z: tz });
            }
        }

        Terrain { tiles, heightmap, config, program, position: [0.0; 3] }
    }

    pub fn get_position(&self) -> [f32; 3] { self.position }

    pub fn set_position(&mut self, pos: [f32; 3]) { self.position = pos; }

    /// Returns the interpolated world-space Y height at the given XZ world position.
    /// Clamps to the terrain edge for positions outside the terrain bounds.
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        let cfg = &self.config;
        let verts = (cfg.resolution + 1) as usize;
        let cell_x = cfg.width  / cfg.resolution as f32;
        let cell_z = cfg.depth  / cfg.resolution as f32;
        let lx = (x - self.position[0] + cfg.width  * 0.5) / cell_x;
        let lz = (z - self.position[2] + cfg.depth  * 0.5) / cell_z;
        bilinear_lookup(&self.heightmap, verts, lx, lz)
    }

    /// Draws all terrain tiles to `target` using the scene camera and lights.
    pub fn draw(
        &self,
        target: &mut impl Surface,
        camera: &Camera,
        lights: &[Light],
        ambient_light: Option<&Light>,
    ) {
        let pos = self.position;
        let model_matrix: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [pos[0], pos[1], pos[2], 1.0],
        ];
        let view_matrix       = camera.get_view_matrix();
        let projection_matrix = camera.get_projection_matrix();

        // Pack up to 4 lights into mat4 rows (same convention as material.rs)
        let n = lights.len().min(4);
        let mut light_pos_arr:   [[f32; 4]; 4] = [[0.0; 4]; 4];
        let mut light_col_arr:   [[f32; 4]; 4] = [[0.0; 4]; 4];
        let mut light_int_arr:   [f32; 4]      = [0.0; 4];
        for i in 0..n {
            light_pos_arr[i] = [lights[i].position[0], lights[i].position[1], lights[i].position[2], 0.0];
            light_col_arr[i] = [lights[i].color[0],    lights[i].color[1],    lights[i].color[2],    0.0];
            light_int_arr[i] = lights[i].intensity;
        }
        let light_amount = n as i32;

        let (amb_color, amb_intensity) = match ambient_light {
            Some(l) => (l.color, l.intensity),
            None    => ([0.0f32; 3], 0.0f32),
        };

        let draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test:  glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        for tile in &self.tiles {
            let uniforms = glium::uniform! {
                model_matrix:            model_matrix,
                view_matrix:             view_matrix,
                projection_matrix:       projection_matrix,
                light_position:          light_pos_arr,
                light_color:             light_col_arr,
                light_intensity:         light_int_arr,
                light_amount:            light_amount,
                ambient_light_color:     amb_color,
                ambient_light_intensity: amb_intensity,
            };
            target.draw(
                &tile.vertex_buffer,
                &tile.index_buffer,
                &self.program,
                &uniforms,
                &draw_params,
            ).expect("Failed to draw terrain tile");
        }
    }
}

/// Bilinear lookup into a row-major (Z-then-X) heightmap. `lx`/`lz` are
/// floating-point grid coordinates; clamped to prevent out-of-bounds access.
pub(crate) fn bilinear_lookup(heightmap: &[f32], verts: usize, lx: f32, lz: f32) -> f32 {
    let max = (verts - 1) as f32 - 0.0001;
    let lx = lx.clamp(0.0, max);
    let lz = lz.clamp(0.0, max);
    let xi = lx.floor() as usize;
    let zi = lz.floor() as usize;
    let fx = lx - xi as f32;
    let fz = lz - zi as f32;
    let idx = |xi: usize, zi: usize| zi * verts + xi;
    let h00 = heightmap[idx(xi,     zi    )];
    let h10 = heightmap[idx(xi + 1, zi    )];
    let h01 = heightmap[idx(xi,     zi + 1)];
    let h11 = heightmap[idx(xi + 1, zi + 1)];
    let h0 = h00 + fx * (h10 - h00);
    let h1 = h01 + fx * (h11 - h01);
    h0 + fz * (h1 - h0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_config() -> TerrainConfig {
        TerrainConfig {
            custom_noise: Some(Box::new(|_, _| 0.0)),
            max_height: 10.0,
            resolution: 4,
            tile_count: 1,
            ..TerrainConfig::default()
        }
    }

    #[test]
    fn terrain_config_defaults() {
        let cfg = TerrainConfig::default();
        assert_eq!(cfg.width, 100.0);
        assert_eq!(cfg.resolution, 128);
        assert_eq!(cfg.tile_count, 1);
        assert_eq!(cfg.noise_octaves, 4);
    }

    #[test]
    fn gradient_noise_range() {
        for i in 0..100i32 {
            let v = gradient_noise(i as f32 * 0.37, i as f32 * 0.53);
            assert!(v >= -2.0 && v <= 2.0, "noise out of range: {v}");
        }
    }

    #[test]
    fn fbm_normalized() {
        for i in 0..100i32 {
            let v = fbm(i as f32 * 0.13, i as f32 * 0.29, 4, 0.5);
            assert!(v >= -1.5 && v <= 1.5, "fbm out of expected range: {v}");
        }
    }

    #[test]
    fn heightmap_flat_custom_noise() {
        let cfg = flat_config();
        let hm = generate_heightmap(&cfg);
        // 5×5 grid (resolution=4 → 5 verts per side)
        assert_eq!(hm.len(), 25);
        for h in &hm { assert_eq!(*h, 0.0); }
    }

    #[test]
    fn heightmap_size() {
        let cfg = TerrainConfig { resolution: 8, tile_count: 1, ..TerrainConfig::default() };
        let hm = generate_heightmap(&cfg);
        assert_eq!(hm.len(), 81); // (8+1)^2
    }

    #[test]
    fn normals_flat_terrain_point_up() {
        let cfg = flat_config();
        let hm = generate_heightmap(&cfg);
        let cell = cfg.width / cfg.resolution as f32;
        let normals = calculate_normals(&hm, cfg.resolution, cell, cell);
        for n in &normals {
            assert!((n[1] - 1.0).abs() < 0.001, "flat terrain normal not pointing up: {n:?}");
        }
    }

    #[test]
    fn vertex_color_flat_low() {
        let cfg = TerrainConfig::default();
        let c = vertex_color([0.0, 1.0, 0.0], 0.0, &cfg);
        // slope_factor=0 (flat), height_t=0 (low) → should equal color_flat_low
        for i in 0..3 {
            assert!((c[i] - cfg.color_flat_low[i]).abs() < 0.001);
        }
    }

    #[test]
    fn vertex_color_steep_slope() {
        let cfg = TerrainConfig::default();
        // normal pointing sideways → full slope color
        let c = vertex_color([1.0, 0.0, 0.0], 5.0, &cfg);
        for i in 0..3 {
            assert!((c[i] - cfg.color_slope[i]).abs() < 0.001);
        }
    }

    #[test]
    fn vertex_color_max_height_zero_no_nan() {
        let cfg = TerrainConfig { max_height: 0.0, ..TerrainConfig::default() };
        let c = vertex_color([0.0, 1.0, 0.0], 0.0, &cfg);
        for comp in c { assert!(!comp.is_nan(), "NaN in vertex color with max_height=0"); }
    }

    fn flat_heightmap(resolution: u32) -> (Vec<f32>, TerrainConfig) {
        let cfg = TerrainConfig {
            resolution,
            tile_count: 1,
            custom_noise: Some(Box::new(|_, _| 0.0)),
            max_height: 10.0,
            width: 10.0,
            depth: 10.0,
            ..TerrainConfig::default()
        };
        let hm = generate_heightmap(&cfg);
        (hm, cfg)
    }

    #[test]
    fn get_height_flat_is_zero() {
        let (hm, cfg) = flat_heightmap(8);
        let verts = (cfg.resolution + 1) as usize;
        let cell_x = cfg.width / cfg.resolution as f32;
        let cell_z = cfg.depth / cfg.resolution as f32;
        let query = |x: f32, z: f32| {
            let lx = (x + cfg.width  * 0.5) / cell_x;
            let lz = (z + cfg.depth  * 0.5) / cell_z;
            bilinear_lookup(&hm, verts, lx, lz)
        };
        assert!(query(0.0, 0.0).abs() < 0.001);
        assert!(query(3.0, -2.5).abs() < 0.001);
        assert!(query(-4.9, 4.9).abs() < 0.001);
    }

    #[test]
    fn get_height_out_of_bounds_clamps() {
        let (hm, cfg) = flat_heightmap(4);
        let verts = (cfg.resolution + 1) as usize;
        // x=1000 is far outside — bilinear_lookup must clamp and return 0.0
        let result = bilinear_lookup(&hm, verts, 1000.0, (cfg.depth * 0.5) / (cfg.depth / cfg.resolution as f32));
        assert_eq!(result, 0.0);
    }
}
