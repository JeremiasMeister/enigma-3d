// chessboard example
//models

#[cfg(feature = "examples")]
pub fn chess_bishop_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/bishop.glb")
}

#[cfg(feature = "examples")]
pub fn chess_board_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/board.glb")
}

#[cfg(feature = "examples")]
pub fn chess_king_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/king.glb")
}

#[cfg(feature = "examples")]
pub fn chess_knight_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/knight.glb")
}

#[cfg(feature = "examples")]
pub fn chess_pawn_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/pawn.glb")
}

#[cfg(feature = "examples")]
pub fn chess_queen_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/queen.glb")
}

#[cfg(feature = "examples")]
pub fn chess_rook_gltf() -> &'static [u8] {
    include_bytes!("res/models/chessboard/rook.glb")
}

#[cfg(feature = "examples")]
pub fn terrain() -> &'static [u8] {
    include_bytes!("res/models/chessboard/terrain.glb")
}

#[cfg(feature = "examples")]
pub fn tree() -> &'static [u8] {
    include_bytes!("res/models/chessboard/tree.glb")
}

// textures
#[cfg(feature = "examples")]
pub fn chess_figures_black_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_Black_basecolor.png")
}
#[cfg(feature = "examples")]
pub fn chess_figures_white_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_White_basecolor.png")
}
#[cfg(feature = "examples")]
pub fn chess_figures_metallic() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_metallic.png")
}

#[cfg(feature = "examples")]
pub fn chess_figures_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_normal.png")
}

#[cfg(feature = "examples")]
pub fn chess_figures_black_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_Black_roughness.png")
}

#[cfg(feature = "examples")]
pub fn chess_figures_white_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Figures_White_roughness.png")
}

#[cfg(feature = "examples")]
pub fn chess_board_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_basecolor.png")
}

#[cfg(feature = "examples")]
pub fn chess_board_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_normal.png")
}

#[cfg(feature = "examples")]
pub fn chess_board_metallic() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_metallic.png")
}

#[cfg(feature = "examples")]
pub fn chess_board_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Board_roughness.png")
}

#[cfg(feature = "examples")]
pub fn terrain_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_basecolor.png")
}

#[cfg(feature = "examples")]
pub fn terrain_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_normal.png")
}

#[cfg(feature = "examples")]
pub fn terrain_metallic() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_metallic.png")
}

#[cfg(feature = "examples")]
pub fn terrain_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Terrain_roughness.png")
}

#[cfg(feature = "examples")]
pub fn tree_albedo() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Tree_basecolor.png")
}

#[cfg(feature = "examples")]
pub fn tree_normal() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Tree_normal.png")
}

#[cfg(feature = "examples")]
pub fn tree_roughness() -> &'static [u8] {
    include_bytes!("res/textures/chessboard/Tree_roughness.png")
}

//engine example
#[cfg(feature = "examples")]
pub fn suzanne() -> &'static [u8] {
    include_bytes!("res/models/engine/suzanne.glb")
}
#[cfg(feature = "examples")]
pub fn skybox_texture_hdr() -> &'static [u8] {
    include_bytes!("res/textures/engine/skybox.hdr")
}
#[cfg(feature = "examples")]
pub fn uv_checker() -> &'static [u8] {
    include_bytes!("res/textures/engine/uv_checker.png")
}