use crate::geometry::Vertex;

pub const TRIANGLE: [Vertex; 3] = [
    Vertex { position: [-0.5, 0.5, 0.0] , color: [1.0, 0.0, 0.0], texcoord: [0.0,0.0] },
    Vertex { position: [0.0, 0.9, 0.0], color: [0.0, 1.0, 0.0], texcoord: [0.0,0.0] },
    Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0], texcoord: [0.0,0.0] },
];

pub const SQUARE: [Vertex; 6] = [
    Vertex { position: [-0.5, -0.8, 0.0], color: [1.0, 0.0, 0.0], texcoord: [0.0,0.0] },
    Vertex { position: [0.5, -0.8, 0.0] , color: [0.0, 1.0, 0.0], texcoord: [0.0,1.0] },
    Vertex { position: [0.5, 0.5, 0.0] , color: [0.0, 0.0, 1.0], texcoord: [1.0,0.0] },
    Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 1.0, 1.0], texcoord: [1.0,0.0] },
    Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 0.0], texcoord: [0.0,1.0] },
    Vertex { position: [-0.5, -0.8, 0.0], color: [1.0, 0.0, 1.0], texcoord: [0.0,0.0] },
];

pub fn get_debug_shapes() -> Vec<&'static [Vertex]> {
    vec![&SQUARE, &TRIANGLE]
}