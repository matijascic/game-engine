use bytemuck;
use nalgebra::{Matrix4, Point3, Vector3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

mod bindings;
use bindings::default as shader;

struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Mesh {
    fn triangle(device: &wgpu::Device) -> Self {
        // Note: VertexInput uses [f32; 4] for alignment, but shader only reads xyz
        let vertices = [
            shader::VertexInput::new([0.0, 0.5, 0.0, 0.0]),
            shader::VertexInput::new([-0.5, -0.5, 0.0, 0.0]),
            shader::VertexInput::new([0.5, -0.5, 0.0, 0.0]),
        ];
        let indices: [u32; 3] = [0, 1, 2];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    ctx: Option<gfx::GfxContext>,
    mesh: Option<Mesh>,
    uniform_buffer: Option<wgpu::Buffer>,
    bind_group: Option<shader::WgpuBindGroup0>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            ctx: None,
            mesh: None,
            uniform_buffer: None,
            bind_group: None,
        }
    }

    fn build_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader_module = shader::create_shader_module_embed_source(device);
        let pipeline_layout = shader::create_pipeline_layout(device);
        let vertex_entry = shader::vs_main_entry(wgpu::VertexStepMode::Vertex);
        let fragment_entry = shader::fs_main_entry([Some(wgpu::ColorTargetState {
            format: surface_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })]);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Default Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: shader::vertex_state(&shader_module, &vertex_entry),
            fragment: Some(shader::fragment_state(&shader_module, &fragment_entry)),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling for now
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
            multiview_mask: None,
            cache: None,
        })
    }

    fn build_uniforms(aspect: f32) -> shader::Uniforms {
        // View-projection matrix (camera looking at origin)
        let proj = Matrix4::new_perspective(aspect, 60.0_f32.to_radians(), 0.1, 100.0);
        let view = Matrix4::look_at_rh(
            &Point3::new(0.0, 0.0, 2.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::y(),
        );
        let view_proj = proj * view;

        // Model matrix (identity for now)
        let model = Matrix4::identity();

        shader::Uniforms::new(view_proj.into(), model.into())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("engine")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)),
                )
                .unwrap(),
        );

        // Create gfx context
        let mut ctx = gfx::GfxContext::new(window.clone());

        // Build pipeline
        let pipeline = Self::build_pipeline(&ctx.device, ctx.surface_format());
        ctx.set_pipeline(pipeline);

        // Create mesh
        let mesh = Mesh::triangle(&ctx.device);

        // Create uniform buffer
        let aspect = ctx.size.width as f32 / ctx.size.height as f32;
        let uniforms = Self::build_uniforms(aspect);
        let uniform_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create bind group using generated helpers
        let bind_group = shader::WgpuBindGroup0::from_bindings(
            &ctx.device,
            shader::WgpuBindGroup0Entries::new(shader::WgpuBindGroup0EntriesParams {
                uniforms: wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                },
            }),
        );

        self.ctx = Some(ctx);
        self.window = Some(window);
        self.mesh = Some(mesh);
        self.uniform_buffer = Some(uniform_buffer);
        self.bind_group = Some(bind_group);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let ctx = match &mut self.ctx {
            Some(c) => c,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                ctx.resize(size);
                // Update uniforms on resize
                if let Some(uniform_buffer) = &self.uniform_buffer {
                    let aspect = size.width as f32 / size.height as f32;
                    let uniforms = Self::build_uniforms(aspect);
                    ctx.queue
                        .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(mesh), Some(bind_group)) = (&self.mesh, &self.bind_group) {
                    let result = ctx.render_with(|pass| {
                        bind_group.set(pass);
                        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                    });
                    if result.is_err() {
                        ctx.resize(ctx.size);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
