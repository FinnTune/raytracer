use clap::Parser;
use nalgebra::Vector3;
use rt::renderer::CameraBuilder;
use rt::scene_file::{default_scene_objects, CameraSettings, SceneFile};
use std::sync::{atomic::AtomicU64, Arc};

/// Renders a scene from the command line, or launches the GUI.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Render headlessly instead of launching the GUI
    #[arg(long)]
    no_gui: bool,

    /// Path to a scene JSON file (as saved from the GUI). Renders the
    /// built-in demo scene if not given.
    #[arg(long)]
    scene: Option<String>,

    /// Output image width in pixels
    #[arg(long, default_value_t = 600)]
    width: u32,

    /// Output image height in pixels
    #[arg(long, default_value_t = 400)]
    height: u32,

    /// Rays per pixel — more samples means less noise, longer render
    #[arg(long, default_value_t = 128)]
    samples: u32,

    /// Maximum ray bounces
    #[arg(long, default_value_t = 32)]
    depth: u32,
}

fn main() {
    let cli = Cli::parse();
    if cli.no_gui {
        headless(&cli);
    } else {
        rt::gui::launch();
    }
}

fn headless(cli: &Cli) {
    let (camera_settings, objects) = match &cli.scene {
        Some(path) => {
            let file = SceneFile::load(path).unwrap_or_else(|e| {
                eprintln!("Failed to load scene from {path}: {e}");
                std::process::exit(1);
            });
            (file.camera, file.objects)
        }
        None => (CameraSettings::default(), default_scene_objects()),
    };

    let mut scene = rt::scene_file::build_scene(&objects);
    let bvh = scene.build_bvh();

    let width = cli.width;
    let height = cli.height;
    let samples = cli.samples;
    let depth = cli.depth;

    let camera = CameraBuilder::new()
        .position(Vector3::new(
            camera_settings.position[0] as f64,
            camera_settings.position[1] as f64,
            camera_settings.position[2] as f64,
        ))
        .look_at(Vector3::new(
            camera_settings.look_at[0] as f64,
            camera_settings.look_at[1] as f64,
            camera_settings.look_at[2] as f64,
        ))
        .fov(camera_settings.fov as f64)
        .resolution(width, height)
        .build();

    println!("Rendering {width}x{height} — {samples} samples, depth {depth}...");
    let start = std::time::Instant::now();
    let progress = Arc::new(AtomicU64::new(0));
    let pixels = camera.render(&scene, &bvh, samples, depth, progress);
    let pixels = rt::renderer::camera::denoise(&pixels, width, height);
    println!("Done in {:.2?}", start.elapsed());

    camera.write_to_ppm("output.ppm", &pixels);
    camera.write_to_png("output.png", &pixels);
    println!("Written to output.ppm and output.png");
}
