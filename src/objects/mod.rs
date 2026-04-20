use crate::renderer::ray::Ray;
use nalgebra::Vector3;

pub mod sphere;
pub mod cube;
pub mod cylinder;
pub mod plane;

pub use sphere::Sphere;
pub use cube::Cube;
pub use cylinder::Cylinder;
pub use plane::Plane;

pub type Vec3 = Vector3<f64>;

/// Everything the renderer needs to know about a ray–object intersection
#[derive(Debug, Clone)]
pub struct HitRecord {
    /// Point in world space where the ray hit
    pub point: Vec3,
    /// Surface normal, always pointing against the incoming ray
    pub normal: Vec3,
    /// Distance along the ray
    pub t: f64,
    /// Whether the ray hit the front face
    pub front_face: bool,
    /// Index into the scene's material list
    pub material_id: usize,
}

impl HitRecord {
    pub fn new(
        point: Vec3,
        outward_normal: Vec3,
        t: f64,
        ray: &Ray,
        material_id: usize,
    ) -> Self {
        let front_face = ray.direction.dot(&outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self { point, normal, t, front_face, material_id }
    }
}

/// Every geometric primitive implements this
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    /// Axis-aligned bounding box — needed for BVH
    fn bounding_box(&self) -> Aabb;
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for axis in 0..3 {
            let inv_d = 1.0 / ray.direction[axis];
            let mut t0 = (self.min[axis] - ray.origin[axis]) * inv_d;
            let mut t1 = (self.max[axis] - ray.origin[axis]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_min = t_min.max(t0);
            t_max = t_max.min(t1);
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    /// Merge two AABBs into one that contains both
    pub fn surrounding(a: Aabb, b: Aabb) -> Aabb {
        let min = Vec3::new(
            a.min.x.min(b.min.x),
            a.min.y.min(b.min.y),
            a.min.z.min(b.min.z),
        );
        let max = Vec3::new(
            a.max.x.max(b.max.x),
            a.max.y.max(b.max.y),
            a.max.z.max(b.max.z),
        );
        Aabb::new(min, max)
    }
}