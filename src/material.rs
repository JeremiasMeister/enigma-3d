use glium::{Display, Surface};
use glium::glutin::surface::WindowSurface;
use glium::texture::{DepthCubemap, RawImage2d};
use glium::uniforms::SamplerWrapFunction;
use crate::{shader, texture};
use crate::camera::Camera;
use crate::light::{Light, LightBlock};

pub struct Material {
    pub name: Option<String>,
    pub color: [f32; 3],
    pub albedo: Option<texture::Texture>,
    pub transparency: f32,
    pub normal: Option<texture::Texture>,
    pub normal_strength: f32,
    pub roughness: Option<texture::Texture>,
    pub roughness_strength: f32,
    pub metallic: Option<texture::Texture>,
    pub metallic_strength: f32,
    pub emissive: Option<texture::Texture>,
    pub emissive_strength: f32,
    pub shader: shader::Shader,
    _tex_white: glium::texture::SrgbTexture2d,
    _tex_black: glium::texture::SrgbTexture2d,
    _tex_gray: glium::texture::SrgbTexture2d,
    _tex_normal: glium::texture::SrgbTexture2d,
    _tex_depth: DepthCubemap,
    pub display: Display<WindowSurface>,
    pub program: glium::Program,
    pub time: f32,
    pub matrix: [[f32; 4]; 4],
    pub render_transparent: bool,
}

pub enum TextureType {
    Albedo,
    Normal,
    Roughness,
    Metallic,
    Emissive,
}

