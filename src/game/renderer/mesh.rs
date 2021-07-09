use wgpu::util::DeviceExt;

use super::vertex::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Mesh {

    pub fn new(device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<u16>) -> Mesh {

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        Mesh {
            vertices,
            indices,

            vertex_buffer,
            index_buffer,
        }

    }

}