pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    shader_src: &'a str,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    surface_format: wgpu::TextureFormat,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        Self {
            device,
            shader_src: "",
            vertex_layouts: vec![],
            surface_format,
        }
    }

    pub fn shader(mut self, src: &'a str) -> Self {
        self.shader_src = src;
        self
    }

    pub fn vertex_layout(mut self, layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    pub fn build(self, label: &str) -> wgpu::RenderPipeline {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(self.shader_src.into()),
        });

        let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &self.vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        })
    }
}