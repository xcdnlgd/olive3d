#![allow(clippy::too_many_arguments)]
use std::path::Path;

use crate::{
    geometry::{Cross, Line2D, Ray, Vector3},
    ppm::save_buffer_to_ppm_file,
};

pub struct Renderer<'b> {
    buffer: &'b mut [u32],
    z_buffer: &'b mut [f32],
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}
impl<'b> Renderer<'b> {
    pub fn new(buffer: &'b mut [u32], z_buffer: &'b mut [f32], width: u32, height: u32) -> Self {
        assert_eq!((width * height) as usize, buffer.len());
        assert_eq!(z_buffer.len(), buffer.len());
        Self {
            buffer,
            z_buffer,
            width,
            height,
            stride: width,
        }
    }
    #[inline]
    pub fn draw_pixel_unchecked(&mut self, x: u32, y: u32, pixel: u32) {
        self.buffer[(y * self.stride + x) as usize] = pixel;
    }
    #[inline]
    pub fn fill(&mut self, pixel: u32) {
        self.buffer.fill(pixel);
        self.z_buffer.fill(f32::MIN);
    }
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pixel: u32) {
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
    pub fn fill_triangle(&mut self, verts: &[Vector3], shader: impl Fn(Vector3) -> u32) {
        let ((x_min, y_min), (x_max, y_max)) = triangle_bunding_box(verts);
        if let Some(((x_min, y_min), (x_max, y_max))) = normalize_rect(
            x_min as i32,
            y_min as i32,
            (x_max - x_min) as i32 + 1,
            (y_max - y_min) as i32 + 1,
            self.width,
            self.height,
        ) {
            for y in y_min..=(y_max + 1) {
                for x in x_min..=(x_max + 1) {
                    let bc = barycentric(
                        x as f32 + 0.5,
                        y as f32 + 0.5,
                        verts[0].x(),
                        verts[0].y(),
                        verts[1].x(),
                        verts[1].y(),
                        verts[2].x(),
                        verts[2].y(),
                    );
                    if bc.x() < 0.0 || bc.y() < 0.0 || bc.z() < 0.0 {
                        continue;
                    }
                    let mut z = 0.0;
                    for i in 0..3 {
                        z += verts[i].z() * bc[i]
                    }
                    if self.z_buffer[(x + y * self.stride) as usize] < z {
                        self.z_buffer[(x + y * self.stride) as usize] = z;
                        self.draw_pixel_unchecked(x, y, shader(bc));
                    }
                }
            }
        }
    }
    pub fn save_to_ppm_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        save_buffer_to_ppm_file(self.buffer, self.width, self.height, self.stride, path)
    }
}

fn triangle_bunding_box(verts: &[Vector3]) -> ((f32, f32), (f32, f32)) {
    let mut xs: Vec<f32> = verts.iter().map(|v| v.x()).collect();
    let mut ys: Vec<f32> = verts.iter().map(|v| v.y()).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ((xs[0], ys[0]), (xs[2], ys[2]))
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

// return (u, v, w)
#[inline]
fn barycentric(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> Vector3 {
    // Barycentric coordinate system
    // https://github.com/ssloy/tinyrenderer/wiki/Lesson-2:-Triangle-rasterization-and-back-face-culling#:~:text=It%20means%20that%20we%20are%20looking%20for%20a%20vector%20(u%2Cv%2C1)%20that%20is%20orthogonal%20to%20(ABx%2CACx%2CPAx)%20and%20(ABy%2CACy%2CPAy)%20at%20the%20same%20time!
    let u = Vector3::new(x2 - x0, x1 - x0, x0 - x).cross(Vector3::new(y2 - y0, y1 - y0, y0 - y));
    if u.z().abs() < 1.0 {
        return Vector3::new(-1.0, 1.0, 1.0);
    }
    Vector3::new(1.0 - (u.x() + u.y()) / u.z(), u.y() / u.z(), u.x() / u.z())
}
