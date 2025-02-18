use std::num::NonZero;
use std::borrow::Cow;
use std::sync::Arc;

use winit::window::Window;

/// Handle rendering with wgpu.
pub struct Renderer {
    adapter: wgpu::Adapter,
    window: Arc<Window>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resolution: u32,
    voxel_buffer: wgpu::Buffer,
    ray_marching_pipeline: wgpu::RenderPipeline,
    ray_marching_bind_group: wgpu::BindGroup,
    ray_marching_texture: wgpu::Texture,
    ray_marching_texture_view: wgpu::TextureView,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Create a new context asynchronously (which will be resolved synchronously with pollster).
    /// Requesting an adapter and device should not take very long, so this is OK.
    pub async fn new_async(window: Arc<Window>, resolution: u32) -> Renderer {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter.");
        // create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::BUFFER_BINDING_ARRAY | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY | wgpu::Features::TEXTURE_BINDING_ARRAY,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create the device.");

        let size = window.inner_size();
        // stop wgpu from panicing if these are less than 1
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();

        surface.configure(&device, &surface_config);

        let ray_marching_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ray Marching Texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let ray_marching_texture_view = ray_marching_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Ray Marching Texture View"),
            base_array_layer: 0,
            base_mip_level: 0,
            dimension: Some(wgpu::TextureViewDimension::D2),
            array_layer_count: None,
            aspect: wgpu::TextureAspect::All,
            format: None,
            mip_level_count: None,
            usage: None,
        });

        let voxel_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Voxel Buffer"),
            size: 134217728,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        queue.write_buffer(&voxel_buffer, 0, &[0, 0, 0, 0]);

        queue.submit([]);

        let ray_marching_pipeline = Renderer::create_ray_marching_pipeline(&device);

        let render_pipeline = Renderer::create_render_pipeline(&device, surface_config.format);

        let ray_marching_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Marching Bind Group"),
            layout: &ray_marching_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { 
                    binding: 0, 
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &voxel_buffer,
                        offset: 0,
                        size: NonZero::new(voxel_buffer.size()),
                    })
                },
            ],
        });

        let render_sampler = device.create_sampler(&wgpu::SamplerDescriptor{
              mag_filter: wgpu::FilterMode::Linear,
              min_filter: wgpu::FilterMode::Linear,
              ..Default::default()
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&render_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&ray_marching_texture_view),
                },
            ],
        });

        Renderer {
            resolution,
            surface,
            surface_config,
            adapter,
            window,
            device,
            queue,
            voxel_buffer,
            ray_marching_pipeline,
            ray_marching_bind_group,
            ray_marching_texture,
            ray_marching_texture_view,
            render_pipeline,
            render_bind_group,
        }
    }

    /// Create the pipeline for ray marching voxels.
    pub fn create_ray_marching_pipeline(
        device: &wgpu::Device,
    ) -> wgpu::RenderPipeline {
        // load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Marching Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/ray_marching.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Marching Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 0,
                    count: NonZero::new(1),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: NonZero::new(134217728),
                    }
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Marching Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
            ],
            ..Default::default()
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ray Marching Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    /// Create the render pipeline.
    pub fn create_render_pipeline(device: &wgpu::Device, swap_chain_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        // load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/render.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 0,
                    count: NonZero::new(1),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
            ],
            ..Default::default()
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(swap_chain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    /// Create a context, using pollster to keep it synchronous.
    pub fn new(window: Arc<Window>, resolution: u32) -> Renderer {
        pollster::block_on(Renderer::new_async(window, resolution))
    }

    /// Update context to match a new size of the window.
    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Queue a change to the voxel buffer.
    pub fn set_voxel_buffer(&mut self, voxels: Vec<u32>) {
        self.queue.write_buffer(&self.voxel_buffer, 0, Renderer::u32_array_to_buffer(&voxels));
    }

    /// Draw the contents to the wgpu surface.
    pub fn draw(&mut self) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire the next swap chain texture.");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Ray Marching Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.ray_marching_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.ray_marching_pipeline);
            rpass.set_bind_group(0, Some(&self.ray_marching_bind_group), &[]);
            rpass.draw(0..4, 0..1);
        }
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, Some(&self.render_bind_group), &[]);
            rpass.draw(0..4, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    /// Encodes a f32 array into a buffer of u8 values.
    ///
    /// This needs to happen to pass the values to the GPU.
    /// They'll be mapped to f32 or vectors in the shader.
    fn f32_array_to_buffer(data: &[f32]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
    }

    /// Encodes a u32 array into a buffer of u8 values.
    ///
    /// This needs to happen to pass the values to the GPU.
    /// They'll be mapped to u32 or vectors in the shader.
    fn u32_array_to_buffer(data: &[u32]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
    }
}
