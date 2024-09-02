use glium::glutin::surface::WindowSurface;
use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::DepthTexture2d;
use crate::postprocessing::PostProcessingEffect;
use crate::{AppState, resources, shader};
use crate::geometry::Vertex;

pub struct Vignette {
    program: glium::Program,
    pub intensity: f32,
    pub falloff: f32,
    pub color: [f32; 3],
    pub opacity: f32,
}

impl Vignette {
    pub fn new(display: &glium::Display<WindowSurface>, intensity: f32, falloff: f32, color: [f32; 3], opacity: f32) -> Self {
        let vignette_shader = shader::Shader::from_strings(
            resources::post_processing_vertex(),
            resources::post_processing_vignette_fragment(),
            None
        );

        let program = glium::Program::from_source(
            display,
            &vignette_shader.get_vertex_shader(),
            &vignette_shader.get_fragment_shader(),
            None
        ).expect("Failed to compile vignette shader program");

        Self {
            program,
            intensity: intensity.clamp(0.0, 1.0),
            falloff: falloff.clamp(0.0, 1.0),
            color,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    pub fn set_falloff(&mut self, falloff: f32) {
        self.falloff = falloff.clamp(0.0, 1.0);
    }

    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
}

impl PostProcessingEffect for Vignette {
    fn render(&self, _app_state: &AppState, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d, _depth_source: &DepthTexture2d, _buffer_textures: &Vec<Texture2d>) {
        let uniforms = uniform! {
            scene: source,
            resolution: [source.width() as f32, source.height() as f32],
            intensity: self.intensity,
            falloff: self.falloff,
            vignette_color: self.color,
            opacity: self.opacity,
        };

        target.draw(
            vertex_buffer,
            index_buffer,
            &self.program,
            &uniforms,
            &Default::default(),
        ).unwrap();
    }
}