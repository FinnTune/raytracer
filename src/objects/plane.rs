use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub struct Plane {
    pub center:      Vector3<f64>,
    pub radius:      f64,
    pub material_id: usize,
}

impl Plane {
    pub fn new(center: Vector3<f64>, radius: f64, material_id: usize) -> Self {
        Self { center, radius, material_id }
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // A horizontal plane has a fixed upward normal
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let denom  = ray.direction.dot(&normal);

        // Ray is parallel to the plane — no intersection
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.center - ray.origin).dot(&normal) / denom;
        if t < t_min || t > t_max {
            return None;
        }

        let hit_point = ray.at(t);

        // Circular plane — reject hits outside the radius
        let dx = hit_point.x - self.center.x;
        let dz = hit_point.z - self.center.z;
        if dx * dx + dz * dz > self.radius * self.radius {
            return None;
        }

        Some(HitRecord::new(hit_point, normal, t, ray, self.material_id))
    }

    fn bounding_box(&self) -> Aabb {
        // Give the plane a tiny Y thickness so the AABB is never degenerate
        let pad = 1e-4;
        Aabb::new(
            Vector3::new(self.center.x - self.radius, self.center.y - pad, self.center.z - self.radius),
            Vector3::new(self.center.x + self.radius, self.center.y + pad, self.center.z + self.radius),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ray(origin: Vector3<f64>, dir: Vector3<f64>) -> Ray {
        Ray::new(origin, dir)
    }

    #[test]
    fn ray_hits_plane_from_above() {
        let plane = Plane::new(Vector3::new(0.0, 0.0, 0.0), 10.0, 0);
        let ray   = make_ray(
            Vector3::new(0.0, 5.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        let hit = plane.hit(&ray, 1e-4, f64::MAX);
        assert!(hit.is_some());
        assert!((hit.unwrap().t - 5.0).abs() < 1e-6);
    }

    #[test]
    fn ray_misses_plane_outside_radius() {
        let plane = Plane::new(Vector3::new(0.0, 0.0, 0.0), 1.0, 0);
        let ray   = make_ray(
            Vector3::new(50.0, 5.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        assert!(plane.hit(&ray, 1e-4, f64::MAX).is_none());
    }

    #[test]
    fn ray_parallel_to_plane_misses() {
        let plane = Plane::new(Vector3::new(0.0, 0.0, 0.0), 10.0, 0);
        let ray   = make_ray(
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        );
        assert!(plane.hit(&ray, 1e-4, f64::MAX).is_none());
    }
}