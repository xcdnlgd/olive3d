use std::ops::Neg;

use olive3d::{
    geometry::{m2v, v2m, Matrix, Matrix4, Vector3},
    model::Model,
    renderer::{self, lookat, viewport, Renderer, Shader},
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const DEPTH: u32 = 255;

struct GouraudShader {
    model: Model,
    transform: Matrix4,
    light_dir: Vector3,
    varying_uv: Matrix<3, 2>,
}
impl Shader for GouraudShader {
    fn vertex(&mut self, iface: usize, nthvert: usize) -> Vector3 {
        self.varying_uv
            .set_row(nthvert, self.model.uv(iface, nthvert));
        let v = self.model.vert(iface, nthvert);
        m2v(&(&self.transform * v2m(&v)))
    }

    fn fregment(&mut self, bc: &Vector3) -> Option<u32> {
        let uv = (Matrix::from_row_vector(bc.clone()) * &self.varying_uv).to_row_vector();
        let n = self.model.normal_uv(&uv).normalize();
        let intensity = n.dot(&self.light_dir).neg().max(0.0);
        let pixel: u32 = self.model.diffuse(&uv);
        // let pixel: u32 = 0xffffffff;
        let mut new_pixel = 0xff000000;
        for i in 0..3 {
            let mut part = ((pixel >> (8 * i)) & 0xff) as f32;
            part *= intensity.abs();
            new_pixel |= ((part.clamp(0.0, 255.0) as u32) & 0xff) << (8 * i)
        }
        Some(new_pixel)
    }
}

fn main() {
    let mut model = Model::new("./obj/african_head.obj");
    model.load_diffuse_map("./obj/african_head_diffuse.ppm");
    model.load_normal_map("./obj/african_head_nm.ppm");
    let model = model;
    let nfaces = model.nfaces();
    let mut buffer = [0u32; WIDTH as usize * HEIGHT as usize];
    let mut z_buffer = [f32::MIN; WIDTH as usize * HEIGHT as usize];

    let mut renderer = Renderer::new(&mut buffer, &mut z_buffer, WIDTH, HEIGHT);
    let light_dir = Vector3::new(-1.0, -1.0, -1.0).normalize();
    let eye = Vector3::new(1.0, 1.0, 3.0);
    let center = Vector3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);

    let model_view = lookat(&eye, &center, &up);

    let projection = renderer::projection(-1.0 / (eye - center).length());

    let viewport = viewport(
        WIDTH as f32 / 8.0,
        HEIGHT as f32 / 8.0,
        WIDTH as f32 * 3.0 / 4.0,
        HEIGHT as f32 * 3.0 / 4.0,
        DEPTH as f32,
    );

    let transform = viewport * projection * model_view;
    // let transform = viewport * projection;

    let mut shader = GouraudShader {
        model,
        transform,
        light_dir,
        varying_uv: Matrix::zero(),
    };

    renderer.fill(0xff000000);
    for i in 0..nfaces {
        let mut screen_coords = Vec::with_capacity(3);
        for j in 0..3 {
            screen_coords.push(shader.vertex(i, j));
        }
        renderer.fill_triangle(&screen_coords, &mut shader);
    }
    renderer.save_to_ppm_file("output/black.ppm").unwrap();
}
