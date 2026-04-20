use nalgebra::Vector3;
use rt::materials::{Diffuse, Emissive, Reflective};
use rt::objects::{Cube, Cylinder, Plane, Sphere};
use rt::renderer::{CameraBuilder, Color, Scene};

fn main() {
    let mut scene = Scene::new(Color::new(0.05, 0.07, 0.12));

    let grey   = scene.add_material(Diffuse::new(Color::new(0.5, 0.5, 0.5)));
    let red    = scene.add_material(Diffuse::new(Color::new(0.8, 0.2, 0.2)));
    let green  = scene.add_material(Diffuse::new(Color::new(0.2, 0.7, 0.3)));
    let blue   = scene.add_material(Diffuse::new(Color::new(0.2, 0.3, 0.9)));
    let mirror = scene.add_material(Reflective::new(Color::new(0.8, 0.8, 0.8), 0.05));
    let light  = scene.add_material(Emissive::new(Color::WHITE, 5.0));

    scene.add_object(Plane::new(Vector3::new(0.0, -0.5, 0.0), 20.0, grey));
    scene.add_object(Sphere::new(Vector3::new(-1.8, 0.0, 0.0), 0.5, red));
    scene.add_object(Cube::new(Vector3::new(0.0, 0.0, 0.0), 0.8, mirror));
    scene.add_object(Cylinder::new(Vector3::new(1.8, -0.5, 0.0), 0.4, 1.0, blue));
    scene.add_object(Sphere::new(Vector3::new(0.0, 0.7, 0.0), 0.2, green));
    scene.add_object(Sphere::new(Vector3::new(0.0, 4.0, 1.0), 0.8, light));

    let bvh = scene.build_bvh();

    let width   = 600u32;
    let height  = 400u32;
    let samples = 128u32;
    let depth   = 32u32;

    let camera = CameraBuilder::new()
        .position(Vector3::new(0.0, 1.5, 6.0))
        .look_at(Vector3::new(0.0, 0.0, 0.0))
        .fov(45.0)
        .resolution(width, height)
        .build();

    println!("Rendering {width}x{height} — {samples} samples, depth {depth}...");
    let start  = std::time::Instant::now();
    let pixels = camera.render(&scene, &bvh, width, height, samples, depth);
    println!("Done in {:.2?}", start.elapsed());

    camera.write_to_ppm("output.ppm", &pixels);
    println!("Written to output.ppm");

    camera.write_to_png("output.png", &pixels);
    println!("Written to output.png");
}