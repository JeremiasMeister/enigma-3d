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
}