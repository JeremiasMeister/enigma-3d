use crate::texture;

pub struct Material {
    color: [f32; 3],
    albedo: Option<texture::Texture>,
    normal: Option<texture::Texture>,
    normal_strength: f32,
    roughness: Option<texture::Texture>,
    roughness_strength: f32,
    metallic: Option<texture::Texture>,
    metallic_strength: f32,
    shader: glium::Program,
}

impl Material {
    pub fn new(
        shader: glium::Program,
        color: Option<[f32; 3]>,
        albedo: Option<texture::Texture>,
        normal: Option<texture::Texture>,
        normal_strength: Option<f32>,
        roughness: Option<texture::Texture>,
        roughness_strength: Option<f32>,
        metallic: Option<texture::Texture>,
        metallic_strength: Option<f32>
    ) -> Self {
        Self {
            shader,
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
        }
    }

    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
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

    pub fn default(shader: glium::Program) -> Self {
        Self{
            shader,
            color: [1.0, 1.0, 1.0],
            albedo: None,
            normal: None,
            normal_strength: 1.0,
            roughness: None,
            roughness_strength: 1.0,
            metallic: None,
            metallic_strength: 1.0,
        }
    }
}