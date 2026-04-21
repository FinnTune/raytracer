use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::sync::mpsc::{self, Receiver};

use crate::materials::{Diffuse, Emissive, Reflective};
use crate::objects::{Cube, Cylinder, Plane, Sphere};
use crate::renderer::{CameraBuilder, Color, Scene};
use crate::renderer::camera::denoise;
use eframe::egui;
use nalgebra::Vector3;

// ── Object types ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum ObjectKind {
    Sphere,
    Cube,
    Cylinder,
    Plane,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Sphere   => write!(f, "Sphere"),
            ObjectKind::Cube     => write!(f, "Cube"),
            ObjectKind::Cylinder => write!(f, "Cylinder"),
            ObjectKind::Plane    => write!(f, "Plane"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum MaterialKind {
    Diffuse,
    Reflective,
    Emissive,
}

impl std::fmt::Display for MaterialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterialKind::Diffuse    => write!(f, "Diffuse"),
            MaterialKind::Reflective => write!(f, "Reflective"),
            MaterialKind::Emissive   => write!(f, "Emissive"),
        }
    }
}

#[derive(Clone)]
pub struct SceneObject {
    pub kind:     ObjectKind,
    pub material: MaterialKind,
    pub x:        f32,
    pub y:        f32,
    pub z:        f32,
    pub size:     f32,
    pub height:   f32,
    pub color:    [f32; 3],
    pub strength: f32,
    pub fuzz:     f32,
}

impl Default for SceneObject {
    fn default() -> Self {
        Self {
            kind:     ObjectKind::Sphere,
            material: MaterialKind::Diffuse,
            x: 0.0, y: 0.0, z: 0.0,
            size:     0.5,
            height:   1.0,
            color:    [0.8, 0.3, 0.2],
            strength: 3.0,
            fuzz:     0.05,
        }
    }
}

// ── Render state ──────────────────────────────────────────────────────────────

struct RenderJob {
    progress: Arc<AtomicU64>,
    total:    u64,
    receiver: Receiver<Vec<u8>>,
    width:    u32,
    height:   u32,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct RtApp {
    cam_x:   f32,
    cam_y:   f32,
    cam_z:   f32,
    look_x:  f32,
    look_y:  f32,
    look_z:  f32,
    fov:     f32,

    width:   u32,
    height:  u32,
    samples: u32,
    depth:   u32,

    objects: Vec<SceneObject>,
    status:  String,
    texture: Option<egui::TextureHandle>,
    job:     Option<RenderJob>,
}

impl Default for RtApp {
    fn default() -> Self {
        Self {
            cam_x:   0.0,
            cam_y:   1.5,
            cam_z:   6.0,
            look_x:  0.0,
            look_y:  0.0,
            look_z:  0.0,
            fov:     45.0,
            width:   600,
            height:  400,
            samples: 256,
            depth:   16,
            objects: default_scene(),
            status:  "Ready.".into(),
            texture: None,
            job:     None,
        }
    }
}

fn default_scene() -> Vec<SceneObject> {
    vec![
        SceneObject {
            kind: ObjectKind::Plane,
            material: MaterialKind::Diffuse,
            x: 0.0, y: -0.5, z: 0.0,
            size: 20.0,
            color: [0.5, 0.5, 0.5],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Diffuse,
            x: -1.8, y: 0.0, z: 0.0,
            size: 0.5,
            color: [0.8, 0.2, 0.2],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Cube,
            material: MaterialKind::Reflective,
            x: 0.0, y: 0.0, z: 0.0,
            size: 0.8,
            color: [0.8, 0.8, 0.8],
            fuzz: 0.05,
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Cylinder,
            material: MaterialKind::Diffuse,
            x: 1.8, y: -0.5, z: 0.0,
            size: 0.4,
            height: 1.0,
            color: [0.2, 0.3, 0.9],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Emissive,
            x: 0.0, y: 4.0, z: 1.0,
            size: 0.8,
            color: [1.0, 1.0, 1.0],
            strength: 5.0,
            ..Default::default()
        },
    ]
}

// ── Scene building ────────────────────────────────────────────────────────────

fn build_scene(objects: &[SceneObject]) -> Scene {
    let mut scene = Scene::new(Color::new(0.05, 0.07, 0.12));

    for obj in objects {
        let [r, g, b] = obj.color;
        let color  = Color::new(r as f64, g as f64, b as f64);
        let pos    = Vector3::new(obj.x as f64, obj.y as f64, obj.z as f64);

        let mat_id = match obj.material {
            MaterialKind::Diffuse    => scene.add_material(Diffuse::new(color)),
            MaterialKind::Reflective => scene.add_material(Reflective::new(color, obj.fuzz as f64)),
            MaterialKind::Emissive   => scene.add_material(Emissive::new(color, obj.strength as f64)),
        };

        match obj.kind {
            ObjectKind::Sphere   => scene.add_object(Sphere::new(pos, obj.size as f64, mat_id)),
            ObjectKind::Cube     => scene.add_object(Cube::new(pos, obj.size as f64, mat_id)),
            ObjectKind::Cylinder => scene.add_object(Cylinder::new(pos, obj.size as f64, obj.height as f64, mat_id)),
            ObjectKind::Plane    => scene.add_object(Plane::new(pos, obj.size as f64, mat_id)),
        }
    }

    scene
}

// ── UI ────────────────────────────────────────────────────────────────────────

impl eframe::App for RtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background render job
        if let Some(job) = &self.job {
            let done  = job.progress.load(Ordering::Relaxed);
            let frac  = done as f32 / job.total as f32;
            self.status = format!("Rendering… {:.0}%", frac * 100.0);

            if let Ok(rgba) = job.receiver.try_recv() {
                let tex = ctx.load_texture(
                    "render",
                    egui::ColorImage::from_rgba_unmultiplied(
                        [job.width as usize, job.height as usize],
                        &rgba,
                    ),
                    egui::TextureOptions::LINEAR,
                );
                self.texture = Some(tex);
                self.status  = "Done. output.png and output.ppm written.".into();
                self.job     = None;
            } else {
                // Keep repainting while job is running
                ctx.request_repaint();
            }
        }

