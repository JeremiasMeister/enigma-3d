use crate::geometry;

pub const TRIANGLE: [geometry::Vertex; 3] = [
    geometry::Vertex { position: [-0.5, 0.5, 0.0] , color: [1.0, 0.0, 0.0], texcoord: [0.0,0.0], normal: [1.0,0.0,1.0], index: 0 },
    geometry::Vertex { position: [0.0, 0.9, 0.0], color: [0.0, 1.0, 0.0], texcoord: [0.5,1.0] , normal: [1.0,0.0,1.0], index: 1 },
    geometry::Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0], texcoord: [0.0,0.0], normal: [1.0,0.0,1.0], index: 2 },
];

pub const SQUARE: [geometry::Vertex; 6] = [
    geometry::Vertex { position: [-0.5, -0.8, 0.0], color: [1.0, 0.0, 0.0], texcoord: [0.0,0.0], normal: [0.5,0.0,1.0], index: 0 },
    geometry::Vertex { position: [0.5, -0.8, 0.0] , color: [0.0, 1.0, 0.0], texcoord: [0.0,1.0], normal: [0.5,0.0,1.0], index: 1 },
    geometry::Vertex { position: [0.5, 0.5, 0.0] , color: [0.0, 0.0, 1.0], texcoord: [1.0,0.0], normal: [0.5,0.0,1.0], index: 2 },
    geometry::Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 1.0, 1.0], texcoord: [1.0,0.0], normal: [0.5,0.0,1.0], index: 3 },
    geometry::Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 0.0], texcoord: [0.0,1.0], normal: [0.5,0.0,1.0], index: 4 },
    geometry::Vertex { position: [-0.5, -0.8, 0.0], color: [1.0, 0.0, 1.0], texcoord: [0.0,0.0], normal: [0.5,0.0,1.0], index: 5 },
];

pub fn get_debug_shapes() -> Vec<&'static [geometry::Vertex]> {
    vec![&SQUARE, &TRIANGLE]
}