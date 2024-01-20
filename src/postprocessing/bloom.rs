use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use crate::geometry::Vertex;
use crate::postprocessing::PostProcessingEffect;
use crate::{AppState, postprocessing, shader};

pub struct Bloom {
    pub threshold: f32,
    program_extract: glium::Program,
    program_blur: glium::Program,
    program_combine: glium::Program,
    program_copy: glium::Program,
    texture: Texture2d,
}

impl Bloom {
    pub fn new(display: &glium::Display<WindowSurface>, threshold: f32, blur_size: u32) -> Self {
        let extract_shader = shader::Shader::from_files("res/shader/post_processing/post_processing_vert.glsl", "res/shader/post_processing/bloom/enigma_bloom_extract.glsl");
        let blur_shader = shader::Shader::from_files("res/shader/post_processing/post_processing_vert.glsl", "res/shader/post_processing/bloom/enigma_bloom_blur.glsl");
        let combine_shader = shader::Shader::from_files("res/shader/post_processing/post_processing_vert.glsl", "res/shader/post_processing/bloom/enigma_bloom_combine.glsl");
        let program_extract = glium::Program::from_source(display, &extract_shader.get_vertex_shader(), &extract_shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let program_blur = glium::Program::from_source(display, &blur_shader.get_vertex_shader(), &blur_shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let program_combine = glium::Program::from_source(display, &combine_shader.get_vertex_shader(), &combine_shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let program_copy = postprocessing::get_screen_program(&display);
        let texture = Texture2d::empty_with_format(display, glium::texture::UncompressedFloatFormat::F32F32F32F32, glium::texture::MipmapsOption::NoMipmap, blur_size, blur_size).unwrap();

        Self {
            program_extract,
            program_blur,
            program_combine,
            program_copy,
            threshold,
            texture
        }
    }
}

impl PostProcessingEffect for Bloom {
    fn render(&self, app_state: &AppState, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d) {
        // first create a temporary framebuffer to render the scene to and then use it as a texture
        let mut work_framebuffer = SimpleFrameBuffer::new(app_state.display.as_ref().unwrap(), &self.texture).unwrap();
        work_framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);
        let uniforms = uniform! {
            scene: source
        };
        work_framebuffer.draw(vertex_buffer, index_buffer, &self.program_copy, &uniforms, &Default::default()).unwrap();

        // extract the bright parts of the scene
        let uniforms = uniform! {
            scene: &self.texture,
        };
        work_framebuffer.draw(vertex_buffer, index_buffer, &self.program_extract, &uniforms, &Default::default()).unwrap();

        // blur the bright parts of the scene
        let uniforms = uniform! {
            scene: &self.texture,
            horizontal: true,
        };
        work_framebuffer.draw(vertex_buffer, index_buffer, &self.program_blur, &uniforms, &Default::default()).unwrap();

        let uniforms = uniform! {
            scene: &self.texture,
            horizontal: false,
        };
        work_framebuffer.draw(vertex_buffer, index_buffer, &self.program_blur, &uniforms, &Default::default()).unwrap();

        // render to original framebuffer
        let uniforms = uniform! {
            scene: source,
            bloomBlur: &self.texture,
        };
        target.draw(vertex_buffer, index_buffer, &self.program_combine, &uniforms, &Default::default()).unwrap();


    }
}