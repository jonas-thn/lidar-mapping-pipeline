use winit::event::MouseScrollDelta;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    pub distance: f32,
    pub yaw: f32, 
    pub pitch: f32,

    pub sensitivity: f32,
    pub zoom_speed: f32,
    pub min_distance: f32
}

impl Camera {
    pub fn new() -> Self {
        Self {
            distance: 2000.0,
            yaw: 0.0,
            pitch: 45.0_f32.to_radians(),
            sensitivity: 0.005,
            zoom_speed: 200.0,
            min_distance: 100.0,
        }
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        self.yaw -= dx as f32 * self.sensitivity;
        self.pitch -= dy as f32 * self.sensitivity;

        let half_pi = std::f32::consts::PI / 2.0 - 0.01;
        self.pitch = self.pitch.clamp(-half_pi, half_pi);
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        match delta {
            //mouse wheel
            MouseScrollDelta::LineDelta(_, y) => {
                self.distance -= *y * self.zoom_speed;
            }
            //touch pad
            MouseScrollDelta::PixelDelta(pos) => {
                self.distance -= pos.y as f32 * 2.0;
            }
        }
        self.distance = self.distance.max(self.min_distance);
    }

    pub fn get_uniform(&self, aspect_ratio: f32) -> CameraUniform {
        let proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 10000.0);
        
        let cam_x = self.distance * self.pitch.cos() * self.yaw.sin();
        let cam_y = self.distance * self.pitch.cos() * self.yaw.cos();
        let cam_z = self.distance * self.pitch.sin();

        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(cam_x, cam_y, cam_z),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );

        CameraUniform {
            view_proj: (proj * view).to_cols_array_2d(),
        }
    }
}