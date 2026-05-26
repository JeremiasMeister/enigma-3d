use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use glium::glutin::surface::WindowSurface;
use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::DepthTexture2d;
use nalgebra::{Matrix4, Vector3};
use crate::{AppState, resources, shader};
use crate::geometry::Vertex;
use crate::postprocessing::PostProcessingEffect;

/// Editor-style world-space grid drawn as a post-process effect so it can
/// sample the scene depth texture and be occluded by 3D geometry per-pixel.
///
/// Reconstructs the Y = 0 intersection of each pixel's view ray, draws an
/// anti-aliased grid pattern, and discards fragments where scene geometry is
/// already in front of the grid plane.
pub struct GridOverlay {
    pub extent: f32,
    pub major_step: f32,
    pub minor_color: [f32; 4],
    pub major_color: [f32; 4],
    pub x_axis_color: [f32; 4],
    pub z_axis_color: [f32; 4],
    // Toggled externally (e.g. from an editor toolbar) without having to
    // remove and re-register the effect in `post_processes`.
    enabled: Arc<AtomicBool>,
    program: glium::Program,
}

impl GridOverlay {
    pub fn new(display: &glium::Display<WindowSurface>) -> Self {
        let s = shader::Shader::from_strings(
            resources::post_processing_vertex(),
            resources::post_processing_grid_overlay_fragment(),
            None,
        );
        let program = glium::Program::from_source(
            display,
            &s.get_vertex_shader(),
            &s.get_fragment_shader(),
            None,
        ).expect("Failed to compile grid overlay shader");

        Self {
            extent: 50.0,
            major_step: 10.0,
            // Straight-alpha (non-premultiplied) — the shader does the over-composite.
            minor_color:  [0.31, 0.31, 0.31, 0.43],
            major_color:  [0.55, 0.55, 0.55, 0.67],
            x_axis_color: [0.78, 0.27, 0.27, 0.86],
            z_axis_color: [0.27, 0.39, 0.78, 0.86],
            enabled: Arc::new(AtomicBool::new(true)),
            program,
        }
    }

    /// Returns a handle that other systems can use to toggle the overlay on or
    /// off without holding a reference to the effect inside the post-process vec.
    pub fn enabled_handle(&self) -> Arc<AtomicBool> {
        self.enabled.clone()
    }
}

impl PostProcessingEffect for GridOverlay {
    fn render(
        &self,
        app_state: &AppState,
        vertex_buffer: &VertexBuffer<Vertex>,
        index_buffer: &IndexBuffer<u32>,
        target: &mut SimpleFrameBuffer,
        source: &Texture2d,
        depth_source: &DepthTexture2d,
        _buffer_textures: &Vec<Texture2d>,
    ) {
        if !self.enabled.load(Ordering::Relaxed) {
            // The engine copies the current scene texture into the ping-pong
            // buffer before each effect, so the target framebuffer already
            // holds the unmodified scene — skipping the draw is a clean pass-
            // through and keeps the chain intact for subsequent effects.
            return;
        }

        let camera = match app_state.camera.as_ref() {
            Some(c) => c,
            None => return,
        };

        let cam_pos = Vector3::from(camera.get_position());
        let forward = Vector3::from(camera.calculate_direction_vector()).normalize();
        let world_up = Vector3::new(0.0, 1.0, 0.0);

        // Same basis the editor's projection math uses; falls back when looking
        // straight up/down so we never produce NaNs.
        let right_raw = forward.cross(&world_up);
        let right = if right_raw.norm_squared() < 1e-6 {
            forward.cross(&Vector3::new(0.0, 0.0, 1.0)).normalize()
        } else {
            right_raw.normalize()
        };
        let up = right.cross(&forward).normalize();

        let aspect = camera.width / camera.height;
        let half_h = (camera.fov * 0.5).tan();
        let half_w = half_h * aspect;

        let view = Matrix4::from(camera.get_view_matrix());
        let proj = Matrix4::from(camera.get_projection_matrix());
        let vp: [[f32; 4]; 4] = (proj * view).into();

        let uniforms = uniform! {
            scene: source,
            depth_tex: depth_source,
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z],
            cam_forward: [forward.x, forward.y, forward.z],
            cam_right: [right.x, right.y, right.z],
            cam_up: [up.x, up.y, up.z],
            half_w: half_w,
            half_h: half_h,
            view_proj: vp,
            extent: self.extent,
            major_step: self.major_step,
            minor_color: self.minor_color,
            major_color: self.major_color,
            x_axis_color: self.x_axis_color,
            z_axis_color: self.z_axis_color,
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            depth: glium::Depth {
                test: glium::DepthTest::Overwrite,
                write: false,
                ..Default::default()
            },
            ..Default::default()
        };

        target
            .draw(vertex_buffer, index_buffer, &self.program, &uniforms, &params)
            .expect("Failed to draw grid overlay");
    }
}
