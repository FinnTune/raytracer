use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub struct Cylinder {
    pub center:      Vector3<f64>, // centre of the bottom cap
    pub radius:      f64,
    pub height:      f64,
    pub material_id: usize,
}

impl Cylinder {
    pub fn new(
        center:      Vector3<f64>,
        radius:      f64,
        height:      f64,
        material_id: usize,
    ) -> Self {
        Self { center, radius, height, material_id }
    }

    fn top(&self) -> f64 {
        self.center.y + self.height
    }

    /// Normal on the curved surface — points radially outward from the Y axis
    fn curved_normal(&self, point: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            point.x - self.center.x,
            0.0,
            point.z - self.center.z,
        )
        .normalize()
    }

}

/// Returns t for a ray hitting a horizontal disc at y=cap_y, or None on miss.
fn cap_hit(
    ray:    &Ray,
    cap_y:  f64,
    center: Vector3<f64>,
    radius: f64,
    t_min:  f64,
    t_max:  f64,
) -> Option<f64> {
    let denom = ray.direction.y;
    if denom.abs() < 1e-8 { return None; }
    let t     = (cap_y - ray.origin.y) / denom;
    if t < t_min || t > t_max { return None; }
    let point = ray.at(t);
    let dx    = point.x - center.x;
    let dz    = point.z - center.z;
    if dx * dx + dz * dz <= radius * radius { Some(t) } else { None }
}

impl Hittable for Cylinder {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut best: Option<HitRecord> = None;
        let mut best_t = t_max;

        // --- Curved surface ---
        let ox = ray.origin.x - self.center.x;
        let oz = ray.origin.z - self.center.z;
        let dx = ray.direction.x;
        let dz = ray.direction.z;

        let a = dx * dx + dz * dz;
        let b = 2.0 * (ox * dx + oz * dz);
        let c = ox * ox + oz * oz - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant >= 0.0 {
            let sqrt_d = discriminant.sqrt();
            for &t in &[(-b - sqrt_d) / (2.0 * a), (-b + sqrt_d) / (2.0 * a)] {
                if t < t_min || t > best_t { continue; }
                let point = ray.at(t);
                if point.y < self.center.y || point.y > self.top() { continue; }
                let normal = self.curved_normal(point);
                best   = Some(HitRecord::new(point, normal, t, ray, self.material_id));
                best_t = t;
            }
        }

        // --- Bottom cap ---
        if let Some(t) = cap_hit(ray, self.center.y, self.center, self.radius, t_min, best_t) {
            let point  = ray.at(t);
            best   = Some(HitRecord::new(point, Vector3::new(0.0, -1.0, 0.0), t, ray, self.material_id));
            best_t = t;
        }

        // --- Top cap ---
        if let Some(t) = cap_hit(ray, self.top(), self.center, self.radius, t_min, best_t) {
            let point = ray.at(t);
            best = Some(HitRecord::new(point, Vector3::new(0.0, 1.0, 0.0), t, ray, self.material_id));
        }

        best
    }

    fn bounding_box(&self) -> Aabb {
        Aabb::new(
            Vector3::new(
                self.center.x - self.radius,
                self.center.y,
                self.center.z - self.radius,
            ),
            Vector3::new(
                self.center.x + self.radius,
                self.top(),
                self.center.z + self.radius,
            ),
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
    fn ray_hits_curved_surface() {
        let cyl = Cylinder::new(Vector3::new(0.0, -1.0, 0.0), 0.5, 2.0, 0);
        let ray = make_ray(
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        assert!(cyl.hit(&ray, 1e-4, f64::MAX).is_some());
    }

    #[test]
    fn ray_hits_top_cap() {
        let cyl = Cylinder::new(Vector3::new(0.0, 0.0, 0.0), 1.0, 2.0, 0);
        let ray = make_ray(
            Vector3::new(0.0, 10.0, 0.0),
            Vector3::new(0.0, -1.0,  0.0),
        );
        let hit = cyl.hit(&ray, 1e-4, f64::MAX).unwrap();
        assert!(hit.normal.y > 0.0); // Top cap normal faces up
    }

    #[test]
    fn ray_hits_bottom_cap() {
        let cyl = Cylinder::new(Vector3::new(0.0, 0.0, 0.0), 1.0, 2.0, 0);
        let ray = make_ray(
            Vector3::new(0.0, -5.0, 0.0),
            Vector3::new(0.0,  1.0,  0.0),
        );
        let hit = cyl.hit(&ray, 1e-4, f64::MAX).unwrap();
        assert!(hit.normal.y < 0.0); // Bottom cap normal faces down
    }

    #[test]
    fn ray_misses_cylinder() {
        let cyl = Cylinder::new(Vector3::new(0.0, 0.0, 0.0), 0.5, 2.0, 0);
        let ray = make_ray(
            Vector3::new(10.0, 0.0, 5.0),
            Vector3::new( 0.0, 0.0, -1.0),
        );
        assert!(cyl.hit(&ray, 1e-4, f64::MAX).is_none());
    }
}