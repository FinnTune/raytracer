use crate::renderer::{ray::Ray, Color};
use nalgebra::Vector3;
use rand::Rng;

pub struct Camera {
    pub position:   Vector3<f64>,
    pub look_at:    Vector3<f64>,
    pub up:         Vector3<f64>,
    pub fov_deg:    f64,
    pub aspect:     f64,
    // Derived — computed once in build()
    lower_left:     Vector3<f64>,
    horizontal:     Vector3<f64>,
    vertical:       Vector3<f64>,
}

pub struct CameraBuilder {
    position:  Vector3<f64>,
    look_at:   Vector3<f64>,
    up:        Vector3<f64>,
    fov_deg:   f64,
    width:     u32,
    height:    u32,
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0,  3.0),
            look_at:  Vector3::new(0.0, 0.0,  0.0),
            up:       Vector3::new(0.0, 1.0,  0.0),
            fov_deg:  60.0,
            width:    800,
            height:   600,
        }
    }

    pub fn position(mut self, p: Vector3<f64>) -> Self { self.position = p; self }
    pub fn look_at(mut self, p: Vector3<f64>)  -> Self { self.look_at  = p; self }
    pub fn fov(mut self, deg: f64)             -> Self { self.fov_deg  = deg; self }
    pub fn resolution(mut self, w: u32, h: u32) -> Self {
        self.width  = w;
        self.height = h;
        self
    }

    pub fn build(self) -> Camera {
        let aspect = self.width as f64 / self.height as f64;
        let theta  = self.fov_deg.to_radians();
        let h      = (theta / 2.0).tan();

        let viewport_height = 2.0 * h;
        let viewport_width  = aspect * viewport_height;

        let w = (self.position - self.look_at).normalize();
        let u = self.up.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal  = u * viewport_width;
        let vertical    = v * viewport_height;
        let lower_left  = self.position
            - horizontal / 2.0
            - vertical   / 2.0
            - w;

        Camera {
            position:   self.position,
            look_at:    self.look_at,
            up:         self.up,
            fov_deg:    self.fov_deg,
            aspect,
            lower_left,
            horizontal,
            vertical,
        }
    }
}

impl Camera {
    /// Generate a ray through pixel (u, v) where both are in [0, 1]
    pub fn ray(&self, u: f64, v: f64) -> Ray {
        let direction = self.lower_left
            + self.horizontal * u
            + self.vertical   * v
            - self.position;
        Ray::new(self.position, direction)
    }

    /// Render the full scene to a flat Vec<Color>, row-major top-to-bottom
    pub fn render(
        &self,
        scene: &crate::renderer::scene::Scene,
        bvh:     &std::sync::Arc<dyn crate::objects::Hittable>,
        width:   u32,
        height:  u32,
        samples: u32,
        depth:   u32,
    ) -> Vec<Color> {
        use rayon::prelude::*;

        (0..height)
            .into_par_iter()
            .rev()
            .flat_map_iter(|row| (0..width).map(move |col| (row, col)))
            .map(|(row, col)| {
                let mut rng   = rand::thread_rng();
                let mut color = Color::BLACK;

                for _ in 0..samples {
                    let u = (col as f64 + rng.gen::<f64>()) / (width  - 1) as f64;
                    let v = (row as f64 + rng.gen::<f64>()) / (height - 1) as f64;
                    color += scene.trace_bvh(bvh, &self.ray(u, v), depth);
                }

                color * (1.0 / samples as f64)
            })
            .collect()
    }
}