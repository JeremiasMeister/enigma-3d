use glium::glutin::surface::WindowSurface;
use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::DepthTexture2d;
use crate::{AppState, resources, shader};
use crate::geometry::Vertex;
use crate::postprocessing::PostProcessingEffect;

pub struct DepthFog {
    pub min_depth: f32,
    pub max_depth: f32,
    pub fog_cutoff: f32,
    pub color: [f32; 3],
    pub opacity: f32,
    program: glium::Program,
}

impl DepthFog {
    pub fn new(display: &glium::Display<WindowSurface>, min_depth: f32, max_depth: f32, fog_cutoff: f32, color: [f32; 3], opacity: f32) -> Self {
        let fog_shader = shader::Shader::from_strings(resources::post_processing_vertex(), resources::post_processing_depth_fog_fragment(), None);

        let program = glium::Program::from_source(display, &fog_shader.get_vertex_shader(), &fog_shader.get_fragment_shader(), None).expect("Failed to compile shader program");

        Self {
            min_depth,
            max_depth,
            opacity,
            program,
            fog_cutoff,
            color,
        }
    }
}

impl PostProcessingEffect for DepthFog {
    fn render(&self, app_state: &AppState, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d, depth_source: &DepthTexture2d, _buffer_textures: &Vec<Texture2d>) {
        let uniforms = uniform! {
            scene: source,
            depth: depth_source,
            minDepth: self.min_depth,
            maxDepth: self.max_depth,
            fogColor: self.color,
            opacity: self.opacity,
            near: app_state.camera.unwrap().near,
            far: app_state.camera.unwrap().far,
            fogCutoff: self.fog_cutoff,
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: false,
                ..Default::default()
            },
            ..Default::default()
        };

        target.draw(
            &*vertex_buffer,
            &*index_buffer,
            &self.program,
            &uniforms,
            &params,
        ).unwrap();
    }
}