// models
pub fn suzanne() -> &'static [u8] {
    include_bytes!("res/models/suzanne.glb")
}

pub fn skybox() -> &'static [u8] {
    include_bytes!("res/models/skybox.glb")
}

// shaders
//// post-processing
pub fn post_processing_vertex() -> &'static str {
    include_str!("res/shader/post_processing/post_processing_vert.glsl")
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

pub fn fragment_unlit_shader() -> &'static str {
    include_str!("res/shader/enigma_fragment_unlit.glsl")
}

// textures
pub fn uv_checker() -> &'static [u8] {
    include_bytes!("res/textures/uv_checker.png")
}

pub fn skybox_texture() -> &'static [u8] {
    include_bytes!("res/textures/skybox.png")
}

pub fn skybox_texture_hdr() -> &'static [u8] {
    include_bytes!("res/textures/skybox.hdr")
}

pub fn icon() -> &'static [u8] {
    include_bytes!("res/textures/icon.png")
}


// chessboard example
//models
pub fn chess_bishop_gltf() -> &'static [u8] {
    include_bytes!("res/models/bishop.glb")
}

pub fn chess_board_gltf() -> &'static [u8] {
    include_bytes!("res/models/board.glb")
}

pub fn chess_king_gltf() -> &'static [u8] {
    include_bytes!("res/models/king.glb")
}

pub fn chess_knight_gltf() -> &'static [u8] {
    include_bytes!("res/models/knight.glb")
}

pub fn chess_pawn_gltf() -> &'static [u8] {
    include_bytes!("res/models/pawn.glb")
}

pub fn chess_queen_gltf() -> &'static [u8] {
    include_bytes!("res/models/queen.glb")
}

pub fn chess_rook_gltf() -> &'static [u8] {
    include_bytes!("res/models/rook.glb")
}

// textures
pub fn chess_figures_black_albedo() -> &'static [u8] {
    include_bytes!("res/textures/Figures_Black_basecolor.png")
}
pub fn chess_figures_white_albedo() -> &'static [u8] {
    include_bytes!("res/textures/Figures_White_basecolor.png")
}
pub fn chess_figures_metallic() -> &'static [u8] {
    include_bytes!("res/textures/Figures_metallic.png")
}
pub fn chess_figures_normal() -> &'static [u8] {
    include_bytes!("res/textures/Figures_normal.png")
}
pub fn chess_figures_black_roughness() -> &'static [u8] {
    include_bytes!("res/textures/Figures_Black_roughness.png")
}
pub fn chess_figures_white_roughness() -> &'static [u8] {
    include_bytes!("res/textures/Figures_White_roughness.png")
}
pub fn chess_board_albedo() -> &'static [u8] {
    include_bytes!("res/textures/Board_basecolor.png")
}
pub fn chess_board_normal() -> &'static [u8] {
    include_bytes!("res/textures/Board_normal.png")
}
pub fn chess_board_metallic() -> &'static [u8] {
    include_bytes!("res/textures/Board_metallic.png")
}
pub fn chess_board_roughness() -> &'static [u8] {
    include_bytes!("res/textures/Board_roughness.png")
}