use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::f32::consts::PI;

use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Color, CommandEncoder, Device, Queue, RenderPipeline,
    SamplerDescriptor, SurfaceConfiguration, TextureView,
};
use wgpu_noboiler::buffer::{BufferCreator, SimpleBuffer};
use wgpu_noboiler::render_pass::RenderPassCreator;
use wgpu_noboiler::render_pipeline::RenderPipelineCreator;
use wgpu_noboiler::vertex::Vertex;

use crate::render::depth_buffer::DepthBuffer;
use crate::render::instance::{Instance, TextureInstance};
use crate::render::vertex::Vertex as OwnVertex;
use crate::shape::image::Image;
use crate::shape::oval::Oval;
use crate::shape::rect::Rect;
use crate::shape::shapes::BasicShape;

/// helps to draw basic [BasicShapes](BasicShape)
pub struct ShapeRenderer {
    shape_render_pipeline: RenderPipeline,
    texture_render_pipeline: RenderPipeline,

    recs: Vec<Rect>,
    ovals: Vec<Oval>,
    images: Vec<Image>,

    frame_group_layout: BindGroupLayout,
    frame_size: (f32, f32),
    frame_offset: (f32, f32),

    background_color: Color,

    depth_texture: DepthBuffer,

    texture_group_layout: BindGroupLayout,
    texture: Option<TextureView>,
    texture_size: u32,
    textures: Vec<DynamicImage>,
    textures_cords: Vec<((f32, f32), (f32, f32))>,
}

