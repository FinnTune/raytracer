use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0 };

    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// Convert from 0–255 integers, e.g. Color::from_rgb(255, 128, 0)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    /// Gamma-correct and clamp to [0,255] for final output
    pub fn to_rgb_u8(self, gamma: f64) -> (u8, u8, u8) {
        let encode = |c: f64| -> u8 {
            (c.max(0.0).min(1.0).powf(1.0 / gamma) * 255.0).round() as u8
        };
        (encode(self.r), encode(self.g), encode(self.b))
    }

    /// Element-wise multiply — used for attenuation
    pub fn attenuate(self, other: Color) -> Color {
        Color::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }

    pub fn is_finite(&self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite()
    }
}

impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

/// Scale by a scalar — used for averaging samples
impl Mul<f64> for Color {
    type Output = Color;
    fn mul(self, t: f64) -> Color {
        Color::new(self.r * t, self.g * t, self.b * t)
    }
}

impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, t: f64) {
        self.r *= t;
        self.g *= t;
        self.b *= t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}