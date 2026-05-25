use glium::{Display, Surface, VertexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::implement_vertex;
use glium::texture::DepthTexture2d;
use crate::camera::Camera;
use crate::logging::EnigmaError;
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
    // fixed_frame overrides per-particle frame state — every particle in the system
    // renders the same cell.
    let raw = fb.fixed_frame.unwrap_or(frame as u32);
    let idx = raw % fb.frame_count.max(1);
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
    renderer: &mut ParticleRenderer,
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

    // Depth-sort back-to-front for blend modes that require ordered geometry.
    // Additive blending is commutative, so skipping the sort there saves work.
    if matches!(blend_mode, BlendMode::Alpha | BlendMode::PremultipliedAlpha) {
        let view = camera.get_view_matrix();
        let view_z = |p: [f32; 3]| view[0][2]*p[0] + view[1][2]*p[1] + view[2][2]*p[2] + view[3][2];
        // OpenGL view-space looks down -Z; "farther from the camera" means more negative z.
        // Back-to-front = ascending view_z.
        instances.sort_by(|a, b| {
            view_z(a.inst_position).partial_cmp(&view_z(b.inst_position))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Reuse a persistent dynamic VB instead of allocating one per system per frame.
    // Grow on demand (next power of two) so steady-state rendering is zero-alloc.
    let needed = instances.len();
    let cap_ok = renderer.sprite_instance_vb.as_ref().map(|v| v.len() >= needed).unwrap_or(false);
    if !cap_ok {
        let new_cap = needed.next_power_of_two().max(64);
        match VertexBuffer::<SpriteInstance>::empty_dynamic(display, new_cap) {
            Ok(vb) => renderer.sprite_instance_vb = Some(vb),
            Err(e) => {
                EnigmaError::new(Some(&format!("particle sprite VB alloc (cap={}): {}", new_cap, e)), false).log();
                return;
            }
        }
    }
    let instance_vb = renderer.sprite_instance_vb.as_ref().expect("sprite_instance_vb just sized");
    instance_vb.slice(0..needed).expect("slice fits buffer just grown to needed").write(&instances);

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

    let instance_slice = instance_vb.slice(0..needed).expect("sprite instance slice fits buffer");
    let active_instances = instance_slice.per_instance().expect("per_instance layout valid for SpriteInstance");
    if let Err(e) = target.draw(
        (&renderer.sprite_quad_vb, active_instances),
        &renderer.sprite_quad_ib,
        &material.program,
        &uniforms,
        &params,
    ) {
        EnigmaError::new(Some(&format!("particle sprite draw: {}", e)), false).log();
    }
}
