use sketchpad_rs::document::{Document, Kind, LineMode, Point};
use sketchpad_rs::render::{GeometryRenderer, Viewport};
use std::{path::PathBuf, sync::Arc};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Point,
    Segment,
    Circle,
}
struct Editor {
    doc: Document,
    tool: Tool,
    pending: Option<usize>,
    dragging: Option<usize>,
    scale: f32,
    center: Point,
    status: String,
}
impl Editor {
    fn new(doc: Document) -> Self {
        let mut e = Self {
            doc,
            tool: Tool::Select,
            pending: None,
            dragging: None,
            scale: 1.,
            center: Point { x: 400., y: 300. },
            status: String::new(),
        };
        e.fit();
        e
    }
    fn fit(&mut self) {
        let pts: Vec<_> = self
            .doc
            .objects
            .iter()
            .filter(|o| o.page == self.doc.active_page && o.visible)
            .filter_map(|o| self.doc.position(o.id))
            .collect();
        if pts.is_empty() {
            return;
        }
        let (mut minx, mut miny, mut maxx, mut maxy) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in pts {
            minx = minx.min(p.x);
            miny = miny.min(p.y);
            maxx = maxx.max(p.x);
            maxy = maxy.max(p.y);
        }
        self.center = Point {
            x: (minx + maxx) / 2.,
            y: (miny + maxy) / 2.,
        };
        self.scale = (650. / (maxx - minx).max(100.) as f32)
            .min(480. / (maxy - miny).max(100.) as f32)
            .clamp(0.2, 3.);
    }
    fn open(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(d) => {
                *self = Self::new(d);
                self.status = format!("Opened {}", path.display());
            }
            Err(e) => self.status = format!("Open failed: {e}"),
        }
    }
    fn save(&mut self, as_new: bool) {
        let path = if as_new || self.doc.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Sketchpad document", &["gsp"])
                .set_file_name("Sketch.gsp")
                .save_file()
        } else {
            self.doc.path.clone()
        };
        if let Some(path) = path {
            self.status = match self.doc.save_to(&path) {
                Ok(()) => format!("Saved {}", path.display()),
                Err(e) => format!("Save refused: {e}"),
            };
        }
    }
    fn hit_point(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        self.doc
            .objects
            .iter()
            .rev()
            .filter(|o| o.page == self.doc.active_page && o.visible)
            .filter_map(|o| {
                self.doc
                    .position(o.id)
                    .map(|at| (o.id, v.screen(at).distance(p)))
            })
            .filter(|(_, d)| *d < 11.)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0)
    }
    fn hit(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        if let Some(id) = self.hit_point(v, p) {
            return Some(id);
        }
        self.doc
            .objects
            .iter()
            .rev()
            .filter(|o| o.page == self.doc.active_page && o.visible)
            .filter_map(|o| {
                let distance = if let Some((a, b, mode)) = self.doc.line_points(o.id) {
                    let (a, b) = (v.screen(a), v.screen(b));
                    let ab = b - a;
                    let t = if ab.length_sq() < 1e-6 {
                        0.
                    } else {
                        (p - a).dot(ab) / ab.length_sq()
                    };
                    let t = match mode {
                        LineMode::Segment => t.clamp(0., 1.),
                        LineMode::Ray => t.max(0.),
                        LineMode::Line => t,
                    };
                    (p - (a + ab * t)).length()
                } else {
                    match o.kind {
                        Kind::Circle(a, b) => {
                            let (a, b) = (
                                v.screen(self.doc.position(a)?),
                                v.screen(self.doc.position(b)?),
                            );
                            ((p - a).length() - (b - a).length()).abs()
                        }
                        _ => return None,
                    }
                };
                (distance < 7.).then_some((o.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0)
    }
    fn point_at(&mut self, v: &Viewport, p: egui::Pos2) -> usize {
        self.hit_point(v, p)
            .unwrap_or_else(|| self.doc.add(Kind::Free(v.world(p))))
    }
    fn click(&mut self, v: &Viewport, p: egui::Pos2, multi: bool) {
        if self.doc.partial_view && self.tool != Tool::Select {
            self.status =
                "This page contains unsupported records; construction cannot be saved safely"
                    .into();
            return;
        }
        match self.tool {
            Tool::Select => {
                let hit = self.hit(v, p);
                if !multi {
                    self.doc.selected.clear();
                }
                if let Some(id) = hit {
                    if multi && !self.doc.selected.insert(id) {
                        self.doc.selected.remove(&id);
                    } else {
                        self.doc.selected.insert(id);
                    }
                }
            }
            Tool::Point => {
                let id = self.point_at(v, p);
                self.doc.selected.clear();
                self.doc.selected.insert(id);
            }
            Tool::Segment | Tool::Circle => {
                let id = self.point_at(v, p);
                if let Some(a) = self.pending.take() {
                    if a != id {
                        let k = if self.tool == Tool::Segment {
                            Kind::Segment(a, id)
                        } else {
                            Kind::Circle(a, id)
                        };
                        let made = self.doc.add(k);
                        self.doc.selected.clear();
                        self.doc.selected.insert(made);
                    }
                } else {
                    self.pending = Some(id);
                }
            }
        }
    }
    fn pair(&mut self, mode: u8) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids: Vec<_> = self
            .doc
            .selected
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        if ids.len() != 2 {
            self.status = "Select exactly two points".into();
            return;
        }
        let (a, b) = (ids[0], ids[1]);
        let kind = match mode {
            0 => Kind::Segment(a, b),
            1 => Kind::Circle(a, b),
            3 => Kind::Line(a, b),
            4 => Kind::Ray(a, b),
            _ => {
                if !self
                    .doc
                    .objects
                    .iter()
                    .any(|o| matches!(o.kind,Kind::Segment(x,y) if (x==a&&y==b)||(x==b&&y==a)))
                {
                    self.doc.add(Kind::Segment(a, b));
                }
                Kind::Midpoint(a, b)
            }
        };
        let id = self.doc.add(kind);
        self.doc.selected.clear();
        self.doc.selected.insert(id);
    }
    fn derived_line(&mut self, perpendicular: bool) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let points: Vec<_> = self
            .doc
            .selected
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        let lines: Vec<_> = self
            .doc
            .selected
            .iter()
            .copied()
            .filter(|id| self.doc.line_points(*id).is_some())
            .collect();
        if points.len() != 1 || lines.len() != 1 || self.doc.selected.len() != 2 {
            self.status = "Select one point and one straight object".into();
            return;
        }
        let kind = if perpendicular {
            Kind::Perpendicular(points[0], lines[0])
        } else {
            Kind::Parallel(points[0], lines[0])
        };
        let id = self.doc.add(kind);
        self.doc.selected.clear();
        self.doc.selected.insert(id);
    }
    fn intersection(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids: Vec<_> = self.doc.selected.iter().copied().collect();
        if ids.len() != 2 {
            self.status = "Select two straight objects, a line and circle, or two circles".into();
            return;
        }
        let (a, b) = (ids[0], ids[1]);
        let kinds = if self.doc.line_points(a).is_some() && self.doc.line_points(b).is_some() {
            vec![Kind::Intersection(a, b)]
        } else if (self.doc.line_points(a).is_some() && self.doc.circle_points(b).is_some())
            || (self.doc.circle_points(a).is_some() && self.doc.line_points(b).is_some())
        {
            let (line, circle) = if self.doc.line_points(a).is_some() {
                (a, b)
            } else {
                (b, a)
            };
            vec![
                Kind::LineCircleIntersection(line, circle, 0),
                Kind::LineCircleIntersection(line, circle, 1),
            ]
        } else if self.doc.circle_points(a).is_some() && self.doc.circle_points(b).is_some() {
            vec![
                Kind::CircleCircleIntersection(a, b, 0),
                Kind::CircleCircleIntersection(a, b, 1),
            ]
        } else {
            self.status = "Select two straight objects, a line and circle, or two circles".into();
            return;
        };
        self.doc.selected.clear();
        for kind in kinds {
            let id = self.doc.add(kind);
            self.doc.selected.insert(id);
        }
    }
    fn ui(&mut self, root: &mut egui::Ui) -> Viewport {
        egui::Panel::top("menu").show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        *self = Self::new(Document::new());
                        ui.close();
                    }
                    if ui.button("Open…").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("Sketchpad document", &["gsp"])
                            .pick_file()
                        {
                            self.open(p);
                        }
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save(false);
                        ui.close();
                    }
                    if ui.button("Save As…").clicked() {
                        self.save(true);
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui
                        .add_enabled(!self.doc.undo.is_empty(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.doc.undo();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.doc.redo.is_empty(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.doc.redo();
                        ui.close();
                    }
                    if ui.button("Select All").clicked() {
                        self.doc.selected = self
                            .doc
                            .objects
                            .iter()
                            .filter(|o| o.page == self.doc.active_page && o.visible)
                            .map(|o| o.id)
                            .collect();
                        ui.close();
                    }
                });
                ui.menu_button("Construct", |ui| {
                    if ui.button("Segment").clicked() {
                        self.pair(0);
                        ui.close();
                    }
                    if ui.button("Ray").clicked() {
                        self.pair(4);
                        ui.close();
                    }
                    if ui.button("Line").clicked() {
                        self.pair(3);
                        ui.close();
                    }
                    if ui.button("Circle by Center and Point").clicked() {
                        self.pair(1);
                        ui.close();
                    }
                    if ui.button("Midpoint").clicked() {
                        self.pair(2);
                        ui.close();
                    }
                    if ui.button("Intersection").clicked() {
                        self.intersection();
                        ui.close();
                    }
                    if ui.button("Parallel Line").clicked() {
                        self.derived_line(false);
                        ui.close();
                    }
                    if ui.button("Perpendicular Line").clicked() {
                        self.derived_line(true);
                        ui.close();
                    }
                });
                ui.menu_button("Display", |ui| {
                    if ui.button("Hide Objects").clicked() {
                        self.doc.set_selected_visibility(false);
                        ui.close();
                    }
                    if ui.button("Show All Hidden").clicked() {
                        self.doc.show_all_hidden();
                        ui.close();
                    }
                    if ui.button("Fit Sketch").clicked() {
                        self.fit();
                        ui.close();
                    }
                });
            });
        });
        egui::Panel::left("tools").min_size(110.).show(root, |ui| {
            ui.heading("Tools");
            if self.doc.page_count > 1 {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.doc.active_page > 0, egui::Button::new("‹"))
                        .clicked()
                    {
                        self.doc.active_page -= 1;
                        self.doc.selected.clear();
                        self.fit();
                    }
                    ui.label(format!(
                        "{}/{}",
                        self.doc.active_page + 1,
                        self.doc.page_count
                    ));
                    if ui
                        .add_enabled(
                            self.doc.active_page + 1 < self.doc.page_count,
                            egui::Button::new("›"),
                        )
                        .clicked()
                    {
                        self.doc.active_page += 1;
                        self.doc.selected.clear();
                        self.fit();
                    }
                });
                ui.separator();
            }
            for (tool, name) in [
                (Tool::Select, "Select"),
                (Tool::Point, "Point"),
                (Tool::Segment, "Segment"),
                (Tool::Circle, "Circle"),
            ] {
                if ui.selectable_label(self.tool == tool, name).clicked() {
                    self.tool = tool;
                    self.pending = None;
                }
            }
            ui.separator();
            ui.label(format!("{} objects", self.doc.objects.len()));
            if self.doc.partial_view {
                ui.colored_label(egui::Color32::DARK_RED, "Partial view: unsupported objects");
            }
            if self.doc.dirty {
                ui.label("Unsaved changes");
            }
        });
        egui::Panel::bottom("status").show(root, |ui| {
            ui.label(if self.status.is_empty() {
                "Drag points. Scroll to zoom. Select two points to construct."
            } else {
                &self.status
            });
        });
        let mut rect = egui::Rect::NOTHING;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                let (r, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                rect = r;
                let vp = Viewport {
                    rect: r,
                    scale: self.scale,
                    center: self.center,
                };
                if response.hovered() {
                    let dy = ui.input(|i| i.smooth_scroll_delta.y);
                    if dy != 0. {
                        self.scale = (self.scale * (dy * 0.001).exp()).clamp(0.05, 20.);
                    }
                }
                if response.drag_started() && self.tool == Tool::Select {
                    if let Some(p) = ui.input(|i| i.pointer.press_origin()) {
                        if let Some(id) = self.hit(&vp, p) {
                            if matches!(self.doc.objects[id - 1].kind, Kind::Free(_)) {
                                self.doc.checkpoint();
                                self.dragging = Some(id);
                            }
                        }
                    }
                }
                if response.dragged() {
                    if let (Some(id), Some(p)) = (self.dragging, response.interact_pointer_pos()) {
                        self.doc.move_point(id, vp.world(p));
                    }
                }
                if response.drag_stopped() {
                    self.dragging = None;
                }
                if response.clicked() {
                    if let Some(p) = response.interact_pointer_pos() {
                        let multi = ui.input(|i| i.modifiers.shift);
                        self.click(&vp, p, multi);
                    }
                }
                for o in self
                    .doc
                    .objects
                    .iter()
                    .filter(|o| o.page == self.doc.active_page && o.visible)
                {
                    if let Some(p) = self.doc.position(o.id) {
                        let at = vp.screen(p);
                        if r.contains(at) {
                            ui.painter().text(
                                at + egui::vec2(7., -9.),
                                egui::Align2::LEFT_TOP,
                                format!("{}", o.id),
                                egui::FontId::proportional(11.),
                                egui::Color32::from_gray(90),
                            );
                        }
                    }
                }
            });
        Viewport {
            rect,
            scale: self.scale,
            center: self.center,
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let doc = match args.get(1).map(PathBuf::from) {
        Some(p) => match Document::open(&p) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not open {}: {e}", p.display());
                Document::new()
            }
        },
        None => Document::new(),
    };
    let mut editor = Editor::new(doc);
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let window: Arc<Window> = Arc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Sketchpad RS")
                    .with_inner_size(winit::dpi::LogicalSize::new(1100., 750.)),
            )
            .unwrap(),
    );
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window.clone()).expect("surface");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("graphics adapter");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("graphics device");
    let mut size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .expect("surface config");
    if let Some(f) = surface
        .get_capabilities(&adapter)
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
    {
        config.format = f;
    }
    surface.configure(&device, &config);
    let ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        ctx.clone(),
        ctx.viewport_id(),
        window.as_ref(),
        Some(window.scale_factor() as f32),
        None,
        None,
    );
    let mut egui_renderer = egui_wgpu::Renderer::new(
        &device,
        config.format,
        egui_wgpu::RendererOptions::default(),
    );
    let geometry = GeometryRenderer::new(&device, config.format);
    window.request_redraw();
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                let response = egui_state.on_window_event(&window, &event);
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(s) => {
                        size = s;
                        if s.width > 0 && s.height > 0 {
                            config.width = s.width;
                            config.height = s.height;
                            surface.configure(&device, &config);
                        }
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        if size.width == 0 || size.height == 0 {
                            return;
                        }
                        let input = egui_state.take_egui_input(&window);
                        let mut viewport = None;
                        let mut output = ctx.run_ui(input, |ui| viewport = Some(editor.ui(ui)));
                        egui_state.handle_platform_output(&window, output.platform_output);
                        let jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
                        let free_textures = output.textures_delta.free.clone();
                        for (id, deltas) in &output.textures_delta.set {
                            for delta in deltas {
                                egui_renderer.update_texture(&device, &queue, *id, delta);
                            }
                        }
                        output.textures_delta.clear();
                        let frame = match surface.get_current_texture() {
                            wgpu::CurrentSurfaceTexture::Success(f)
                            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                            wgpu::CurrentSurfaceTexture::Lost
                            | wgpu::CurrentSurfaceTexture::Outdated => {
                                surface.configure(&device, &config);
                                return;
                            }
                            _ => return,
                        };
                        let view = frame.texture.create_view(&Default::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("frame"),
                            });
                        let screen = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [size.width, size.height],
                            pixels_per_point: output.pixels_per_point,
                        };
                        geometry.render(
                            &device,
                            &mut encoder,
                            &view,
                            &editor.doc,
                            &viewport.unwrap(),
                            [size.width, size.height],
                            output.pixels_per_point,
                        );
                        let mut extra = egui_renderer.update_buffers(
                            &device,
                            &queue,
                            &mut encoder,
                            &jobs,
                            &screen,
                        );
                        {
                            let mut pass = encoder
                                .begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("interface"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        depth_slice: None,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                    multiview_mask: None,
                                })
                                .forget_lifetime();
                            egui_renderer.render(&mut pass, &jobs, &screen);
                        }
                        extra.push(encoder.finish());
                        queue.submit(extra);
                        queue.present(frame);
                        for id in &free_textures {
                            egui_renderer.free_texture(id);
                        }
                    }
                    _ => {
                        let _ = response;
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        })
        .expect("window loop");
}
