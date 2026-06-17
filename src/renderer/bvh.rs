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