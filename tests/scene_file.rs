use rt::scene_file::{
    build_scene, default_scene_objects, CameraSettings, MaterialKind, ObjectKind, RenderSettings,
    SceneFile, SceneObject,
};

fn sample_scene_file() -> SceneFile {
    SceneFile {
        camera: CameraSettings {
            position: [1.0, 2.0, 3.0],
            look_at: [0.0, 0.5, 0.0],
            fov: 50.0,
        },
        render: RenderSettings {
            width: 320,
            height: 240,
            samples: 64,
            depth: 8,
        },
        objects: vec![SceneObject {
            kind: ObjectKind::Cube,
            material: MaterialKind::Dielectric,
            x: 0.1,
            y: 0.2,
            z: 0.3,
            size: 0.6,
            ior: 1.7,
            ..Default::default()
        }],
    }
}

#[test]
fn round_trips_through_json_on_disk() {
    let dir = std::env::temp_dir().join(format!("rt-scene-file-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("scene.json");
    let path = path.to_str().unwrap();

    let original = sample_scene_file();
    original.save(path).expect("save should succeed");

    let loaded = SceneFile::load(path).expect("load should succeed");

    assert_eq!(loaded.camera.position, original.camera.position);
    assert_eq!(loaded.camera.look_at, original.camera.look_at);
    assert_eq!(loaded.camera.fov, original.camera.fov);
    assert_eq!(loaded.render.width, original.render.width);
    assert_eq!(loaded.render.height, original.render.height);
    assert_eq!(loaded.render.samples, original.render.samples);
    assert_eq!(loaded.render.depth, original.render.depth);
    assert_eq!(loaded.objects.len(), original.objects.len());
    assert_eq!(loaded.objects[0].kind, original.objects[0].kind);
    assert_eq!(loaded.objects[0].material, original.objects[0].material);
    assert_eq!(loaded.objects[0].ior, original.objects[0].ior);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn load_reports_an_error_for_a_missing_file() {
    let result = SceneFile::load("/nonexistent/path/does-not-exist.json");
    assert!(result.is_err());
}

#[test]
fn load_reports_an_error_for_invalid_json() {
    let dir = std::env::temp_dir().join(format!("rt-scene-file-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bad.json");
    std::fs::write(&path, "not valid json").unwrap();

    let result = SceneFile::load(path.to_str().unwrap());
    assert!(result.is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn default_scene_objects_build_into_a_non_empty_scene() {
    let objects = default_scene_objects();
    assert!(!objects.is_empty());

    let mut scene = build_scene(&objects);
    // build_bvh panics on an empty scene, so this alone proves objects made it through.
    scene.build_bvh();
}
