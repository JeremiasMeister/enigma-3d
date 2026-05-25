// models
pub fn skybox() -> &'static [u8] {
    include_bytes!("res/models/skybox.glb")
}

// shaders
//// post-processing
pub fn post_processing_vertex() -> &'static str {
    include_str!("res/shader/post_processing/post_processing_vert.glsl")
}

pub fn post_processing_depth_fog_fragment() -> &'static str {
    include_str!("res/shader/post_processing/depth_fog/depth_fog.glsl")
}

pub fn post_processing_bloom_blur_fragment() -> &'static str {
    include_str!("res/shader/post_processing/bloom/enigma_bloom_blur.glsl")
}

pub fn post_processing_bloom_combine_fragment() -> &'static str {
    include_str!("res/shader/post_processing/bloom/enigma_bloom_combine.glsl")
}

pub fn post_processing_bloom_extract_fragment() -> &'static str {
    include_str!("res/shader/post_processing/bloom/enigma_bloom_extract.glsl")
}

pub fn post_processing_edge_fragment() -> &'static str {
    include_str!("res/shader/post_processing/edge/enigma_edge_detection.glsl")
}

pub fn post_processing_grayscale_fragment() -> &'static str {
    include_str!("res/shader/post_processing/grayscale/enigma_grayscale.glsl")
}

pub fn post_processing_vignette_fragment() -> &'static str {
    include_str!("res/shader/post_processing/vignette/enigma_vignette.glsl")
}

pub fn post_processing_lens_dirt_fragment() -> &'static str {
    include_str!("res/shader/post_processing/lens_dirt/enigma_lens_dirt.glsl")
}

//// other
pub fn fragment_shader() -> &'static str {
    include_str!("res/shader/enigma_fragment_shader.glsl")
}

pub fn vertex_shader() -> &'static str {
    include_str!("res/shader/enigma_vertex_shader.glsl")
}

pub fn vertex_wind_shader() -> &'static str {
    include_str!("res/shader/enigma_vertex_wind_shader.glsl")
}

pub fn geometry_grass_shader() -> &'static str {
    include_str!("res/shader/enigma_geometry_grass_shader.glsl")
}

pub fn geometry_shader() -> &'static str {
    include_str!("res/shader/enigma_geometry_shader.glsl")
}

pub fn fragment_unlit_shader() -> &'static str {
    include_str!("res/shader/enigma_fragment_unlit.glsl")
}

pub fn shadow_depth_vert_shader() -> &'static str {
    include_str!("res/shader/shadow_depth_vert.glsl")
}

pub fn shadow_depth_dir_frag_shader() -> &'static str {
    include_str!("res/shader/shadow_depth_dir_frag.glsl")
}

pub fn shadow_depth_point_frag_shader() -> &'static str {
    include_str!("res/shader/shadow_depth_point_frag.glsl")
}

// textures
pub fn skybox_texture() -> &'static [u8] {
    include_bytes!("res/textures/skybox.png")
}

pub fn lens_dirt_texture() -> &'static [u8] {
    include_bytes!("res/textures/lens_dirt.png")
}

pub fn icon() -> &'static [u8] {
    include_bytes!("res/textures/icon.png")
}

pub fn terrain_vert_shader() -> &'static str {
    include_str!("res/shader/terrain_vert.glsl")
}

pub fn terrain_frag_shader() -> &'static str {
    include_str!("res/shader/terrain_frag.glsl")
}

pub fn particle_sprite_vert() -> &'static str {
    include_str!("res/shader/particle_sprite_vert.glsl")
}
pub fn particle_sprite_frag() -> &'static str {
    include_str!("res/shader/particle_sprite_frag.glsl")
}
pub fn particle_ribbon_vert() -> &'static str {
    include_str!("res/shader/particle_ribbon_vert.glsl")
}
pub fn particle_ribbon_frag() -> &'static str {
    include_str!("res/shader/particle_ribbon_frag.glsl")
}

// vfx — particle system JSON configs (embedded)
pub fn vfx_fireball() -> &'static str {
    include_str!("res/vfx/fireball.json")
}
pub fn vfx_fireball_smoke() -> &'static str {
    include_str!("res/vfx/fireball_smoke.json")
}
pub fn vfx_hit_sparks() -> &'static str {
    include_str!("res/vfx/hit_sparks.json")
}
pub fn vfx_spark_dust() -> &'static str {
    include_str!("res/vfx/spark_dust.json")
}
pub fn vfx_magic_orb() -> &'static str {
    include_str!("res/vfx/magic_orb.json")
}

// vfx — particle textures (embedded)
pub fn vfx_explosion_flipbook() -> &'static [u8] {
    include_bytes!("res/textures/particles/explosion_flipbook.png")
}

