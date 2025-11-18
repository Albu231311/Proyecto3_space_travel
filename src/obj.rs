use std::fs::File;
use std::io::{BufRead, BufReader};
use nalgebra_glm::{Vec2, Vec3};
use crate::vertex::Vertex;

pub struct Obj {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Obj {
    pub fn load(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "v" if parts.len() >= 4 => {
                    let x: f32 = parts[1].parse()?;
                    let y: f32 = parts[2].parse()?;
                    let z: f32 = parts[3].parse()?;
                    positions.push(Vec3::new(x, y, z));
                }
                "vn" if parts.len() >= 4 => {
                    let x: f32 = parts[1].parse()?;
                    let y: f32 = parts[2].parse()?;
                    let z: f32 = parts[3].parse()?;
                    normals.push(Vec3::new(x, y, z));
                }
                "vt" if parts.len() >= 3 => {
                    let u: f32 = parts[1].parse()?;
                    let v: f32 = parts[2].parse()?;
                    tex_coords.push(Vec2::new(u, v));
                }
                "f" if parts.len() >= 4 => {
                    // Parse face (assuming triangles)
                    for i in 1..4 {
                        let face_data: Vec<&str> = parts[i].split('/').collect();
                        let pos_idx: usize = face_data[0].parse::<usize>()? - 1;
                        
                        let position = positions.get(pos_idx).copied().unwrap_or(Vec3::new(0.0, 0.0, 0.0));
                        let normal = if face_data.len() > 2 && !face_data[2].is_empty() {
                            let norm_idx: usize = face_data[2].parse::<usize>()? - 1;
                            normals.get(norm_idx).copied().unwrap_or(Vec3::new(0.0, 1.0, 0.0))
                        } else {
                            Vec3::new(0.0, 1.0, 0.0)
                        };
                        
                        let tex_coord = if face_data.len() > 1 && !face_data[1].is_empty() {
                            let tex_idx: usize = face_data[1].parse::<usize>()? - 1;
                            tex_coords.get(tex_idx).copied().unwrap_or(Vec2::new(0.0, 0.0))
                        } else {
                            Vec2::new(0.0, 0.0)
                        };
                        
                        vertices.push(Vertex::new(position, normal, tex_coord));
                        indices.push(vertices.len() as u32 - 1);
                    }
                }
                _ => {}
            }
        }
        
        println!("Modelo cargado: {} vértices, {} triángulos", vertices.len(), indices.len() / 3);
        
        Ok(Obj { vertices, indices })
    }
    
    pub fn get_vertex_and_index_arrays(&self) -> (Vec<Vertex>, Vec<u32>) {
        (self.vertices.clone(), self.indices.clone())
    }
}