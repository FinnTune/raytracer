mod common;

use nalgebra::Vector3;
use rt::objects::{Cylinder, Hittable};

use common::make_ray;

#[test]
fn ray_hits_curved_surface() {
    let cyl = Cylinder::new(Vector3::new(0.0, -1.0, 0.0), 0.5, 2.0, 0);
    let ray = make_ray(Vector3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(cyl.hit(&ray, 1e-4, f64::MAX).is_some());
}

#[test]
fn ray_hits_top_cap() {
    let cyl = Cylinder::new(Vector3::new(0.0, 0.0, 0.0), 1.0, 2.0, 0);
    let ray = make_ray(Vector3::new(0.0, 10.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
    let hit = cyl.hit(&ray, 1e-4, f64::MAX).unwrap();
    assert!(hit.normal.y > 0.0);
}

#[test]
fn ray_hits_bottom_cap() {
    let cyl = Cylinder::new(Vector3::new(0.0, 0.0, 0.0), 1.0, 2.0, 0);
    let ray = make_ray(Vector3::new(0.0, -5.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
    let hit = cyl.hit(&ray, 1e-4, f64::MAX).unwrap();
    assert!(hit.normal.y < 0.0);
}

#[test]
fn ray_misses_cylinder() {
    let cyl = Cylinder::new(Vector3::new(0.0, 0.0, 0.0), 0.5, 2.0, 0);
    let ray = make_ray(Vector3::new(10.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(cyl.hit(&ray, 1e-4, f64::MAX).is_none());
}
