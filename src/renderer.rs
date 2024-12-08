#![allow(clippy::too_many_arguments)]
use std::path::Path;

use crate::{
    geometry::{Line2D, Matrix, Matrix4, Ray, Vector3},
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
    pub fn fill_triangle(&mut self, verts: &[Vector3], shader: &mut impl Shader) {
        let ((x_min, y_min), (x_max, y_max)) = triangle_bunding_box(verts);
        let x_min = (x_min.round() as i32).clamp(0, self.width as i32) as u32;
        let y_min = (y_min.round() as i32).clamp(0, self.height as i32) as u32;
        let x_max = (x_max.round() as i32).clamp(0, self.width as i32) as u32;
        let y_max = (y_max.round() as i32).clamp(0, self.height as i32) as u32;
        for y in y_min..y_max {
            for x in x_min..x_max {
                // TODO: why it doesn't consider z
                let mut bc = barycentric(
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
                    bc[i] /= verts[i].z();
                    z += bc[i];
                }
                let z = 1.0 / z;
                if self.z_buffer[(x + y * self.stride) as usize] < z {
                    for i in 0..3 {
                        bc[i] *= z;
                    }
                    self.z_buffer[(x + y * self.stride) as usize] = z;
                    if let Some(color) = shader.fregment(&bc) {
                        self.draw_pixel_unchecked(x, y, color);
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

// return (u, v, w)
#[inline]
fn barycentric(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> Vector3 {
    // Barycentric coordinate system
    // https://github.com/ssloy/tinyrenderer/wiki/Lesson-2:-Triangle-rasterization-and-back-face-culling#:~:text=It%20means%20that%20we%20are%20looking%20for%20a%20vector%20(u%2Cv%2C1)%20that%20is%20orthogonal%20to%20(ABx%2CACx%2CPAx)%20and%20(ABy%2CACy%2CPAy)%20at%20the%20same%20time!
    let u = Vector3::new(x2 - x0, x1 - x0, x0 - x).cross(&Vector3::new(y2 - y0, y1 - y0, y0 - y));
    if u.z().abs() < 1.0 {
        return Vector3::new(-1.0, 1.0, 1.0);
    }
    Vector3::new(1.0 - (u.x() + u.y()) / u.z(), u.y() / u.z(), u.x() / u.z())
}

#[rustfmt::skip]
pub fn viewport(x: f32, y: f32, w: f32, h: f32, depth: f32) -> Matrix4 {
    let d = depth;
    [
        [w/2.0,    0.0,   0.0, x+w/2.0],
        [  0.0, -h/2.0,   0.0, y+h/2.0],
        [  0.0,    0.0, d/2.0,   d/2.0],
        [  0.0,    0.0,   0.0,     1.0],
    ]
    .into()
}

// coeff = -1.0 / c
pub fn projection(coeff: f32) -> Matrix4 {
    let mut projection = Matrix4::identity();
    projection[3][2] = coeff;
    projection
}

#[rustfmt::skip]
pub fn lookat(eye: &Vector3, center: &Vector3, up: &Vector3) -> Matrix4 {
    let z = (eye - center).normalize();
    let x = up.cross(&z).normalize();
    let y = z.cross(&x).normalize();
    // TODO: figure it out
    let minv = Matrix::from_rows([
        [x[0], x[1], x[2], 0.0],
        [y[0], y[1], y[2], 0.0],
        [z[0], z[1], z[2], 0.0],
        [ 0.0,  0.0,  0.0, 1.0],
    ]);
    // move center to [0,0,0]
    let tr = Matrix::from_rows([
        [1.0, 0.0, 0.0, -center[0]],
        [0.0, 1.0, 0.0, -center[1]],
        [0.0, 0.0, 1.0, -center[2]],
        [0.0, 0.0, 0.0,        1.0],
    ]);
    minv * tr
}

pub trait Shader {
    fn vertex(&mut self, iface: usize, nthvert: usize) -> Vector3;
    fn fregment(&mut self, bc: &Vector3) -> Option<u32>;
}
