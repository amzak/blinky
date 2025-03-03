use std::ops::{Add, Div, Mul, Sub};

use embedded_graphics::prelude::{Point, Size};

#[derive(Clone, Copy)]
pub struct RelativeSize {
    size: u32,
}

impl RelativeSize {
    pub fn squared(&self) -> u32 {
        self.size * self.size
    }

    pub fn as_f32(&self) -> f32 {
        self.size as f32
    }

    pub fn as_u32(&self) -> u32 {
        self.size as u32
    }

    pub fn as_i32(&self) -> i32 {
        self.size as i32
    }

    pub fn as_u16(&self) -> u16 {
        self.size as u16
    }

    pub fn to_absolute(&self, screen_width: usize) -> i32 {
        let scale = screen_width as f32 / 1000f32;

        (self.size as f32 * scale) as i32
    }

    pub fn to_absolute_u32(&self, screen_width: usize) -> u32 {
        let scale = screen_width as f32 / 1000f32;

        (self.size as f32 * scale) as u32
    }
}

impl From<u16> for RelativeSize {
    fn from(value: u16) -> Self {
        Self { size: value as u32 }
    }
}

impl From<u32> for RelativeSize {
    fn from(value: u32) -> Self {
        Self { size: value }
    }
}

impl From<i32> for RelativeSize {
    fn from(value: i32) -> Self {
        Self { size: value as u32 }
    }
}

impl Div<u16> for RelativeSize {
    type Output = Self;

    fn div(self, rhs: u16) -> Self::Output {
        Self {
            size: self.size / rhs as u32,
        }
    }
}

impl Div<i32> for RelativeSize {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            size: self.size / rhs as u32,
        }
    }
}

impl Div<u32> for RelativeSize {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self {
            size: self.size / rhs,
        }
    }
}

impl Add<RelativeSize> for RelativeSize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            size: self.size + rhs.size,
        }
    }
}

impl Mul<u16> for RelativeSize {
    type Output = Self;

    fn mul(self, rhs: u16) -> Self::Output {
        Self {
            size: self.size * rhs as u32,
        }
    }
}

impl Mul<u32> for RelativeSize {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            size: self.size * rhs,
        }
    }
}

impl Sub<RelativeSize> for RelativeSize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            size: self.size - rhs.size,
        }
    }
}

impl Into<Point> for RelativeSize {
    fn into(self) -> Point {
        Point {
            x: self.size as i32,
            y: self.size as i32,
        }
    }
}

pub struct RelativeCoordinate {
    x: u16,
    y: u16,
}

impl RelativeCoordinate {
    pub fn to_absolute(&self, screen_width: usize) -> Point {
        let scale = screen_width as f32 / 1000f32;

        Point {
            x: (self.x as f32 * scale) as i32,
            y: (self.y as f32 * scale) as i32,
        }
    }

    pub fn new(x: RelativeSize, y: RelativeSize) -> Self {
        Self {
            x: x.as_u16(),
            y: y.as_u16(),
        }
    }
}

impl From<(u16, u16)> for RelativeCoordinate {
    fn from(value: (u16, u16)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Add<RelativeCoordinate> for RelativeCoordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<RelativeCoordinate> for RelativeCoordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
