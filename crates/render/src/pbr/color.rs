use std::fmt::Debug;
use flatbox_core::math::glm::Vec3;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Byte(u8, u8, u8),
    Float(f32, f32, f32),
}

impl Color {
    pub const WHITE: Color = Color::Byte(255, 255, 255);
    pub const BLACK: Color = Color::Byte(0, 0, 0);
    pub const RED: Color = Color::Byte(200, 0, 0);
    pub const GREEN: Color = Color::Byte(0, 200, 0);
    pub const BLUE: Color = Color::Byte(0, 0, 200);
    pub const NORMAL: Color = Color::Byte(128, 128, 255);

    pub fn to_byte_repr(&self) -> Color {
        match *self {
            Color::Float(r, g, b) => Color::Byte(
                (r * 255.0) as u8, 
                (g * 255.0) as u8, 
                (b * 255.0) as u8,
            ),
            _ => *self,
        }
    }

    pub fn to_float_repr(&self) -> Color {
        match *self {
            Color::Byte(r, g, b) => Color::Float(
                (r as f32) / 255.0, 
                (g as f32) / 255.0, 
                (b as f32) / 255.0,
            ),
            _ => *self,
        }
    }

    pub fn grayscale(value: u8) -> Self {
        Color::Byte(value, value, value)
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Color::Float(value.x, value.y, value.z)
    }
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        let Color::Byte(r, g, b) = color.to_byte_repr() else { unreachable!() };
        [r, g, b, 255]
    }
}

impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self {
        let Color::Float(r, g, b) = color.to_float_repr() else { unreachable!() };
        [r, g, b]
    }
}