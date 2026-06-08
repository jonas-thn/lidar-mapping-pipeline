mod context;
mod network;
mod types;
mod gpu;

use context::GraphicsContext;
use std::sync::{Arc, Mutex};
use types::Point3D;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::gpu::GpuState;

struct App {
    state: Option<GpuState>,
    window: Option<Arc<Window>>,
    point_cloud: Arc<Mutex<Vec<Point3D>>>,
    last_logged_count: usize
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("3D LiDAR Visualizer");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let context = pollster::block_on(GraphicsContext::new(window.clone()));
        self.state = Some(pollster::block_on(GpuState::new(context)));
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

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.ctx.resize(physical_size);
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

                state.render(&points);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
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
        last_logged_count: 0
    };
    event_loop.run_app(&mut app).unwrap();
}
