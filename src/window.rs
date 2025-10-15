use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use winit::event_loop::{self};
use winit::{application::ApplicationHandler, dpi::PhysicalPosition, event::{KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use wgpu::{wgt::TextureViewDescriptor, *};

pub struct State {
    surface: Surface<'static>,          // the render target essentially
    device: Device,                     // the GPU
    queue: Queue,                       // the work queue for submitting commands to the GPU
    config: SurfaceConfiguration,       // the surface settings
    brown_render_pipeline: RenderPipeline,    // render pipeline handle
    barycentric_render_pipeline: RenderPipeline,    // render pipeline handle
    is_surface_configured: bool,
    triangle_toggle: bool,
    window: Arc<Window>,
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

        // could also use include_wgsl! macro
        let brown_triangle_shader = device.create_shader_module(
            ShaderModuleDescriptor { 
                label: Some("Shader"), 
                source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()) 
            }
        );

        let barycentric_triangle_shader = device.create_shader_module(include_wgsl!("barycentric.wgsl"));

        let render_pipeline_layout  = device.create_pipeline_layout(
            &PipelineLayoutDescriptor { 
                label: Some("Render Pipeline Layout"), 
                bind_group_layouts: &[], 
                push_constant_ranges: &[] 
            }
        );

        let brown_render_pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor { 
                label: Some("Render Pipeline"), 
                layout: Some(&render_pipeline_layout), 
                vertex: VertexState { 
                    module: &brown_triangle_shader, 
                    entry_point: Some("vs_main"), 
                    compilation_options: PipelineCompilationOptions::default(), 
                    buffers: &[] 
                }, 
                fragment: Some(FragmentState {
                    module: &brown_triangle_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(), 
                    targets: &[Some(ColorTargetState { 
                        format: config.format, 
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
        );


        let barycentric_render_pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor { 
                label: Some("Render Pipeline"), 
                layout: Some(&render_pipeline_layout), 
                vertex: VertexState { 
                    module: &barycentric_triangle_shader, 
                    entry_point: Some("vs_main"), 
                    compilation_options: PipelineCompilationOptions::default(), 
                    buffers: &[] 
                }, 
                fragment: Some(FragmentState {
                    module: &barycentric_triangle_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(), 
                    targets: &[Some(ColorTargetState { 
                        format: config.format, 
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
        );
        
        Ok(Self {
            surface,
            window,
            device,
            queue,
            config,
            is_surface_configured: false,
            brown_render_pipeline,
            barycentric_render_pipeline,
            mouse_pos: (0.0, 0.0),
            triangle_toggle: true
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

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::Space, true) => self.triangle_toggle = !self.triangle_toggle,
            _ => ()
        }
    }
     
    fn handle_mouse_moved(&mut self, _event_loop: &ActiveEventLoop, pos: PhysicalPosition<f64>) {
        self.mouse_pos = (pos.x, pos.y);
    }

    fn update(&mut self) {

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
                                        r: self.mouse_pos.0 / self.config.width as f64, 
                                        g: self.mouse_pos.1 / self.config.height as f64, 
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
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}


pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>
}

impl App {

    #[allow(dead_code)]
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::UnwrapThrowExt;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // not on web, use pollster to await. otherwise make it async
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            State::new(window)
                                .await
                                .expect("Unable to create canvas!")
                        )
                        .is_ok()
                    ) 
                });
            }
        }
    }


    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height
            );
        }
        self.state = Some(event);
    } 
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => (),
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    },
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            },

            WindowEvent::KeyboardInput { 
                event: KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(&event_loop, code, key_state.is_pressed()),

            WindowEvent::CursorMoved { 
                position,
                ..
            } => state.handle_mouse_moved(&event_loop, position),

            _ => ()
        }
    }
}


#[allow(dead_code)]
pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::UnwrapThrowExt;
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}