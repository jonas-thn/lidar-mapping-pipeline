mod camera;
mod context;
mod gpu;
mod gui;
mod network;
mod types;

use context::GraphicsContext;
use gui::DashboardStats;
use std::sync::{Arc, Mutex};
use types::{CloudState};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event::{DeviceEvent, ElementState, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use crate::gpu::GpuState;
use gui::GuiEvent;
use std::sync::mpsc;

struct App {
    state: Option<GpuState>,
    window: Option<Arc<Window>>,
    shared_cloud: Arc<Mutex<CloudState>>,
    mouse_pressed: bool,

    gui_rx: Option<mpsc::Receiver<GuiEvent>>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("3D LiDAR Visualizer");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let (tx, rx) = mpsc::channel();
        self.gui_rx = Some(rx);

        let context = pollster::block_on(GraphicsContext::new(window.clone()));
        self.state = Some(pollster::block_on(GpuState::new(context, &window, tx)));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut state) = self.state else {
            return;
        };

        let Some(window) = self.window.as_ref() else {
            return;
        };

        let ui_consumed_event = state.gui.handle_event(window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                let (points, stats) = {
                    let cloud = self.shared_cloud.lock().unwrap();

                    let is_connected = std::time::Instant::now()
                        .duration_since(cloud.last_packet_time)
                        .as_secs_f32()
                        < 1.0;
                    let display_pps = if is_connected { cloud.current_pps } else { 0 };

                    let stats = DashboardStats {
                        total_points: cloud.points.len(),
                        pps: display_pps,
                        is_connected,
                    };
                    (cloud.points.clone(), stats)
                };

                state.render(&points, window, &stats);

                if let Some(rx) = &self.gui_rx {
                    while let Ok(event) = rx.try_recv() {
                        match event {
                            GuiEvent::ClearCloud => {
                                let mut cloud = self.shared_cloud.lock().unwrap();
                                cloud.points.clear();
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => {
                if !ui_consumed_event {
                    self.mouse_pressed = button_state == ElementState::Pressed;
                } else if button_state == ElementState::Released {
                    self.mouse_pressed = false;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if !ui_consumed_event {
                    state.camera.process_scroll(&delta);
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.mouse_pressed {
                if let Some(ref mut state) = self.state {
                    state.camera.process_mouse(delta.0, delta.1);
                }
            }
        }
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let point_cloud = Arc::new(Mutex::new(CloudState::new()));
    let point_cloud_network_clone = Arc::clone(&point_cloud);
    let udp_listen_port: u16 = 4242;

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            network::run_udp_listener(udp_listen_port, point_cloud_network_clone).await;
        });
    });

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        state: None,
        window: None,
        shared_cloud: point_cloud,
        mouse_pressed: false,
        gui_rx: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