        egui::SidePanel::left("controls")
            .min_width(280.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_camera_section(ui);
                    ui.separator();
                    self.draw_render_settings(ui);
                    ui.separator();
                    self.draw_objects_section(ui);
                    ui.separator();
                    self.draw_render_button(ui, ctx);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_viewport(ui);
        });
    }
}

impl RtApp {
    fn draw_camera_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Camera");
        ui.label("Position");
        ui.horizontal(|ui| {
            ui.label("X"); ui.add(egui::DragValue::new(&mut self.cam_x).speed(0.1));
            ui.label("Y"); ui.add(egui::DragValue::new(&mut self.cam_y).speed(0.1));
            ui.label("Z"); ui.add(egui::DragValue::new(&mut self.cam_z).speed(0.1));
        });
        ui.label("Look at");
        ui.horizontal(|ui| {
            ui.label("X"); ui.add(egui::DragValue::new(&mut self.look_x).speed(0.1));
            ui.label("Y"); ui.add(egui::DragValue::new(&mut self.look_y).speed(0.1));
            ui.label("Z"); ui.add(egui::DragValue::new(&mut self.look_z).speed(0.1));
        });
        ui.add(egui::Slider::new(&mut self.fov, 10.0..=120.0).text("FOV"));
    }

    fn draw_render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Render");
        ui.horizontal(|ui| {
            ui.label("Width");
            ui.add(egui::DragValue::new(&mut self.width).clamp_range(100u32..=3840u32));
            ui.label("Height");
            ui.add(egui::DragValue::new(&mut self.height).clamp_range(100u32..=2160u32));
        });
        ui.add(egui::Slider::new(&mut self.samples, 1..=4096).text("Samples").logarithmic(true));
        ui.add(egui::Slider::new(&mut self.depth, 1..=64).text("Depth"));
    }

    fn draw_objects_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Objects");

        if ui.button("+ Add object").clicked() {
            self.objects.push(SceneObject::default());
        }

        let mut to_delete: Option<usize> = None;

        for (i, obj) in self.objects.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                egui::CollapsingHeader::new(format!("{} {}", obj.kind, i + 1))
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ComboBox::from_label("Type")
                            .selected_text(obj.kind.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut obj.kind, ObjectKind::Sphere,   "Sphere");
                                ui.selectable_value(&mut obj.kind, ObjectKind::Cube,     "Cube");
                                ui.selectable_value(&mut obj.kind, ObjectKind::Cylinder, "Cylinder");
                                ui.selectable_value(&mut obj.kind, ObjectKind::Plane,    "Plane");
                            });

                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.label("X"); ui.add(egui::DragValue::new(&mut obj.x).speed(0.05));
                            ui.label("Y"); ui.add(egui::DragValue::new(&mut obj.y).speed(0.05));
                            ui.label("Z"); ui.add(egui::DragValue::new(&mut obj.z).speed(0.05));
                        });

                        let size_label = match obj.kind {
                            ObjectKind::Cube => "Side length: ",
                            _                => "Radius: ",
                        };
                        ui.add(egui::DragValue::new(&mut obj.size).speed(0.05).prefix(size_label));

                        if obj.kind == ObjectKind::Cylinder {
                            ui.add(egui::DragValue::new(&mut obj.height).speed(0.05).prefix("Height: "));
                        }

                        egui::ComboBox::from_label("Material")
                            .selected_text(obj.material.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut obj.material, MaterialKind::Diffuse,    "Diffuse");
                                ui.selectable_value(&mut obj.material, MaterialKind::Reflective, "Reflective");
                                ui.selectable_value(&mut obj.material, MaterialKind::Emissive,   "Emissive");
                            });

                        ui.horizontal(|ui| {
                            ui.label("Color");
                            ui.color_edit_button_rgb(&mut obj.color);
                        });

                        match obj.material {
                            MaterialKind::Reflective => {
                                ui.add(egui::Slider::new(&mut obj.fuzz, 0.0..=1.0).text("Fuzz"));
                            }
                            MaterialKind::Emissive => {
                                ui.add(egui::Slider::new(&mut obj.strength, 0.1..=20.0).text("Strength"));
                            }
                            _ => {}
                        }

                        if ui.button("Delete").clicked() {
                            to_delete = Some(i);
                        }
                    });
            });
        }

        if let Some(i) = to_delete {
            self.objects.remove(i);
        }
    }

    fn draw_render_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(8.0);

        let rendering = self.job.is_some();

        // Progress bar — only visible while rendering
        if let Some(job) = &self.job {
            let done = job.progress.load(Ordering::Relaxed);
            let frac = done as f32 / job.total as f32;
            ui.add(
                egui::ProgressBar::new(frac)
                    .show_percentage()
                    .animate(true)
            );
        }

        let label  = if rendering { "Rendering…" } else { "Render" };
        let button = egui::Button::new(egui::RichText::new(label).size(16.0));

        if ui.add_enabled(
            !rendering,
            egui::widgets::Button::new(egui::RichText::new(label).size(16.0))
                .min_size(egui::vec2(ui.available_width(), 36.0)),
        ).clicked() {
            if self.objects.is_empty() {
                self.status = "Add at least one object first.".into();
                return;
            }
            self.start_render(ctx);
        }
        let _ = button; // suppress unused warning

        ui.label(&self.status);
    }

    fn start_render(&mut self, ctx: &egui::Context) {
        let objects  = self.objects.clone();
        let cam_pos  = Vector3::new(self.cam_x as f64, self.cam_y as f64, self.cam_z as f64);
        let look_at  = Vector3::new(self.look_x as f64, self.look_y as f64, self.look_z as f64);
        let fov      = self.fov as f64;
        let width    = self.width;
        let height   = self.height;
        let samples  = self.samples;
        let depth    = self.depth;

        let progress = Arc::new(AtomicU64::new(0));
        let total    = (width * height) as u64;
        let (tx, rx) = mpsc::channel();

        let progress_clone = Arc::clone(&progress);
        let ctx_clone      = ctx.clone();

        std::thread::spawn(move || {
            let mut scene  = build_scene(&objects);
            let bvh        = scene.build_bvh();
            let camera     = CameraBuilder::new()
                .position(cam_pos)
                .look_at(look_at)
                .fov(fov)
                .resolution(width, height)
                .build();

            let pixels = camera.render(&scene, &bvh, width, height, samples, depth, progress_clone);
            let pixels = denoise(&pixels, width, height);

            camera.write_to_ppm("output.ppm", &pixels);
            camera.write_to_png("output.png", &pixels);

            let rgba: Vec<u8> = pixels
                .iter()
                .flat_map(|c| {
                    let (r, g, b) = c.to_rgb_u8(2.2);
                    [r, g, b, 255u8]
                })
                .collect();

            tx.send(rgba).ok();
            ctx_clone.request_repaint();
        });

        self.job    = Some(RenderJob { progress, total, receiver: rx, width, height });
        self.status = "Rendering…".into();
    }

    fn draw_viewport(&self, ui: &mut egui::Ui) {
        if let Some(tex) = &self.texture {
            let available = ui.available_size();
            let img_size  = tex.size_vec2();
            let scale     = (available.x / img_size.x).min(available.y / img_size.y);
            let size      = egui::vec2(img_size.x * scale, img_size.y * scale);
            ui.centered_and_justified(|ui| {
                ui.image((tex.id(), size));
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Configure your scene and press Render.")
                        .color(egui::Color32::GRAY)
                        .size(16.0),
                );
            });
        }
    }
}

pub fn launch() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rt — ray tracer")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "rt",
        options,
        Box::new(|_cc| Box::new(RtApp::default())),
    )
    .expect("Failed to launch GUI");
}