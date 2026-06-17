mod common;

use nalgebra::Vector3;
use rt::objects::{Hittable, Sphere};

use common::make_ray;

#[test]
fn ray_hits_sphere() {
    let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5, 0);
    let ray = make_ray(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(sphere.hit(&ray, 0.001, f64::MAX).is_some());
}

#[test]
fn ray_misses_sphere() {
    let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5, 0);
    let ray = make_ray(Vector3::new(0.0, 10.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(sphere.hit(&ray, 0.001, f64::MAX).is_none());
}

#[test]
fn normal_points_outward_from_outside() {
    let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5, 0);
    let ray = make_ray(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
    let hit = sphere.hit(&ray, 0.001, f64::MAX).unwrap();
    assert!(hit.front_face);
    assert!(hit.normal.z > 0.0);
}
