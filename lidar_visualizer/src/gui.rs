use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use std::sync::mpsc::Sender;
use winit::window::Window;

pub enum GuiEvent {
    ClearCloud,
}

pub struct DashboardStats {
    pub total_points: usize,
    pub pps: usize,
    pub is_connected: bool,
}

pub struct Gui {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,

    pub show_grid: bool,
    tx: Sender<GuiEvent>,
}

impl Gui {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        window: &Window,
        tx: Sender<GuiEvent>,
    ) -> Self {
        let context = Context::default();
        let viewport_id = context.viewport_id();
        let state = State::new(context.clone(), viewport_id, window, None, None, None);
        let renderer = Renderer::new(device, format, RendererOptions::PREDICTABLE);

        Self {
            context,
            state,
            renderer,

            show_grid: true,
            tx,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn draw(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        stats: &DashboardStats,
    ) -> Vec<wgpu::CommandBuffer> {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);

        let ui_width = 130.0;

        egui::Window::new("Status")
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .default_width(ui_width)
            .show(&self.context, |ui| {
                ui.set_min_width(ui_width);

                ui.horizontal(|ui| {
                    ui.label(format!("{} pts/s", stats.pps));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if stats.is_connected {
                            ui.label(egui::RichText::new("Online").color(egui::Color32::GREEN));
                        } else {
                            ui.label(egui::RichText::new("Offline").color(egui::Color32::RED));
                        }
                    });
                });

                ui.add_space(6.0);

                let fill_ratio = stats.total_points as f32 / 50_000.0;
                let visuals = ui.visuals();

                ui.add(
                    egui::ProgressBar::new(fill_ratio)
                        .desired_width(ui.available_width())
                        .fill(visuals.widgets.active.bg_fill)
                        .text(
                            egui::RichText::new(format!("Buffer: {} / 50k", stats.total_points))
                                .color(visuals.widgets.inactive.fg_stroke.color),
                        ),
                );
            });

        egui::Window::new("Controls")
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .default_width(ui_width)
            .show(&self.context, |ui| {
                ui.set_min_width(ui_width);

                ui.checkbox(&mut self.show_grid, "Show Grid");

                ui.add_space(5.0);

                if ui.button("🗑 Clear Point Cloud").clicked() {
                    let _ = self.tx.send(GuiEvent::ClearCloud);
                }
            });

        let full_output = self.context.end_pass();
        self.state
            .handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        let user_cmd_bufs =
            self.renderer
                .update_buffers(device, queue, encoder, &paint_jobs, &screen_desc);

        {
            let mut gui_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GUI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                })
                .forget_lifetime();

            self.renderer
                .render(&mut gui_pass, &paint_jobs, &screen_desc);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        user_cmd_bufs
    }
}
