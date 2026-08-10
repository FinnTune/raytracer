use rt::renderer::camera::pixel_uv;

/// A dimension of 1 divides by (dimension - 1) = 0 unless guarded — this
/// exercises exactly that edge case for both width and height independently.
#[test]
fn pixel_uv_stays_finite_at_one_pixel_resolution() {
    let (u, v) = pixel_uv(0, 0, 1, 1, 0.5, 0.5);
    assert!(u.is_finite() && v.is_finite(), "got u={u}, v={v}");

    let (u, v) = pixel_uv(0, 3, 1, 10, 0.25, 0.75);
    assert!(u.is_finite() && v.is_finite(), "got u={u}, v={v}");

    let (u, v) = pixel_uv(4, 0, 10, 1, 0.75, 0.25);
    assert!(u.is_finite() && v.is_finite(), "got u={u}, v={v}");
}

#[test]
fn pixel_uv_spans_zero_to_one_for_normal_resolutions() {
    let (u, v) = pixel_uv(0, 0, 100, 50, 0.0, 0.0);
    assert_eq!((u, v), (0.0, 0.0));

    let (u, v) = pixel_uv(99, 49, 100, 50, 0.0, 0.0);
    assert_eq!((u, v), (1.0, 1.0));
}
