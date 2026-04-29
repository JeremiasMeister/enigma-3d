# Pawn Rush — Design Spec

**Date:** 2026-04-29  
**Status:** Approved

## Overview

A small survival shooter example for enigma-3d. Chess pawns spawn in waves and march toward the camera. The player fires projectiles by left-clicking to eliminate them before they arrive. Demonstrates: raycasting/click events, per-frame update logic, typed state data, procedural mesh generation, audio, post-processing, and egui HUD — all within the existing example asset set.

---

## Engine Addition: Primitive Constructors

**File:** `src/object.rs`

Two new `Object` constructors, placed alongside the existing GLTF loaders:

```rust
pub fn cube(half_extent: f32) -> Self
pub fn sphere(radius: f32, stacks: u32, slices: u32) -> Self
```

- `cube`: 24 vertices (4 per face × 6 faces) with per-face normals. 36 indices (2 triangles × 6 faces).
- `sphere`: UV sphere. `stacks` latitude bands, `slices` longitude segments. Per-vertex normals computed as normalized position. Degenerate triangles at poles handled with a fan.
- Both zero-initialize `bone_indices`/`bone_weights`, set `color` to `[1.0, 1.0, 1.0]`, and set `texcoord` appropriately.
- Bounding box is computed from vertices after construction (same path as GLTF loading).

The projectile uses `Object::cube(0.08)`.

---

## Scene Layout

| Element | Asset | Notes |
|---|---|---|
| Ground | `terrain.glb` | Positioned as in chessboard example |
| Tree | `tree.glb` | Background decoration |
| Knight | `skinned_knight.glb` | Mid-field, idle animation playing |
| Pawns | `pawn.glb` | Spawned per wave, white material |
| Projectile | `Object::cube(0.08)` | Unlit emissive material |

**Camera:** Fixed at `[0.0, 3.5, 8.0]`, looking at `[-15.0, -10.0, 0.0]` rotation. No free camera movement during gameplay.

**Pawn spawn band:** Z = -12, X spread `[-3.5, 3.5]` in random steps matching chess grid spacing.  
**Pawn death plane:** Z > 4.0 → costs 1 life.  
**Projectile max range:** 30 units from spawn; removed if exceeded.

---

## Game State

A single `GameState` struct stored via `app_state.add_state_data("game_state", Box::new(...))`.

```rust
#[derive(Clone, PartialEq)]
enum GamePhase { Menu, Playing, GameOver }

struct GameState {
    phase: GamePhase,
    score: u32,
    lives: u32,         // starts at 3
    wave: u32,
    wave_timer: f32,    // counts up; new wave at >= WAVE_INTERVAL
    projectile_ids: Vec<(Uuid, [f32; 3])>,  // (uuid, velocity vector)
    pawn_ids: Vec<Uuid>,
}
```

Constants (top of file):
```rust
const WAVE_INTERVAL: f32 = 8.0;
const PAWN_SPEED: f32 = 0.8;       // units/sec; multiplied by wave number
const PROJECTILE_SPEED: f32 = 18.0;
const PROJECTILE_MAX_RANGE: f32 = 30.0;
const STARTING_LIVES: u32 = 3;
```

---

## Update Function

One injected update function `game_update(app_state: &mut AppState)` handles all per-frame logic:

1. **Gate:** if phase != `Playing`, return early.
2. **Wave timer:** increment `wave_timer` by `app_state.delta_time`. When `>= WAVE_INTERVAL`, call `spawn_wave`.
3. **Move pawns:** for each UUID in `pawn_ids`, find object and translate by `[0, 0, PAWN_SPEED * wave_speed * delta]`. If Z > 4.0, remove object, remove from list, decrement lives. If lives == 0, set phase to `GameOver`.
4. **Move projectiles:** for each `(uuid, velocity)` in `projectile_ids`, translate object by `velocity * delta`. Track total distance via velocity magnitude × elapsed; remove if > `PROJECTILE_MAX_RANGE`.
5. **Collision:** for each projectile × pawn pair, fetch bounding boxes and call `collision_world::is_colliding`. On hit: remove both objects from scene, remove from both lists, increment score, play `"click"` audio.

Wave speed multiplier: `1.0 + (wave as f32 - 1.0) * 0.15`.

---

## Input

```rust
// Left mouse click → fire projectile
EventCharacteristic::MousePress(MouseButton::Left) → fire_projectile
```

`fire_projectile` reads camera position and constructs a forward direction from camera rotation (yaw + pitch → direction vector). Spawns a cube object at camera position with the emissive material, adds its UUID + velocity to `projectile_ids`.

---

## GUI (egui)

Single injected GUI function with three conditional panels:

**Menu panel** (phase == Menu):
- Title: "Pawn Rush"
- "How to play" text (left-click to shoot, survive waves)
- `[Start Game]` button → sets phase to `Playing`, calls `reset_game`

**HUD** (phase == Playing):
- Score, wave number
- Lives displayed as `♥` characters

**Game Over panel** (phase == GameOver):
- "Game Over" heading
- Final score
- `[Play Again]` button → calls `reset_game`, sets phase to `Playing`

---

## Audio

- `background_music` looping throughout
- `click_sound` played on each pawn kill

---

## Post-Processing

- `Bloom` (threshold 0.95, radius 12) — makes emissive projectiles glow
- `DepthFog` — atmospheric distance fade
- `Vignette` — frames the arena

---

## Reset Logic

`reset_game(app_state)` helper:
- Removes all objects whose UUID is in `pawn_ids` or `projectile_ids`
- Resets `GameState` fields to defaults (score 0, lives 3, wave 1, timer 0, empty lists)
- Re-adds scene objects (terrain, tree, knight) if they were removed — or simpler: only pawns and projectiles are ever removed/added dynamically, so reset just clears those lists

---

## File

`examples/pawn-rush.rs` — gated behind `features=examples`, runnable with:
```bash
cargo run --example=pawn-rush --features=examples
```

Added to `Cargo.toml` alongside the existing examples.
