use crate::document::{Document, Kind, LineMode, Point};
use egui::{Pos2, Rect};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

pub struct Viewport {
    pub rect: Rect,
    pub scale: f32,
    pub center: Point,
}
impl Viewport {
    pub fn screen(&self, p: Point) -> Pos2 {
        self.rect.center()
            + egui::vec2(
                (p.x - self.center.x) as f32 * self.scale,
                (p.y - self.center.y) as f32 * self.scale,
            )
    }
    pub fn world(&self, p: Pos2) -> Point {
        let d = p - self.rect.center();
        Point {
            x: self.center.x + d.x as f64 / self.scale as f64,
            y: self.center.y + d.y as f64 / self.scale as f64,
        }
    }
}
fn clipped_line(a: Pos2, b: Pos2, mode: LineMode, rect: Rect) -> Option<(Pos2, Pos2)> {
    let d = b - a;
    if d.length_sq() < 1e-8 {
        return None;
    }
    let (mut lo, mut hi) = match mode {
        LineMode::Segment => (0., 1.),
        LineMode::Ray => (0., f32::INFINITY),
        LineMode::Line => (f32::NEG_INFINITY, f32::INFINITY),
    };
    for (origin, delta, min, max) in [
        (a.x, d.x, rect.min.x, rect.max.x),
        (a.y, d.y, rect.min.y, rect.max.y),
    ] {
        if delta.abs() < 1e-8 {
            if origin < min || origin > max {
                return None;
            }
            continue;
        }
        let t0 = (min - origin) / delta;
        let t1 = (max - origin) / delta;
        lo = lo.max(t0.min(t1));
        hi = hi.min(t0.max(t1));
    }
    (lo <= hi && lo.is_finite() && hi.is_finite()).then_some((a + d * lo, a + d * hi))
}
struct Mesh {
    vertices: Vec<Vertex>,
    width: f32,
    height: f32,
    ppp: f32,
}
impl Mesh {
    fn v(&self, p: Pos2, c: [f32; 4]) -> Vertex {
        Vertex {
            pos: [
                2. * p.x * self.ppp / self.width - 1.,
                1. - 2. * p.y * self.ppp / self.height,
            ],
            color: c,
        }
    }
    fn tri(&mut self, a: Pos2, b: Pos2, c: Pos2, col: [f32; 4]) {
        self.vertices
            .extend_from_slice(&[self.v(a, col), self.v(b, col), self.v(c, col)]);
    }
    fn line(&mut self, a: Pos2, b: Pos2, w: f32, col: [f32; 4]) {
        let d = b - a;
        let n = d.length();
        if n < 0.01 {
            return;
        }
        let off = egui::vec2(-d.y / n, d.x / n) * w / 2.;
        self.tri(a + off, b + off, b - off, col);
        self.tri(a + off, b - off, a - off, col);
    }
    fn disk(&mut self, p: Pos2, r: f32, col: [f32; 4]) {
        for i in 0..20 {
            let a = i as f32 * std::f32::consts::TAU / 20.;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 20.;
            self.tri(
                p,
                p + egui::vec2(a.cos(), a.sin()) * r,
                p + egui::vec2(b.cos(), b.sin()) * r,
                col,
            );
        }
    }
}
pub struct GeometryRenderer {
    pipeline: wgpu::RenderPipeline,
}
impl GeometryRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sketch geometry"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sketch geometry"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self { pipeline }
    }
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        doc: &Document,
        vp: &Viewport,
        size: [u32; 2],
        ppp: f32,
    ) {
        let mut mesh = Mesh {
            vertices: Vec::new(),
            width: size[0] as f32,
            height: size[1] as f32,
            ppp,
        };
        let dark = [0.14, 0.18, 0.24, 1.];
        let blue = [0.09, 0.42, 0.84, 1.];
        let selected = [0.94, 0.32, 0.10, 1.];
        // Draw paths first and points last.
        for o in &doc.objects {
            if !o.visible || o.page != doc.active_page {
                continue;
            }
            let col = if doc.selected.contains(&o.id) {
                selected
            } else {
                dark
            };
            if let Some((a, b, mode)) = doc.line_points(o.id) {
                if let Some((a, b)) = clipped_line(vp.screen(a), vp.screen(b), mode, vp.rect) {
                    mesh.line(a, b, 2., col);
                }
            }
            match o.kind {
                Kind::Circle(a, b) => {
                    if let (Some(a), Some(b)) = (doc.position(a), doc.position(b)) {
                        let c = vp.screen(a);
                        let r = (a.distance(b) as f32 * vp.scale).abs();
                        if r > 0.5 && r < 100000. {
                            let n = ((r * 0.45) as usize).clamp(32, 256);
                            for i in 0..n {
                                let t = i as f32 * std::f32::consts::TAU / n as f32;
                                let u = (i + 1) as f32 * std::f32::consts::TAU / n as f32;
                                mesh.line(
                                    c + egui::vec2(t.cos(), t.sin()) * r,
                                    c + egui::vec2(u.cos(), u.sin()) * r,
                                    2.,
                                    col,
                                );
                            }
                        }
                    }
                }
                Kind::ThreePointArc(_, _, _) => {
                    if let Some(points) = doc.arc_points(o.id, 128) {
                        for pair in points.windows(2) {
                            mesh.line(vp.screen(pair[0]), vp.screen(pair[1]), 2., col);
                        }
                    }
                }
                _ => {}
            }
        }
        for o in &doc.objects {
            if !o.visible || o.page != doc.active_page {
                continue;
            }
            if let Some(p) = doc.position(o.id) {
                mesh.disk(
                    vp.screen(p),
                    if doc.selected.contains(&o.id) {
                        5.5
                    } else {
                        4.
                    },
                    if doc.selected.contains(&o.id) {
                        selected
                    } else {
                        blue
                    },
                );
            }
        }
        let buffer = (!mesh.vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sketch vertices"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sketch"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.96,
                        g: 0.97,
                        b: 0.985,
                        a: 1.,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if let Some(buffer) = buffer {
            let x = (vp.rect.min.x * ppp).max(0.) as u32;
            let y = (vp.rect.min.y * ppp).max(0.) as u32;
            let right = (vp.rect.max.x * ppp).min(size[0] as f32) as u32;
            let bottom = (vp.rect.max.y * ppp).min(size[1] as f32) as u32;
            if right > x && bottom > y {
                pass.set_scissor_rect(x, y, right - x, bottom - y);
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..mesh.vertices.len() as u32, 0..1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clips_line_ray_and_segment() {
        let r = Rect::from_min_max(Pos2::new(0., 0.), Pos2::new(100., 100.));
        let a = Pos2::new(50., 50.);
        let b = Pos2::new(60., 50.);
        assert_eq!(
            clipped_line(a, b, LineMode::Line, r),
            Some((Pos2::new(0., 50.), Pos2::new(100., 50.)))
        );
        assert_eq!(
            clipped_line(a, b, LineMode::Ray, r),
            Some((a, Pos2::new(100., 50.)))
        );
        assert_eq!(clipped_line(a, b, LineMode::Segment, r), Some((a, b)));
        assert_eq!(clipped_line(a, a, LineMode::Line, r), None);
    }
}
