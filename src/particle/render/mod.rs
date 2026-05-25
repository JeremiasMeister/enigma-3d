pub mod sprite;
pub mod ribbon;

use std::collections::HashMap;
use glium::{Display, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::texture::{DepthTexture2d, SrgbTexture2d, RawImage2d};
use uuid::Uuid;
use crate::camera::Camera;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::RenderStyle;

pub struct ParticleRenderer {
    pub sprite_quad_vb: VertexBuffer<sprite::SpriteCorner>,
    pub sprite_quad_ib: IndexBuffer<u16>,
    /// 1×1 white texture used when a particle material has no albedo bound,
    /// so unauthored materials still render (final color = white × instance color).
    pub fallback_albedo: SrgbTexture2d,
    /// 1×1 depth texture used when the caller passes no scene depth (disables soft-particles).
    pub fallback_depth: DepthTexture2d,
}

impl ParticleRenderer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let corners = vec![
            sprite::SpriteCorner { corner: [-0.5, -0.5, 0.0], corner_uv: [0.0, 0.0] },
            sprite::SpriteCorner { corner: [ 0.5, -0.5, 0.0], corner_uv: [1.0, 0.0] },
            sprite::SpriteCorner { corner: [ 0.5,  0.5, 0.0], corner_uv: [1.0, 1.0] },
            sprite::SpriteCorner { corner: [-0.5,  0.5, 0.0], corner_uv: [0.0, 1.0] },
        ];
        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];
        let white = RawImage2d::from_raw_rgba(vec![255u8, 255, 255, 255], (1, 1));
        ParticleRenderer {
            sprite_quad_vb: VertexBuffer::new(display, &corners).unwrap(),
            sprite_quad_ib: IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap(),
            fallback_albedo: SrgbTexture2d::new(display, white).unwrap(),
            fallback_depth: DepthTexture2d::empty(display, 1, 1).unwrap(),
        }
    }
}

pub fn render<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    depth_texture: Option<&DepthTexture2d>,
    camera: &Camera,
    systems: &[ParticleSystem],
    materials: &HashMap<Uuid, &Material>,
    renderer: &ParticleRenderer,
    screen_size: [f32; 2],
) {
    use crate::particle::config::BlendMode;
    let cam_pos = camera.get_position();

    // Sort: additive first (order-independent), then alpha back-to-front
    let blend_rank = |m: BlendMode| match m {
        BlendMode::Additive => 0,
        BlendMode::PremultipliedAlpha => 1,
        BlendMode::Alpha => 2,
    };
    let mut order: Vec<usize> = (0..systems.len()).collect();
    order.sort_by(|&a, &b| {
        let pa = systems[a].transform[3];
        let pb = systems[b].transform[3];
        let da = (pa[0]-cam_pos[0]).powi(2) + (pa[1]-cam_pos[1]).powi(2) + (pa[2]-cam_pos[2]).powi(2);
        let db = (pb[0]-cam_pos[0]).powi(2) + (pb[1]-cam_pos[1]).powi(2) + (pb[2]-cam_pos[2]).powi(2);
        let mode_a = match systems[a].config.render { RenderStyle::Sprite { blend_mode, .. } | RenderStyle::Ribbon { blend_mode, .. } => blend_mode };
        let mode_b = match systems[b].config.render { RenderStyle::Sprite { blend_mode, .. } | RenderStyle::Ribbon { blend_mode, .. } => blend_mode };
        blend_rank(mode_a).cmp(&blend_rank(mode_b))
            .then(db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal))
    });

    for idx in order {
        let system = &systems[idx];
        if system.live_count() == 0 { continue; }
        let mat_id = match system.material_id { Some(id) => id, None => continue };
        let mat = match materials.get(&mat_id) { Some(m) => *m, None => continue };
        match system.config.render {
            RenderStyle::Sprite { .. } => {
                sprite::render_one(display, target, depth_texture, camera, system, mat, renderer, screen_size);
            }
            RenderStyle::Ribbon { .. } => {
                ribbon::render_one(display, target, camera, system, mat, renderer);
            }
        }
    }
}
