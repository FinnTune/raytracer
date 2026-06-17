mod common;

use nalgebra::Vector3;
use rt::objects::{Cube, Hittable};

use common::make_ray;

#[test]
fn ray_hits_cube_head_on() {
    let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
    let ray = make_ray(Vector3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(cube.hit(&ray, 1e-4, f64::MAX).is_some());
}

#[test]
fn ray_misses_cube() {
    let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
    let ray = make_ray(Vector3::new(10.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    assert!(cube.hit(&ray, 1e-4, f64::MAX).is_none());
}

#[test]
fn hit_normal_points_toward_ray() {
    let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
    let ray = make_ray(Vector3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, -1.0));
    let hit = cube.hit(&ray, 1e-4, f64::MAX).unwrap();
    assert!(hit.normal.z > 0.0);
}

#[test]
fn ray_hits_cube_from_below() {
    let cube = Cube::new(Vector3::new(0.0, 0.0, 0.0), 2.0, 0);
    let ray = make_ray(Vector3::new(0.0, -5.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
    let hit = cube.hit(&ray, 1e-4, f64::MAX).unwrap();
    assert!(hit.normal.y < 0.0);
}
