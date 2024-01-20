use glium::{Display, IndexBuffer, Texture2d, VertexBuffer};
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use crate::AppState;
use crate::geometry::Vertex;

pub mod grayscale;
pub mod bloom;

pub trait PostProcessingEffect {
    fn render(&self, _app_state: &AppState, _vertex_buffer: &VertexBuffer<Vertex>, _index_buffer: &IndexBuffer<u32>, _target: &mut SimpleFrameBuffer, _source: &Texture2d) {
        println!("PostProcessingEffect::render() not implemented. Please implement this method in your postprocessing struct.");
    }
}

pub fn get_screen_vert_rect(display: &Display<WindowSurface>) -> glium::VertexBuffer<Vertex> {
    let vertices = vec![
        Vertex { position: [-1.0, -1.0, 0.0], texcoord: [0.0, 0.0], color: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [-1.0, 1.0, 0.0], texcoord: [0.0, 1.0], color: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [1.0, 1.0, 0.0], texcoord: [1.0, 1.0], color: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [1.0, -1.0, 0.0], texcoord: [1.0, 0.0], color: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
    ];
    glium::VertexBuffer::new(display, &vertices).unwrap()
}

pub fn get_screen_indices_rect(display: &Display<WindowSurface>) -> glium::IndexBuffer<u32> {
    let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
    glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap()
}

pub fn get_screen_program(display: &Display<WindowSurface>) -> glium::Program {
    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec2 texcoord;
        in vec3 color;
        in vec3 normal;

        out vec2 TEXCOORD;


        void main() {
            TEXCOORD = texcoord;
            gl_Position = vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 TEXCOORD;

        out vec4 color;

        uniform sampler2D scene;

        void main() {
            color = texture(scene, TEXCOORD);
        }
    "#;

    glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).expect("Failed to compile shader program")
}