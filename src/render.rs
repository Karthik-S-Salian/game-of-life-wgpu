use bytemuck::{Pod, Zeroable};
use wgpu::PipelineCompilationOptions;

use crate::Config;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    grid_size: u32,
    // uniforms: Uniforms,
    // uniform_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    display_bindgroup: wgpu::BindGroup,
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    grid_size: f32,
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

        let bindgroup_layout = create_bindgroup_layout(&device);
        let render_pipeline = create_render_pipeline(&device, &shader_module, &bindgroup_layout);
        let uniforms = Uniforms {
            grid_size: config.grid_size as f32,
        };
        let uniform_buffer = create_uniform_buffer(&device, &uniforms);
        let bindgroup = create_bindgroup(&device, &bindgroup_layout, &uniform_buffer);

        Renderer {
            device,
            queue,
            grid_size: config.grid_size,
            render_pipeline,
            display_bindgroup: bindgroup,
        }
    }

    pub fn render_frame(&self, target: &wgpu::TextureView) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render frame"),
            });

        // {
        //     let compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        //         label: Some("compute pass"),
        //         timestamp_writes: None,
        //     });

        // }

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

        render_pass.set_bind_group(0, &self.display_bindgroup, &[]);

        render_pass.draw(0..6, 0..(&self.grid_size * &self.grid_size) as u32);

        drop(render_pass);

        let commmand_buffer = encoder.finish();
        self.queue.submit(Some(commmand_buffer));
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
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_bindgroup(
    device: &wgpu::Device,
    layout:& wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: uniform_buffer,
                offset: 0,
                size: None,
            }),
        }],
    })
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
