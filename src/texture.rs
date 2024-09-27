use image::{RgbaImage};

#[derive(Debug, Clone)]
pub struct Texture {
    pub image: RgbaImage,
}

impl Texture {
    pub fn from_file(path: &str) -> Self {
        let img = image::open(path).expect("Failed to open texture image");
        Self { image: img.to_rgba8() }
    }

    pub fn get_color_at(&self, uv: (f32, f32)) -> [u8; 4] {
        let (u, v) = uv;

        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let x = (u * (self.image.width() - 1) as f32).round() as u32;  
        let y = ((1.0 - v) * (self.image.height() - 1) as f32).round() as u32; 

        let x = x.clamp(0, self.image.width() - 1);
        let y = y.clamp(0, self.image.height() - 1);

        let pixel = self.image.get_pixel(x, y);

        [pixel[0], pixel[1], pixel[2], pixel[3]] 
    }
}
