use glium::Display;
use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;
use crate::{shader, texture};

pub struct Material {
    pub color: [f32; 3],
    pub albedo: Option<texture::Texture>,
    pub normal: Option<texture::Texture>,
    pub normal_strength: f32,
    pub roughness: Option<texture::Texture>,
    pub roughness_strength: f32,
    pub metallic: Option<texture::Texture>,
    pub metallic_strength: f32,
    pub shader: shader::Shader,
    _tex_white: glium::texture::SrgbTexture2d,
    _tex_black: glium::texture::SrgbTexture2d,
    _tex_gray: glium::texture::SrgbTexture2d,
    _tex_normal: glium::texture::SrgbTexture2d, //this should be a raw image
    pub display: glium::Display<WindowSurface>,
    pub program: glium::Program,
    pub time: f32,
    pub matrix: [[f32; 4]; 4],
}

pub enum TextureType {
    Albedo,
    Normal,
    Roughness,
    Metallic,
}

impl Material {
    pub fn default(shader: shader::Shader, display: glium::Display<WindowSurface>) -> Self {
        Material::new(shader, display, None, None, None, None, None, None, None, None)
    }
    pub fn new(
        shader: shader::Shader,
        display: glium::Display<WindowSurface>,
        color: Option<[f32; 3]>,
        albedo: Option<texture::Texture>,
        normal: Option<texture::Texture>,
        normal_strength: Option<f32>,
        roughness: Option<texture::Texture>,
        roughness_strength: Option<f32>,
        metallic: Option<texture::Texture>,
        metallic_strength: Option<f32>,
    ) -> Self {
        let _program = glium::Program::from_source(&display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let _tex_white = {
            let raw = Material::tex_raw_from_array([1.0,1.0,1.0,1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };
        let _tex_black = {
            let raw = Material::tex_raw_from_array([0.0,0.0,0.0,1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };
        let _tex_gray = {
            let raw = Material::tex_raw_from_array([0.5,0.5,0.5,1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };
        let _tex_normal = {
            let raw = Material::tex_raw_from_array([0.5,0.5,1.0,1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };

        Self {
            shader,
            display,
            color: match color {
                Some(color) => color,
                None => [1.0, 1.0, 1.0],
            },
            albedo: match albedo {
                Some(albedo) => Some(albedo),
                None => None,
            },
            normal: match normal {
                Some(normal) => Some(normal),
                None => None,
            },
            normal_strength: match normal_strength {
                Some(normal_strength) => normal_strength,
                None => 1.0,
            },
            roughness: match roughness {
                Some(roughness) => Some(roughness),
                None => None,
            },
            roughness_strength: match roughness_strength {
                Some(roughness_strength) => roughness_strength,
                None => 1.0,
            },
            metallic: match metallic {
                Some(metallic) => Some(metallic),
                None => None,
            },
            metallic_strength: match metallic_strength {
                Some(metallic_strength) => metallic_strength,
                None => 1.0,
            },
            _tex_white,
            _tex_black,
            _tex_gray,
            _tex_normal,
            program: _program,
            time: 0.0,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
        }
    }

    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }

    pub fn set_shader(&mut self, shader: shader::Shader) {
        self.shader = shader;
    }

    pub fn set_albedo(&mut self, albedo: texture::Texture) {
        self.albedo = Some(albedo);
    }

    pub fn set_normal(&mut self, normal: texture::Texture) {
        self.normal = Some(normal);
    }

    pub fn set_normal_strength(&mut self, normal_strength: f32) {
        self.normal_strength = normal_strength;
    }

    pub fn set_roughness(&mut self, roughness: texture::Texture) {
        self.roughness = Some(roughness);
    }

    pub fn set_roughness_strength(&mut self, roughness_strength: f32) {
        self.roughness_strength = roughness_strength;
    }

    pub fn set_metallic(&mut self, metallic: texture::Texture) {
        self.metallic = Some(metallic);
    }

    pub fn set_metallic_strength(&mut self, metallic_strength: f32) {
        self.metallic_strength = metallic_strength;
    }

    pub fn set_texture_from_file(&mut self, path: &str, texture_type: TextureType) {
        match texture_type {
            TextureType::Albedo => self.albedo = Some(texture::Texture::new(&self.display, path)),
            TextureType::Normal => self.normal = Some(texture::Texture::new(&self.display, path)),
            TextureType::Roughness => self.roughness = Some(texture::Texture::new(&self.display, path)),
            TextureType::Metallic => self.metallic = Some(texture::Texture::new(&self.display, path)),
        }
    }

    pub fn lit_pbr(display: Display<WindowSurface>) -> Self {
        Material::default(shader::Shader::from_files("res/shader/enigma_vertex_shader.glsl", "res/shader/enigma_fragment_shader.glsl"), display)
    }

    pub fn get_uniforms(&self) -> impl glium::uniforms::Uniforms + '_ {
        glium::uniform! {
            time: self.time,
            matrix: self.matrix,
            mat_color: self.color,
            mat_albedo: match &self.albedo {
                Some(albedo) => &albedo.texture,
                None => &self._tex_white
            },
            mat_normal: match &self.normal {
                Some(normal) => &normal.texture,
                None => &self._tex_normal,
            },
            mat_normal_strength: self.normal_strength,
            mat_roughness: match &self.roughness {
                Some(roughness) => &roughness.texture,
                None => &self._tex_gray
            },
            mat_roughness_strength: self.roughness_strength,
            mat_metallic: match &self.metallic {
                Some(metallic) => &metallic.texture,
                None => &self._tex_black
            },
            mat_metallic_strength: self.metallic_strength,
        }
    }

    fn tex_raw_from_array(color: [f32; 4]) -> RawImage2d<'static, u8> {
        let byte_color: [u8; 4] = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];

        RawImage2d::from_raw_rgba_reversed(&byte_color, (1, 1))
    }

    pub fn update(&mut self) {
        self.time += 0.001;
    }
}
