mod common;

use nalgebra::Vector3;
use rt::objects::{Hittable, Plane};

use common::make_ray;

#[test]
fn ray_hits_plane_from_above() {
    let plane = Plane::new(Vector3::new(0.0, 0.0, 0.0), 10.0, 0);
    let ray = make_ray(Vector3::new(0.0, 5.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
    let hit = plane.hit(&ray, 1e-4, f64::MAX);
    assert!(hit.is_some());
    assert!((hit.unwrap().t - 5.0).abs() < 1e-6);
}

#[test]
fn ray_misses_plane_outside_radius() {
    let plane = Plane::new(Vector3::new(0.0, 0.0, 0.0), 1.0, 0);
    let ray = make_ray(Vector3::new(50.0, 5.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
    assert!(plane.hit(&ray, 1e-4, f64::MAX).is_none());
}

#[test]
fn ray_parallel_to_plane_misses() {
    let plane = Plane::new(Vector3::new(0.0, 0.0, 0.0), 10.0, 0);
    let ray = make_ray(Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
    assert!(plane.hit(&ray, 1e-4, f64::MAX).is_none());
}
