mod common;

use std::sync::Arc;

use nalgebra::Vector3;
use rt::objects::{Hittable, Sphere};
use rt::renderer::BvhNode;

use common::make_ray;

fn sphere(x: f64) -> Arc<dyn Hittable> {
    Arc::new(Sphere::new(Vector3::new(x, 0.0, 0.0), 0.5, 0))
}

#[test]
fn bvh_hits_sphere_through_tree() {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![sphere(-2.0), sphere(0.0), sphere(2.0)];
    let bvh = BvhNode::build(&mut objects);

    let ray = make_ray(Vector3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(bvh.hit(&ray, 0.001, f64::MAX).is_some());
}

#[test]
fn bvh_misses_when_ray_avoids_all() {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![sphere(-2.0), sphere(0.0), sphere(2.0)];
    let bvh = BvhNode::build(&mut objects);

    let ray = make_ray(Vector3::new(100.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(bvh.hit(&ray, 0.001, f64::MAX).is_none());
}

#[test]
fn bvh_returns_closest_hit() {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![sphere(0.0), sphere(-3.0)];
    let bvh = BvhNode::build(&mut objects);

    let ray = make_ray(Vector3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    let hit = bvh.hit(&ray, 0.001, f64::MAX).unwrap();
    assert!(hit.t < 5.0);
}

#[test]
fn bvh_single_object() {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![sphere(0.0)];
    let bvh = BvhNode::build(&mut objects);

    let ray = make_ray(Vector3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(bvh.hit(&ray, 0.001, f64::MAX).is_some());
}
