use crate::geometry::Vertex;

pub const TRIANGLE: [Vertex; 3] = [
    Vertex { position: [-0.5, 0.52] },
    Vertex { position: [0.0, 0.9] },
    Vertex { position: [0.5, 0.52] },
];

pub const SQUARE: [Vertex; 6] = [
    Vertex { position: [-0.5, -0.8] },
    Vertex { position: [0.5, -0.8] },
    Vertex { position: [0.5, 0.5] },
    Vertex { position: [0.5, 0.5] },
    Vertex { position: [-0.5, 0.5] },
    Vertex { position: [-0.5, -0.8] },
];

pub fn get_debug_shapes() -> Vec<&'static [Vertex]> {
    vec![&SQUARE, &TRIANGLE]
}