use std::{f32::consts::PI, sync::Arc};

use nalgebra::Vector3;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::{self};
use winit::{dpi::PhysicalPosition, event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use wgpu::{util::{BufferInitDescriptor, DeviceExt}, wgt::TextureViewDescriptor, *};

use crate::{render, shader_structs::{Camera, CameraController, CameraUniform, INDICES, VERTICES, Vertex}};
use crate::texture::Texture;

pub struct State {
    surface: Surface<'static>,          // the render target essentially
    device: Device,                     // the GPU
    queue: Queue,                       // the work queue for submitting commands to the GPU
    config: SurfaceConfiguration,       // the surface settings
    brown_render_pipeline: RenderPipeline,    // render pipeline handle
    barycentric_render_pipeline: RenderPipeline,    // render pipeline handle
    vertex_buffer: Buffer,
    index_buffer: Buffer,

    diffuse_bind_group: BindGroup,
    diffuse_texture: Texture,

    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    is_surface_configured: bool,
    triangle_toggle: bool,
    num_indices: u32,

    pub window: Arc<Window>,
    mouse_pos: (f64, f64)
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance_descriptor = 
            InstanceDescriptor {
                #[cfg(target_arch = "wasm32")]
                backends: Backends::BROWSER_WEBGPU,
                #[cfg(not(target_arch = "wasm32"))]
                backends: Backends::PRIMARY,
                ..Default::default()
            };

        let instance = Instance::new(
            &instance_descriptor
        );

        let surface = instance.create_surface(window.clone()).unwrap();

        // adapter is a handle for our graphics card
        let adapter = instance.request_adapter( 
            &RequestAdapterOptions { 
                power_preference: PowerPreference::default(), 
                force_fallback_adapter: false, 
                compatible_surface: Some(&surface)
            }
        )
        .await?;

        println!("GPU: {}", adapter.get_info().name);

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor { 
                label: None, 
                required_features: Features::empty(), 
                required_limits: 
                    if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    }
                , 
                memory_hints: Default::default(), 
                trace: Trace::Off 
            }
        )
        .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_fmt = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_fmt,
            
            #[cfg(not(target_arch = "wasm32"))]
            width: size.width,
            #[cfg(not(target_arch = "wasm32"))]
            height: size.height,

            #[cfg(target_arch = "wasm32")]
            width: size.width.min(2048),
            #[cfg(target_arch = "wasm32")]
            height: size.height.min(2048),

            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let diffuse_bytes = include_bytes!("../happy-tree.png");
        let texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "Happy Tree Texture").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture { 
                            sample_type: TextureSampleType::Float { filterable: true }, 
                            view_dimension: TextureViewDimension::D2, 
                            multisampled: false
                        },
                        count: None
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None
                    }
                ],
                label: Some("texture_bind_group_layout")
            },
        );

        let diffuse_bind_group = device.create_bind_group(
            &BindGroupDescriptor { 
                label: Some("Diffuse Bind Group"), 
                layout: &texture_bind_group_layout, 
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture.view)
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&texture.sampler)
                    }
                ] 
            }
        );

        let brown_triangle_shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let barycentric_triangle_shader = device.create_shader_module(include_wgsl!("barycentric.wgsl"));
        
        let vertex_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX
            }
        );

        let index_buffer = device.create_buffer_init(
            &BufferInitDescriptor { 
                label: Some("Index Buffer"), 
                contents: bytemuck::cast_slice(INDICES), 
                usage: BufferUsages::INDEX
            }
        );

        let camera = Camera {
            sphericals: [4.0, PI / 4.0, PI / 4.0].into(),
            target: [0.0, 0.0, 0.0].into(),
            up: Vector3::y(),
            aspect_ratio: config.width as f32 / config.height as f32,
            fovy: 45.0,
            zfar: 100.0,
            znear: 0.1,
            cam_controller: CameraController {
                w: false,
                a: false,
                d: false,
                e: false,
                q: false,
                s: false
            }
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        
        let camera_buffer = device.create_buffer_init(
            &BufferInitDescriptor { 
                label: Some("Camera Uniform Buffer"), 
                contents: bytemuck::cast_slice(&[camera_uniform]), 
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST 
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor { 
                label: Some("Camera Bind Group Layout"), 
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        count: None,
                        ty: BindingType::Buffer { 
                            ty: BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None 
                        }
                    }
                ] 
            }
        );

        let camera_bind_group = device.create_bind_group(
            &BindGroupDescriptor { 
                label: Some("Camera Bind Group"), 
                layout: &camera_bind_group_layout, 
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding()
                    }
                ]
            }
        );
        
        let render_pipeline_layout  = device.create_pipeline_layout(
            &PipelineLayoutDescriptor { 
                label: Some("Render Pipeline Layout"), 
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout], 
                push_constant_ranges: &[] 
            }
        );

        let brown_render_pipeline = make_pipeline_desc_from_shader(&device, &render_pipeline_layout, &brown_triangle_shader, config.format);
        let barycentric_render_pipeline = make_pipeline_desc_from_shader(&device, &render_pipeline_layout, &barycentric_triangle_shader, config.format);

        Ok(Self {
            surface,
            window,
            device,
            queue,
            config,
            is_surface_configured: false,
            brown_render_pipeline,
            barycentric_render_pipeline,
            vertex_buffer,
            mouse_pos: (0.0, 0.0),
            triangle_toggle: true,
            num_indices: INDICES.len() as u32,
            index_buffer,
            diffuse_bind_group,
            diffuse_texture: texture,
            camera,
            camera_bind_group,
            camera_buffer
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.config.width = width;
                self.config.height = height;
            }
            #[cfg(target_arch = "wasm32")]
            {
                self.config.width = width.min(2048);
                self.config.height = height.min(2048);
            }
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::Space, true) => self.triangle_toggle = !self.triangle_toggle,

            (KeyCode::KeyQ, x) => self.camera.cam_controller.q = x,
            (KeyCode::KeyE, x) => self.camera.cam_controller.e = x,
            (KeyCode::KeyA, x) => self.camera.cam_controller.a = x,
            (KeyCode::KeyD, x) => self.camera.cam_controller.d = x,
            (KeyCode::KeyW, x) => self.camera.cam_controller.w = x,
            (KeyCode::KeyS, x) => self.camera.cam_controller.s = x,

            _ => ()
        }
    }
     
    pub fn handle_mouse_moved(&mut self, _event_loop: &ActiveEventLoop, pos: PhysicalPosition<f64>) {
        self.mouse_pos = (pos.x, pos.y);
    }

    pub fn update(&mut self) {
        self.camera.update();
        
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), SurfaceError>{
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder( &CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(
                &RenderPassDescriptor { 
                    label: Some("Render Pass"), 
                    color_attachments: &[Some(
                        RenderPassColorAttachment { 
                            view: &view, 
                            resolve_target: None, 
                            ops: Operations { 
                                load: LoadOp::Clear(
                                    Color { 
                                        r: 0.0, 
                                        g: 0.0, 
                                        b: 0.0, 
                                        a: 1.0 
                                    }
                                ), 
                                store: StoreOp::Store
                            },
                            depth_slice: None, 
                        }
                    )], 
                    depth_stencil_attachment: None, 
                    timestamp_writes: None, 
                    occlusion_query_set: None 
                }
            );

            render_pass.set_pipeline(if self.triangle_toggle { &self.brown_render_pipeline } else { &self.barycentric_render_pipeline });
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}






//helper fn for render pipeline descriptors
fn make_pipeline_desc_from_shader(device: &Device, layout: &PipelineLayout, shader: &ShaderModule, fmt: TextureFormat) -> RenderPipeline {
    let vertex_buffer_layout = Vertex::desc();
    
    device.create_render_pipeline(
        &RenderPipelineDescriptor { 
            label: Some("Render Pipeline"), 
            layout: Some(layout), 
            vertex: VertexState { 
                module: shader, 
                entry_point: Some("vs_main"), 
                compilation_options: PipelineCompilationOptions::default(), 
                buffers: &[vertex_buffer_layout] 
            }, 
            fragment: Some(FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(), 
                targets: &[Some(ColorTargetState { 
                    format: fmt, 
                    blend: Some(BlendState::REPLACE), 
                    write_mask: ColorWrites::ALL 
                })]
            }),
            primitive: PrimitiveState { 
                topology: PrimitiveTopology::TriangleList, 
                strip_index_format: None, 
                front_face: FrontFace::Ccw, 
                cull_mode: Some(Face::Back), 
                unclipped_depth: false, 
                polygon_mode: PolygonMode::Fill, 
                conservative: false 
            }, 
            depth_stencil: None,
            multiview: None, 
            cache: None,
            multisample: MultisampleState { 
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false 
            }, 
        }
    )
}
