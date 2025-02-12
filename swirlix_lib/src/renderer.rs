use std::num::NonZero;
use std::borrow::Cow;
use std::sync::Arc;

use winit::window::Window;

use wgpu::MemoryHints::Performance;
use wgpu::{BindGroupLayoutDescriptor, PipelineLayoutDescriptor, ShaderSource};

/// Handle rendering with wgpu.
pub struct Renderer {
    adapter: wgpu::Adapter,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    voxel_buffer: wgpu::Buffer,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
}

impl Renderer {
    /// Create a new context asynchronously (which will be resolved synchronously with pollster).
    /// Requesting an adapter and device should not take very long, so this is OK.
    pub async fn new_async(window: Arc<Window>) -> Renderer {
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
                    required_features: wgpu::Features::BUFFER_BINDING_ARRAY | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: Performance,
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

        let voxel_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Voxel Buffer"),
            size: 65536 * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        queue.write_buffer(&voxel_buffer, 0, &[0, 0, 0, 0]);

        queue.submit([]);

        let render_pipeline = Renderer::create_render_pipeline(&device, surface_config.format);

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group For Voxel Buffer"),
            layout: &render_pipeline.get_bind_group_layout(0),
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

        Renderer {
            surface,
            surface_config,
            adapter,
            window,
            device,
            queue,
            render_pipeline,
            render_bind_group,
            voxel_buffer,
        }
    }

    /// Create the render pipeline.
    pub fn create_render_pipeline(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        // load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Renderer Shader Module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/render.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout for Voxel Buffer"),
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
                        min_binding_size: NonZero::new(65536 * 4),
                    }
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(& PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            ..Default::default()
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
                // strip_index_format: None,
                // front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                // // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // // or Features::POLYGON_MODE_POINT
                // polygon_mode: wgpu::PolygonMode::Fill,
                // // Requires Features::DEPTH_CLIP_CONTROL
                // unclipped_depth: false,
                // // Requires Features::CONSERVATIVE_RASTERIZATION
                // conservative: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    /// Create a context, using pollster to keep it synchronous.
    pub fn new(window: Arc<Window>) -> Renderer {
        pollster::block_on(Renderer::new_async(window))
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
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
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
