use nalgebra::Vector3;
use rt::renderer::ray::Ray;

pub fn make_ray(origin: Vector3<f64>, dir: Vector3<f64>) -> Ray {
    Ray::new(origin, dir)
}
