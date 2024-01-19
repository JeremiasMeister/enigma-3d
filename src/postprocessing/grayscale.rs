use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use crate::geometry::Vertex;
use crate::shader;
use crate::postprocessing::{get_screen_indices_rect, get_screen_vert_rect, PostProcessingEffect};

pub struct GrayScale {
    program: glium::Program,
}

impl PostProcessingEffect for GrayScale {
    fn render(&self, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d) {
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
        let shader = shader::Shader::from_files("res/shader/post_processing/post_processing_vert.glsl", "res/shader/post_processing/grayscale/enigma_grayscale.glsl");
        let program = glium::Program::from_source(display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        Self {
            program,
        }
    }
}