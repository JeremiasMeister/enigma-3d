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

// ── Placeholder for remaining types (added in later tasks) ───────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
        // gradient_noise should stay within [-2, 2] for any input
        for i in 0..100i32 {
            let x = i as f32 * 0.37;
            let z = i as f32 * 0.53;
            let v = gradient_noise(x, z);
            assert!(v >= -2.0 && v <= 2.0, "noise out of range: {v}");
        }
    }

    #[test]
    fn fbm_normalized() {
        // fBm output should be in roughly [-1, 1]
        for i in 0..100i32 {
            let x = i as f32 * 0.13;
            let z = i as f32 * 0.29;
            let v = fbm(x, z, 4, 0.5);
            assert!(v >= -1.5 && v <= 1.5, "fbm out of expected range: {v}");
        }
    }
}
