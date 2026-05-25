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
pub fn gun_gltf() -> &'static [u8] {
    include_bytes!("res/models/pawn-shooter/gun.glb")
}

#[cfg(feature = "examples")]
pub fn bullet_gltf() -> &'static [u8] {
    include_bytes!("res/models/pawn-shooter/bullet.glb")
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

// skinned-mesh example
#[cfg(feature = "examples")]
pub fn skinned_knight() -> &'static [u8] {
    include_bytes!("res/models/skinned-mesh/knight.glb")
}

#[cfg(feature = "examples")]
pub fn skinned_wiggle() -> &'static [u8] { include_bytes!("res/models/skinned-mesh/wiggle.glb")}

#[cfg(feature = "examples")]
pub fn skinned_knight_albedo() -> &'static [u8] { include_bytes!("res/textures/skinned-mesh/knight_albedo.png") }

#[cfg(feature = "examples")]
pub fn skinned_knight_normal() -> &'static [u8] { include_bytes!("res/textures/skinned-mesh/knight_normal.png") }

#[cfg(feature = "examples")]
pub fn skinned_knight_roughness() -> &'static [u8] { include_bytes!("res/textures/skinned-mesh/knight_roughness.png") }

// audio
#[cfg(feature = "examples")]
pub fn click_sound() -> &'static [u8] { include_bytes!("res/sounds/click-sound.ogg")}

#[cfg(feature = "examples")]
pub fn background_music() -> &'static [u8] { include_bytes!("res/sounds/background-audio.ogg")}

#[cfg(feature = "examples")]
pub fn shot_sound() -> &'static [u8] { include_bytes!("res/sounds/shot.ogg")}

#[cfg(feature = "examples")]
pub fn break_sound() -> &'static [u8] { include_bytes!("res/sounds/break.ogg")}

#[cfg(feature = "examples")]
pub fn pickup_sound() -> &'static [u8] { include_bytes!("res/sounds/pickup.ogg")}

// vfx — particle system JSON configs (embedded)
#[cfg(feature = "examples")]
pub fn vfx_fireball() -> &'static str {
    include_str!("res/vfx/fireball.json")
}
#[cfg(feature = "examples")]
pub fn vfx_fireball_smoke() -> &'static str {
    include_str!("res/vfx/fireball_smoke.json")
}
#[cfg(feature = "examples")]
pub fn vfx_hit_sparks() -> &'static str {
    include_str!("res/vfx/hit_sparks.json")
}
#[cfg(feature = "examples")]
pub fn vfx_spark_dust() -> &'static str {
    include_str!("res/vfx/spark_dust.json")
}
#[cfg(feature = "examples")]
pub fn vfx_magic_orb() -> &'static str {
    include_str!("res/vfx/magic_orb.json")
}
#[cfg(feature = "examples")]
pub fn vfx_aura() -> &'static str {
    include_str!("res/vfx/aura.json")
}
#[cfg(feature = "examples")]
pub fn vfx_vortex_ribbon() -> &'static str {
    include_str!("res/vfx/vortex_ribbon.json")
}
#[cfg(feature = "examples")]
pub fn vfx_lightning() -> &'static str {
    include_str!("res/vfx/lightning.json")
}

// vfx — particle textures (embedded)
#[cfg(feature = "examples")]
pub fn vfx_explosion_flipbook() -> &'static [u8] {
    include_bytes!("res/textures/particles/explosion_flipbook.png")
}
#[cfg(feature = "examples")]
pub fn vfx_sprite_sheet() -> &'static [u8] {
    include_bytes!("res/textures/particles/sprite_sheet.png")
}

#[cfg(feature = "examples")]
pub fn vfx_ribbon_texture() -> &'static [u8] {
    include_bytes!("res/textures/particles/ribbon_tex.png")
}
