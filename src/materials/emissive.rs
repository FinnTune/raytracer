use crate::materials::{Material, Scatter};
use crate::objects::HitRecord;
use crate::renderer::{ray::Ray, Color};

pub struct Emissive {
    pub color: Color,
    pub strength: f64,
}

impl Emissive {
    pub fn new(color: Color, strength: f64) -> Self {
        Self { color, strength }
    }
}

impl Material for Emissive {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord) -> Option<Scatter> {
        None // Light sources don't scatter
    }

    fn emitted(&self) -> Color {
        self.color * self.strength
    }
}