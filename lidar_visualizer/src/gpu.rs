use crate::camera::Camera;
use crate::context::GraphicsContext;
use crate::gui::{Gui, GuiEvent};
use crate::types::{GridVertex, Point3D, PointInstance, QuadVertex};
use wgpu::util::DeviceExt;
use crate::gui::DashboardStats;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct GpuState {
    pub ctx: GraphicsContext,

    pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    max_points: u32,

    grid_pipeline: wgpu::RenderPipeline,
    grid_vertex_buffer: wgpu::Buffer,
    grid_vertex_count: u32,

    camera_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,

    pub camera: Camera,
    pub gui: Gui,
}

fn create_grid_vertices(size: f32, step: f32) -> Vec<GridVertex> {
    let mut vertices = Vec::new();
    let color = [0.05, 0.05, 0.05];

    let mut i = -size;

    while i <= size {
        vertices.push(GridVertex {
            position: [i, -size, 0.0],
            color,
        });
        vertices.push(GridVertex {
            position: [i, size, 0.0],
            color,
        });

        vertices.push(GridVertex {
            position: [-size, i, 0.0],
            color,
        });
        vertices.push(GridVertex {
            position: [size, i, 0.0],
            color,
        });

        i += step;
    }

    vertices
}

fn hue_to_rgb(hue_degrees: f32) -> [f32; 3] {
    let h = (hue_degrees / 60.0) % 6.0;

    let x = 1.0 - (h % 2.0 - 1.0).abs(); //triangle wave

    let (r, g, b) = if h < 1.0 {
        (1.0, x, 0.0)
    } else if h < 2.0 {
        (x, 1.0, 0.0)
    } else if h < 3.0 {
        (0.0, 1.0, x)
    } else if h < 4.0 {
        (0.0, x, 1.0)
    } else if h < 5.0 {
        (x, 0.0, 1.0)
    } else {
        (1.0, 0.0, x)
    };

    [r, g, b]
}

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

impl GpuState {
    pub async fn new(ctx: GraphicsContext, window: &winit::window::Window, gui_tx: std::sync::mpsc::Sender<GuiEvent>) -> Self {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Lidar Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let camera = Camera::new();
        let aspect = ctx.config.width as f32 / ctx.config.height as f32;
        let camera_uniform = camera.get_uniform(aspect);

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
                    entry_point: Some("vs_lidar"),
                    buffers: &[QuadVertex::desc(), PointInstance::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_lidar"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    front_face: wgpu::FrontFace::Ccw,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let grid_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Grid Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_grid"),
                    buffers: &[GridVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_grid"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    front_face: wgpu::FrontFace::Ccw,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let grid_vertices = create_grid_vertices(5000.0, 500.0);
        let grid_vertex_count = grid_vertices.len() as u32;

        let grid_vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Vertex Buffer"),
                contents: bytemuck::cast_slice(&grid_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let depth_texture_view = create_depth_texture(&ctx.device, &ctx.config);

        let quad_vertices: &[QuadVertex] = &[
            QuadVertex {
                position: [-1.0, -1.0],
            },
            QuadVertex {
                position: [1.0, -1.0],
            },
            QuadVertex {
                position: [-1.0, 1.0],
            },
            QuadVertex {
                position: [1.0, 1.0],
            },
        ];
        let quad_vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(quad_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let max_points = 50_000;
        let instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (max_points as usize * std::mem::size_of::<PointInstance>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let gui = Gui::new(&ctx.device, ctx.config.format, window, gui_tx);

        return GpuState {
            ctx,
            pipeline,
            camera_buffer,
            bind_group,
            depth_texture_view,
            max_points,
            camera,
            grid_pipeline,
            grid_vertex_buffer,
            grid_vertex_count,
            quad_vertex_buffer,
            instance_buffer,
            gui,
        };
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.resize(new_size);
            self.depth_texture_view = create_depth_texture(&self.ctx.device, &self.ctx.config);
        }
    }

    pub fn render(&mut self, raw_points: &[Point3D], window: &winit::window::Window, stats: &DashboardStats) {
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

        let aspect = self.ctx.config.width as f32 / self.ctx.config.height as f32;
        let camera_uniform = self.camera.get_uniform(aspect);

        self.ctx.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let point_count = raw_points.len().min(self.max_points as usize);
        let mut gpu_instances = Vec::with_capacity(point_count);
        let max_quality = 255.0;

        for p in raw_points.iter().take(point_count) {
            let quality_factor = (p.quality as f32 / max_quality).clamp(0.0, 1.0);
            let hue = (1.0 - quality_factor) * 240.0; //in degrees
            let heat_color = hue_to_rgb(hue);

            gpu_instances.push(PointInstance {
                position: [p.x_mm as f32, p.y_mm as f32, p.z_mm as f32],
                color: heat_color,
            });
        }

        if !gpu_instances.is_empty() {
            self.ctx.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&gpu_instances),
            );
        }

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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0025,
                            g: 0.0025,
                            b: 0.0025,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            if self.gui.show_grid {
                render_pass.set_pipeline(&self.grid_pipeline);
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
                render_pass.draw(0..self.grid_vertex_count, 0..1);
            }

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..4 as u32, 0..point_count as u32);
        }

        let user_cmd_bufs = self.gui.draw(
            window,
            &self.ctx.device,
            &self.ctx.queue,
            &mut encoder,
            &view,
            self.ctx.config.width,
            self.ctx.config.height,
            stats
        );

        self.ctx.queue.submit(
            user_cmd_bufs
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );

        output.present();
    }
}
