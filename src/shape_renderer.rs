use wgpu::{Buffer, Device, include_wgsl, RenderPass, RenderPipeline, SurfaceConfiguration};
use wgpu::util::DeviceExt;

use crate::instance::Instance;
use crate::rect::Rect;
use crate::vertex::Vertex;

pub struct ShapeRenderer {
    render_pipeline: RenderPipeline,

    recs: Vec<Rect>,
}

impl ShapeRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> ShapeRenderer {
        let shader = device.create_shader_module(include_wgsl!("../resources/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    Instance::desc()
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });


        ShapeRenderer {
            render_pipeline,
            recs: vec![],
        }
    }

    pub fn render<'a, 'b : 'a>(&'b self, render_pass: RenderPass<'a>, device: &Device) {
        let (vertex_buffer, (indices_buffer, indices_count), (instance, instance_count)) = self.generate_rect_buffer(device);

        let mut render_pass = render_pass;

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, instance.slice(..));

        render_pass.draw_indexed(0..indices_count, 0, 0..instance_count);
    }

    pub fn clear(&mut self) {
        self.recs.clear();
    }

    pub fn rect(&mut self) -> &mut Rect {
        self.recs.push(Rect::default());
        self.recs.last_mut().unwrap()
    }

    fn generate_rect_buffer(&self, device: &Device) -> (Buffer, (Buffer, u32), (Buffer, u32)) {
        let vertices: Vec<_> = vec![
            Vertex { position: [1.0, 1.0] },
            Vertex { position: [-1.0, 1.0] },
            Vertex { position: [-1.0, -1.0] },
            Vertex { position: [1.0, -1.0] },
        ];

        let indices: Vec<_> = vec![0, 1, 2, 0, 2, 3];

        let mut instances: Vec<_> = vec![];

        for rect in &self.recs {
            instances.push(Instance {
                position: [rect.pos.0, rect.pos.1],
                scale: [rect.width, rect.height],
                color: [rect.color.0, rect.color.1, rect.color.2],
            })
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        (vertex_buffer, (index_buffer, indices.len() as u32), (instance_buffer, instances.len() as u32))
    }
}