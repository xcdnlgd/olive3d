use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{
    geometry::{Vector2, Vector3},
    ppm::{load_ppm_file_to_buffer, Image},
};

pub struct Model {
    verts: Vec<Vector3>,     // array of vertices
    tex_coord: Vec<Vector2>, // per-vertex array of tex coords
    norms: Vec<Vector3>,     // per-vertex array of normal vectors
    facet_vrt: Vec<usize>,
    facet_tex: Vec<usize>, // per-triangle indices in the above arrays
    facet_nrm: Vec<usize>,
    diffuse_map: Option<Image>,
    normal_map: Option<Image>,
}

macro_rules! load_map {
    ($func_name:ident, $map_field:ident) => {
        pub fn $func_name(&mut self, path: impl AsRef<std::path::Path>) {
            let mut img = load_ppm_file_to_buffer(path);
            img.vflip();
            self.$map_field = Some(img);
        }
    };
}

impl Model {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let file = File::open(path).unwrap();
        let file = BufReader::new(file);
        let mut verts = Vec::new();
        let mut norms = Vec::new();
        let mut tex_coord = Vec::new();
        let mut facet_vrt = Vec::new();
        let mut facet_tex = Vec::new();
        let mut facet_nrm = Vec::new();

        for line in file
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| !line.trim().is_empty())
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts[0] {
                "v" => {
                    let mut v = Vector3::zero();
                    for i in 0..3 {
                        v[i] = parts[i + 1].parse().unwrap();
                    }
                    verts.push(v);
                }
                "vn" => {
                    let mut n = Vector3::zero();
                    for i in 0..3 {
                        n[i] = parts[i + 1].parse().unwrap();
                    }
                    norms.push(n);
                }
                "vt" => {
                    let mut uv = Vector2::zero();
                    for i in 0..2 {
                        uv[i] = parts[i + 1].parse().unwrap();
                    }
                    tex_coord.push(uv);
                }
                "f" => {
                    for i in 0..3 {
                        let v: Vec<&str> = parts[i + 1].split('/').collect();
                        facet_vrt.push(v[0].parse::<usize>().unwrap() - 1);
                        facet_tex.push(v[1].parse::<usize>().unwrap() - 1);
                        facet_nrm.push(v[2].parse::<usize>().unwrap() - 1);
                    }
                }
                _ => {}
            }
        }
        println!(
            "# v# {} f# {} vt# {} vn# {}",
            verts.len(),
            facet_vrt.len() / 3,
            tex_coord.len(),
            norms.len()
        );
        Self {
            verts,
            norms,
            tex_coord,
            facet_vrt,
            facet_tex,
            facet_nrm,
            diffuse_map: None,
            normal_map: None,
        }
    }
    load_map!(load_diffuse_map, diffuse_map);
    load_map!(load_normal_map, normal_map);
    pub fn nverts(&self) -> usize {
        self.verts.len()
    }
    pub fn nfaces(&self) -> usize {
        self.facet_vrt.len() / 3
    }
    pub fn vert(&self, iface: usize, nthvert: usize) -> Vector3 {
        self.verts[self.facet_vrt[iface * 3 + nthvert]].clone()
    }
    pub fn uv(&self, iface: usize, nthvert: usize) -> Vector2 {
        self.tex_coord[self.facet_tex[iface * 3 + nthvert]].clone()
    }
    pub fn normal_vert(&self, iface: usize, nthvert: usize) -> Vector3 {
        self.norms[self.facet_nrm[iface * 3 + nthvert]].clone()
    }
    pub fn normal_uv(&self, uv: &Vector2) -> Vector3 {
        if let Some(ref normal_map) = self.normal_map {
            let x = uv.x() * normal_map.width as f32;
            let y = uv.y() * normal_map.height as f32;
            let pixel = normal_map.buffer[x as usize + y as usize * normal_map.width as usize];
            if pixel == 0xff000000 {
                Vector3::zero()
            } else {
                let r = (pixel & 0xff) as f32;
                let g = ((pixel >> 8) & 0xff) as f32;
                let b = ((pixel >> 16) & 0xff) as f32;
                Vector3::new(r, g, b) * 2.0 / 255.0 - Vector3::new(1.0, 1.0, 1.0)
            }
        } else {
            Vector3::zero()
        }
    }
    pub fn diffuse(&self, uv: &Vector2) -> u32 {
        let pixel: u32 = if let Some(ref diffuse_map) = self.diffuse_map {
            let x = uv.x() * diffuse_map.width as f32;
            let y = uv.y() * diffuse_map.height as f32;
            diffuse_map.buffer[x as usize + y as usize * diffuse_map.width as usize]
        } else {
            0xffffffff
        };
        pixel
    }
}
