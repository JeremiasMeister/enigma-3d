pub struct Shader {
    pub fragment_shader: String,
    pub vertex_shader: String,
}


impl Shader {
    pub fn new() -> Self {
        Self {
            fragment_shader: String::from(""),
            vertex_shader: String::from(""),
        }
    }
    pub fn set_fragment_shader(&mut self, fragment_shader: String) {
        self.fragment_shader = fragment_shader;
    }
    pub fn set_vertex_shader(&mut self, vertex_shader: String) {
        self.vertex_shader = vertex_shader;
    }
    pub fn get_fragment_shader(&self) -> String {
        self.fragment_shader.clone()
    }
    pub fn get_vertex_shader(&self) -> String {
        self.vertex_shader.clone()
    }

    pub fn from_files(vertex_shader: &str, fragment_shader: &str) -> Self {
        let vertex_shader = std::fs::read_to_string(vertex_shader).expect("Unable to read file");
        let fragment_shader = std::fs::read_to_string(fragment_shader).expect("Unable to read file");
        Self {
            fragment_shader,
            vertex_shader,
        }
    }

    pub fn log(&self) {
        println!("Vertex Shader:\n{}", self.vertex_shader);
        println!("Fragment Shader:\n{}", self.fragment_shader);
    }

    pub fn default() -> Self {
        let vertex_shader = r#"
            #version 140
            uniform float time;
            uniform mat4 matrix;
            in vec3 position;
            void main() {
                gl_Position = matrix * vec4(position, 1.0);
            }
        "#;

        let fragment_shader = r#"
            #version 140
            uniform float time;
            out vec4 color;
            void main() {
                color = vec4(1.0, 0.0, 1.0, 1.0);
            }
        "#;

        Self {
            fragment_shader: String::from(fragment_shader),
            vertex_shader: String::from(vertex_shader),
        }
    }
}

impl Default for Shader {
    fn default() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.vertex_shader, self.fragment_shader)
    }
}

impl std::fmt::Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.vertex_shader, self.fragment_shader)
    }
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            fragment_shader: self.fragment_shader.clone(),
            vertex_shader: self.vertex_shader.clone(),
        }
    }
}