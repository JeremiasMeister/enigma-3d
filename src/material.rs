use glium::Display;
use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;
use glium::uniforms::SamplerWrapFunction;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{resources, shader, texture};
use crate::camera::Camera;
use crate::light::{Light, LightBlock};

#[derive(Serialize, Deserialize, Clone)]
pub struct MaterialSerializer {
    name : String,
    color: [f32; 3],
    albedo: Option<texture::TextureSerializer>,
    transparency: f32,
    normal: Option<texture::TextureSerializer>,
    normal_strength: f32,
    roughness: Option<texture::TextureSerializer>,
    roughness_strength: f32,
    metallic: Option<texture::TextureSerializer>,
    metallic_strength: f32,
    emissive: Option<texture::TextureSerializer>,
    emissive_strength: f32,
    shader: shader::ShaderSerializer,
    matrix: [[f32;4]; 4],
    render_transparent: bool,
    uuid: String
}

pub struct Material {
    pub name: String,
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
    //this should be a raw image
    pub display: glium::Display<WindowSurface>,
    pub program: glium::Program,
    pub time: f32,
    pub matrix: [[f32; 4]; 4],
    pub render_transparent: bool,
    pub uuid: Uuid
}

pub enum TextureType {
    Albedo,
    Normal,
    Roughness,
    Metallic,
    Emissive,
}

impl Clone for Material {
    fn clone(&self) -> Self {
        let mut material = Material::default(self.shader.clone(), &self.display);
        let _tex_white = {
            let raw = Material::tex_raw_from_array([1.0, 1.0, 1.0, 1.0]);
            glium::texture::SrgbTexture2d::new(&self.display, raw).unwrap()
        };
        let _tex_black = {
            let raw = Material::tex_raw_from_array([0.0, 0.0, 0.0, 1.0]);
            glium::texture::SrgbTexture2d::new(&self.display, raw).unwrap()
        };
        let _tex_gray = {
            let raw = Material::tex_raw_from_array([0.5, 0.5, 0.5, 1.0]);
            glium::texture::SrgbTexture2d::new(&self.display, raw).unwrap()
        };
        let _tex_normal = {
            let raw = Material::tex_raw_from_array([0.5, 0.5, 1.0, 1.0]);
            glium::texture::SrgbTexture2d::new(&self.display, raw).unwrap()
        };
        material.name = self.name.clone();
        material.color = self.color.clone();
        material.albedo = match &self.albedo {
            Some(tex) => Some(tex.get_texture_clone(&self.display)),
            None => None
        };
        material.transparency = self.transparency.clone();
        material.normal = match &self.normal {
            Some(tex) => Some(tex.get_texture_clone(&self.display)),
            None => None
        };
        material.normal_strength = self.normal_strength.clone();
        material.roughness = match &self.roughness {
            Some(tex) => Some(tex.get_texture_clone(&self.display)),
            None => None
        };
        material.roughness_strength = self.roughness_strength.clone();
        material.metallic = match &self.metallic {
            Some(tex) => Some(tex.get_texture_clone(&self.display)),
            None => None
        };
        material.metallic_strength = self.metallic_strength.clone();
        material.emissive = match &self.emissive {
            Some(tex) => Some(tex.get_texture_clone(&self.display)),
            None => None
        };
        material.emissive_strength = self.emissive_strength.clone();
        material.matrix = self.matrix.clone();
        material.time = self.time.clone();
        material.render_transparent = self.render_transparent.clone();
        material.uuid = self.uuid.clone();
        material
    }
}

impl Material {
    pub fn default(shader: shader::Shader, display: &glium::Display<WindowSurface>) -> Self {
        Material::new(shader, display.clone(), None, None, None, None, None, None, None, None, None, None)
    }

