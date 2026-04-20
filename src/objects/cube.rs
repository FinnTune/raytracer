use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub struct Cube {
    pub center: Vector3<f64>,
    pub size: f64,
    pub material_id: usize,
}

impl Cube {
    pub fn new(center: Vector3<f64>, size: f64, material_id: usize) -> Self {
        Self { center, size, material_id }
    }
}

impl Hittable for Cube {
    fn hit(&self, _ray: &Ray, _t_min: f64, _t_max: f64) -> Option<HitRecord> {
        todo!()
    }
    fn bounding_box(&self) -> Aabb {
        todo!()
    }
}