use crate::materials::Material;
use crate::objects::{HitRecord, Hittable};
use crate::renderer::{bvh::BvhNode, ray::Ray, Color};
use std::sync::Arc;

pub struct Scene {
    pub objects:   Vec<Arc<dyn Hittable>>,
    pub materials: Vec<Arc<dyn Material>>,
    pub background: Color,
}

impl Scene {
    pub fn new(background: Color) -> Self {
        Self {
            objects:   Vec::new(),
            materials: Vec::new(),
            background,
        }
    }

    pub fn add_material(&mut self, material: impl Material + 'static) -> usize {
        self.materials.push(Arc::new(material));
        self.materials.len() - 1
    }

    pub fn add_object(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Arc::new(object));
    }

    /// Build the BVH once and return it. Call this after all objects are added.
    pub fn build_bvh(&mut self) -> Arc<dyn Hittable> {
        assert!(!self.objects.is_empty(), "Scene has no objects");
        BvhNode::build(&mut self.objects)
    }

    /// Trace using a pre-built BVH root (fast path)
    pub fn trace_bvh(
        &self,
        bvh: &Arc<dyn Hittable>,
        ray: &Ray,
        depth: u32,
    ) -> Color {
        if depth == 0 {
            return Color::BLACK;
        }

        match bvh.hit(ray, 1e-4, f64::MAX) {
            None => self.background,
            Some(hit) => {
                let material = &self.materials[hit.material_id];
                let emitted  = material.emitted();
                match material.scatter(ray, &hit) {
                    None => emitted,
                    Some(scatter) => {
                        let incoming = self.trace_bvh(bvh, &scatter.ray, depth - 1);
                        emitted + incoming.attenuate(scatter.attenuation)
                    }
                }
            }
        }
    }

    /// Linear scan fallback — still useful for small scenes and testing
    pub fn trace(&self, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::BLACK;
        }

        let mut closest    = f64::MAX;
        let mut result_hit = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, 1e-4, closest) {
                closest    = hit.t;
                result_hit = Some(hit);
            }
        }

        match result_hit {
            None => self.background,
            Some(hit) => {
                let material = &self.materials[hit.material_id];
                let emitted  = material.emitted();
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