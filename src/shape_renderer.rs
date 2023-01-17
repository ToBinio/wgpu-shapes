use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::f32::consts::PI;
use image::GenericImageView;

use wgpu::{BindGroupLayout, Color, CommandEncoder, Device, include_wgsl, Queue, RenderPipeline, Sampler, SurfaceConfiguration, TextureView};
use wgpu::util::DeviceExt;
use wgpu_noboiler::buffer::{BufferCreator, SimpleBuffer};
use wgpu_noboiler::render_pass::RenderPassCreator;
use wgpu_noboiler::vertex::Vertex;

use crate::depth_buffer::Texture;
use crate::instance::Instance;
use crate::oval::Oval;
use crate::rect::Rect;
use crate::shapes::BasicShape;
use crate::vertex::Vertex as OwnVertex;

/// helps to draw basic [BasicShapes](BasicShape)
pub struct ShapeRenderer {
    render_pipeline: RenderPipeline,

    recs: Vec<Rect>,
    ovals: Vec<Oval>,

    frame_group_layout: BindGroupLayout,
    frame_size: (f32, f32),
    frame_offset: (f32, f32),

    background_color: Color,

    depth_texture: Texture,

    textures: Vec<(TextureView, Sampler)>,
    textures_bind_groups: Vec<BindGroupLayout>,
}

impl ShapeRenderer {
    /// creates a new [ShapeRenderer] which can render [BasicShape].
    /// these can be created with [ShapeRenderer::rect], [ShapeRenderer::oval]
    ///
    /// can be reused with [ShapeRenderer::clear] function
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
                    OwnVertex::descriptor(),
                    Instance::descriptor()
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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

            background_color: Color::WHITE,

            depth_texture: Texture::create_depth_texture(device, config, "depth_texture"),

