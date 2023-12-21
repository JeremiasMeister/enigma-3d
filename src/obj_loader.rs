use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::geometry::Vertex;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct VertexData {
    pos_idx: usize,
    tex_idx: Option<usize>,
    norm_idx: Option<usize>,
}

pub fn parse_obj_file(path: &str) -> io::Result<Vec<Vertex>> {
    let file = File::open(Path::new(path))?;
    let reader = io::BufReader::new(file);

    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut texcoords = Vec::new();
    let mut normals = Vec::new();
    let mut vertex_data_map = HashMap::new();
    let mut vertices = Vec::new();
    let mut next_index = 0;

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                let pos = [
                    parse_or_warn(parts.get(1), 0.0, line_number, "position"),
                    parse_or_warn(parts.get(2), 0.0, line_number, "position"),
                    parse_or_warn(parts.get(3), 0.0, line_number, "position"),
                ];
                let color = if parts.len() >= 7 {
                    [
                        parse_or_warn(parts.get(4), 1.0, line_number, "color"),
                        parse_or_warn(parts.get(5), 1.0, line_number, "color"),
                        parse_or_warn(parts.get(6), 1.0, line_number, "color"),
                    ]
                } else {
                    println!("Warning: Default color used at line {}", line_number + 1);
                    [1.0, 1.0, 1.0] // Default color if not specified
                };
                positions.push(pos);
                colors.push(color);
            }
            "vt" => {
                let tex = [
                    parse_or_warn(parts.get(1), 0.0, line_number, "texture coordinate"),
                    parse_or_warn(parts.get(2), 0.0, line_number, "texture coordinate"),
                ];
                texcoords.push(tex);
            }
            "vn" => {
                let norm = [
                    parse_or_warn(parts.get(1), 0.0, line_number, "normal"),
                    parse_or_warn(parts.get(2), 0.0, line_number, "normal"),
                    parse_or_warn(parts.get(3), 0.0, line_number, "normal"),
                ];
                normals.push(norm);
            }
            "f" => {
                for i in 1..=3 {
                    let indices: Vec<&str> = parts[i].split('/').collect();
                    let pos_idx = indices[0].parse::<usize>().unwrap_or(0) - 1;
                    let tex_idx = indices.get(1).and_then(|&s| s.parse::<usize>().ok());
                    let norm_idx = indices.get(2).and_then(|&s| s.parse::<usize>().ok());

                    let vertex_data = VertexData {
                        pos_idx,
                        tex_idx: tex_idx.map(|idx| idx - 1),
                        norm_idx: norm_idx.map(|idx| idx - 1),
                    };

                    let _ = *vertex_data_map.entry(vertex_data).or_insert_with(|| {
                        let index = next_index;
                        next_index += 1;

                        let position = positions[vertex_data.pos_idx];
                        let color = colors[vertex_data.pos_idx];
                        let texcoord = tex_idx
                            .and_then(|idx| texcoords.get(idx))
                            .unwrap_or(&[0.0, 0.0]);
                        let normal = norm_idx
                            .and_then(|idx| normals.get(idx))
                            .unwrap_or(&[0.0, 0.0, 0.0]);

                        vertices.push(Vertex {
                            position,
                            texcoord: *texcoord,
                            color,
                            normal: *normal,
                            index: index as u32,
                        });

                        index
                    });
                }
            }
            _ => {}
        }
    }

    Ok(vertices)
}

fn parse_or_warn(value: Option<&&str>, fallback: f32, line_number: usize, attribute: &str) -> f32 {
    match value {
        Some(val) => val.parse::<f32>().unwrap_or_else(|_| {
            println!("Warning: Invalid {} at line {}, using fallback {}", attribute, line_number + 1, fallback);
            fallback
        }),
        None => {
            println!("Warning: Missing {} at line {}, using fallback {}", attribute, line_number + 1, fallback);
            fallback
        }
    }
}