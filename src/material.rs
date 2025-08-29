use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
  pub diffuse: Color,
  pub specular: f32,
  /// albedo: [diffuse, specular, reflection, refraction]
  pub albedo: [f32; 4],
  /// index of refraction (1.0 = air, ~1.5 = glass)
  pub ior: f32,
}

impl Material {
  pub fn new(
    diffuse: Color,
    specular: f32,
    albedo: [f32; 4],
    ior: f32,
  ) -> Self {
    Material { diffuse, specular, albedo, ior }
  }

  pub fn black() -> Self {
    Material {
      diffuse: Color::new(0, 0, 0),
      specular: 0.0,
      albedo: [0.0, 0.0, 0.0, 0.0],
      ior: 1.0,
    }
  }
}