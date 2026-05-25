use glium::{Display, Surface, VertexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::implement_vertex;
use glium::texture::DepthTexture2d;
use crate::camera::Camera;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::{RenderStyle, BlendMode, FlipbookConfig};
use super::ParticleRenderer;

#[derive(Copy, Clone)]
pub struct SpriteCorner {
    pub corner: [f32; 3],
    pub corner_uv: [f32; 2],
}
implement_vertex!(SpriteCorner, corner, corner_uv);

#[derive(Copy, Clone)]
pub struct SpriteInstance {
    pub inst_position:    [f32; 3],
    pub inst_size:        f32,
    pub inst_color:       [f32; 4],
    pub inst_velocity:    [f32; 3],
    pub inst_rotation:    f32,
    pub inst_flipbook_uv: [f32; 4],
}
implement_vertex!(SpriteInstance, inst_position, inst_size, inst_color, inst_velocity, inst_rotation, inst_flipbook_uv);

pub fn blend_params(mode: BlendMode) -> glium::Blend {
    use glium::{Blend, BlendingFunction, LinearBlendingFactor::*};
    match mode {
        BlendMode::Additive => Blend {
            color: BlendingFunction::Addition { source: SourceAlpha, destination: One },
            alpha: BlendingFunction::Addition { source: One, destination: One },
            ..Default::default()
        },
        BlendMode::Alpha => Blend::alpha_blending(),
        BlendMode::PremultipliedAlpha => Blend {
            color: BlendingFunction::Addition { source: One, destination: OneMinusSourceAlpha },
            alpha: BlendingFunction::Addition { source: One, destination: OneMinusSourceAlpha },
            ..Default::default()
        },
    }
}

fn flipbook_uv(frame: f32, fb: &FlipbookConfig) -> [f32; 4] {
    let idx = (frame as u32) % fb.frame_count.max(1);
    let col = (idx % fb.cols.max(1)) as f32;
    let row = (idx / fb.cols.max(1)) as f32;
    let w = 1.0 / fb.cols.max(1) as f32;
    let h = 1.0 / fb.rows.max(1) as f32;
    [col * w, 1.0 - (row + 1.0) * h, w, h]
}

pub fn render_one<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    depth_texture: Option<&DepthTexture2d>,
    camera: &Camera,
    system: &ParticleSystem,
    material: &Material,
    renderer: &ParticleRenderer,
    screen_size: [f32; 2],
) {
    let (flipbook, blend_mode, soft_fade, vel_stretch) = match &system.config.render {
        RenderStyle::Sprite { flipbook, blend_mode, soft_particles, soft_fade_distance, velocity_stretch } => {
            let fade = if *soft_particles { *soft_fade_distance } else { 0.0 };
            (flipbook.clone(), *blend_mode, fade, *velocity_stretch)
        }
        _ => return,
    };

    // Build instance data
    let mut instances: Vec<SpriteInstance> = Vec::with_capacity(system.live_count());
    for i in 0..system.live_count() {
        let uv = match &flipbook {
            Some(fb) => flipbook_uv(system.buffer.flipbook_frame[i], fb),
            None => [0.0, 0.0, 1.0, 1.0],
        };
        instances.push(SpriteInstance {
            inst_position:    system.buffer.position[i],
            inst_size:        system.buffer.size[i],
            inst_color:       system.buffer.color[i],
            inst_velocity:    system.buffer.velocity[i],
            inst_rotation:    system.buffer.rotation[i],
            inst_flipbook_uv: uv,
        });
    }
    if instances.is_empty() { return; }

    let instance_vb = match VertexBuffer::dynamic(display, &instances) {
        Ok(vb) => vb,
        Err(_) => return,
    };

    let albedo_sampler = match material.albedo.as_ref() {
        Some(t) => &t.texture,
        None => &renderer.fallback_albedo,
    }.sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

    let depth_sampler = depth_texture.unwrap_or(&renderer.fallback_depth).sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest);

    let view = camera.get_view_matrix();
    let proj = camera.get_projection_matrix();
    let near = camera.get_near();
    let far = camera.get_far();
    let use_flipbook: i32 = if flipbook.is_some() { 1 } else { 0 };

    let uniforms = glium::uniform! {
        view_matrix:       view,
        projection_matrix: proj,
        albedo:            albedo_sampler,
        scene_depth:       depth_sampler,
        soft_fade_distance: soft_fade,
        use_flipbook:      use_flipbook,
        screen_size:       screen_size,
        near_plane:        near,
        far_plane:         far,
        velocity_stretch:  vel_stretch,
    };

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: false,
            ..Default::default()
        },
        blend: blend_params(blend_mode),
        ..Default::default()
    };

    let _ = target.draw(
        (&renderer.sprite_quad_vb, instance_vb.per_instance().unwrap()),
        &renderer.sprite_quad_ib,
        &material.program,
        &uniforms,
        &params,
    );
}
