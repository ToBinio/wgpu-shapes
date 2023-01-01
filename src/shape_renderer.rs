use std::f32::consts::PI;

use wgpu::{BindGroupLayout, Buffer, Device, include_wgsl, RenderPass, RenderPipeline, SurfaceConfiguration};
use wgpu::util::DeviceExt;

use crate::instance::Instance;
use crate::oval::Oval;
use crate::rect::Rect;
use crate::vertex::Vertex;

pub struct ShapeRenderer {
    render_pipeline: RenderPipeline,

    recs: Vec<Rect>,
    ovals: Vec<Oval>,

    frame_group_layout: BindGroupLayout,
    frame_size: (f32, f32),
    frame_offset: (f32, f32),
}

impl ShapeRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> ShapeRenderer {
        let shader = device.create_shader_module(include_wgsl!("../resources/shader.wgsl"));

        let frame_size_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Frame Bind group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &frame_size_group_layout
                ],
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
            ovals: vec![],

            frame_group_layout: frame_size_group_layout,
            frame_size: (800.0, 600.0),
            frame_offset: (0.0, 0.0),
        }
    }

    pub fn render<'a, 'b : 'a>(&'b self, render_pass: RenderPass<'a>, device: &Device) {
        let frame_size_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Frame size Buffer"),
                contents: bytemuck::cast_slice(&[self.frame_size.0, self.frame_size.1]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let frame_offset_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Frame offset Buffer"),
                contents: bytemuck::cast_slice(&[self.frame_offset.0, self.frame_offset.1]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.frame_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: frame_offset_buffer.as_entire_binding(),
                }
            ],
            label: Some("frame_size_bind_group"),
        });

        let (rect_vertex_buffer, (rect_indices_buffer, rect_indices_count), (rect_instance, rect_instance_count)) = self.generate_rect_buffer(device);
        let (oval_vertex_buffer, (oval_indices_buffer, oval_indices_count), (oval_instance, oval_instance_count)) = self.generate_oval_buffer(device);

        let mut render_pass = render_pass;

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &frame_bind_group, &[]);

        //rects
        render_pass.set_vertex_buffer(0, rect_vertex_buffer.slice(..));
        render_pass.set_index_buffer(rect_indices_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, rect_instance.slice(..));

        render_pass.draw_indexed(0..rect_indices_count, 0, 0..rect_instance_count);

        //ovals
        render_pass.set_vertex_buffer(0, oval_vertex_buffer.slice(..));
        render_pass.set_index_buffer(oval_indices_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, oval_instance.slice(..));

        render_pass.draw_indexed(0..oval_indices_count, 0, 0..oval_instance_count);
    }

    pub fn clear(&mut self) {
        self.recs.clear();
    }

    pub fn update_frame_size(&mut self, frame_size: (f32, f32)) -> &mut Self {
        self.frame_size = frame_size;
        self
    }

    pub fn update_frame_offset(&mut self, frame_offset: (f32, f32)) -> &mut Self {
        self.frame_offset = frame_offset;
        self
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

        let instances: Vec<_> = self.recs.iter()
            .map(|rect| Instance {
                position: [rect.pos.0, rect.pos.1],
                scale: [rect.width, rect.height],
                rotation: rect.rotation,
                color: [rect.color.0, rect.color.1, rect.color.2],
            })
            .collect();

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

    pub fn oval(&mut self) -> &mut Oval{
        self.ovals.push(Oval::default());
        self.ovals.last_mut().unwrap()
    }

    fn generate_oval_buffer(&self, device: &Device) -> (Buffer, (Buffer, u32), (Buffer, u32)) {
        let mut vertices: Vec<_> = vec![];

        for i in 0..16 {
            let angle = PI * 2.0 / 16.0 * i as f32;

            vertices.push(Vertex {
                position: [angle.cos(), angle.sin()],
            })
        }

        let mut indices: Vec<_> = vec![];

        for i in 0..(16 - 2) {
            indices.push(0);
            indices.push(i + 1);
            indices.push(i + 2);
        }

        let instances: Vec<_> = self.ovals.iter()
            .map(|oval| Instance {
                position: [oval.pos.0, oval.pos.1],
                scale: [oval.width, oval.height],
                rotation: 0.0,
                color: [oval.color.0, oval.color.1, oval.color.2],
            })
            .collect();

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