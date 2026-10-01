use super::{Editor, PendingAction};
use crate::render::{GeometryRenderer, RenderPreferences};
use sketchpad_engine::document::Document;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct GraphicsState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    geometry: GeometryRenderer,
    next_animation_frame: Instant,
    next_ui_frame: Option<Instant>,
}

impl GraphicsState {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Sketchpad RS")
                        .with_inner_size(winit::dpi::LogicalSize::new(1100., 750.)),
                )
                .expect("window"),
        );
        // Give GLES the compositor connection up front.  Creating the default
        // instance without it can leave Vulkan as the only active backend on
        // software/Xvfb displays, even though a working GL adapter is present.
        // `with_env` also makes WGPU_BACKEND useful for deterministic demos and
        // troubleshooting without changing normal backend selection.
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
                event_loop.owned_display_handle(),
            ))
            .with_env(),
        );
        let surface = instance.create_surface(window.clone()).expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("graphics adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("graphics device");
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface config");
        if let Some(format) = surface
            .get_capabilities(&adapter)
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
        {
            config.format = format;
        }
        surface.configure(&device, &config);

        let ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            ctx.clone(),
            ctx.viewport_id(),
            window.as_ref(),
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions::default(),
        );
        let geometry = GeometryRenderer::new(&device, config.format);
        window.request_redraw();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            ctx,
            egui_state,
            egui_renderer,
            geometry,
            next_animation_frame: Instant::now(),
            next_ui_frame: None,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
        self.window.request_redraw();
    }

    fn redraw(&mut self, editor: &mut Editor, event_loop: &ActiveEventLoop) {
        if self.size.width == 0 || self.size.height == 0 {
            return;
        }

        editor.tick_animation_clock();
        editor.prune_faded_traces();
        let input = self.egui_state.take_egui_input(&self.window);
        editor.sync_document_fonts(&self.ctx);
        let mut viewport = None;
        let mut output = self.ctx.run_ui(input, |ui| viewport = Some(editor.ui(ui)));
        self.next_ui_frame = output
            .viewport_output
            .get(&self.ctx.viewport_id())
            .and_then(|viewport| Instant::now().checked_add(viewport.repaint_delay));
        if editor.exit_requested {
            event_loop.exit();
            return;
        }

        self.egui_state
            .handle_platform_output(&self.window, output.platform_output);
        let jobs = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        let free_textures = output.textures_delta.free.clone();
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, *id, delta);
            }
        }
        output.textures_delta.clear();

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: output.pixels_per_point,
        };
        self.geometry.render(
            crate::render::RenderTarget {
                device: &self.device,
                encoder: &mut encoder,
                view: &view,
                size: [self.size.width, self.size.height],
                pixels_per_point: output.pixels_per_point,
            },
            &editor.doc,
            &viewport.expect("editor viewport"),
            &editor.trace_marks,
            RenderPreferences {
                background_color: editor.preferences.background_color,
                selection_color: editor.preferences.selection_color,
                trace_fade_seconds: editor
                    .preferences
                    .fade_traces
                    .then_some(editor.preferences.trace_fade_seconds.max(0.1)),
            },
        );
        let mut command_buffers = self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
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
            self.egui_renderer.render(&mut pass, &jobs, &screen);
        }
        command_buffers.push(encoder.finish());
        self.queue.submit(command_buffers);
        self.window.pre_present_notify();
        self.queue.present(frame);
        for id in &free_textures {
            self.egui_renderer.free_texture(id);
        }
    }

    fn about_to_wait(&mut self, editor: &Editor, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if self.next_ui_frame.is_some_and(|deadline| now >= deadline) {
            self.window.request_redraw();
            self.next_ui_frame = None;
        }

        let mut next_frame = self.next_ui_frame;
        if editor.preferences.fade_traces && !editor.trace_marks.is_empty()
            || (!editor.animations.is_empty() && !editor.animation_paused)
            || !editor.movements.is_empty()
            || !editor.presentations.is_empty()
        {
            if now >= self.next_animation_frame {
                self.window.request_redraw();
                self.next_animation_frame = now + Duration::from_millis(16);
            }
            next_frame = Some(next_frame.map_or(self.next_animation_frame, |deadline| {
                deadline.min(self.next_animation_frame)
            }));
        } else {
            self.next_animation_frame = now;
        }
        event_loop.set_control_flow(next_frame.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }
}

struct Application {
    editor: Editor,
    graphics: Option<GraphicsState>,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_none() {
            self.graphics = Some(GraphicsState::new(event_loop));
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(graphics) = &mut self.graphics else {
            return;
        };
        if window_id != graphics.window.id() {
            return;
        }

        let _ = graphics
            .egui_state
            .on_window_event(&graphics.window, &event);
        match event {
            WindowEvent::CloseRequested => {
                if self.editor.doc.dirty {
                    self.editor.pending_action = Some(PendingAction::Quit);
                    graphics.window.request_redraw();
                } else {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => graphics.resize(size),
            WindowEvent::RedrawRequested => graphics.redraw(&mut self.editor, event_loop),
            _ => graphics.window.request_redraw(),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(graphics) = &mut self.graphics {
            graphics.about_to_wait(&self.editor, event_loop);
        }
    }
}

pub(crate) fn run() {
    let args: Vec<_> = std::env::args_os().collect();
    let doc = match args.get(1).map(PathBuf::from) {
        Some(path) if path.as_os_str() == "--demo" => {
            match sketchpad_engine::demo::geometry_document(sketchpad_engine::demo::GEOMETRY_STEPS)
            {
                Ok(document) => document,
                Err(error) => {
                    eprintln!("Could not create the built-in demo: {error}");
                    Document::new()
                }
            }
        }
        Some(path) => match Document::open(&path) {
            Ok(document) => document,
            Err(error) => {
                eprintln!("Could not open {}: {error}", path.display());
                Document::new()
            }
        },
        None => Document::new(),
    };
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut application = Application {
        editor: Editor::new(doc),
        graphics: None,
    };
    event_loop
        .run_app(&mut application)
        .expect("window application");
}
