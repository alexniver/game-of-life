use std::time::Instant;

use glam::Mat4;
use wgpu::{
    util::DeviceExt, Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    ColorWrites, ComputePipeline, Device, Features, Instance, Queue, RenderPipeline,
    ShaderModuleDescriptor, ShaderStages, Surface, SurfaceConfiguration, TextureUsages,
};
use winit::{
    event::{ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    grid::{self, Grid},
    vertex::Vertex,
};

pub struct Core {
    pub instance: Instance,
    pub surface: Surface,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,

    pub proj_bind_group: BindGroup,
    pub proj_buffer: Buffer,

    pub grid_bind_group_arr: [BindGroup; 2],

    pub vertex_arr: Vec<Vertex>,
    pub vertex_buffer: Buffer,

    pub render_pipline: RenderPipeline,
    pub compute_pipline: ComputePipeline,
    pub compute_bind_group_arr: [BindGroup; 2],

    pub grid: Grid,

    pub start_time: Instant,
    pub last_cell_swap_time: Instant,
    pub step: u32,
}

impl Core {
    pub async fn new(_event_loop: &EventLoop<()>, window: &Window) -> Self {
        let window_size = window.inner_size();

        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window).unwrap() };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: Features::empty(),
                    limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let proj_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Proj Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let proj = Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 1.0, -1.0);
        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Proj Buffer"),
            contents: bytemuck::cast_slice(&proj.to_cols_array_2d()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Proj Bind Group"),
            layout: &proj_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(proj_buffer.as_entire_buffer_binding()),
            }],
        });

        let grid_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Grid Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let grid = Grid::new();
        let grid_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Size Buffer"),
            contents: bytemuck::cast_slice(&[grid::GRID_SIZE as f32, grid::GRID_SIZE as f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let grid_pixel_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Pilex Size"),
            contents: bytemuck::cast_slice(&[
                grid::GRID_PIXEL_SIZE as f32,
                grid::GRID_PIXEL_SIZE as f32,
            ]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let grid_cell_buffer_arr = [
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Cell Buffer"),
                contents: bytemuck::cast_slice(&grid.cell_arr),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Cell Buffer"),
                contents: bytemuck::cast_slice(&grid.cell_arr),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            }),
        ];

        let grid_bind_group_arr = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Grid Bind Group"),
                layout: &grid_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            grid_size_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(
                            grid_pixel_size_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(
                            grid_cell_buffer_arr[0].as_entire_buffer_binding(),
                        ),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Grid Bind Group"),
                layout: &grid_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            grid_size_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(
                            grid_pixel_size_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(
                            grid_cell_buffer_arr[1].as_entire_buffer_binding(),
                        ),
                    },
                ],
            }),
        ];

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // let compute_bind_groups

        let vertex_arr = Vertex::rect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_arr),
            usage: BufferUsages::VERTEX,
        });

        let render_pipline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipline Layout"),
                bind_group_layouts: &[&proj_bind_group_layout, &grid_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mesh_wgsl = std::fs::read_to_string("assets/shader/mesh.wgsl").unwrap();
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(mesh_wgsl.into()),
        });

        let render_pipline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipline"),
            layout: Some(&render_pipline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::vertex_buffer_layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let compute_wgsl = std::fs::read_to_string("assets/shader/compute.wgsl").unwrap();
        let compute_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(compute_wgsl.into()),
        });

        let compute_pipline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipline"),
            layout: Some(&compute_pipline_layout),
            module: &compute_shader,
            entry_point: "cp_main",
        });

        let compute_bind_group_arr = [
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Compute Bind Group A"),
                layout: &compute_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            grid_size_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(
                            grid_cell_buffer_arr[0].as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(
                            grid_cell_buffer_arr[1].as_entire_buffer_binding(),
                        ),
                    },
                ],
            }),
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Compute Bind Group B"),
                layout: &compute_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            grid_size_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(
                            grid_cell_buffer_arr[1].as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(
                            grid_cell_buffer_arr[0].as_entire_buffer_binding(),
                        ),
                    },
                ],
            }),
        ];

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            proj_bind_group,
            proj_buffer,
            grid_bind_group_arr,
            vertex_arr: vertex_arr.into(),
            vertex_buffer,
            render_pipline,
            compute_bind_group_arr,
            compute_pipline,
            grid,
            start_time: Instant::now(),
            last_cell_swap_time: Instant::now(),
            step: 0,
        }
    }

    pub fn set_vertex_arr(&mut self, vertex_arr: Vec<Vertex>) {
        self.vertex_arr = vertex_arr;
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertex_arr),
                usage: BufferUsages::VERTEX,
            });
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        let proj = Mat4::orthographic_rh(
            -(width as f32 / 2.0),
            width as f32 / 2.0,
            -(height as f32 / 2.0),
            height as f32 / 2.0,
            1.0,
            -1.0,
        );

        self.queue.write_buffer(
            &self.proj_buffer,
            0,
            bytemuck::cast_slice(&proj.to_cols_array_2d()),
        );
    }

    pub fn update(&mut self) {
        let last_time = (Instant::now() - self.last_cell_swap_time).as_secs_f32();
        if last_time >= 1.0 {
            self.last_cell_swap_time = Instant::now();
            let time = (Instant::now() - self.start_time).as_secs_f32().floor() as u32;

            self.step = time % 2;
        }
    }

    pub fn render(&self) {
        let render_pipline = &self.render_pipline;

        let current_texture = self.surface.get_current_texture().unwrap();
        let texture_view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(render_pipline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.proj_bind_group, &[]);
            render_pass.set_bind_group(1, &self.grid_bind_group_arr[self.step as usize], &[]);
            render_pass.draw(
                0..self.vertex_arr.len() as _,
                0..grid::GRID_SIZE * grid::GRID_SIZE,
            );
        }

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.compute_pipline);
            compute_pass.set_bind_group(0, &self.compute_bind_group_arr[self.step as usize], &[]);
            let workgroup_count = (grid::GRID_SIZE as f32 / 8.0).ceil();
            compute_pass.dispatch_workgroups(workgroup_count as _, workgroup_count as _, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        current_texture.present();
    }

    pub fn block_loop(mut self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _, control_flow| match event {
            winit::event::Event::MainEventsCleared => window.request_redraw(),
            winit::event::Event::RedrawRequested(window_id) => {
                if window_id == window.id() {
                    self.update();
                    self.render();
                }
            }
            winit::event::Event::WindowEvent { window_id, event } => {
                if window_id == window.id() {
                    match event {
                        winit::event::WindowEvent::Resized(new_size) => {
                            self.resize(new_size.width, new_size.height);
                        }
                        winit::event::WindowEvent::CloseRequested
                        | winit::event::WindowEvent::KeyboardInput {
                            input:
                                winit::event::KeyboardInput {
                                    state: ElementState::Released,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            _ => {}
        });
    }
}
