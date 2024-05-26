// models
pub const SUZANNE: &'static [u8] = include_bytes!("res/models/suzanne.glb");
pub const SKYBOX: &'static [u8] = include_bytes!("res/models/skybox.glb");

// shaders
//// post processing
pub const POST_PROCESSING_VERTEX: &str =
    include_str!("res/shader/post_processing/post_processing_vert.glsl");
pub const POST_PROCESSING_BLOOM_BLUR_FRAGMENT: &str =
    include_str!("res/shader/post_processing/bloom/enigma_bloom_blur.glsl");
pub const POST_PROCESSING_BLOOM_COMBINE_FRAGMENT: &str =
    include_str!("res/shader/post_processing/bloom/enigma_bloom_combine.glsl");
pub const POST_PROCESSING_BLOOM_EXTRACT_FRAGMENT: &str =
    include_str!("res/shader/post_processing/bloom/enigma_bloom_extract.glsl");
pub const POST_PROCESSING_EDGE_FRAGMENT: &str =
    include_str!("res/shader/post_processing/edge/enigma_edge_detection.glsl");
pub const POST_PROCESSING_GRAYSCALE_FRAGMENT: &str =
    include_str!("res/shader/post_processing/grayscale/enigma_grayscale.glsl");

//// other
pub const FRAGMENT_SHADER: &str = include_str!("res/shader/enigma_fragment_shader.glsl");
pub const VERTEX_SHADER: &str = include_str!("res/shader/enigma_vertex_shader.glsl");
pub const FRAGMENT_UNLIT_SHADER: &str = include_str!("res/shader/enigma_fragment_unlit.glsl");

// textures
pub const UV_CHECKER: &'static [u8] = include_bytes!("res/textures/uv_checker.png");
pub const SKYBOX_TEXTURE: &'static [u8] = include_bytes!("res/textures/skybox.png");
pub const SKYBOX_TEXTURE_HDR: &'static [u8] = include_bytes!("res/textures/skybox.hdr");
pub const ICON: &'static [u8] = include_bytes!("res/textures/icon.png");
