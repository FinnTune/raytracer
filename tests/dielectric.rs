use nalgebra::Vector3;
use rt::materials::dielectric::{reflectance, refract};
use rt::materials::{reflect, Dielectric, Material};
use rt::objects::HitRecord;
use rt::renderer::ray::Ray;
use rt::renderer::Color;

#[test]
fn refract_at_normal_incidence_does_not_bend() {
    // Straight through the boundary, refraction should not change direction
    // regardless of the index-of-refraction ratio.
    let incoming = Vector3::new(0.0, 0.0, -1.0);
    let normal = Vector3::new(0.0, 0.0, 1.0);

    let refracted = refract(incoming, normal, 1.0 / 1.5);

    assert!((refracted - incoming).norm() < 1e-9, "got {refracted:?}");
}

#[test]
fn reflectance_is_low_head_on_and_full_at_grazing() {
    let ratio = 1.0 / 1.5;
    let head_on = reflectance(1.0, ratio); // cos(theta) = 1 -> straight on
    let grazing = reflectance(0.0, ratio); // cos(theta) = 0 -> perpendicular graze

    assert!(
        head_on < 0.1,
        "expected low reflectance head-on, got {head_on}"
    );
    assert!(
        (grazing - 1.0).abs() < 1e-9,
        "expected full reflectance at grazing incidence, got {grazing}"
    );
}

#[test]
fn total_internal_reflection_always_reflects() {
    // A ray inside glass (ior 1.5) hitting the boundary at 60 degrees from
    // the normal exceeds the ~41.8 degree critical angle, so it must
    // reflect deterministically — the probabilistic Schlick branch is
    // short-circuited by `cannot_refract`, so this needs no RNG seeding.
    let glass = Dielectric::new(Color::WHITE, 1.5);

    let angle: f64 = 60.0_f64.to_radians();
    let direction = Vector3::new(angle.sin(), 0.0, angle.cos());
    let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), direction);
    let outward_normal = Vector3::new(0.0, 0.0, 1.0);
    let hit = HitRecord::new(Vector3::new(0.0, 0.0, 0.0), outward_normal, 1.0, &ray, 0);

    assert!(
        !hit.front_face,
        "test setup should model a ray exiting the medium"
    );

    let scatter = glass
        .scatter(&ray, &hit)
        .expect("dielectric always scatters");
    let expected = reflect(ray.direction, hit.normal).normalize();

    assert!(
        (scatter.ray.direction - expected).norm() < 1e-9,
        "expected reflection {expected:?}, got {:?}",
        scatter.ray.direction
    );
}
