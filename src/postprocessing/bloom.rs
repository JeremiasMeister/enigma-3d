use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::texture::DepthTexture2d;
use crate::geometry::Vertex;
use crate::postprocessing::PostProcessingEffect;
use crate::{AppState, postprocessing, resources, shader};

pub struct Bloom {
    pub threshold: f32,
    pub iterations: i32,
    program_extract: glium::Program,
    program_blur: glium::Program,
    program_combine: glium::Program,
    program_copy: glium::Program,
}

impl Bloom {
    pub fn new(display: &glium::Display<WindowSurface>, threshold: f32, iterations: i32) -> Self {
        let extract_shader = shader::Shader::from_strings(resources::POST_PROCESSING_VERTEX, resources::POST_PROCESSING_BLOOM_EXTRACT_FRAGMENT);
        let blur_shader = shader::Shader::from_strings(resources::POST_PROCESSING_VERTEX, resources::POST_PROCESSING_BLOOM_BLUR_FRAGMENT);
        let combine_shader = shader::Shader::from_strings(resources::POST_PROCESSING_VERTEX, resources::POST_PROCESSING_BLOOM_COMBINE_FRAGMENT);
        let program_extract = glium::Program::from_source(display, &extract_shader.get_vertex_shader(), &extract_shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let program_blur = glium::Program::from_source(display, &blur_shader.get_vertex_shader(), &blur_shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let program_combine = glium::Program::from_source(display, &combine_shader.get_vertex_shader(), &combine_shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let program_copy = postprocessing::get_screen_program(&display);

        Self {
            program_extract,
            program_blur,
            program_combine,
            program_copy,
            threshold,
            iterations,
        }
    }
}

impl PostProcessingEffect for Bloom {
    fn render(&self, app_state: &AppState, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d, _depth_texture: &DepthTexture2d, buffer_textures: &Vec<Texture2d>) {
        // first create a temporary framebuffer to render the scene to and then use it as a texture
        let mut work_framebuffer_1 = SimpleFrameBuffer::new(app_state.display.as_ref().unwrap(), &buffer_textures[0]).unwrap();
        let mut work_framebuffer_2 = SimpleFrameBuffer::new(app_state.display.as_ref().unwrap(), &buffer_textures[1]).unwrap();
        work_framebuffer_1.clear_color(0.0, 0.0, 0.0, 0.0);
        work_framebuffer_2.clear_color(0.0, 0.0, 0.0, 0.0);

        // creating copies of the incomming scene
        let uniforms = uniform! {
            scene: source.sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
        };
        work_framebuffer_1.draw(vertex_buffer, index_buffer, &self.program_copy, &uniforms, &Default::default()).unwrap();
        work_framebuffer_2.draw(vertex_buffer, index_buffer, &self.program_copy, &uniforms, &Default::default()).unwrap();

        // extract the bright parts of the scene
        let uniforms = uniform! {
            scene: source.sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            threshold: self.threshold,
        };
        work_framebuffer_1.draw(vertex_buffer, index_buffer, &self.program_extract, &uniforms, &Default::default()).unwrap();

        // blur the bright parts of the scene
        let uniforms = uniform! {
            scene: buffer_textures[0].sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            horizontal: true,
            iterations: self.iterations,
        };
        work_framebuffer_2.draw(vertex_buffer, index_buffer, &self.program_blur, &uniforms, &Default::default()).unwrap();
        work_framebuffer_1.clear_color(0.0, 0.0, 0.0, 0.0);

        let uniforms = uniform! {
            scene: buffer_textures[1].sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            horizontal: false,
            iterations: self.iterations,
        };
        work_framebuffer_1.draw(vertex_buffer, index_buffer, &self.program_blur, &uniforms, &Default::default()).unwrap();

        // render to original framebuffer
        let uniforms = uniform! {
            scene: source.sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            bloomBlur: buffer_textures[0].sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
        };
        target.draw(vertex_buffer, index_buffer, &self.program_combine, &uniforms, &Default::default()).unwrap();
    }
}