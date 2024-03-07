use glium::glutin::surface::WindowSurface;
use std::path::Path;
use std::vec::Vec;

pub struct Texture {
    pub path: String,
    pub texture: glium::texture::SrgbTexture2d,
    pub width: u32,
    pub height: u32,
    pub binary_data: Option<Vec<u8>>
}

impl Texture {
    pub fn new(display: &glium::Display<WindowSurface>, path: &str) -> Self {
        let image = image::open(path).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();
        Self {
            texture,
            path: String::from(path),
            width: image_dimensions.0,
            height: image_dimensions.1,
            binary_data: None
        }
    }
    pub fn from_resource(display: &glium::Display<WindowSurface>, data: &[u8]) -> Self {
        let image = image::load_from_memory(data).expect("Failed to load image").to_rgba8();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();
        Self {
            texture,
            path: String::from("INTERNAL ENIGMA RESOURCE"),
            width: image_dimensions.0,
            height: image_dimensions.1,
            binary_data: Some(data.to_vec())
        }
    }
    pub fn get_texture_clone(&self, display: &glium::Display<WindowSurface>) -> Self {
        let path_str = self.path.clone();
        let path = Path::new(&path_str);
        if !path.is_file() {
            match &self.binary_data {
                Some(data) => {
                   return Texture::from_resource(display, data.as_slice());
                }
                None => {
                    println!("could not clone texture , returned texture is empty");
                    let empty_tex = glium::texture::SrgbTexture2d::empty(display, self.width, self.height);
                    return Self {
                        texture: empty_tex.expect("Could not create Empty Texture"),
                        path: String::from("RESOURCE"),
                        width: self.width,
                        height: self.height,
                        binary_data: None,
                    }
                }
            }
        } else {
            return Texture::new(display, path_str.as_str());
        }
    }
}

