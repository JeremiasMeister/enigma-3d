use glium::glutin::surface::WindowSurface;
use glium::implement_uniform_block;
use nalgebra::{Matrix4, Perspective3, Point3, Unit, Vector3};

pub enum LightType {
    Point,
    Ambient,
}

#[derive(Copy, Clone)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadow: bool,
}

pub struct LightBlock {
    pub position: [[f32; 4]; 4],
    pub color: [[f32; 4]; 4],
    pub intensity: [f32; 4],
    pub amount: i32,
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
}
impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32, cast_shadow: bool) -> Self {
        Self {
            position,
            color,
            intensity,
            cast_shadow,
        }
    }

    pub fn calculate_view_matrix_for_cubemap_face(&self, face_index: usize) -> Matrix4<f32> {
        let light_position = Point3::from(self.position);
        let (target, up) = match face_index {
            0 => (light_position + Vector3::x(), Vector3::y()),
            1 => (light_position - Vector3::x(), Vector3::y()),
            2 => (light_position + Vector3::y(), -Vector3::z()),
            3 => (light_position - Vector3::y(), Vector3::z()),
            4 => (light_position + Vector3::z(), Vector3::y()),
            5 => (light_position - Vector3::z(), Vector3::y()),
            _ => panic!("Invalid cubemap face index"),
        };

        Matrix4::look_at_rh(&light_position, &target, &Unit::new_normalize(up))
    }

    pub fn calculate_projection_matrix_for_point_light(&self, near_plane: f32, far_plane: f32) -> Matrix4<f32> {
        let perspective = Perspective3::new(1.0, std::f32::consts::FRAC_PI_2, near_plane, far_plane);
        perspective.to_homogeneous()
    }
}

impl std::fmt::Debug for LightBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LightBlock")
            .field("position", &self.position)
            .field("color", &self.color)
            .field("intensity", &self.intensity)
            .field("amount", &self.amount)
            .field("ambient_color", &self.ambient_color)
            .finish()
    }
}

glium::implement_uniform_block!(Light, position, color, intensity);
glium::implement_uniform_block!(LightBlock, position, color, intensity, amount, ambient_color, ambient_intensity);


pub fn get_shadow_map_program(display: &glium::Display<WindowSurface>) -> glium::Program {
    let vertex_shader = r#"
        #version 330 core
        in vec3 position;
        uniform mat4 depth_mvp;
        void main() {
            gl_Position = depth_mvp * vec4(position, 1.0);
        }
    "#;
    let fragment_shader = r#"
        #version 330 core
        layout(location = 0) out float fragmentdepth;
        void main(){
            fragmentdepth = gl_FragCoord.z;
        }
    "#;

    glium::Program::from_source(display, &vertex_shader, &fragment_shader, None).expect("Failed to compile shader program")
}