    pub fn from_serializer(serializer: MaterialSerializer, display: &glium::Display<WindowSurface>) -> Self {
        let shader = shader::Shader::from_serializer(serializer.shader);
        let albedo = match serializer.albedo {
            Some(albedo) => Some(texture::Texture::from_serializer(albedo, &display)),
            None => None,
        };
        let normal = match serializer.normal {
            Some(normal) => Some(texture::Texture::from_serializer(normal, &display)),
            None => None,
        };
        let roughness = match serializer.roughness {
            Some(roughness) => Some(texture::Texture::from_serializer(roughness, &display)),
            None => None,
        };
        let metallic = match serializer.metallic {
            Some(metallic) => Some(texture::Texture::from_serializer(metallic, &display)),
            None => None,
        };
        let emissive = match serializer.emissive {
            Some(emissive) => Some(texture::Texture::from_serializer(emissive, &display)),
            None => None,
        };

        let mut mat = Material::new(shader, display.clone(), Some(serializer.color), albedo, normal, Some(serializer.normal_strength), roughness, Some(serializer.roughness_strength), metallic, Some(serializer.metallic_strength), emissive, Some(serializer.emissive_strength));
        mat.name = serializer.name;
        mat.matrix = serializer.matrix;
        mat.set_transparency_strength(serializer.transparency);
        mat.set_transparency(serializer.render_transparent);
        mat.uuid = Uuid::parse_str(serializer.uuid.as_str()).expect("Failed parsing Uuid");
        mat
    }

    pub fn to_serializer(&self) -> MaterialSerializer {
        MaterialSerializer {
            name: self.name.clone(),
            color: self.color,
            albedo: match &self.albedo {
                Some(albedo) => Some(albedo.to_serializer()),
                None => None,
            },
            transparency: self.transparency,
            normal: match &self.normal {
                Some(normal) => Some(normal.to_serializer()),
                None => None,
            },
            normal_strength: self.normal_strength,
            roughness: match &self.roughness {
                Some(roughness) => Some(roughness.to_serializer()),
                None => None,
            },
            roughness_strength: self.roughness_strength,
            metallic: match &self.metallic {
                Some(metallic) => Some(metallic.to_serializer()),
                None => None,
            },
            metallic_strength: self.metallic_strength,
            emissive: match &self.emissive {
                Some(emissive) => Some(emissive.to_serializer()),
                None => None,
            },
            emissive_strength: self.emissive_strength,
            shader: self.shader.to_serializer(),
            matrix: self.matrix,
            render_transparent: self.render_transparent,
            uuid: self.uuid.to_string()
        }
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
        let geometry_shader = match &shader.geometry_shader {
            Some(shader) => Some(shader.as_str()),
            None => Some(resources::geometry_shader())
        };

        let _program = glium::Program::from_source(&display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), geometry_shader).expect("Failed to compile shader program");
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