impl ShapeRenderer {
    /// creates a new [ShapeRenderer] which can render [BasicShape].
    /// these can be created with [ShapeRenderer::rect], [ShapeRenderer::oval]
    ///
    /// can be reused with [ShapeRenderer::clear] function
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> ShapeRenderer {
        let frame_size_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    },
                ],
                label: Some("Frame Bind group"),
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let shape_render_pipeline = RenderPipelineCreator::from_shader_code(
            include_str!("../../resources/shape_shader.wgsl"),
            device,
            config,
        )
        .add_bind_group(&frame_size_group_layout)
        .add_vertex_buffer(OwnVertex::descriptor())
        .add_vertex_buffer(Instance::descriptor())
        .depth_stencil(wgpu::DepthStencilState {
            format: DepthBuffer::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
        .build();

        let texture_render_pipeline = RenderPipelineCreator::from_shader_code(
            include_str!("../../resources/texture_shader.wgsl"),
            device,
            config,
        )
        .add_bind_group(&frame_size_group_layout)
        .add_bind_group(&texture_bind_group_layout)
        .add_vertex_buffer(OwnVertex::descriptor())
        .add_vertex_buffer(TextureInstance::descriptor())
        .depth_stencil(wgpu::DepthStencilState {
            format: DepthBuffer::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
        .build();

        ShapeRenderer {
            shape_render_pipeline,
            texture_render_pipeline,

            recs: vec![],
            ovals: vec![],
            images: vec![],

            frame_group_layout: frame_size_group_layout,
            frame_size: (800.0, 600.0),
            frame_offset: (0.0, 0.0),

            background_color: Color::WHITE,

            depth_texture: DepthBuffer::create_depth_texture(device, config, "depth_texture"),

            texture_group_layout: texture_bind_group_layout,
            texture: None,
            texture_size: 512,
            textures: vec![],
            textures_cords: vec![],
        }
    }

    fn frame_bind_group(&self, device: &Device) -> BindGroup {
        let frame_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame size Buffer"),
            contents: bytemuck::cast_slice(&[self.frame_size.0, self.frame_size.1]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let frame_offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame offset Buffer"),
            contents: bytemuck::cast_slice(&[self.frame_offset.0, self.frame_offset.1]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.frame_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: frame_offset_buffer.as_entire_binding(),
                },
            ],
            label: Some("frame_size_bind_group"),
        })
    }

    fn texture_bind_group(&self, device: &Device) -> Option<BindGroup> {
        self.texture.as_ref()?;

        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.texture.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &device.create_sampler(&SamplerDescriptor::default()),
                    ),
                },
            ],
            label: Some("diffuse_bind_group"),
        }))
    }

    /// renders the current [BasicShapes](BasicShape) which can be added with [ShapeRenderer::rect], [ShapeRenderer::oval]
    pub fn render<'a, 'b: 'a>(
        &'b self,
        encoder: &mut CommandEncoder,
        texture_view: &TextureView,
        device: &Device,
    ) {
        let InstanceBufferGroup(rect_vertex_buffer, rect_indices_buffer, rect_instance_buffer) =
            self.generate_rect_buffer(device);
        let oval_buffers = self.generate_oval_buffer(device);

        let InstanceBufferGroup(
            texture_vertex_buffer,
            texture_indices_buffer,
            texture_instance_buffer,
        ) = self.generate_image_buffer(device);

        let frame_bind_group = self.frame_bind_group(device);
        let texture_bind_group = self.texture_bind_group(device);

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

        render_pass.set_pipeline(&self.shape_render_pipeline);
        render_pass.set_bind_group(0, &frame_bind_group, &[]);

        //rects
        render_pass.set_vertex_buffer(0, rect_vertex_buffer.slice());
        render_pass.set_index_buffer(rect_indices_buffer.slice(), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, rect_instance_buffer.slice());

        render_pass.draw_indexed(
            0..rect_indices_buffer.size(),
            0,
            0..rect_instance_buffer.size(),
        );

        //ovals

        for InstanceBufferGroup(oval_vertex_buffer, oval_indices_buffer, oval_instance_buffer) in
            &oval_buffers
        {
            render_pass.set_vertex_buffer(0, oval_vertex_buffer.slice());
            render_pass.set_index_buffer(oval_indices_buffer.slice(), wgpu::IndexFormat::Uint32);

            render_pass.set_vertex_buffer(1, oval_instance_buffer.slice());

            render_pass.draw_indexed(
                0..oval_indices_buffer.size(),
                0,
                0..oval_instance_buffer.size(),
            );
        }

        //texture

        if texture_bind_group.is_none() {
            return;
        }

        render_pass.set_pipeline(&self.texture_render_pipeline);
        render_pass.set_bind_group(0, &frame_bind_group, &[]);
        render_pass.set_bind_group(1, texture_bind_group.as_ref().unwrap(), &[]);

        render_pass.set_vertex_buffer(0, texture_vertex_buffer.slice());
        render_pass.set_index_buffer(texture_indices_buffer.slice(), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, texture_instance_buffer.slice());

        render_pass.draw_indexed(
            0..texture_indices_buffer.size(),
            0,
            0..texture_instance_buffer.size(),
        );
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
        self.depth_texture = DepthBuffer::create_depth_texture(device, config, "depth_texture");
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
                OwnVertex {
                    position: [1.0, 1.0],
                },
                OwnVertex {
                    position: [-1.0, 1.0],
                },
                OwnVertex {
                    position: [-1.0, -1.0],
                },
                OwnVertex {
                    position: [1.0, -1.0],
                },
            ])
            .build();

        let indices_buffer = BufferCreator::indices(device)
            .label("Rect IndicesBuffer")
            .data(vec![0, 1, 2, 0, 2, 3])
            .build();

        let instances: Vec<_> = self.recs.iter().map(|rect| rect.to_instance()).collect();

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

                    OwnVertex {
                        position: [angle.cos(), angle.sin()],
                    }
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

            let instances: Vec<_> = ovals.iter().map(|oval| oval.to_instance()).collect();

            let instances_buffer = BufferCreator::vertex(device)
                .label("Rect InstanceBuffer")
                .data(instances)
                .build();

            instance_buffer_groups.push(InstanceBufferGroup(
                vertex_buffer,
                indices_buffer,
                instances_buffer,
            ));
        }

        instance_buffer_groups
    }

    pub fn image(&mut self, texture_index: usize) -> &mut Image {
        let mut image = Image::default();
        match self.textures_cords.get(texture_index) {
            None => {
                println!("No texture with the id: {} could be found", texture_index);
            }
            Some(cords) => {
                image.texture_pos = cords.0.clone();
                image.texture_scale = cords.1.clone();
            }
        };

        self.images.push(image);
        self.images.last_mut().unwrap()
    }

    fn generate_image_buffer(&self, device: &Device) -> InstanceBufferGroup {
        let vertex_buffer = BufferCreator::vertex(device)
            .label("Rect VertexBuffer")
            .data(vec![
                OwnVertex {
                    position: [1.0, 1.0],
                },
                OwnVertex {
                    position: [-1.0, 1.0],
                },
                OwnVertex {
                    position: [-1.0, -1.0],
                },
                OwnVertex {
                    position: [1.0, -1.0],
                },
            ])
            .build();

        let indices_buffer = BufferCreator::indices(device)
            .label("Rect IndicesBuffer")
            .data(vec![0, 1, 2, 0, 2, 3])
            .build();

        let instances: Vec<_> = self
            .images
            .iter()
            .map(|texture| texture.to_instance())
            .collect();

        let instances_buffer = BufferCreator::vertex(device)
            .label("Rect InstanceBuffer")
            .data(instances)
            .build();

        InstanceBufferGroup(vertex_buffer, indices_buffer, instances_buffer)
    }

    pub fn add_texture_from_bytes(
        &mut self,
        bytes: &[u8],
        device: &Device,
        queue: &Queue,
    ) -> &mut Self {
        self.textures.push(image::load_from_memory(bytes).unwrap());

        self.upload_textures(device, queue);

        self
    }

    pub fn add_textures_from_bytes(
        &mut self,
        bytes: &Vec<&[u8]>,
        device: &Device,
        queue: &Queue,
    ) -> &mut Self {
        for bytes in bytes {
            self.textures.push(image::load_from_memory(bytes).unwrap());
        }

        self.upload_textures(device, queue);

        self
    }

    fn upload_textures(&mut self, device: &Device, queue: &Queue) {
        let mut rects_to_place: GroupedRectsToPlace<usize, usize> = GroupedRectsToPlace::new();

        for (index, image) in self.textures.iter().enumerate() {
            let dimensions = image.dimensions();

            rects_to_place.push_rect(
                index,
                None,
                RectToInsert::new(dimensions.0, dimensions.1, 1),
            );
        }

        let mut target_bins = BTreeMap::new();
        target_bins.insert(0, TargetBin::new(self.texture_size, self.texture_size, 1));

        let rectangle_placements = pack_rects(
            &rects_to_place,
            &mut target_bins,
            &volume_heuristic,
            &contains_smallest_box,
        );

        let rectangle_placements = match rectangle_placements {
            Ok(rectangle_pack) => rectangle_pack,
            Err(_) => {
                self.texture_size *= 2;
                self.upload_textures(device, queue);

                return;
            }
        };

        self.textures_cords.clear();
        let mut buffer = ImageBuffer::new(self.texture_size, self.texture_size);

        for (index, (_, location)) in rectangle_placements.packed_locations() {
            buffer
                .copy_from(
                    self.textures.get(*index).unwrap(),
                    location.x(),
                    location.y(),
                )
                .expect("TODO: panic message");

            self.textures_cords.push((
                (
                    location.x() as f32 / self.texture_size as f32,
                    location.y() as f32 / self.texture_size as f32,
                ),
                (
                    location.width() as f32 / self.texture_size as f32,
                    location.height() as f32 / self.texture_size as f32,
                ),
            ))
        }

        let dimensions = buffer.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
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
        });

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &buffer,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.texture = Some(diffuse_texture_view);
    }
}

struct InstanceBufferGroup(SimpleBuffer, SimpleBuffer, SimpleBuffer);
