pub mod sprite;

use std::collections::HashMap;
use glium::{Display, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::texture::DepthTexture2d;
use uuid::Uuid;
use crate::camera::Camera;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::RenderStyle;

pub struct ParticleRenderer {
    pub sprite_quad_vb: VertexBuffer<sprite::SpriteCorner>,
    pub sprite_quad_ib: IndexBuffer<u16>,
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
        ParticleRenderer {
            sprite_quad_vb: VertexBuffer::new(display, &corners).unwrap(),
            sprite_quad_ib: IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap(),
        }
    }
}

pub fn render<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    depth_texture: Option<&DepthTexture2d>,
    camera: &Camera,
    systems: &mut [ParticleSystem],
    materials: &HashMap<Uuid, &Material>,
    renderer: &ParticleRenderer,
    screen_size: [f32; 2],
) {
    for system in systems.iter_mut() {
        if system.live_count() == 0 { continue; }
        let mat_id = match system.material_id { Some(id) => id, None => continue };
        let mat = match materials.get(&mat_id) { Some(m) => *m, None => continue };
        match system.config.render {
            RenderStyle::Sprite { .. } => {
                sprite::render_one(display, target, depth_texture, camera, system, mat, renderer, screen_size);
            }
            RenderStyle::Ribbon { .. } => {
                // task 13
            }
        }
    }
}
