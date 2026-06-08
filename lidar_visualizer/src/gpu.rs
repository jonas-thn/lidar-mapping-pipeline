use crate::context::GraphicsContext;
use crate::types::{GpuVertex, Point3D};
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct GpuState {
    pub ctx: GraphicsContext,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    max_points: u32,
    angle: f32
}

impl GpuState {
    pub async fn new(ctx: GraphicsContext) -> Self {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Lidar Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let aspect = ctx.config.width as f32 / ctx.config.height as f32;
        let proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 10000.0);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, -2500.0, 1500.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );

        let camera_uniform = CameraUniform {
            view_proj: (proj * view).to_cols_array_2d(),
        };
        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Lidar Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Lidar Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[GpuVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::PointList,
                    front_face: wgpu::FrontFace::Ccw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let max_points = 50_000;
        let vertex_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Vertex Buffer"),
            size: (max_points as usize * std::mem::size_of::<GpuVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        return GpuState {
            ctx,
            pipeline,
            vertex_buffer,
            camera_buffer,
            bind_group,
            max_points,
            angle: 0.0,
        };
    }

    pub fn render(&mut self, raw_points: &[Point3D]) {
        let output = match self.ctx.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.ctx.resize(self.ctx.size);
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => return,
            _ => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let point_count = raw_points.len().min(self.max_points as usize);
        let mut gpu_vertices = Vec::with_capacity(point_count);

        for p in raw_points.iter().take(point_count) {
            gpu_vertices.push(GpuVertex {
                position: [p.x_mm as f32, p.y_mm as f32, p.z_mm as f32],
                color: [0.0, 1.0, 1.0],
            });
        }

        if !gpu_vertices.is_empty() {
            self.ctx.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&gpu_vertices),
            );
        }

        self.angle += 0.005; 

        let aspect = self.ctx.config.width as f32 / self.ctx.config.height as f32;
        let proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 10000.0);
        
        let cam_x = self.angle.sin() * 2500.0;
        let cam_y = self.angle.cos() * 2500.0;
        
        let view_cam = glam::Mat4::look_at_rh(
            glam::Vec3::new(cam_x, cam_y, 1500.0), 
            glam::Vec3::ZERO,                     
            glam::Vec3::Z,
        );

        let camera_uniform = CameraUniform {
            view_proj: (proj * view_cam).to_cols_array_2d(),
        };

        self.ctx.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lidar Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..point_count as u32, 0..1);
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
