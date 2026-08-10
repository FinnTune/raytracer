use std::sync::mpsc::{self, Receiver};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::renderer::camera::denoise;
use crate::renderer::CameraBuilder;
use crate::scene_file::{
    build_scene, default_scene_objects, CameraSettings, MaterialKind, ObjectKind, RenderSettings,
    SceneFile, SceneObject,
};
use eframe::egui;
use nalgebra::Vector3;

// ── Render state ──────────────────────────────────────────────────────────────

struct RenderJob {
    progress: Arc<AtomicU64>,
    total: u64,
    receiver: Receiver<Vec<u8>>,
    width: u32,
    height: u32,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct RtApp {
    cam_x: f32,
    cam_y: f32,
    cam_z: f32,
    look_x: f32,
    look_y: f32,
    look_z: f32,
    fov: f32,

    width: u32,
    height: u32,
    samples: u32,
    depth: u32,

    objects: Vec<SceneObject>,
    scene_path: String,
    status: String,
    texture: Option<egui::TextureHandle>,
    job: Option<RenderJob>,
}

impl Default for RtApp {
    fn default() -> Self {
        Self {
            cam_x: 0.0,
            cam_y: 1.5,
            cam_z: 6.0,
            look_x: 0.0,
            look_y: 0.0,
            look_z: 0.0,
            fov: 45.0,
            width: 600,
            height: 400,
            samples: 256,
            depth: 16,
            objects: default_scene_objects(),
            scene_path: "scene.json".into(),
            status: "Ready.".into(),
            texture: None,
            job: None,
        }
    }
}

// ── UI ────────────────────────────────────────────────────────────────────────

impl eframe::App for RtApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(job) = &self.job {
            let done = job.progress.load(Ordering::Relaxed);
            let frac = done as f32 / job.total as f32;
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
                self.status = "Done. output.png and output.ppm written.".into();
                self.job = None;
            } else {
                ctx.request_repaint();
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("controls")
            .min_size(280.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_scene_file_section(ui);
                    ui.separator();
                    self.draw_camera_section(ui);
                    ui.separator();
                    self.draw_render_settings(ui);
                    ui.separator();
                    self.draw_objects_section(ui);
                    ui.separator();
                    self.draw_render_button(ui);
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.draw_viewport(ui);
        });
    }
}

