use crate::materials::{Material, Scatter};
use crate::objects::HitRecord;
use crate::renderer::{ray::Ray, Color};

pub struct Diffuse {
    pub albedo: Color,
}

impl Diffuse {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Diffuse {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord) -> Option<Scatter> {
        todo!()
    }
}