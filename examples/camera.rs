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
        [w/2.0,    0.0,   0.0, x+w/2.0],
        [  0.0, -h/2.0,   0.0, y+h/2.0],
        [  0.0,    0.0, d/2.0,   d/2.0],
        [  0.0,    0.0,   0.0,     1.0],
    ]
    .into()
}

#[rustfmt::skip]
fn lookat(eye: &Vector3, center: &Vector3, up: &Vector3) -> Matrix4 {
    let z = (eye - center).normalize();
    let x = up.cross(&z).normalize();
    let y = (&z).cross(&x).normalize();
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

lazy_static! {
    static ref MODEL: Model = {
        let mut model = Model::new("./obj/african_head.obj");
        model.load_diffuse_map("./obj/african_head_diffuse.ppm");
        model
    };
}

static mut T: f32 = 0.0;

pub fn render(buffer: &mut [u32], z_buffer: &mut [f32], dt: f32) {
    let mut renderer = Renderer::new(buffer, z_buffer, WIDTH, HEIGHT);
    // let light_dir = Vector3::new(0.0, 0.0, -1.0);
    let eye = unsafe {
        T += dt;
        Vector3::new(T.cos(), T.sin(), 3.0)
    };
    let center = Vector3::new(0.0, 0.0, 0.0);
    let light_dir = (&eye - &center).normalize();

    let model_view = lookat(&eye, &center, &Vector3::new(0.0, 1.0, 0.0));

    let mut projection = Matrix4::identity();
    projection[3][2] = -1.0 / (eye - center).length();

    let viewport = viewport(
        WIDTH as f32 / 8.0,
        HEIGHT as f32 / 8.0,
        WIDTH as f32 * 3.0 / 4.0,
        HEIGHT as f32 * 3.0 / 4.0,
    );

    let transform = viewport * projection * model_view;

    renderer.fill(0xff000000);
    for i in 0..MODEL.nfaces() {
        let mut screen_coords = Vec::with_capacity(3);
        let mut intensities = Vec::with_capacity(3);
        for j in 0..3 {
            let v = MODEL.vert(i, j);
            screen_coords.push(m2v(&(&transform * v2m(&v))));
            intensities.push(MODEL.normal(i, j).normalize().dot(&light_dir));
        }
        renderer.fill_triangle(&screen_coords, |bc| {
            let mut uv = Vector2::zero();
            let mut intensity = 0.0;
            for j in 0..3 {
                let c_uv = MODEL.uv(i, j);
                uv[0] += c_uv[0] * bc[j];
                uv[1] += c_uv[1] * bc[j];
                intensity += intensities[j] * bc[j];
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
                new_pixel |= ((part.clamp(0.0, 255.0) as u32) & 0xff) << (8 * i)
            }
            new_pixel
        });
    }
    // renderer.save_to_ppm_file("output/black.ppm").unwrap();
}

pub fn init() {}

include!("../common/main.rs");
