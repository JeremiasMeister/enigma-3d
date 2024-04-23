use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::texture::DepthTexture2d;
use crate::geometry::Vertex;
use crate::{AppState, resources, shader};
use crate::postprocessing::PostProcessingEffect;

pub struct GrayScale {
    program: glium::Program,
}

impl PostProcessingEffect for GrayScale {
    fn render(&self, _app_state: &AppState, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d, _depth_source: &DepthTexture2d, _buffer_textures: &Vec<Texture2d>) {
        let uniforms = uniform! {
            scene: source,
        };

        target.draw(
            &*vertex_buffer,
            &*index_buffer,
            &self.program,
            &uniforms,
            &Default::default(),
        ).unwrap();
    }
}

impl GrayScale {
    pub fn new(display: &glium::Display<WindowSurface>) -> Self {
        let shader = shader::Shader::from_strings(resources::POST_PROCESSING_VERTEX, resources::POST_PROCESSING_GRAYSCALE_FRAGMENT, None);
        let program = glium::Program::from_source(display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        Self {
            program,
        }
    }
}