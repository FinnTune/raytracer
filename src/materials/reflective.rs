use crate::materials::{Material, Scatter};
use crate::objects::HitRecord;
use crate::renderer::{ray::Ray, Color};

pub struct Reflective {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Reflective {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz: fuzz.min(1.0) }
    }
}

impl Material for Reflective {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord) -> Option<Scatter> {
        todo!()
    }
}