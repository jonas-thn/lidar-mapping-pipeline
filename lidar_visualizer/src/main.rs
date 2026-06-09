mod camera;
mod context;
mod gpu;
mod gui;
mod network;
mod types;

use context::GraphicsContext;
use std::sync::{Arc, Mutex};
use types::Point3D;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event::{DeviceEvent, ElementState, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::gpu::GpuState;

struct App {
    state: Option<GpuState>,
    window: Option<Arc<Window>>,
    point_cloud: Arc<Mutex<Vec<Point3D>>>,
    last_logged_count: usize,
    mouse_pressed: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("3D LiDAR Visualizer");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let context = pollster::block_on(GraphicsContext::new(window.clone()));
        self.state = Some(pollster::block_on(GpuState::new(context, &window)));
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
                let points = {
                    let cloud = self.point_cloud.lock().unwrap();
                    cloud.clone()
                };

                let count = points.len();
                if count % 100 == 0 && count > 0 && count != self.last_logged_count {
                    log::info!("Point count: {}", count);
                    self.last_logged_count = count;
                }

                state.render(&points, window);
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

    let point_cloud = Arc::new(Mutex::new(Vec::new()));
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
        point_cloud,
        last_logged_count: 0,
        mouse_pressed: false,
    };
    event_loop.run_app(&mut app).unwrap();
}
