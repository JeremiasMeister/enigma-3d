// chessboard example
//models
pub fn chess_bishop_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/bishop.glb")
}

pub fn chess_board_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/board.glb")
}

pub fn chess_king_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/king.glb")
}

pub fn chess_knight_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/knight.glb")
}

pub fn chess_pawn_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/pawn.glb")
}

pub fn chess_queen_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/queen.glb")
}

pub fn chess_rook_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/rook.glb")
}

pub fn terrain() -> &'static [u8] {
    include_bytes!("res/models/chessboard/terrain.glb")
}

// textures
pub fn chess_figures_black_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_Black_basecolor.png")
}
pub fn chess_figures_white_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_White_basecolor.png")
}
pub fn chess_figures_metallic() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_metallic.png")
}
pub fn chess_figures_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_normal.png")
}
pub fn chess_figures_black_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_Black_roughness.png")
}
pub fn chess_figures_white_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_White_roughness.png")
}
pub fn chess_board_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_basecolor.png")
}
pub fn chess_board_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_normal.png")
}
pub fn chess_board_metallic() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_metallic.png")
}
pub fn chess_board_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_roughness.png")
}

pub fn terrain_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_basecolor.png")
}

pub fn terrain_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_normal.png")
}

pub fn terrain_metallic() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_metallic.png")
}

pub fn terrain_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_roughness.png")
}

//engine example
pub fn suzanne() -> &'static [u8] {
    include_bytes!("res/models/engine/suzanne.glb")
}

pub fn skybox_texture_hdr() -> &'static [u8] {
    include_bytes!("res/textures/skybox.hdr")
}

pub fn uv_checker() -> &'static [u8] {
    include_bytes!("res/textures/uv_checker.png")
}