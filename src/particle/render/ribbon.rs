use glium::{Display, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::implement_vertex;
use crate::camera::Camera;
use crate::logging::EnigmaError;
use crate::material::Material;
use crate::particle::ParticleSystem;
use crate::particle::config::RenderStyle;
use super::sprite::blend_params;

#[derive(Copy, Clone)]
pub struct RibbonVertex {
    pub position: [f32; 3],
    pub uv:       [f32; 2],
    pub color:    [f32; 4],
}
implement_vertex!(RibbonVertex, position, uv, color);

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if l < 1e-6 { [0.0, 0.0, 0.0] } else { [v[0]/l, v[1]/l, v[2]/l] }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

pub fn render_one<S: Surface>(
    display: &Display<WindowSurface>,
    target: &mut S,
    camera: &Camera,
    system: &ParticleSystem,
    material: &Material,
    renderer: &mut super::ParticleRenderer,
) {
    let (max_pts, width_curve, blend_mode) = match &system.config.render {
        RenderStyle::Ribbon { max_trail_points, width_over_life, blend_mode } => {
            (*max_trail_points, width_over_life.clone(), *blend_mode)
        }
        _ => return,
    };
    let trails = match &system.buffer.trail { Some(t) => t, None => return };
    if trails.is_empty() { return; }

    let cam_pos = camera.get_position();
    let mut verts: Vec<RibbonVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..system.live_count() {
        let trail = &trails[i];
        if trail.len() < 2 { continue; }
        let color = system.buffer.color[i];
        let base_size = system.buffer.size[i];

        for s in 0..trail.len() - 1 {
            let p0 = trail[s];
            let p1 = trail[s + 1];
            let tangent = normalize([p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]]);
            let view_dir0 = normalize([cam_pos[0]-p0[0], cam_pos[1]-p0[1], cam_pos[2]-p0[2]]);
            let side0 = normalize(cross(tangent, view_dir0));
            let view_dir1 = normalize([cam_pos[0]-p1[0], cam_pos[1]-p1[1], cam_pos[2]-p1[2]]);
            let side1 = normalize(cross(tangent, view_dir1));

            let denom = (max_pts.max(1) - 1).max(1) as f32;
            let t0 = s as f32 / denom;
            let t1 = (s as f32 + 1.0) / denom;
            let w0 = base_size * width_curve.eval(t0);
            let w1 = base_size * width_curve.eval(t1);

            let base_idx = verts.len() as u32;
            verts.push(RibbonVertex { position: [p0[0]+side0[0]*w0, p0[1]+side0[1]*w0, p0[2]+side0[2]*w0], uv: [0.0, t0], color });
            verts.push(RibbonVertex { position: [p0[0]-side0[0]*w0, p0[1]-side0[1]*w0, p0[2]-side0[2]*w0], uv: [1.0, t0], color });
            verts.push(RibbonVertex { position: [p1[0]+side1[0]*w1, p1[1]+side1[1]*w1, p1[2]+side1[2]*w1], uv: [0.0, t1], color });
            verts.push(RibbonVertex { position: [p1[0]-side1[0]*w1, p1[1]-side1[1]*w1, p1[2]-side1[2]*w1], uv: [1.0, t1], color });
            indices.extend_from_slice(&[base_idx, base_idx+1, base_idx+2, base_idx+1, base_idx+3, base_idx+2]);
        }
    }
    if verts.is_empty() { return; }

    // Reuse persistent dynamic VB+IB. Grow on demand to next-power-of-two so steady
    // state is zero-alloc; the buffers never shrink.
    let v_needed = verts.len();
    let i_needed = indices.len();
    let vb_ok = renderer.ribbon_vb.as_ref().map(|v| v.len() >= v_needed).unwrap_or(false);
    if !vb_ok {
        let new_cap = v_needed.next_power_of_two().max(256);
        match VertexBuffer::<RibbonVertex>::empty_dynamic(display, new_cap) {
            Ok(vb) => renderer.ribbon_vb = Some(vb),
            Err(e) => {
                EnigmaError::new(Some(&format!("particle ribbon VB alloc (cap={}): {}", new_cap, e)), false).log();
                return;
            }
        }
    }
    let ib_ok = renderer.ribbon_ib.as_ref().map(|i| i.len() >= i_needed).unwrap_or(false);
    if !ib_ok {
        let new_cap = i_needed.next_power_of_two().max(384);
        match IndexBuffer::<u32>::empty_dynamic(display, glium::index::PrimitiveType::TrianglesList, new_cap) {
            Ok(ib) => renderer.ribbon_ib = Some(ib),
            Err(e) => {
                EnigmaError::new(Some(&format!("particle ribbon IB alloc (cap={}): {}", new_cap, e)), false).log();
                return;
            }
        }
    }
    let vb = renderer.ribbon_vb.as_ref().expect("ribbon_vb just sized");
    let ib = renderer.ribbon_ib.as_ref().expect("ribbon_ib just sized");
    vb.slice(0..v_needed).expect("ribbon VB slice fits").write(&verts);
    ib.slice(0..i_needed).expect("ribbon IB slice fits").write(&indices);

    let albedo_sampler = match material.albedo.as_ref() {
        Some(t) => &t.texture,
        None => &renderer.fallback_albedo,
    }.sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

    let view = camera.get_view_matrix();
    let proj = camera.get_projection_matrix();
    let uniforms = glium::uniform! {
        view_matrix: view,
        projection_matrix: proj,
        albedo: albedo_sampler,
    };
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: false,
            ..Default::default()
        },
        blend: blend_params(blend_mode),
        ..Default::default()
    };
    let vb_slice = vb.slice(0..v_needed).expect("ribbon VB draw slice");
    let ib_slice = ib.slice(0..i_needed).expect("ribbon IB draw slice");
    if let Err(e) = target.draw(vb_slice, ib_slice, &material.program, &uniforms, &params) {
        EnigmaError::new(Some(&format!("particle ribbon draw: {}", e)), false).log();
    }
}
