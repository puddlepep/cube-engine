use crate::game::renderer::{mesh::Mesh, vertex::Vertex};
use cgmath::Vector3;
use super::texture::Texture;

// relative x/y & offset x/y
pub struct UIVector {
    pub rx: f32,
    pub ry: f32,
    pub ox: f32,
    pub oy: f32,
}

pub struct UIRect {
    pub position: UIVector,
    pub scale: UIVector,
    pub texture: Texture,

    pub mesh: Option<Mesh>,
}

impl UIRect {

    pub fn new(position: UIVector, scale: UIVector, device: &wgpu::Device, queue: &wgpu::Queue, texture_path: String) -> UIRect {

        let texture = Texture::from_path(device, queue, texture_path);

        UIRect {
            position,
            scale,
            texture,
            mesh: None,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, window_width: u32, window_height: u32) {

        let pos = &self.position;
        let scl = &self.scale;

        let pos_base = [ (pos.rx * window_width as f32) + pos.ox, (pos.ry * window_height as f32) + pos.oy ];
        let p0 = Vector3::new(pos_base[0] + scl.ox, pos_base[1] + scl.oy, 0.0);
        let p1 = Vector3::new(pos_base[0], pos_base[1] + scl.oy, 0.0);
        let p2 = Vector3::new(pos_base[0], pos_base[1], 0.0);
        let p3 = Vector3::new(pos_base[0] + scl.ox, pos_base[1], 0.0);

        let mut vertices: Vec<Vertex> = Vec::new();
        vertices.push(Vertex { position: p0.into(), normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0] });
        vertices.push(Vertex { position: p1.into(), normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0] });
        vertices.push(Vertex { position: p2.into(), normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0] });
        vertices.push(Vertex { position: p3.into(), normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] });

        let mut indices: Vec<u16> = Vec::new();
        indices.push(0);
        indices.push(1);
        indices.push(2);

        indices.push(0);
        indices.push(2);
        indices.push(3);

        self.mesh = Some(Mesh::new(device, vertices, indices));

    }
}

pub struct UIManager {
    pub uniforms: super::uniforms::UIUniforms,
    pub pipeline: wgpu::RenderPipeline,
    pub render_pool: Vec<UIRect>,
}

impl UIManager {

    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, queue: &wgpu::Queue) -> UIManager {

        let uniforms = super::uniforms::UIUniforms::new(device);

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
            flags: wgpu::ShaderFlags::empty(),
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                }
            ],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniforms.bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[super::vertex::Vertex::layout()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[
                    wgpu::ColorTargetState {
                        format: swap_chain_desc.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                ],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            multisample: wgpu::MultisampleState::default(),

            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),

        });

        let mut crosshair = UIRect::new(
            UIVector { rx: 0.5, ry: 0.5, ox: -8.0, oy: -8.0 }, 
            UIVector { rx: 0.0, ry: 0.0, ox: 16.0, oy: 16.0 },
            device,
            queue,
            "./src/game/data/ui/crosshair.png".into(),
        );
        crosshair.update(device, swap_chain_desc.width, swap_chain_desc.height);

        let mut render_pool = Vec::new();
        render_pool.push(crosshair);

        UIManager {
            uniforms,
            pipeline,
            render_pool,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {

        for rect in &mut self.render_pool {
            rect.update(device, width, height);
        }
    }
}