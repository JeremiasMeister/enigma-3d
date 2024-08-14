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

// textures
pub fn skybox_texture() -> &'static [u8] {
    include_bytes!("res/textures/skybox.png")
}

pub fn icon() -> &'static [u8] {
    include_bytes!("res/textures/icon.png")
}

