use olive3d::{
    geometry::{Cross, Dot, Vector3},
    model::Model,
    renderer::Renderer,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn main() {
    let model = Model::new("./obj/african_head.obj");
    let mut buffer = [0u32; WIDTH as usize * HEIGHT as usize];
    let mut renderer = Renderer::new(&mut buffer, WIDTH, HEIGHT);
    let light_dir = Vector3::new(0.0, 0.0, -1.0);
    renderer.fill(0xff181818);
    for i in 0..model.nfaces() {
        let mut s = Vec::with_capacity(3);
        let mut world_coords = Vec::with_capacity(3);
        for j in 0..3 {
            let v = model.vert(i, j);
            let x = ((v.x() + 1f32) * WIDTH as f32 / 2f32) as i32;
            let y = ((-v.y() + 1f32) * HEIGHT as f32 / 2f32) as i32;
            s.push((x, y));
            world_coords.push(v);
        }
        let mut n =
            (&world_coords[2] - &world_coords[0]).cross(&world_coords[1] - &world_coords[0]);
        n = n.normalize();
        let intensity = n.dot(&light_dir);
        if intensity > 0.0 {
            let r = (intensity * 255.0) as u32;
            let g = (intensity * 255.0) as u32;
            let b = (intensity * 255.0) as u32;
            let mut pixel = 0xff000000;
            pixel |= r & 0xff;
            pixel |= (g & 0xff) << 8;
            pixel |= (b & 0xff) << (2 * 8);
            renderer.fill_triangle(s[0].0, s[0].1, s[1].0, s[1].1, s[2].0, s[2].1, pixel);
        }
    }
    renderer.save_to_ppm_file("output/black.ppm").unwrap();
}
