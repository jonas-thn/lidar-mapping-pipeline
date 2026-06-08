mod context;

use context::GraphicsContext;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{self, WindowEvent};
use winit::event_loop::{self, ControlFlow, EventLoop};
use winit::window::Window;

struct App {
    state: Option<GraphicsContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("3D LiDAR Visualizer");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let context = pollster::block_on(GraphicsContext::new(window.clone()));
        self.state = Some(context);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
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
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                //TODO
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(ref _state) = self.state {}
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
