use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::geometry::{Vector2, Vector3};

pub struct Model {
    verts: Vec<Vector3>,     // array of vertices
    tex_coord: Vec<Vector2>, // per-vertex array of tex coords
    norms: Vec<Vector3>,     // per-vertex array of normal vectors
    facet_vrt: Vec<usize>,
    facet_tex: Vec<usize>, // per-triangle indices in the above arrays
    facet_nrm: Vec<usize>,
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
                    uv[1] = 1f32 - uv[1];
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
        }
    }
    pub fn nverts(&self) -> usize {
        self.verts.len()
    }
    pub fn nfaces(&self) -> usize {
        self.facet_vrt.len() / 3
    }
    pub fn vert(&self, iface: usize, nthvert: usize) -> Vector3 {
        self.verts[self.facet_vrt[iface * 3 + nthvert]].clone()
    }
}