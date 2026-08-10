use crate::materials::{reflect, Material, Scatter};
use crate::objects::HitRecord;
use crate::renderer::{ray::Ray, Color};
use nalgebra::Vector3;
use rand::RngExt;

pub struct Dielectric {
    pub tint: Color,
    /// Index of refraction — 1.0 = vacuum/air, ~1.5 = glass, ~2.4 = diamond
    pub ior: f64,
}

impl Dielectric {
    pub fn new(tint: Color, ior: f64) -> Self {
        Self { tint, ior }
    }
}

/// Bend a unit direction across a boundary per Snell's law.
pub fn refract(uv: Vector3<f64>, n: Vector3<f64>, etai_over_etat: f64) -> Vector3<f64> {
    let cos_theta = (-uv).dot(&n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.norm_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

/// Schlick's approximation for the fraction of light reflected vs. refracted.
pub fn reflectance(cosine: f64, refraction_ratio: f64) -> f64 {
    let r0 = ((1.0 - refraction_ratio) / (1.0 + refraction_ratio)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scatter> {
        let refraction_ratio = if hit.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let cos_theta = (-ray.direction).dot(&hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let mut rng = rand::rng();
        let direction =
            if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.random::<f64>() {
                reflect(ray.direction, hit.normal)
            } else {
                refract(ray.direction, hit.normal, refraction_ratio)
            };

        Some(Scatter {
            ray: Ray::new(hit.point, direction),
            attenuation: self.tint,
        })
    }
}
