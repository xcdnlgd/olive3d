use lazy_static::lazy_static;
use olive3d::{
    geometry::{Cross, Dot, Vector2, Vector3},
    model::Model,
    renderer::Renderer,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

lazy_static!{
    static ref MODEL: Model = {
        let mut model = Model::new("./obj/african_head.obj");
        model.load_diffuse_map("./obj/african_head_diffuse.ppm");
        model
    };
}

static mut T: f32 = 0.0;

pub fn render(buffer: &mut [u32], z_buffer: &mut [f32], dt: f32) {
    let mut renderer = Renderer::new(buffer, z_buffer, WIDTH, HEIGHT);
    let light_dir =  unsafe {
        T += dt;
        let light_dir = Vector3::new(T.cos(), T.sin(), -1.0);
        light_dir.normalize()
    };
    renderer.fill(0xff000000);
    for i in 0..MODEL.nfaces() {
        let mut s = Vec::with_capacity(3);
        let mut world_coords = Vec::with_capacity(3);
        for j in 0..3 {
            let v = MODEL.vert(i, j);
            let x = (v.x() + 1f32) * WIDTH as f32 / 2f32;
            let y = (-v.y() + 1f32) * HEIGHT as f32 / 2f32;
            s.push(Vector3::new(x, y, v.z()));
            world_coords.push(v);
        }
        let mut n =
            (&world_coords[2] - &world_coords[0]).cross(&world_coords[1] - &world_coords[0]);
        n = n.normalize();
        let intensity = n.dot(&light_dir);
        renderer.fill_triangle(&s, |bc| {
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
