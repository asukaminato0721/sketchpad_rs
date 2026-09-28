use super::{Editor, PendingAction};
use sketchpad_engine::document::Document;
use crate::render::GeometryRenderer;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

pub(crate) fn run() {
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
    let mut next_animation_frame = Instant::now();
    let mut next_ui_frame = None;
    window.request_redraw();
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                let _ = egui_state.on_window_event(&window, &event);
                match event {
                    WindowEvent::CloseRequested => {
                        if editor.doc.dirty {
                            editor.pending_action = Some(PendingAction::Quit);
                            window.request_redraw();
                        } else {
                            elwt.exit();
                        }
                    }
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
                        editor.tick_animation_clock();
                        let input = egui_state.take_egui_input(&window);
                        editor.sync_document_fonts(&ctx);
                        let mut viewport = None;
                        let mut output = ctx.run_ui(input, |ui| viewport = Some(editor.ui(ui)));
                        next_ui_frame =
                            output
                                .viewport_output
                                .get(&ctx.viewport_id())
                                .and_then(|viewport| {
                                    Instant::now().checked_add(viewport.repaint_delay)
                                });
                        if editor.exit_requested {
                            elwt.exit();
                            return;
                        }
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
                            &editor.trace_marks,
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
                        window.request_redraw();
                    }
                }
            }
            Event::AboutToWait => {
                let now = Instant::now();
                if next_ui_frame.is_some_and(|deadline| now >= deadline) {
                    window.request_redraw();
                    next_ui_frame = None;
                }
                let mut next_frame = next_ui_frame;
                if (!editor.animations.is_empty() && !editor.animation_paused)
                    || !editor.movements.is_empty()
                    || !editor.presentations.is_empty()
                {
                    if now >= next_animation_frame {
                        window.request_redraw();
                        next_animation_frame = now + Duration::from_millis(16);
                    }
                    next_frame = Some(next_frame.map_or(next_animation_frame, |deadline| {
                        deadline.min(next_animation_frame)
                    }));
                } else {
                    next_animation_frame = now;
                }
                elwt.set_control_flow(next_frame.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
            }
            _ => {}
        })
        .expect("window loop");
}
