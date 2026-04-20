use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub struct Plane {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub material_id: usize,
}

impl Plane {
    pub fn new(center: Vector3<f64>, radius: f64, material_id: usize) -> Self {
        Self { center, radius, material_id }
    }
}

impl Hittable for Plane {
    fn hit(&self, _ray: &Ray, _t_min: f64, _t_max: f64) -> Option<HitRecord> {
        todo!()
    }
    fn bounding_box(&self) -> Aabb {
        todo!()
    }
}