use std::sync::Arc;

use crate::objects::{Aabb, HitRecord, Hittable};
use crate::renderer::ray::Ray;

pub struct BvhNode {
    left:  Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox:  Aabb,
}

impl BvhNode {
    /// Build a BVH from a slice of objects.
    ///
    /// Sorts along the longest axis of the combined AABB, splits in half,
    /// and recurses. Single objects become a leaf by cloning the Arc.
    pub fn build(objects: &mut [Arc<dyn Hittable>]) -> Arc<dyn Hittable> {
        match objects.len() {
            0 => panic!("BvhNode::build called with empty slice"),

            // Single object — return it directly, no node needed
            1 => Arc::clone(&objects[0]),

            // Two objects — leaf node with both children
            2 => {
                let left  = Arc::clone(&objects[0]);
                let right = Arc::clone(&objects[1]);
                let bbox  = Aabb::surrounding(left.bounding_box(), right.bounding_box());
                Arc::new(BvhNode { left, right, bbox })
            }

            // Three or more — sort along longest axis, split at midpoint
            _ => {
                let combined = objects
                    .iter()
                    .map(|o| o.bounding_box())
                    .reduce(Aabb::surrounding)
                    .unwrap();

                let axis = longest_axis(&combined);

                objects.sort_by(|a, b| {
                    let a_min = a.bounding_box().min[axis];
                    let b_min = b.bounding_box().min[axis];
                    a_min.partial_cmp(&b_min).unwrap()
                });

                let mid = objects.len() / 2;
                let (left_slice, right_slice) = objects.split_at_mut(mid);

                let left  = BvhNode::build(left_slice);
                let right = BvhNode::build(right_slice);
                let bbox  = Aabb::surrounding(left.bounding_box(), right.bounding_box());

                Arc::new(BvhNode { left, right, bbox })
            }
        }
    }
}

/// Returns 0, 1, or 2 for the x, y, or z axis that spans the most distance.
fn longest_axis(bbox: &Aabb) -> usize {
    let extent = bbox.max - bbox.min;
    if extent.x > extent.y && extent.x > extent.z {
        0 // x
    } else if extent.y > extent.z {
        1 // y
    } else {
        2 // z
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // If ray misses this node's box, skip both children entirely
        if !self.bbox.hit(ray, t_min, t_max) {
            return None;
        }

        // Test left child first
        let left_hit = self.left.hit(ray, t_min, t_max);

        // For the right child, tighten t_max to the left hit distance if we
        // already have a hit — no point finding something farther away
        let t_max_right = left_hit
            .as_ref()
            .map(|h| h.t)
            .unwrap_or(t_max);

        let right_hit = self.right.hit(ray, t_min, t_max_right);

        // Return whichever hit is closer (right_hit wins on tie since it used
        // a tighter t_max, so if it returned Some it is strictly closer)
        right_hit.or(left_hit)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Sphere;
    use nalgebra::Vector3;

    fn sphere(x: f64) -> Arc<dyn Hittable> {
        Arc::new(Sphere::new(Vector3::new(x, 0.0, 0.0), 0.5, 0))
    }

    fn make_ray(origin: Vector3<f64>, dir: Vector3<f64>) -> Ray {
        Ray::new(origin, dir)
    }

    #[test]
    fn bvh_hits_sphere_through_tree() {
        let mut objects: Vec<Arc<dyn Hittable>> = vec![
            sphere(-2.0),
            sphere(0.0),
            sphere(2.0),
        ];
        let bvh = BvhNode::build(&mut objects);

        // Ray aimed at the centre sphere
        let ray = make_ray(
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        assert!(bvh.hit(&ray, 0.001, f64::MAX).is_some());
    }

    #[test]
    fn bvh_misses_when_ray_avoids_all() {
        let mut objects: Vec<Arc<dyn Hittable>> = vec![
            sphere(-2.0),
            sphere(0.0),
            sphere(2.0),
        ];
        let bvh = BvhNode::build(&mut objects);

        // Ray aimed far to the side
        let ray = make_ray(
            Vector3::new(100.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        assert!(bvh.hit(&ray, 0.001, f64::MAX).is_none());
    }

    #[test]
    fn bvh_returns_closest_hit() {
        let mut objects: Vec<Arc<dyn Hittable>> = vec![
            sphere(0.0),   // at z=0, closer
            sphere(-3.0),  // at z=-3, farther
        ];
        let bvh = BvhNode::build(&mut objects);

        let ray = make_ray(
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        let hit = bvh.hit(&ray, 0.001, f64::MAX).unwrap();
        // The closer sphere (z=0) surface is at roughly z=0.5, so t ≈ 4.5
        assert!(hit.t < 5.0);
    }

    #[test]
    fn bvh_single_object() {
        let mut objects: Vec<Arc<dyn Hittable>> = vec![sphere(0.0)];
        let bvh = BvhNode::build(&mut objects);

        let ray = make_ray(
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        assert!(bvh.hit(&ray, 0.001, f64::MAX).is_some());
    }
}