        Self {
            name: "New Material".to_string(),
            shader,
            display,
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
            roughness_strength: roughness_strength.unwrap_or_else(|| 1.0),
            metallic: match metallic {
                Some(metallic) => Some(metallic),
                None => None,
            },
            metallic_strength: metallic_strength.unwrap_or_else(|| 1.0),
            emissive: match emissive {
                Some(emissive) => Some(emissive),
                None => None,
            },
            emissive_strength: emissive_strength.unwrap_or_else(|| 1.0),
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
            render_transparent: false,
            uuid: Uuid::new_v4()
        }
    }

    pub fn set_name(&mut self, name: &str){
        self.name = name.to_string()
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
        self.shader = shader.clone();
        self.program = glium::Program::from_source(&self.display, &shader.get_vertex_shader(), &shader.get_fragment_shader(), shader.get_geometry_shader().as_deref()).expect("Failed to compile shader program");
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

    pub fn set_texture_from_resource(&mut self, data: &[u8], texture_type: TextureType) {
        match texture_type {
            TextureType::Albedo => self.albedo = Some(texture::Texture::from_resource(&self.display, data)),
            TextureType::Normal => self.normal = Some(texture::Texture::from_resource(&self.display, data)),
            TextureType::Roughness => self.roughness = Some(texture::Texture::from_resource(&self.display, data)),
            TextureType::Metallic => self.metallic = Some(texture::Texture::from_resource(&self.display, data)),
            TextureType::Emissive => self.emissive = Some(texture::Texture::from_resource(&self.display, data)),
        }
    }

    pub fn set_texture(&mut self, texture: texture::Texture, texture_type: TextureType){
        match texture_type {
            TextureType::Albedo => self.albedo = Some(texture),
            TextureType::Normal => self.normal = Some(texture),
            TextureType::Roughness => self.roughness = Some(texture),
            TextureType::Metallic => self.metallic = Some(texture),
            TextureType::Emissive => self.emissive = Some(texture),
        }
    }

    pub fn lit_pbr(display: Display<WindowSurface>, transparency: bool) -> Self {
        let mut mat = Material::default(shader::Shader::from_strings(resources::vertex_shader(), resources::fragment_shader(), None), &display);
        mat.set_transparency(transparency);
        mat
    }

    pub fn unlit(display: Display<WindowSurface>, transparency: bool) -> Self {
        let mut mat = Material::default(shader::Shader::from_strings(resources::vertex_shader(), resources::fragment_unlit_shader(), None), &display);
        mat.set_transparency(transparency);
        mat
    }

    fn light_block_from_vec(lights: Vec<Light>, ambient_light: Option<Light>) -> LightBlock {
        let mut light_amount = lights.len() as i32;
        if light_amount > 4 {
            light_amount = 4;
        }

        let mut light_position: [[f32; 4];4] = [[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0]];
        let mut light_color: [[f32; 4];4] = [[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0]];
        let mut light_intensity: [f32;4] = [0.0, 0.0, 0.0, 0.0];
        let mut light_direction: [[f32; 4];4] = [[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0]];
        let mut cast_shadow: [i32;4] = [0, 0, 0, 0];

        for i in 0..5 {
            if i < light_amount as usize {
                light_position[i] = [lights[i].position[0], lights[i].position[1], lights[i].position[2], 0.0];
                light_color[i] = [lights[i].color[0], lights[i].color[1], lights[i].color[2], 0.0];
                light_intensity[i] = lights[i].intensity;
                light_direction[i] = {
                    let direction = lights[i].direction;
                    if direction == [0.0, 0.0, 0.0] {
                        [0.0,0.0,0.0,0.0]
                    } else {
                        [lights[i].direction[0], lights[i].direction[1], lights[i].direction[2], 1.0]
                    }
                };
                cast_shadow[i] = if lights[i].cast_shadow { 1 } else { 0 };
            }
        }

        LightBlock {
            position: light_position,
            directions: light_direction,
            cast_shadow,
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

    pub fn get_uniforms<'a>(&'a self, lights: Vec<Light>, ambient_light: Option<Light>, camera: Option<Camera>, model_matrix: Option<&[[f32; 4]; 4]>, skybox: &'a texture::Texture) -> impl glium::uniforms::Uniforms + '_ {

        let light_block = Material::light_block_from_vec(lights, ambient_light);

        glium::uniform! {
            time: self.time,
            matrix: self.matrix,
            camera_position: match camera {
                Some(camera) => {
                    let pos = camera.transform.get_position();
                    [pos.x, pos.y, pos.z]
                }
                None => [0.0,0.0,0.0],
            },
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
                Some(albedo) => {
                    if albedo.tileable{
                        albedo.texture.sampled().wrap_function(SamplerWrapFunction::Repeat)
                    } else{
                        albedo.texture.sampled()
                    }
                },
                None => self._tex_white.sampled()
            },
            mat_normal: match &self.normal {
                Some(normal) => {
                    if normal.tileable {
                        normal.texture.sampled().wrap_function(SamplerWrapFunction::Repeat)
                    } else {
                        normal.texture.sampled()
                    }

                },
                None => self._tex_normal.sampled(),
            },
            mat_normal_strength: self.normal_strength,
            mat_roughness: match &self.roughness {
                Some(roughness) => {
                    if roughness.tileable {
                        roughness.texture.sampled().wrap_function(SamplerWrapFunction::Repeat)
                    } else {
                        roughness.texture.sampled()
                    }
                },
                None => self._tex_gray.sampled()
            },
            mat_roughness_strength: self.roughness_strength,
            mat_metallic: match &self.metallic {
                Some(metallic) => {
                    if metallic.tileable{
                        metallic.texture.sampled().wrap_function(SamplerWrapFunction::Repeat)
                    } else {
                        metallic.texture.sampled()
                    }
                }
                None => self._tex_black.sampled()
            },
            mat_metallic_strength: self.metallic_strength,
            mat_emissive: match &self.emissive {
                Some(emissive) => {
                    if emissive.tileable{
                        emissive.texture.sampled().wrap_function(SamplerWrapFunction::Repeat)
                    } else {
                        emissive.texture.sampled()
                    }
                },
                None => self._tex_black.sampled()
            },
            mat_emissive_strength: self.emissive_strength,
            mat_transparency_strength: self.transparency,
            light_position: light_block.position,
            light_direction: light_block.directions,
            light_color: light_block.color,
            light_intensity: light_block.intensity,
            light_amount: light_block.amount,
            ambient_light_color: light_block.ambient_color,
            ambient_light_intensity: light_block.ambient_intensity,
            model_matrix: match model_matrix {
                Some(matrix) => matrix.clone(),
                None => self.matrix
            },
            skybox: &skybox.texture,
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
