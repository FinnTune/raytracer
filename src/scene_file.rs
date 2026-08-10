//! A GUI-independent, serializable description of a scene — the on-disk
//! counterpart to `renderer::Scene`, which holds `Arc<dyn Hittable>` /
//! `Arc<dyn Material>` trait objects and isn't itself serializable. Both the
//! GUI (save/load) and headless mode (`--scene`) build a `renderer::Scene`
//! from this shape rather than hardcoding one of their own.

use crate::materials::{Dielectric, Diffuse, Emissive, Reflective};
use crate::objects::{Cube, Cylinder, Plane, Sphere};
use crate::renderer::{Color, Scene};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectKind {
    Sphere,
    Cube,
    Cylinder,
    Plane,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Sphere => write!(f, "Sphere"),
            ObjectKind::Cube => write!(f, "Cube"),
            ObjectKind::Cylinder => write!(f, "Cylinder"),
            ObjectKind::Plane => write!(f, "Plane"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialKind {
    Diffuse,
    Reflective,
    Emissive,
    Dielectric,
}

impl std::fmt::Display for MaterialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterialKind::Diffuse => write!(f, "Diffuse"),
            MaterialKind::Reflective => write!(f, "Reflective"),
            MaterialKind::Emissive => write!(f, "Emissive"),
            MaterialKind::Dielectric => write!(f, "Dielectric"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SceneObject {
    pub kind: ObjectKind,
    pub material: MaterialKind,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub size: f32,
    pub height: f32,
    pub color: [f32; 3],
    pub strength: f32,
    pub fuzz: f32,
    pub ior: f32,
}

impl Default for SceneObject {
    fn default() -> Self {
        Self {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Diffuse,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            size: 0.5,
            height: 1.0,
            color: [0.8, 0.3, 0.2],
            strength: 3.0,
            fuzz: 0.05,
            ior: 1.5,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub fov: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            position: [0.0, 1.5, 6.0],
            look_at: [0.0, 0.0, 0.0],
            fov: 45.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub depth: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            width: 600,
            height: 400,
            samples: 256,
            depth: 16,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SceneFile {
    pub camera: CameraSettings,
    pub render: RenderSettings,
    pub objects: Vec<SceneObject>,
}

impl SceneFile {
    pub fn load(path: &str) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

/// Build a renderer `Scene` (materials, geometry, background) from a flat
/// object list. Camera and render settings are handled separately by the
/// caller, since they belong to `Camera`/render loop, not `Scene`.
pub fn build_scene(objects: &[SceneObject]) -> Scene {
    let mut scene = Scene::new(Color::new(0.05, 0.07, 0.12));

    for obj in objects {
        let [r, g, b] = obj.color;
        let color = Color::new(r as f64, g as f64, b as f64);
        let pos = Vector3::new(obj.x as f64, obj.y as f64, obj.z as f64);

        let mat_id = match obj.material {
            MaterialKind::Diffuse => scene.add_material(Diffuse::new(color)),
            MaterialKind::Reflective => scene.add_material(Reflective::new(color, obj.fuzz as f64)),
            MaterialKind::Emissive => scene.add_material(Emissive::new(color, obj.strength as f64)),
            MaterialKind::Dielectric => scene.add_material(Dielectric::new(color, obj.ior as f64)),
        };

        match obj.kind {
            ObjectKind::Sphere => scene.add_object(Sphere::new(pos, obj.size as f64, mat_id)),
            ObjectKind::Cube => scene.add_object(Cube::new(pos, obj.size as f64, mat_id)),
            ObjectKind::Cylinder => scene.add_object(Cylinder::new(
                pos,
                obj.size as f64,
                obj.height as f64,
                mat_id,
            )),
            ObjectKind::Plane => scene.add_object(Plane::new(pos, obj.size as f64, mat_id)),
        }
    }

    scene
}

/// The demo scene shown by default in the GUI and rendered by `--no-gui`
/// when no `--scene` file is given.
pub fn default_scene_objects() -> Vec<SceneObject> {
    vec![
        SceneObject {
            kind: ObjectKind::Plane,
            material: MaterialKind::Diffuse,
            x: 0.0,
            y: -0.5,
            z: 0.0,
            size: 20.0,
            color: [0.5, 0.5, 0.5],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Diffuse,
            x: -1.8,
            y: 0.0,
            z: 0.0,
            size: 0.5,
            color: [0.8, 0.2, 0.2],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Cube,
            material: MaterialKind::Reflective,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            size: 0.8,
            color: [0.8, 0.8, 0.8],
            fuzz: 0.05,
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Cylinder,
            material: MaterialKind::Diffuse,
            x: 1.8,
            y: -0.5,
            z: 0.0,
            size: 0.4,
            height: 1.0,
            color: [0.2, 0.3, 0.9],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Diffuse,
            x: 0.0,
            y: 0.7,
            z: 0.0,
            size: 0.2,
            color: [0.2, 0.7, 0.3],
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Emissive,
            x: 0.0,
            y: 4.0,
            z: 1.0,
            size: 0.8,
            color: [1.0, 1.0, 1.0],
            strength: 5.0,
            ..Default::default()
        },
        SceneObject {
            kind: ObjectKind::Sphere,
            material: MaterialKind::Dielectric,
            x: -0.7,
            y: -0.2,
            z: 1.5,
            size: 0.3,
            color: [1.0, 1.0, 1.0],
            ior: 1.5,
            ..Default::default()
        },
    ]
}
