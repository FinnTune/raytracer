use crate::materials::{Material, Scatter};
use crate::objects::HitRecord;
use crate::renderer::{ray::Ray, Color};
use nalgebra::Vector3;
use rand::Rng;

pub struct Reflective {
    pub albedo: Color,
    /// 0.0 = perfect mirror, 1.0 = very rough
    pub fuzz: f64,
}

impl Reflective {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz: fuzz.min(1.0) }
    }
}

fn reflect(v: Vector3<f64>, n: Vector3<f64>) -> Vector3<f64> {
    v - 2.0 * v.dot(&n) * n
}

fn random_in_unit_sphere() -> Vector3<f64> {
    let mut rng = rand::thread_rng();
    loop {
        let v = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        if v.norm_squared() < 1.0 {
            return v;
        }
    }
}

impl Material for Reflective {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scatter> {
        let reflected = reflect(ray.direction, hit.normal);
        let direction = reflected + random_in_unit_sphere() * self.fuzz;

        // Absorb rays that scatter below the surface
        if direction.dot(&hit.normal) <= 0.0 {
            return None;
        }

        Some(Scatter {
            ray: Ray::new(hit.point, direction),
            attenuation: self.albedo,
        })
    }
}