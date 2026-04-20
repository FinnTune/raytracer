use nalgebra::Vector3;
use rt::materials::{Diffuse, Emissive, Reflective};
use rt::objects::Sphere;
use rt::renderer::{CameraBuilder, Color, Scene};

fn main() {
    // --- Scene ---
    let mut scene = Scene::new(Color::new(0.05, 0.07, 0.12)); // dark blue sky

    let ground   = scene.add_material(Diffuse::new(Color::new(0.5, 0.5, 0.5)));
    let red      = scene.add_material(Diffuse::new(Color::new(0.8, 0.2, 0.2)));
    let mirror   = scene.add_material(Reflective::new(Color::new(0.8, 0.8, 0.8), 0.05));
    let light    = scene.add_material(Emissive::new(Color::WHITE, 4.0));

    scene.add_object(Sphere::new(Vector3::new( 0.0, -100.5, -1.0), 100.0, ground));
    scene.add_object(Sphere::new(Vector3::new( 0.0,    0.0, -1.0),   0.5, red));
    scene.add_object(Sphere::new(Vector3::new( 1.2,    0.0, -1.0),   0.5, mirror));
    scene.add_object(Sphere::new(Vector3::new( 0.0,    2.0, -1.0),   0.5, light));

    // --- Camera ---
    let width   = 400u32;
    let height  = 225u32;
    let samples = 64u32;
    let depth   = 32u32;

    let camera = CameraBuilder::new()
        .position(Vector3::new(0.0, 0.5, 3.0))
        .look_at(Vector3::new(0.0, 0.0, -1.0))
        .fov(60.0)
        .resolution(width, height)
        .build();

    println!("Rendering {width}x{height} — {samples} samples, depth {depth}...");
    let start  = std::time::Instant::now();
    let pixels = camera.render(&scene, width, height, samples, depth);
    println!("Done in {:.2?}", start.elapsed());

    // --- Write PPM ---
    use std::io::Write;
    let mut f = std::fs::File::create("output.ppm").unwrap();
    writeln!(f, "P3\n{width} {height}\n255").unwrap();
    for color in &pixels {
        let (r, g, b) = color.to_rgb_u8(2.2);
        writeln!(f, "{r} {g} {b}").unwrap();
    }
    println!("Written to output.ppm");
}