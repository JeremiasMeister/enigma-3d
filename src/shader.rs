use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ShaderSerializer {
    fragment_shader: String,
    vertex_shader: String,
    geometry_shader: Option<String>
}

pub struct Shader {
    pub fragment_shader: String,
    pub vertex_shader: String,
    pub geometry_shader: Option<String>
}

impl Shader {
    pub fn new() -> Self {
        Self {
            fragment_shader: String::from(""),
            vertex_shader: String::from(""),
            geometry_shader: None,
        }
    }

    pub fn from_serializer(serializer: ShaderSerializer) -> Self {
        Self {
            fragment_shader: serializer.fragment_shader,
            vertex_shader: serializer.vertex_shader,
            geometry_shader: serializer.geometry_shader,
        }
    }

    pub fn to_serializer(&self) -> ShaderSerializer {
        ShaderSerializer {
            fragment_shader: self.fragment_shader.clone(),
            vertex_shader: self.vertex_shader.clone(),
            geometry_shader: self.geometry_shader.clone(),
        }
    }

    pub fn set_fragment_shader(&mut self, fragment_shader: String) {
        self.fragment_shader = fragment_shader;
    }
    pub fn set_vertex_shader(&mut self, vertex_shader: String) {
        self.vertex_shader = vertex_shader;
    }

    pub fn set_geometry_shader(&mut self, geometry_shader: String) {
        self.geometry_shader = Some(geometry_shader);
    }

    pub fn get_fragment_shader(&self) -> String {
        self.fragment_shader.clone()
    }
    pub fn get_vertex_shader(&self) -> String {
        self.vertex_shader.clone()
    }

    pub fn get_geometry_shader(&self) -> Option<String> {
        self.geometry_shader.clone()
    }

    pub fn from_files(vertex_shader: &str, fragment_shader: &str, geometry_shader: Option<&str>) -> Self {
        let vertex_shader = std::fs::read_to_string(vertex_shader).expect("Unable to read file");
        let fragment_shader = std::fs::read_to_string(fragment_shader).expect("Unable to read file");
        let geometry_shader = match geometry_shader {
            Some(geometry_shader) => Some(std::fs::read_to_string(geometry_shader).expect("Unable to read file")),
            None => None,
        };
        Self {
            fragment_shader,
            vertex_shader,
            geometry_shader,
        }
    }

    pub fn from_strings(vertex_shader: &str, fragment_shader: &str, geometry_shader: Option<&str>) -> Self {
        Self {
            fragment_shader: String::from(fragment_shader),
            vertex_shader: String::from(vertex_shader),
            geometry_shader: match geometry_shader {
                Some(geometry_shader) => Some(String::from(geometry_shader)),
                None => None,
            },
        }
    }

    pub fn log(&self) {
        println!("Vertex Shader:\n{}", self.vertex_shader);
        println!("Fragment Shader:\n{}", self.fragment_shader);
        match &self.geometry_shader {
            Some(geometry_shader) => println!("Geometry Shader:\n{}", geometry_shader),
            None => (),
        }
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
            geometry_shader: None,
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
        write!(f, "{}{}{}", self.vertex_shader, self.fragment_shader, match &self.geometry_shader {
            Some(geometry_shader) => geometry_shader,
            None => "",
        })
    }
}

impl std::fmt::Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.vertex_shader, self.fragment_shader, match &self.geometry_shader {
            Some(geometry_shader) => geometry_shader,
            None => "",
        })
    }
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            fragment_shader: self.fragment_shader.clone(),
            vertex_shader: self.vertex_shader.clone(),
            geometry_shader: match &self.geometry_shader {
                Some(geometry_shader) => Some(geometry_shader.clone()),
                None => None,
            },
        }
    }
}