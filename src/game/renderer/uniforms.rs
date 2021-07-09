use wgpu::{Queue, util::DeviceExt};
use crate::game::color::Color;
use cgmath::{InnerSpace, Vector3};

// IMPORANT!! wgpu requires that uniforms are spaced by 16 bytes! 
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Data {
    pub view_proj: [[f32; 4]; 4],

    pub light_color: [f32; 3],
    pub padding: u32,
    pub light_direction: [f32; 3],
}

pub struct Uniforms {
    pub data: Data,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Uniforms {

    pub fn new(device: &wgpu::Device) -> Uniforms {
        use cgmath::SquareMatrix;

        let data = Data {
            view_proj: cgmath::Matrix4::identity().into(),

            light_color: Color::from_rgb(1.0, 1.0, 1.0).into(),
            padding: 0,
            light_direction: Vector3::new(0.0, 1.0, 0.0).normalize().into(),
        };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: None,
        });

        Uniforms { data, buffer, bind_group_layout, bind_group }
    }

    pub fn write(&self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }

    pub fn update_view_proj(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.data.view_proj = matrix.into();
    }
}