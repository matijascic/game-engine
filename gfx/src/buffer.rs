use wgpu::util::DeviceExt;

pub struct VertexBuffer {
    pub buffer: wgpu::Buffer,
    pub count: u32,
}

impl VertexBuffer {
    pub fn new<T: bytemuck::Pod>(device: &wgpu::Device, data: &[T], label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self { buffer, count: data.len() as u32 }
    }
}

pub struct IndexBuffer {
    pub buffer: wgpu::Buffer,
    pub count: u32,
}

impl IndexBuffer {
    pub fn new(device: &wgpu::Device, data: &[u16], label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self { buffer, count: data.len() as u32 }
    }
}