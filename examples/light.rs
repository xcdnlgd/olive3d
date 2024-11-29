use lazy_static::lazy_static;
use olive3d::{
    geometry::{Cross, Dot, Matrix, Matrix4, Vector2, Vector3},
    model::Model,
    renderer::Renderer,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const DEPTH: u32 = 255;

fn m2v(m: &Matrix<4, 1>) -> Vector3 {
    Vector3::new(m[0][0] / m[3][0], m[1][0] / m[3][0], m[2][0] / m[3][0])
}

#[rustfmt::skip]
fn v2m(v: &Vector3) -> Matrix<4, 1> {
    [
        [v.x()],
        [v.y()],
        [v.z()],
        [  1.0],
    ].into()
}

#[rustfmt::skip]
fn viewport(x: f32, y: f32, w: f32, h: f32) -> Matrix4 {
    let d = DEPTH as f32;
    [
        [w/2.0,   0.0,   0.0, x+w/2.0],
        [  0.0, -h/2.0,   0.0, y+h/2.0],
        [  0.0,   0.0, d/2.0,   d/2.0],
        [  0.0,   0.0,   0.0,     1.0],
    ]
    .into()
}

lazy_static! {
    static ref MODEL: Model = {
        let mut model = Model::new("./obj/african_head.obj");
        model.load_diffuse_map("./obj/african_head_diffuse.ppm");
        model
    };
    static ref transform: Matrix4 = {
        let camera = Vector3::new(0.0, 0.0, 3.0);

        let mut projection = Matrix4::identity();
        projection[3][2] = -1.0 / camera.z();
        let viewport = viewport(
            WIDTH as f32 / 8.0,
            HEIGHT as f32 / 8.0,
            WIDTH as f32 * 3.0 / 4.0,
            HEIGHT as f32 * 3.0 / 4.0,
        );
        viewport * projection
    };
}

static mut T: f32 = 0.0;

pub fn render(buffer: &mut [u32], z_buffer: &mut [f32], dt: f32) {
    let mut renderer = Renderer::new(buffer, z_buffer, WIDTH, HEIGHT);
    let light_dir = unsafe {
        T += dt;
        let light_dir = Vector3::new(T.cos(), T.sin(), -1.0);
        light_dir.normalize()
    };
    renderer.fill(0xff000000);
    for i in 0..MODEL.nfaces() {
        let mut screen_coords = Vec::with_capacity(3);
        let mut world_coords = Vec::with_capacity(3);
        for j in 0..3 {
            let v = MODEL.vert(i, j);
            screen_coords.push(m2v(&(&(*transform) * v2m(&v))));
            world_coords.push(v);
        }
        let mut n =
            (&world_coords[2] - &world_coords[0]).cross(&world_coords[1] - &world_coords[0]);
        n = n.normalize();
        let intensity = n.dot(&light_dir);
        renderer.fill_triangle(&screen_coords, |bc| {
            let mut uv = Vector2::zero();
            for j in 0..3 {
                let c_uv = MODEL.uv(i, j);
                uv[0] += c_uv[0] * bc[j];
                uv[1] += c_uv[1] * bc[j];
            }
            let pixel: u32 = if let Some(ref diffuse_map) = MODEL.diffuse_map {
                let x = uv.x() * diffuse_map.width as f32;
                let y = uv.y() * diffuse_map.height as f32;
                diffuse_map.buffer[x as usize + y as usize * diffuse_map.width as usize]
            } else {
                0xffffffff
            };
            let mut new_pixel = 0xff000000;
            for i in 0..3 {
                let mut part = ((pixel >> (8 * i)) & 0xff) as f32;
                part *= intensity.abs();
                new_pixel |= ((part as u32) & 0xff) << (8 * i)
            }
            new_pixel
        });
    }
    // renderer.save_to_ppm_file("output/black.ppm").unwrap();
}

pub fn init() {}

include!("../common/main.rs");
