use crate::{
    objects::HitRecord,
    renderer::{ray::Ray, Color},
};
use nalgebra::Vector3;

pub mod dielectric;
pub mod diffuse;
pub mod emissive;
pub mod reflective;

pub use dielectric::Dielectric;
pub use diffuse::Diffuse;
pub use emissive::Emissive;
pub use reflective::Reflective;

/// Reflect a direction around a surface normal.
pub fn reflect(v: Vector3<f64>, n: Vector3<f64>) -> Vector3<f64> {
    v - 2.0 * v.dot(&n) * n
}

pub struct Scatter {
    /// The new ray after scattering
    pub ray: Ray,
    /// How much the surface attenuates the incoming light
    pub attenuation: Color,
}

pub trait Material: Send + Sync {
    /// Returns Some(scatter) if the ray bounces, None if it is absorbed
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scatter>;

    /// Light emitted by this material (only non-zero for Emissive)
    fn emitted(&self) -> Color {
        Color::BLACK
    }
}
