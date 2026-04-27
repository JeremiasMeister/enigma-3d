use glium::{Display, Texture2d};
use glium::framebuffer::DepthRenderBuffer;
use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;

pub struct ShadowMaps {
    pub directional_maps: [Option<Texture2d>; 4],
    pub point_maps: [Option<Texture2d>; 4],
    pub light_space_matrices: [[[f32; 4]; 4]; 4],
    pub point_far_planes: [f32; 4],
    pub resolution: u32,
    pub dir_depth_rb: DepthRenderBuffer,
    pub point_depth_rb: DepthRenderBuffer,
    pub dummy: Texture2d,
}

const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

impl ShadowMaps {
    pub fn new(display: &Display<WindowSurface>, resolution: u32) -> Self {
        let dir_depth_rb = DepthRenderBuffer::new(
            display,
            glium::texture::DepthFormat::F32,
            resolution,
            resolution,
        ).expect("Failed to create directional shadow depth renderbuffer");

        let point_depth_rb = DepthRenderBuffer::new(
            display,
            glium::texture::DepthFormat::F32,
            resolution * 2,
            resolution * 3,
        ).expect("Failed to create point shadow depth renderbuffer");

        let dummy = Texture2d::new(
            display,
            RawImage2d::from_raw_rgba_reversed(&[255u8, 0, 0, 255], (1, 1)),
        ).expect("Failed to create dummy shadow texture");

        Self {
            directional_maps: [None, None, None, None],
            point_maps: [None, None, None, None],
            light_space_matrices: [IDENTITY; 4],
            point_far_planes: [100.0; 4],
            resolution,
            dir_depth_rb,
            point_depth_rb,
            dummy,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..4 {
            self.directional_maps[i] = None;
            self.point_maps[i] = None;
        }
    }
}

// --- Matrix helpers (column-major, matching camera.rs) ---

pub fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let len = (direction[0]*direction[0] + direction[1]*direction[1] + direction[2]*direction[2]).sqrt();
    let f = [-direction[0]/len, -direction[1]/len, -direction[2]/len];

    let s = [
        up[1]*f[2] - up[2]*f[1],
        up[2]*f[0] - up[0]*f[2],
        up[0]*f[1] - up[1]*f[0],
    ];
    let s_len = (s[0]*s[0] + s[1]*s[1] + s[2]*s[2]).sqrt();
    let s = [s[0]/s_len, s[1]/s_len, s[2]/s_len];

    let u = [
        f[1]*s[2] - f[2]*s[1],
        f[2]*s[0] - f[0]*s[2],
        f[0]*s[1] - f[1]*s[0],
    ];

    let p = [
        -position[0]*s[0] - position[1]*s[1] - position[2]*s[2],
        -position[0]*u[0] - position[1]*u[1] - position[2]*u[2],
        -position[0]*f[0] - position[1]*f[1] - position[2]*f[2],
    ];

    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

pub fn ortho_matrix(l: f32, r: f32, b: f32, t: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    [
        [2.0/(r-l), 0.0, 0.0, 0.0],
        [0.0, 2.0/(t-b), 0.0, 0.0],
        [0.0, 0.0, -2.0/(far-near), 0.0],
        [-(r+l)/(r-l), -(t+b)/(t-b), -(far+near)/(far-near), 1.0],
    ]
}

pub fn perspective_90_matrix(near: f32, far: f32) -> [[f32; 4]; 4] {
    // f = 1.0/tan(45°) = 1.0, aspect = 1.0
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, (far+near)/(near-far), -1.0],
        [0.0, 0.0, (2.0*far*near)/(near-far), 0.0],
    ]
}

pub fn mat4_mul(a: [[f32;4];4], b: [[f32;4];4]) -> [[f32;4];4] {
    let mut r = [[0.0f32;4];4];
    for col in 0..4 {
        for row in 0..4 {
            r[col][row] = a[0][row]*b[col][0] + a[1][row]*b[col][1]
                        + a[2][row]*b[col][2] + a[3][row]*b[col][3];
        }
    }
    r
}

pub fn directional_light_space_matrix(
    light_dir: [f32; 3],
    cam_pos: [f32; 3],
    half_extent: f32,
) -> [[f32; 4]; 4] {
    let len = (light_dir[0]*light_dir[0] + light_dir[1]*light_dir[1] + light_dir[2]*light_dir[2]).sqrt();
    let dir = [light_dir[0]/len, light_dir[1]/len, light_dir[2]/len];
    let pos = [
        cam_pos[0] - dir[0] * half_extent,
        cam_pos[1] - dir[1] * half_extent,
        cam_pos[2] - dir[2] * half_extent,
    ];
    let up = if dir[1].abs() < 0.99 { [0.0f32, 1.0, 0.0] } else { [1.0f32, 0.0, 0.0] };
    let view = view_matrix(&pos, &dir, &up);
    let ortho = ortho_matrix(-half_extent, half_extent, -half_extent, half_extent, 0.1, half_extent * 2.0);
    mat4_mul(ortho, view)
}

// Cube face directions and ups, ordered: +X -X +Y -Y +Z -Z
pub const CUBE_FACE_DIRS: [([f32;3], [f32;3]); 6] = [
    ([1.0, 0.0, 0.0],  [0.0, -1.0, 0.0]),
    ([-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
    ([0.0, 1.0, 0.0],  [0.0, 0.0, 1.0]),
    ([0.0, -1.0, 0.0], [0.0, 0.0, -1.0]),
    ([0.0, 0.0, 1.0],  [0.0, -1.0, 0.0]),
    ([0.0, 0.0, -1.0], [0.0, -1.0, 0.0]),
];

// Atlas viewport for face i in a 2*res x 3*res texture (y=0 at bottom)
pub fn face_viewport(face: usize, res: u32) -> glium::Rect {
    let col = (face % 2) as u32;
    let row = (face / 2) as u32;
    glium::Rect {
        left: col * res,
        bottom: row * res,
        width: res,
        height: res,
    }
}
