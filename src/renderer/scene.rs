use crate::materials::Material;
use crate::objects::{HitRecord, Hittable};
use crate::renderer::{ray::Ray, Color};
use std::sync::Arc;

pub struct Scene {
    pub objects: Vec<Arc<dyn Hittable>>,
    pub materials: Vec<Arc<dyn Material>>,
    pub background: Color,
}

impl Scene {
    pub fn new(background: Color) -> Self {
        Self {
            objects: Vec::new(),
            materials: Vec::new(),
            background,
        }
    }

    /// Add a material, returns its id for use when building objects
    pub fn add_material(&mut self, material: impl Material + 'static) -> usize {
        self.materials.push(Arc::new(material));
        self.materials.len() - 1
    }

    /// Add a hittable object
    pub fn add_object(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Arc::new(object));
    }

    /// Find the closest hit across all objects in [t_min, t_max]
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, t_min, closest) {
                closest = hit.t;
                result = Some(hit);
            }
        }

        result
    }

    /// Recursively trace a ray, returning the accumulated color
    pub fn trace(&self, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::BLACK;
        }

        match self.hit(ray, 1e-4, f64::MAX) {
            None => self.background,
            Some(hit) => {
                let material = &self.materials[hit.material_id];
                let emitted = material.emitted();

                match material.scatter(ray, &hit) {
                    None => emitted,
                    Some(scatter) => {
                        let incoming = self.trace(&scatter.ray, depth - 1);
                        emitted + incoming.attenuate(scatter.attenuation)
                    }
                }
            }
        }
    }
}