impl Material {
    pub fn default(shader: shader::Shader, display: &glium::Display<WindowSurface>) -> Self {
        Material::new(shader, display.clone(), None, None, None, None, None, None, None, None, None, None)
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
        emissive: Option<texture::Texture>,
        emissive_strength: Option<f32>,
    ) -> Self {
        let _program = glium::Program::from_source(&display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), None).expect("Failed to compile shader program");
        let _tex_white = {
            let raw = Material::tex_raw_from_array([1.0, 1.0, 1.0, 1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };
        let _tex_black = {
            let raw = Material::tex_raw_from_array([0.0, 0.0, 0.0, 1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };
        let _tex_gray = {
            let raw = Material::tex_raw_from_array([0.5, 0.5, 0.5, 1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };
        let _tex_normal = {
            let raw = Material::tex_raw_from_array([0.5, 0.5, 1.0, 1.0]);
            glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
        };

        let _tex_depth = DepthCubemap::empty(&display, 32).unwrap();
        for i in 0..6 {
            let face = texture::cube_layer_from_index(i);
            let mut temp_fbo = glium::framebuffer::SimpleFrameBuffer::depth_only(&display, _tex_depth.main_level().image(face)).unwrap();
            temp_fbo.clear_depth(1.0);
        }

        Self {
            name: None,
            shader,
            display: display.clone(),
            color: color.unwrap_or_else(|| [1.0, 1.0, 1.0]),
            albedo: match albedo {
                Some(albedo) => Some(albedo),
                None => None,
            },
            transparency: 1.0,
            normal: match normal {
                Some(normal) => Some(normal),
                None => None,
            },
            normal_strength: normal_strength.unwrap_or_else(|| 1.0),
            roughness: match roughness {
                Some(roughness) => Some(roughness),
                None => None,
            },
            roughness_strength: roughness_strength.unwrap_or_else(|| 0.5),
            metallic: match metallic {
                Some(metallic) => Some(metallic),
                None => None,
            },
            metallic_strength: metallic_strength.unwrap_or_else(|| 0.0),
            emissive: match emissive {
                Some(emissive) => Some(emissive),
                None => None,
            },
            emissive_strength: emissive_strength.unwrap_or_else(|| 1.0),
            _tex_white,
            _tex_black,
            _tex_gray,
            _tex_normal,
            _tex_depth,
            program: _program,
            time: 0.0,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
            render_transparent: false,
        }
    }

    pub fn set_transparency(&mut self, transparent: bool) {
        self.render_transparent = transparent;
    }

    pub fn set_emissive(&mut self, emissive: texture::Texture) {
        self.emissive = Some(emissive);
    }

    pub fn set_emissive_strength(&mut self, emissive_strength: f32) {
        self.emissive_strength = emissive_strength;
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

    pub fn set_transparency_strength(&mut self, transparency: f32) {
        self.transparency = transparency;
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
            TextureType::Emissive => self.emissive = Some(texture::Texture::new(&self.display, path)),
        }
    }

    pub fn lit_pbr(display: Display<WindowSurface>, transparency: bool) -> Self {
        let mut mat = Material::default(shader::Shader::from_files("res/shader/enigma_vertex_shader.glsl", "res/shader/enigma_fragment_shader.glsl"), &display);
        mat.set_transparency(transparency);
        mat
    }

    pub fn unlit(display: Display<WindowSurface>, transparency: bool) -> Self {
        let mut mat = Material::default(shader::Shader::from_files("res/shader/enigma_vertex_shader.glsl", "res/shader/enigma_fragment_unlit.glsl"), &display);
        mat.set_transparency(transparency);
        mat
    }

    fn light_block_from_vec(lights: Vec<Light>, ambient_light: Option<Light>) -> LightBlock {
        let mut light_amount = lights.len() as i32;
        if light_amount > 4 {
            light_amount = 4;
        }

        let mut light_position: [[f32; 4]; 4] = [[0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]];
        let mut light_color: [[f32; 4]; 4] = [[0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]];
        let mut light_intensity: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        // a maximum of 4 lights can be passed to the shader
        for i in 0..5 {
            if i < light_amount as usize {
                light_position[i] = [lights[i].position[0], lights[i].position[1], lights[i].position[2], 0.0];
                light_color[i] = [lights[i].color[0], lights[i].color[1], lights[i].color[2], 0.0];
                light_intensity[i] = lights[i].intensity;
            }
        }

        LightBlock {
            position: light_position,
            color: light_color,
            intensity: light_intensity,
            amount: light_amount,
            ambient_color: match ambient_light {
                Some(ambient_light) => ambient_light.color,
                None => [0.0, 0.0, 0.0],
            },
            ambient_intensity: match ambient_light {
                Some(ambient_light) => ambient_light.intensity,
                None => 0.0,
            },
        }
    }

    pub fn get_uniforms<'a>(&'a self, lights: Vec<Light>, ambient_light: Option<Light>, camera: Option<Camera>, model_matrix: Option<[[f32; 4]; 4]>, skybox: &'a texture::Texture, shadow_maps: &mut Vec<&'a glium::texture::DepthCubemap>) -> impl glium::uniforms::Uniforms + '_ {

        let light_block = Material::light_block_from_vec(lights, ambient_light);
        for i in 0..5 { // ensure to have exactly 4 shadow maps for the shader to calculate on
            if shadow_maps.len() < i {
                shadow_maps.push(&self._tex_depth);
            }
        }

        glium::uniform! {
            time: self.time,
            matrix: self.matrix,
            projection_matrix: match camera {
                Some(camera) => camera.get_projection_matrix(),
                None => Camera::new(None, None, None, None, None, None).get_projection_matrix(),
            },
            view_matrix: match camera {
                Some(camera) => camera.get_view_matrix(),
                None => Camera::new(None, None, None, None, None, None).get_view_matrix(),
            },
            mat_color: self.color,
            mat_albedo: match &self.albedo {
                Some(albedo) => albedo.texture.sampled(),
                None => self._tex_white.sampled()
            },
            mat_normal: match &self.normal {
                Some(normal) => normal.texture.sampled(),
                None => self._tex_normal.sampled(),
            },
            mat_normal_strength: self.normal_strength,
            mat_roughness: match &self.roughness {
                Some(roughness) => roughness.texture.sampled(),
                None => self._tex_gray.sampled()
            },
            mat_roughness_strength: self.roughness_strength,
            mat_metallic: match &self.metallic {
                Some(metallic) => metallic.texture.sampled(),
                None => self._tex_black.sampled()
            },
            mat_metallic_strength: self.metallic_strength,
            mat_emissive: match &self.emissive {
                Some(emissive) => emissive.texture.sampled(),
                None => self._tex_black.sampled()
            },
            mat_emissive_strength: self.emissive_strength,
            mat_transparency_strength: self.transparency,
            light_position: light_block.position,
            light_color: light_block.color,
            light_intensity: light_block.intensity,
            light_amount: light_block.amount,
            ambient_light_color: light_block.ambient_color,
            ambient_light_intensity: light_block.ambient_intensity,
            model_matrix: model_matrix.unwrap_or_else(|| self.matrix),
            skybox: &skybox.texture,
            far: match camera {
                Some(camera) => camera.far,
                None => 100.0f32,
            },
            near: match camera {
                Some(camera) => camera.near,
                None => 0.1f32,
            },
            shadow_far: 100.0f32, // TODO: expose this to the user
            shadow_map0: shadow_maps[0].sampled().wrap_function(SamplerWrapFunction::Clamp),
            shadow_map1: shadow_maps[1].sampled().wrap_function(SamplerWrapFunction::Clamp),
            shadow_map2: shadow_maps[2].sampled().wrap_function(SamplerWrapFunction::Clamp),
            shadow_map3: shadow_maps[3].sampled().wrap_function(SamplerWrapFunction::Clamp),
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
