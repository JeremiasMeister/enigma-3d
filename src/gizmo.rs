use glium::{Display, implement_vertex, program, Surface, uniform};
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use nalgebra::{Matrix4, Point3, Vector3};
use crate::camera::Camera;

#[derive(Copy, Clone)]
struct GizmoVertex {
    position: [f32; 3],
    color: [f32; 4],
}

implement_vertex!(GizmoVertex, position, color);

pub struct Gizmo {
    position_vertices: Vec<GizmoVertex>,
    line_vertices: Vec<GizmoVertex>,
}

impl Gizmo {
    pub fn new() -> Self {
        Gizmo {
            position_vertices: Vec::new(),
            line_vertices: Vec::new(),
        }
    }

    pub fn draw_position(&mut self, center: Point3<f32>, radius: f32, segments: u32, color: [f32; 4]) {
        let mut circle_vertices = Vec::with_capacity((segments as usize + 1) * 2);

        // Add center point
        circle_vertices.push(GizmoVertex { position: [center.x, center.y, center.z], color });

        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            let z = center.z;

            // Add vertex on the circumference
            circle_vertices.push(GizmoVertex { position: [x, y, z], color });

            // Add center point again (except for the last iteration)
            if i < segments {
                circle_vertices.push(GizmoVertex { position: [center.x, center.y, center.z], color });
            }
        }

        // Add the vertices to the main buffer
        self.position_vertices.extend_from_slice(&circle_vertices);
    }

    pub fn draw_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: [f32;4]) {
        self.line_vertices.push(GizmoVertex { position: [start.x, start.y, start.z], color });
        self.line_vertices.push(GizmoVertex { position: [end.x, end.y, end.z], color });
    }

    pub fn render(&self, display: &Display<WindowSurface>, frame: &mut SimpleFrameBuffer, camera: &Camera) {
        if self.position_vertices.is_empty() && self.line_vertices.is_empty() {
            return;
        }

        let circle_vertex_buffer = glium::VertexBuffer::new(display, &self.position_vertices).unwrap();
        let line_vertex_buffer = glium::VertexBuffer::new(display, &self.line_vertices).unwrap();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let line_indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

        let view_matrix = Matrix4::look_at_lh(
            &Point3::from(camera.transform.get_position()),
            &Point3::from(camera.transform.get_position() + camera.transform.forward()),
            &Vector3::y(),
        );

        let projection_matrix: Matrix4<f32> = Matrix4::from(camera.get_projection_matrix());
        let combined_matrix = projection_matrix * view_matrix;

        let program = program!(display,
            140 => {
                vertex: "
                    #version 140
                    uniform mat4 matrix;
                    in vec3 position;
                    in vec4 color;
                    out vec4 v_col;
                    void main() {
                        gl_Position = matrix * vec4(position, 1.0);
                        v_col = color;
                    }
                ",
                fragment: "
                    #version 140
                    in vec4 v_col;
                    out vec4 color;
                    void main() {
                        color = v_col;
                    }
                "
            }
        ).unwrap();
        let conv_matrix: [[f32; 4]; 4] = combined_matrix.into();
        let uniforms = uniform! {
            matrix: conv_matrix
        };

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::Overwrite,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        if !self.position_vertices.is_empty() {
            frame.draw(
                &circle_vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &draw_parameters
            ).unwrap();
        }

        if !self.line_vertices.is_empty() {
            frame.draw(&line_vertex_buffer, &line_indices, &program, &uniforms, &draw_parameters).unwrap();
        }
    }

    pub fn clear(&mut self) {
        self.position_vertices.clear();
        self.line_vertices.clear();
    }
}