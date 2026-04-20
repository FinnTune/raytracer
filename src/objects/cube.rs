use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub struct Cube {
    pub center:      Vector3<f64>,
    pub size:        f64,
    pub material_id: usize,
    // Pre-computed — avoids recalculating on every hit test
    min: Vector3<f64>,
    max: Vector3<f64>,
}

impl Cube {
    pub fn new(center: Vector3<f64>, size: f64, material_id: usize) -> Self {
        let half = size / 2.0;
        let min  = center - Vector3::new(half, half, half);
        let max  = center + Vector3::new(half, half, half);
        Self { center, size, material_id, min, max }
    }

    fn normal_at(&self, point: Vector3<f64>) -> Vector3<f64> {
        // Find which face was hit by seeing which component is closest to a bound
        let local   = point - self.center;
        let half    = self.size / 2.0;
        let epsilon = 1e-4 * self.size;

        if (local.x.abs() - half).abs() < epsilon {
            Vector3::new(local.x.signum(), 0.0, 0.0)
        } else if (local.y.abs() - half).abs() < epsilon {
            Vector3::new(0.0, local.y.signum(), 0.0)
        } else {
            Vector3::new(0.0, 0.0, local.z.signum())
        }
    }
}

impl Hittable for Cube {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Slab method — intersect ray with three pairs of parallel planes
        let mut t_near = t_min;
        let mut t_far  = t_max;

        for axis in 0..3 {
            let inv_d = 1.0 / ray.direction[axis];
            let mut t0 = (self.min[axis] - ray.origin[axis]) * inv_d;
            let mut t1 = (self.max[axis] - ray.origin[axis]) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            t_near = t_near.max(t0);
            t_far  = t_far.min(t1);

            if t_far < t_near {
                return None;
            }
        }

        // Pick the entry point (t_near) if it is in range, else the exit (t_far)
        let t = if t_near >= t_min { t_near } else { t_far };
        if t < t_min || t > t_max {
            return None;
        }

        let hit_point      = ray.at(t);
        let outward_normal = self.normal_at(hit_point);

        Some(HitRecord::new(hit_point, outward_normal, t, ray, self.material_id))
    }

    fn bounding_box(&self) -> Aabb {
        Aabb::new(self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ray(origin: Vector3<f64>, dir: Vector3<f64>) -> Ray {
        Ray::new(origin, dir)
    }

    #[test]
    fn ray_hits_cube_head_on() {
        let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
        let ray  = make_ray(
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        assert!(cube.hit(&ray, 1e-4, f64::MAX).is_some());
    }

    #[test]
    fn ray_misses_cube() {
        let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
        let ray  = make_ray(
            Vector3::new(10.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        assert!(cube.hit(&ray, 1e-4, f64::MAX).is_none());
    }

    #[test]
    fn hit_normal_points_toward_ray() {
        let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
        let ray  = make_ray(
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        let hit = cube.hit(&ray, 1e-4, f64::MAX).unwrap();
        // Front face normal should point toward +Z (toward the camera)
        assert!(hit.normal.z > 0.0);
    }

    #[test]
    fn ray_hits_cube_from_below() {
        let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
        let ray  = make_ray(
            Vector3::new(0.0, -5.0, 0.0),
            Vector3::new(0.0,  1.0, 0.0),
        );
        let hit = cube.hit(&ray, 1e-4, f64::MAX).unwrap();
        assert!(hit.normal.y < 0.0); // Bottom face, normal points down
    }
}