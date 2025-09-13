
use crate::color::Color;
use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct Material {
  pub diffuse: Color,
  pub texture: Option<Texture>,
}

impl Material {
  pub fn from_color(diffuse: Color) -> Self {
    Material { diffuse, texture: None }
  }

  pub fn from_texture(tex: Texture) -> Self {
    Material { diffuse: Color::new(255,255,255), texture: Some(tex) }
  }

  #[inline]
  pub fn color_at(&self, u: f32, v: f32) -> Color {
    if let Some(tex) = &self.texture {
      tex.sample(u, v)
    } else {
      self.diffuse
    }
  }

  pub fn black() -> Self {
    Material { diffuse: Color::new(0, 0, 0), texture: None }
  }
}
