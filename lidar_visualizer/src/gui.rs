use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use winit::window::Window;

pub struct Gui {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,

    pub show_grid: bool,
}

impl Gui {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, window: &Window) -> Self {
        let context = Context::default();
        let viewport_id = context.viewport_id();
        let state = State::new(context.clone(), viewport_id, window, None, None, None);
        let renderer = Renderer::new(device, format, RendererOptions::PREDICTABLE);

        Self {
            context,
            state,
            renderer,

            show_grid: true, // Standardmäßig an
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn build_ui(&mut self) {
        egui::Window::new("Lidar Dashboard")
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .show(&self.context, |ui| {
                ui.checkbox(&mut self.show_grid, "Show Grid");
            });
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
    ) -> Vec<wgpu::CommandBuffer> {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);

        self.build_ui();

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
