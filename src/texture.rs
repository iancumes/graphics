use crate::color::Color;
use image::io::Reader as ImageReader; // quitamos GenericImageView para evitar la warning

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<Color>,
}

impl Texture {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let img = ImageReader::open(path)
            .map_err(|e| format!("Failed to open texture '{}': {}", path, e))?
            .decode()
            .map_err(|e| format!("Failed to decode texture '{}': {}", path, e))?
            .to_rgb8();

        // usamos width/height directos, así no necesitamos GenericImageView
        let (w, h) = (img.width(), img.height());
        let mut data = Vec::with_capacity((w * h) as usize);

        for p in img.pixels() {
            data.push(Color::new(p[0], p[1], p[2]));
        }

        Ok(Texture { width: w, height: h, data })
    }

    #[inline]
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // volteamos V

        let x = (u * (self.width  as f32 - 1.0)).round() as u32;
        let y = (v * (self.height as f32 - 1.0)).round() as u32;
        let idx = (y * self.width + x) as usize;
        self.data[idx]
    }
}
