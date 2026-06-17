use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub material_id: usize,
}

impl Sphere {
    pub fn new(center: Vector3<f64>, radius: f64, material_id: usize) -> Self {
        Self { center, radius, material_id }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.norm_squared();
        let half_b = oc.dot(&ray.direction);
        let c = oc.norm_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();

        // Find the nearest root in [t_min, t_max]
        let mut t = (-half_b - sqrt_d) / a;
        if t < t_min || t > t_max {
            t = (-half_b + sqrt_d) / a;
            if t < t_min || t > t_max {
                return None;
            }
        }

        let point = ray.at(t);
        let outward_normal = (point - self.center) / self.radius;

        Some(HitRecord::new(point, outward_normal, t, ray, self.material_id))
    }

    fn bounding_box(&self) -> Aabb {
        let r = Vector3::new(self.radius, self.radius, self.radius);
        Aabb::new(self.center - r, self.center + r)
    }
}