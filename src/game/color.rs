
use std::ops::*;

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(dead_code)]
impl Color {

    pub fn from_u32(r: u32, g: u32, b: u32) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    pub fn lerp(from: Color, to: Color, t: f32) -> Color {
        let r = from.r + t * (to.r - from.r);
        let g = from.g + t * (to.g - from.g);
        let b = from.b + t * (to.b - from.b);
        let a = from.a + t * (to.a - from.a);

        Color { r, g, b, a }
    }

}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}

impl Sub for Color {
    type Output = Color;

    fn sub(self, other: Color) -> Color {
        Color {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
            a: self.a - other.a,
        }
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a,
        }
    }
}

impl Div for Color {
    type Output = Color;

    fn div(self, other: Color) -> Color {
        Color {
            r: self.r / other.r,
            g: self.g / other.g,
            b: self.b / other.b,
            a: self.a / other.a,
        }
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl From<[f32; 3]> for Color {
    fn from(item: [f32; 3]) -> Color {
        Color {
            r: item[0],
            g: item[1],
            b: item[2],
            a: 1.0,
        }
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<[f32; 4]> for Color {
    fn from(item: [f32; 4]) -> Color {
        Color {
            r: item[0],
            g: item[1],
            b: item[2],
            a: item[3],
        }
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}