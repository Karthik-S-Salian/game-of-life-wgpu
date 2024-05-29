use bytemuck::{Pod, Zeroable};
use wgpu::PipelineCompilationOptions;

use crate::Config;
extern crate rand;
use rand::Rng;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    grid_size: u32,
    // uniforms: Uniforms,
    // uniform_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    display_bindgroups: [wgpu::BindGroup; 2],
    frame_count: usize,
    workgroup_count: (u32, u32, u32),
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    grid_size: [f32; 2],
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, config: &Config) -> Renderer {
        device.on_uncaptured_error(Box::new(|error| {
            panic!("Aborting due to an error: {}", error);
        }));

        let code = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/shaders/render.wgsl"
        ));

        let shader_module = compile_shader_module(&device, code);

        let code = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/shaders/compute.wgsl"
        ));

        let compute_shader_module = compile_shader_module(&device, code);

        let bindgroup_layout = create_bindgroup_layout(&device);
        let compute_pipeline =
            create_compute_pipeline(&device, &bindgroup_layout, &compute_shader_module);
        let render_pipeline = create_render_pipeline(&device, &shader_module, &bindgroup_layout);
        let uniforms = Uniforms {
            grid_size: [config.grid_size as f32, config.grid_size as f32],
        };
        let uniform_buffer = create_uniform_buffer(&device, &uniforms);

        let mut cell_state: Vec<u32> = vec![0; (config.grid_size * config.grid_size) as usize];

        {
            let mut rng = rand::thread_rng();
            for i in 0..cell_state.len() {
                if rng.gen_bool(0.4) {
                    cell_state[i] = 1;
                }
            }
        }
        let storage_buffers = create_storage_buffers(&device, cell_state);

        let display_bindgroups = create_bindgroups(
            &device,
            &bindgroup_layout,
            &uniform_buffer,
            [&storage_buffers[0], &storage_buffers[1]],
        );

        let workgroup_count: (u32, u32, u32) = (
            (config.grid_size as f32 / 8.).ceil() as u32,
            (config.grid_size as f32 / 8.).ceil() as u32,
            1 as u32,
        );

        Renderer {
            device,
            queue,
            compute_pipeline,
            grid_size: config.grid_size,
            render_pipeline,
            display_bindgroups,
            frame_count: 0,
            workgroup_count,
        }
    }

    pub fn render_frame(&mut self, target: &wgpu::TextureView) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render frame"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &(self.display_bindgroups[self.frame_count % 2]), &[]);
            compute_pass.dispatch_workgroups(
                self.workgroup_count.0,
                self.workgroup_count.1,
                self.workgroup_count.2,
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("display pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.display_bindgroups[self.frame_count % 2], &[]);

        render_pass.draw(0..6, 0..(&self.grid_size * &self.grid_size) as u32);

        drop(render_pass);

        let commmand_buffer = encoder.finish();
        self.queue.submit(Some(commmand_buffer));
        self.frame_count += 1;
    }
}

fn compile_shader_module(device: &wgpu::Device, code: &str) -> wgpu::ShaderModule {
    use std::borrow::Cow;

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("render"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
    })
}

fn create_render_pipeline(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("display"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[bind_group_layout],
                ..Default::default()
            }),
        ),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
        vertex: wgpu::VertexState {
            module: shader_module,
            entry_point: "display_vs",
            buffers: &[],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader_module,
            entry_point: "display_fs",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}

fn create_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn create_bindgroups(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    storagebuffers: [&wgpu::Buffer; 2],
) -> [wgpu::BindGroup; 2] {
    [
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: storagebuffers[0],
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: storagebuffers[1],
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        }),
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: storagebuffers[1],
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: storagebuffers[0],
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        }),
    ]
}

fn create_uniform_buffer(device: &wgpu::Device, uniforms: &Uniforms) -> wgpu::Buffer {
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniforms"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    uniform_buffer
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(bytemuck::bytes_of(uniforms));
    uniform_buffer.unmap();
    uniform_buffer
}

fn create_storage_buffers(device: &wgpu::Device, initial_state: Vec<u32>) -> [wgpu::Buffer; 2] {
    let buffer1 = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("storage buffer 1"),
        size: (initial_state.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: true,
    });
    buffer1
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(bytemuck::cast_slice(&initial_state));
    buffer1.unmap();

    let buffer2 = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("storage buffer 2"),
        size: (initial_state.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: true,
    });
    buffer2
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(bytemuck::cast_slice(&initial_state));
    buffer2.unmap();

    [buffer1, buffer2]
}

fn create_compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    shader_module: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("add source"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[layout],
                ..Default::default()
            }),
        ),
        module: shader_module,
        entry_point: "computeMain",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    })
}
