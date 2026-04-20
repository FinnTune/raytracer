use crate::materials::{Material, Scatter};
use crate::objects::HitRecord;
use crate::renderer::{ray::Ray, Color};
use nalgebra::Vector3;
use rand::Rng;

pub struct Diffuse {
    pub albedo: Color,
}

impl Diffuse {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

fn random_unit_vector() -> Vector3<f64> {
    let mut rng = rand::thread_rng();
    loop {
        let v = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        if v.norm_squared() < 1.0 {
            return v.normalize();
        }
    }
}

impl Material for Diffuse {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<Scatter> {
        let mut direction = hit.normal + random_unit_vector();

        // Catch degenerate scatter directions
        if direction.iter().all(|c| c.abs() < 1e-8) {
            direction = hit.normal;
        }

        Some(Scatter {
            ray: Ray::new(hit.point, direction),
            attenuation: self.albedo,
        })
    }
}