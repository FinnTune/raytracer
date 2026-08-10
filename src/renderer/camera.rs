use crate::renderer::{ray::Ray, Color};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use image;
use nalgebra::Vector3;
use rand::RngExt;

pub struct Camera {
    pub position: Vector3<f64>,
    pub look_at: Vector3<f64>,
    pub up: Vector3<f64>,
    pub fov_deg: f64,
    pub aspect: f64,
    pub width: u32,
    pub height: u32,
    lower_left: Vector3<f64>,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
}

pub struct CameraBuilder {
    position: Vector3<f64>,
    look_at: Vector3<f64>,
    up: Vector3<f64>,
    fov_deg: f64,
    width: u32,
    height: u32,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 3.0),
            look_at: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fov_deg: 60.0,
            width: 800,
            height: 600,
        }
    }

    pub fn position(mut self, p: Vector3<f64>) -> Self {
        self.position = p;
        self
    }
    pub fn look_at(mut self, p: Vector3<f64>) -> Self {
        self.look_at = p;
        self
    }
    pub fn fov(mut self, deg: f64) -> Self {
        self.fov_deg = deg;
        self
    }
    pub fn resolution(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    pub fn build(self) -> Camera {
        let aspect = self.width as f64 / self.height as f64;
        let theta = self.fov_deg.to_radians();
        let h = (theta / 2.0).tan();

        let viewport_height = 2.0 * h;
        let viewport_width = aspect * viewport_height;

        let w = (self.position - self.look_at).normalize();
        let u = self.up.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal = u * viewport_width;
        let vertical = v * viewport_height;
        let lower_left = self.position - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            position: self.position,
            look_at: self.look_at,
            up: self.up,
            fov_deg: self.fov_deg,
            aspect,
            width: self.width,
            height: self.height,
            lower_left,
            horizontal,
            vertical,
        }
    }
}

/// Map a pixel coordinate plus per-sample jitter to normalized (u, v) in [0, 1].
///
/// A dimension of 1 has no span to normalize against — dividing by
/// `dimension - 1` would divide by zero — so in that case the jitter alone
/// (already in [0, 1)) is used directly.
pub fn pixel_uv(
    col: u32,
    row: u32,
    width: u32,
    height: u32,
    jitter_u: f64,
    jitter_v: f64,
) -> (f64, f64) {
    let u_denom = if width > 1 { (width - 1) as f64 } else { 1.0 };
    let v_denom = if height > 1 { (height - 1) as f64 } else { 1.0 };
    let u = (col as f64 + jitter_u) / u_denom;
    let v = (row as f64 + jitter_v) / v_denom;
    (u, v)
}

impl Camera {
    /// Generate a ray through pixel (u, v) where both are in [0, 1]
    pub fn ray(&self, u: f64, v: f64) -> Ray {
        let direction = self.lower_left + self.horizontal * u + self.vertical * v - self.position;
        Ray::new(self.position, direction)
    }

    /// Render the full scene to a flat Vec<Color>, row-major top-to-bottom
    pub fn render(
        &self,
        scene: &crate::renderer::scene::Scene,
        bvh: &Arc<dyn crate::objects::Hittable>,
        samples: u32,
        depth: u32,
        progress: Arc<AtomicU64>,
    ) -> Vec<Color> {
        use rayon::prelude::*;

        let (width, height) = (self.width, self.height);

        (0..height)
            .into_par_iter()
            .rev()
            .flat_map_iter(|row| (0..width).map(move |col| (row, col)))
            .map(|(row, col)| {
                let mut rng = rand::rng();
                let mut color = Color::BLACK;

                for _ in 0..samples {
                    let (u, v) = pixel_uv(col, row, width, height, rng.random(), rng.random());
                    color += scene.trace_bvh(bvh, &self.ray(u, v), depth);
                }

                progress.fetch_add(1, Ordering::Relaxed);
                color * (1.0 / samples as f64)
            })
            .collect()
    }

    pub fn write_to_ppm(&self, path: &str, pixels: &[Color]) {
        use std::io::Write;
        let (width, height) = (self.width, self.height);
        let mut f = std::fs::File::create(path).unwrap();
        writeln!(f, "P3\n{width} {height}\n255").unwrap();
        for color in pixels {
            let (r, g, b) = color.to_rgb_u8(2.2);
            writeln!(f, "{r} {g} {b}").unwrap();
        }
    }

    pub fn write_to_png(&self, path: &str, pixels: &[Color]) {
        let (width, height) = (self.width, self.height);
        let mut img = image::RgbImage::new(width, height);

        for (i, color) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            let (r, g, b) = color.to_rgb_u8(2.2);
            img.put_pixel(x, y, image::Rgb([r, g, b]));
        }

        img.save(path).expect("Failed to write PNG");
    }
}

pub fn denoise(pixels: &[Color], width: u32, height: u32) -> Vec<Color> {
    let w = width as usize;
    let h = height as usize;
    let radius = 1usize; // 3x3 neighbourhood
    let sigma_s = 1.0f64; // spatial falloff
    let sigma_r = 0.1f64; // color similarity falloff

    let mut output = vec![Color::BLACK; pixels.len()];

    for y in 0..h {
        for x in 0..w {
            let center = pixels[y * w + x];
            let mut total_weight = 0.0f64;
            let mut total_color = Color::BLACK;

            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    // Skip out-of-bounds neighbours
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }

                    let neighbour = pixels[ny as usize * w + nx as usize];

                    // Spatial weight — prefer nearby pixels
                    let spatial = -(dx * dx + dy * dy) as f64 / (2.0 * sigma_s * sigma_s);

                    // Range weight — prefer similar-colored pixels
                    let dr = center.r - neighbour.r;
                    let dg = center.g - neighbour.g;
                    let db = center.b - neighbour.b;
                    let range = -(dr * dr + dg * dg + db * db) / (2.0 * sigma_r * sigma_r);

                    let weight = (spatial + range).exp();
                    total_weight += weight;
                    total_color += neighbour * weight;
                }
            }

            output[y * w + x] = total_color * (1.0 / total_weight);
        }
    }

    output
}