impl RtApp {
    fn draw_scene_file_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene File");
        ui.horizontal(|ui| {
            ui.label("Path");
            ui.text_edit_singleline(&mut self.scene_path);
        });
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                match self.save_scene() {
                    Ok(()) => self.status = format!("Saved scene to {}", self.scene_path),
                    Err(e) => self.status = format!("Failed to save scene: {e}"),
                }
            }
            if ui.button("Load").clicked() {
                match self.load_scene() {
                    Ok(()) => self.status = format!("Loaded scene from {}", self.scene_path),
                    Err(e) => self.status = format!("Failed to load scene: {e}"),
                }
            }
        });
    }

    fn save_scene(&self) -> Result<(), String> {
        let file = SceneFile {
            camera: CameraSettings {
                position: [self.cam_x, self.cam_y, self.cam_z],
                look_at: [self.look_x, self.look_y, self.look_z],
                fov: self.fov,
            },
            render: RenderSettings {
                width: self.width,
                height: self.height,
                samples: self.samples,
                depth: self.depth,
            },
            objects: self.objects.clone(),
        };
        file.save(&self.scene_path)
    }

    fn load_scene(&mut self) -> Result<(), String> {
        let file = SceneFile::load(&self.scene_path)?;

        self.cam_x = file.camera.position[0];
        self.cam_y = file.camera.position[1];
        self.cam_z = file.camera.position[2];
        self.look_x = file.camera.look_at[0];
        self.look_y = file.camera.look_at[1];
        self.look_z = file.camera.look_at[2];
        self.fov = file.camera.fov;

        self.width = file.render.width;
        self.height = file.render.height;
        self.samples = file.render.samples;
        self.depth = file.render.depth;

        self.objects = file.objects;
        self.texture = None;

        Ok(())
    }

    fn draw_camera_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Camera");
        ui.label("Position");
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut self.cam_x).speed(0.1));
            ui.label("Y");
            ui.add(egui::DragValue::new(&mut self.cam_y).speed(0.1));
            ui.label("Z");
            ui.add(egui::DragValue::new(&mut self.cam_z).speed(0.1));
        });
        ui.label("Look at");
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut self.look_x).speed(0.1));
            ui.label("Y");
            ui.add(egui::DragValue::new(&mut self.look_y).speed(0.1));
            ui.label("Z");
            ui.add(egui::DragValue::new(&mut self.look_z).speed(0.1));
        });
        ui.add(egui::Slider::new(&mut self.fov, 10.0..=120.0).text("FOV"));
    }

    fn draw_render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Render");
        ui.horizontal(|ui| {
            ui.label("Width");
            ui.add(egui::DragValue::new(&mut self.width).range(100u32..=3840u32));
            ui.label("Height");
            ui.add(egui::DragValue::new(&mut self.height).range(100u32..=2160u32));
        });
        ui.add(
            egui::Slider::new(&mut self.samples, 1..=4096)
                .text("Samples")
                .logarithmic(true),
        );
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
                                ui.selectable_value(&mut obj.kind, ObjectKind::Sphere, "Sphere");
                                ui.selectable_value(&mut obj.kind, ObjectKind::Cube, "Cube");
                                ui.selectable_value(
                                    &mut obj.kind,
                                    ObjectKind::Cylinder,
                                    "Cylinder",
                                );
                                ui.selectable_value(&mut obj.kind, ObjectKind::Plane, "Plane");
                            });

                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(egui::DragValue::new(&mut obj.x).speed(0.05));
                            ui.label("Y");
                            ui.add(egui::DragValue::new(&mut obj.y).speed(0.05));
                            ui.label("Z");
                            ui.add(egui::DragValue::new(&mut obj.z).speed(0.05));
                        });

                        let size_label = match obj.kind {
                            ObjectKind::Cube => "Side length: ",
                            _ => "Radius: ",
                        };
                        ui.add(
                            egui::DragValue::new(&mut obj.size)
                                .speed(0.05)
                                .prefix(size_label),
                        );

                        if obj.kind == ObjectKind::Cylinder {
                            ui.add(
                                egui::DragValue::new(&mut obj.height)
                                    .speed(0.05)
                                    .prefix("Height: "),
                            );
                        }

                        egui::ComboBox::from_label("Material")
                            .selected_text(obj.material.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut obj.material,
                                    MaterialKind::Diffuse,
                                    "Diffuse",
                                );
                                ui.selectable_value(
                                    &mut obj.material,
                                    MaterialKind::Reflective,
                                    "Reflective",
                                );
                                ui.selectable_value(
                                    &mut obj.material,
                                    MaterialKind::Emissive,
                                    "Emissive",
                                );
                                ui.selectable_value(
                                    &mut obj.material,
                                    MaterialKind::Dielectric,
                                    "Dielectric",
                                );
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
                                ui.add(
                                    egui::Slider::new(&mut obj.strength, 0.1..=20.0)
                                        .text("Strength"),
                                );
                            }
                            MaterialKind::Dielectric => {
                                ui.add(
                                    egui::Slider::new(&mut obj.ior, 1.0..=2.5)
                                        .text("Index of refraction"),
                                );
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

    fn draw_render_button(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        let rendering = self.job.is_some();

        // Progress bar — only visible while rendering
        if let Some(job) = &self.job {
            let done = job.progress.load(Ordering::Relaxed);
            let frac = done as f32 / job.total as f32;
            ui.add(egui::ProgressBar::new(frac).show_percentage().animate(true));
        }

        let label = if rendering { "Rendering…" } else { "Render" };
        let button = egui::Button::new(egui::RichText::new(label).size(16.0));

        if ui
            .add_enabled(
                !rendering,
                egui::widgets::Button::new(egui::RichText::new(label).size(16.0))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            if self.objects.is_empty() {
                self.status = "Add at least one object first.".into();
                return;
            }
            self.start_render(ui.ctx());
        }
        let _ = button; // suppress unused warning

        ui.label(&self.status);
    }

    fn start_render(&mut self, ctx: &egui::Context) {
        let objects = self.objects.clone();
        let cam_pos = Vector3::new(self.cam_x as f64, self.cam_y as f64, self.cam_z as f64);
        let look_at = Vector3::new(self.look_x as f64, self.look_y as f64, self.look_z as f64);
        let fov = self.fov as f64;
        let width = self.width;
        let height = self.height;
        let samples = self.samples;
        let depth = self.depth;

        let progress = Arc::new(AtomicU64::new(0));
        let total = (width * height) as u64;
        let (tx, rx) = mpsc::channel();

        let progress_clone = Arc::clone(&progress);
        let ctx_clone = ctx.clone();

        std::thread::spawn(move || {
            let mut scene = build_scene(&objects);
            let bvh = scene.build_bvh();
            let camera = CameraBuilder::new()
                .position(cam_pos)
                .look_at(look_at)
                .fov(fov)
                .resolution(width, height)
                .build();

            let pixels = camera.render(&scene, &bvh, samples, depth, progress_clone);
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

        self.job = Some(RenderJob {
            progress,
            total,
            receiver: rx,
            width,
            height,
        });
        self.status = "Rendering…".into();
    }

    fn draw_viewport(&self, ui: &mut egui::Ui) {
        if let Some(tex) = &self.texture {
            let available = ui.available_size();
            let img_size = tex.size_vec2();
            let scale = (available.x / img_size.x).min(available.y / img_size.y);
            let size = egui::vec2(img_size.x * scale, img_size.y * scale);
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
        Box::new(|_cc| Ok(Box::new(RtApp::default()))),
    )
    .expect("Failed to launch GUI");
}
