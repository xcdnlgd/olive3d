#![allow(clippy::too_many_arguments)]
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::geometry::{Line2D, Ray};

pub trait Pixel: Clone + Copy {}
impl Pixel for u32 {}

pub struct Renderer<'b, T: Pixel> {
    buffer: &'b mut [T],
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}
impl<'b, T: Pixel> Renderer<'b, T> {
    pub fn new(buffer: &'b mut [T], width: u32, height: u32) -> Self {
        assert_eq!((width * height) as usize, buffer.len());
        Self {
            buffer,
            width,
            height,
            stride: width,
        }
    }
    #[inline]
    fn draw_pixel_unchecked(&mut self, x: u32, y: u32, pixel: T) {
        self.buffer[(y * self.stride + x) as usize] = pixel;
    }
    #[inline]
    pub fn fill(&mut self, pixel: T) {
        self.buffer.fill(pixel);
    }
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pixel: T) {
        let (x0, y0, x1, y1) = if let Some(Line2D { x0, y0, x1, y1 }) = (Line2D {
            x0: x0 as f32,
            y0: y0 as f32,
            x1: x1 as f32,
            y1: y1 as f32,
        })
        .box_clip(
            0f32,
            0f32,
            self.width as f32 - 0.1,
            self.height as f32 - 0.1,
        ) {
            (x0 as i32, y0 as i32, x1 as i32, y1 as i32)
        } else {
            return;
        };
        self.draw_pixel_unchecked(x1 as u32, y1 as u32, pixel);

        let mut ray = Ray::new(x0, y0, x1, y1);
        while !ray.reached {
            let (x, y) = ray.next_xy();
            self.draw_pixel_unchecked(x as u32, y as u32, pixel)
        }
    }
    pub fn fill_triangle(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        pixel: T,
    ) {
        let ((x_min, y_min), (x_max, y_max)) = triangle_bunding_box(x0, y0, x1, y1, x2, y2);
        if let Some(((x_min, y_min), (x_max, y_max))) = normalize_rect(
            x_min,
            y_min,
            x_max - x_min + 1,
            y_max - y_min + 1,
            self.width,
            self.height,
        ) {
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    let (u, v, w) = barycentric(
                        x as f32, y as f32, x0 as f32, y0 as f32, x1 as f32, y1 as f32, x2 as f32,
                        y2 as f32,
                    );
                    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                        self.draw_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
    }
}

impl<'b> Renderer<'b, u32> {
    pub fn save_to_ppm_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        write!(file, "P6\n{} {} 255\n", self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.buffer[(y * self.width + x) as usize];
                let rgb: [u8; 3] = [
                    ((pixel) & 0xFF) as u8,
                    ((pixel >> 8) & 0xFF) as u8,
                    ((pixel >> (8 * 2)) & 0xFF) as u8,
                ];
                file.write_all(&rgb)?;
            }
        }
        Ok(())
    }
}

fn triangle_bunding_box(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) -> ((i32, i32), (i32, i32)) {
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }

    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
    }
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }

    ((x0, y0), (x2, y2))
}

/// ```
/// if let Some((x0, y0), (x1, y1)) = normalize_rect(x, y, w, h, bound_width, bound_height) {
///     for y in y0..=y1 {
///         for x in x0..=x1 {
///             // do things on (x, y)
///         }
///     }
/// }
/// ```
fn normalize_rect(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bound_width: u32,
    bound_height: u32,
) -> Option<((u32, u32), (u32, u32))> {
    if w == 0 || h == 0 {
        return None;
    }
    let x1 = if w > 0 { x + w - 1 } else { x + w + 1 };
    let y1 = if h > 0 { y + h - 1 } else { y + h + 1 };
    let mut x0 = x.clamp(0, bound_width as i32 - 1);
    let mut y0 = y.clamp(0, bound_height as i32 - 1);
    let mut x1 = x1.clamp(0, bound_width as i32 - 1);
    let mut y1 = y1.clamp(0, bound_height as i32 - 1);
    if x1 < x0 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y1 < y0 {
        std::mem::swap(&mut y0, &mut y1);
    }
    Some(((x0 as u32, y0 as u32), (x1 as u32, y1 as u32)))
}

// return (x, y, z)
#[inline]
fn vector3_a_cross_b(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> (f32, f32, f32) {
    let x = ay * bz - az * by;
    let y = az * bx - ax * bz;
    let z = ax * by - ay * bx;
    (x, y, z)
}

// return (u, v, w)
#[inline]
fn barycentric(
    x: f32,
    y: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> (f32, f32, f32) {
    // Barycentric coordinate system
    // https://github.com/ssloy/tinyrenderer/wiki/Lesson-2:-Triangle-rasterization-and-back-face-culling#:~:text=It%20means%20that%20we%20are%20looking%20for%20a%20vector%20(u%2Cv%2C1)%20that%20is%20orthogonal%20to%20(ABx%2CACx%2CPAx)%20and%20(ABy%2CACy%2CPAy)%20at%20the%20same%20time!
    let (x, y, z) = vector3_a_cross_b(x1 - x0, x2 - x0, x0 - x, y1 - y0, y2 - y0, y0 - y);
    let v = x / z;
    let w = y / z;
    let u = 1.0 - w - v;
    (u, v, w)
}
