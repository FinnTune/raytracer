use rt::renderer::Color;

#[test]
fn gamma_output_clamps() {
    let c = Color::new(1.5, -0.1, 0.5);
    let (r, g, b) = c.to_rgb_u8(2.0);
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 180);
}

#[test]
fn attenuate_is_elementwise() {
    let a = Color::new(0.5, 0.5, 0.5);
    let b = Color::new(0.5, 0.5, 0.5);
    let result = a.attenuate(b);
    assert!((result.r - 0.25).abs() < 1e-10);
}