            textures: vec![],
            textures_bind_groups: vec![],
        }
    }

    /// renders the current [BasicShapes](BasicShape) which can be added with [ShapeRenderer::rect], [ShapeRenderer::oval]
    pub fn render<'a, 'b : 'a>(&'b self, encoder: &mut CommandEncoder, texture_view: &TextureView, device: &Device) {
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

        let InstanceBufferGroup(rect_vertex_buffer, rect_indices_buffer, rect_instance_buffer) = self.generate_rect_buffer(device);
        let oval_buffers = self.generate_oval_buffer(device);

        let mut render_pass = RenderPassCreator::new(texture_view)
            .depth_stencil_attachment(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            })
            .clear_color(self.background_color)
            .build(encoder);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &frame_bind_group, &[]);

        //rects
        render_pass.set_vertex_buffer(0, rect_vertex_buffer.slice());
        render_pass.set_index_buffer(rect_indices_buffer.slice(), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, rect_instance_buffer.slice());

        render_pass.draw_indexed(0..rect_indices_buffer.size(), 0, 0..rect_instance_buffer.size());

        //ovals

        for InstanceBufferGroup(oval_vertex_buffer, oval_indices_buffer, oval_instance_buffer) in &oval_buffers {
            render_pass.set_vertex_buffer(0, oval_vertex_buffer.slice());
            render_pass.set_index_buffer(oval_indices_buffer.slice(), wgpu::IndexFormat::Uint32);

            render_pass.set_vertex_buffer(1, oval_instance_buffer.slice());

            render_pass.draw_indexed(0..oval_indices_buffer.size(), 0, 0..oval_instance_buffer.size());
        }
    }

    /// clears the current drawn [BasicShapes](BasicShape) which can be added with [ShapeRenderer::rect], [ShapeRenderer::oval]
    pub fn clear(&mut self) {
        self.recs.clear();
        self.ovals.clear();
    }

    /// sets the current [frame_size](ShapeRenderer::frame_size)
    pub fn set_frame_size(&mut self, frame_size: (f32, f32)) -> &mut Self {
        self.frame_size = frame_size;
        self
    }

    /// frame_size.0 is the with in which [BasicShape] get displayed.
    ///
    /// frame_size.1 is the height in which [BasicShape] get displayed
    ///
    /// (0,0) -> center of screen
    pub fn frame_size(&self) -> (f32, f32) {
        self.frame_size
    }

    /// sets the current [frame_offset](ShapeRenderer::frame_offset)
    pub fn set_frame_offset(&mut self, frame_offset: (f32, f32)) -> &mut Self {
        self.frame_offset = frame_offset;
        self
    }

    /// frame_offset are values which get added to all [BasicShape]
    pub fn frame_offset(&self) -> (f32, f32) {
        self.frame_offset
    }

    /// resizes the depthBuffer should be called on every window resize
    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) -> &mut Self {
        self.depth_texture = Texture::create_depth_texture(device, config, "depth_texture");
        self
    }

    /// sets the clearColor/ backgroundColor
    pub fn background_color(&mut self, background_color: Color) -> &mut Self {
        self.background_color = background_color;
        self
    }

    /// renders [Rect] and returns a Ref to it
    pub fn rect(&mut self) -> &mut Rect {
        self.recs.push(Rect::default());
        self.recs.last_mut().unwrap()
    }

    fn generate_rect_buffer(&self, device: &Device) -> InstanceBufferGroup {
        let vertex_buffer = BufferCreator::vertex(device)
            .label("Rect VertexBuffer")
            .data(vec![
                OwnVertex { position: [1.0, 1.0] },
                OwnVertex { position: [-1.0, 1.0] },
                OwnVertex { position: [-1.0, -1.0] },
                OwnVertex { position: [1.0, -1.0] },
            ]).build();

        let indices_buffer = BufferCreator::indices(device)
            .label("Rect IndicesBuffer")
            .data(vec![0, 1, 2, 0, 2, 3])
            .build();

        let instances: Vec<_> = self.recs.iter()
            .map(|rect| rect.to_instance())
            .collect();

        let instances_buffer = BufferCreator::vertex(device)
            .label("Rect InstanceBuffer")
            .data(instances)
            .build();

        InstanceBufferGroup(vertex_buffer, indices_buffer, instances_buffer)
    }

    /// renders [Oval] and returns a Ref to it
    pub fn oval(&mut self) -> &mut Oval {
        self.ovals.push(Oval::default());
        self.ovals.last_mut().unwrap()
    }

    fn generate_oval_buffer(&self, device: &Device) -> Vec<InstanceBufferGroup> {
        let mut ovals = HashMap::<u32, Vec<&Oval>>::new();

        for oval in &self.ovals {
            if let Entry::Vacant(e) = ovals.entry(oval.detail) {
                e.insert(vec![oval]);
            } else {
                ovals.get_mut(&oval.detail).unwrap().push(oval);
            }
        }

        let mut instance_buffer_groups = vec![];

        for (detail, ovals) in ovals {
            let vertices: Vec<_> = (0..detail)
                .map(|i| {
                    let angle = PI * 2.0 / detail as f32 * i as f32;

                    OwnVertex { position: [angle.cos(), angle.sin()] }
                })
                .collect();

            let vertex_buffer = BufferCreator::vertex(device)
                .label("Rect VertexBuffer")
                .data(vertices)
                .build();

            let indices: Vec<_> = (0..(detail as i32 - 2))
                .flat_map(|i| [0, i + 1, i + 2])
                .collect();

            let indices_buffer = BufferCreator::indices(device)
                .label("Rect IndicesBuffer")
                .data(indices)
                .build();

            let instances: Vec<_> = ovals.iter()
                .map(|oval| oval.to_instance())
                .collect();

            let instances_buffer = BufferCreator::vertex(device)
                .label("Rect InstanceBuffer")
                .data(instances)
                .build();

            instance_buffer_groups.push(InstanceBufferGroup(vertex_buffer, indices_buffer, instances_buffer));
        }

        instance_buffer_groups
    }

    pub fn add_texture_from_bytes(&mut self, bytes: &[u8], device: &Device, queue: &Queue) {
        let diffuse_image = image::load_from_memory(bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
            }
        );

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        self.textures.push((diffuse_texture_view, diffuse_sampler));
    }
}

struct InstanceBufferGroup(SimpleBuffer, SimpleBuffer, SimpleBuffer);