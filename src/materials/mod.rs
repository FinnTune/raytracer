use crate::{
    objects::HitRecord,
    renderer::{ray::Ray, Color},
};

pub mod diffuse;
pub mod reflective;
pub mod emissive;

pub use diffuse::Diffuse;
pub use reflective::Reflective;
pub use emissive::Emissive;

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