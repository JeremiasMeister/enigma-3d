use glium::glutin::surface::WindowSurface;
use std::path::Path;
use std::vec::Vec;
use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::time::Instant;
use image::{ImageFormat, DynamicImage, RgbaImage};
use glium::Display;
use glium::backend::glutin::DisplayCreationError;
use glium::texture::{RawImage2d, SrgbTexture2d, MipmapsOption};
use glium::uniforms::SamplerWrapFunction;
use lru::LruCache;
use std::num::NonZeroUsize;
use rayon::prelude::*;

thread_local! {
    static IMAGE_CACHE: RefCell<LruCache<Vec<u8>, RgbaImage>> = RefCell::new(LruCache::new(NonZeroUsize::new(20).unwrap()));
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextureSerializer {
    path: String,
    width: u32,
    height: u32,
    binary_data: Option<Vec<u8>>,
}

pub struct Texture {
    pub path: String,
    pub texture: glium::texture::SrgbTexture2d,
    pub width: u32,
    pub height: u32,
    pub binary_data: Option<Vec<u8>>,
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
            binary_data: None,
        }
    }

    pub fn from_serializer(serializer: TextureSerializer, display: &glium::Display<WindowSurface>) -> Self {
        let path = Path::new(&serializer.path);
        if !path.is_file() {
            match &serializer.binary_data {
                Some(data) => {
                    return Texture::from_resource(display, data.as_slice());
                }
                None => {}
            }
        } else {
            return Texture::new(display, path.to_str().unwrap());
        }
        println!("could not create texture from serializer, returned texture is empty");
        let empty_tex = glium::texture::SrgbTexture2d::empty(display, serializer.width, serializer.height);
        return Self {
            texture: empty_tex.expect("Could not create Empty Texture"),
            path: String::from("RESOURCE"),
            width: serializer.width,
            height: serializer.height,
            binary_data: None,
        };
    }

    pub fn to_serializer(&self) -> TextureSerializer {
        TextureSerializer {
            path: self.path.clone(),
            width: self.width,
            height: self.height,
            binary_data: self.binary_data.clone(),
        }
    }

    pub fn from_resource(display: &Display<WindowSurface>, data: &[u8]) -> Self {

        let image = IMAGE_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.get_or_insert(data.to_vec(), || {
                let img = image::load_from_memory(data).unwrap_or_else(|_| {
                    DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 255, 255])))
                });
                img.to_rgba8()
            }).clone()
        });

        let dimensions = image.dimensions();
        let raw_image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);
        let texture = SrgbTexture2d::with_mipmaps(display, raw_image, MipmapsOption::AutoGeneratedMipmaps).unwrap();

        Self {
            texture,
            path: String::from("INTERNAL ENIGMA RESOURCE"),
            width: dimensions.0,
            height: dimensions.1,
            binary_data: Some(data.to_vec()),
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
                    };
                }
            }
        } else {
            return Texture::new(display, path_str.as_str());
        }
    }

    pub fn pink_texture(display: &glium::Display<WindowSurface>) -> Self {
        let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 255, 255]));
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();
        Self {
            texture,
            path: String::from("PINK"),
            width: image_dimensions.0,
            height: image_dimensions.1,
            binary_data: None,
        }
    }


}

