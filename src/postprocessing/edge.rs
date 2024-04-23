use glium::glutin::surface::WindowSurface;
use glium::{IndexBuffer, Surface, Texture2d, uniform, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::DepthTexture2d;
use crate::postprocessing::PostProcessingEffect;
use crate::{AppState, resources, shader};
use crate::geometry::Vertex;

pub struct Edge {
    pub threshold: f32,
    pub color: [f32; 3],
    program: glium::Program,
}

impl Edge {
    pub fn new(display: &glium::Display<WindowSurface>, threshold: f32, color: [f32; 3]) -> Self {
        let edge_shader = shader::Shader::from_strings(resources::POST_PROCESSING_VERTEX, resources::POST_PROCESSING_EDGE_FRAGMENT, None);

        let program = glium::Program::from_source(display, &edge_shader.get_vertex_shader(), &edge_shader.get_fragment_shader(), None).expect("Failed to compile shader program");

        Self {
            program,
            threshold,
            color,
        }
    }
}

impl PostProcessingEffect for Edge {
    fn render(&self, app_state: &AppState, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &IndexBuffer<u32>, target: &mut SimpleFrameBuffer, source: &Texture2d, depth_source: &DepthTexture2d, _buffer_textures: &Vec<Texture2d>) {
        let uniforms = uniform! {
            scene: source,
            depth: depth_source,
            threshold: self.threshold,
            screenSize: [source.width() as f32, source.height() as f32],
            outlineColor: self.color,
            near: app_state.camera.unwrap().near,
            far: app_state.camera.unwrap().far,
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: false,
                .. Default::default()
            },
            .. Default::default()
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