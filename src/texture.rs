use glium::glutin::surface::WindowSurface;

pub struct Texture {
    pub path: String,
    pub texture: glium::texture::SrgbTexture2d,
    pub width: u32,
    pub height: u32,
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
        }
    }
}