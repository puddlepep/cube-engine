
mod uniforms;
pub mod texture;
pub mod vertex;
pub mod mesh;

use mesh::Mesh;
use vertex::Vertex;

use super::color::Color;

pub struct Renderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub uniforms: uniforms::Uniforms,
    pub depth_texture: texture::Texture,
    pub block_atlas: texture::Texture,
}

pub const WIREFRAME_MODE: bool = false;

impl Renderer {

    pub async fn new(window: &winit::window::Window) -> Renderer {

        let size = window.inner_size();
        let wgpu_instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { wgpu_instance.create_surface(window) };
        let adapter = wgpu_instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }).await.expect("Failed to create adapter.");

        // Creates a device and a command queue.
        // The device is a connection to the GPU, and the command queue
        // is the list of commands for the GPU to perform.
        let (device, queue) = adapter.request_device(
            
            &wgpu::DeviceDescriptor 
            {
                label: None,
                features: if WIREFRAME_MODE { wgpu::Features::NON_FILL_POLYGON_MODE } else { wgpu::Features::empty() },
                limits: wgpu::Limits::default(),
            },
            None
        ).await.expect("Failed to get device & queue.");

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            flags: wgpu::ShaderFlags::empty(),
        });

        // Defines and creates the swap chain, which is a series of images that are displayed to the window.
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
        let depth_texture = texture::Texture::create_depth_texture(&device, &swap_chain_desc);
        
        let block_atlas = texture::Texture::from_path(&device, &queue, String::from("./src/game/data/blocks/atlas.png"));

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

        let uniforms = uniforms::Uniforms::new(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniforms.bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[Vertex::layout()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[
                    wgpu::ColorTargetState {
                        format: swap_chain_desc.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                ],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
                polygon_mode: if WIREFRAME_MODE { wgpu::PolygonMode::Line} else { wgpu::PolygonMode::Fill }, 
                conservative: false,
            },
            multisample: wgpu::MultisampleState::default(),

            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),

        });

        Renderer {
            render_pipeline,
            swap_chain_desc,
            swap_chain,
            device,
            queue,
            surface,
            uniforms,
            depth_texture,
            block_atlas,
        }
    }

    pub fn render(&mut self, camera: &super::camera::Camera, pool: &Vec<&Mesh>, sky_color: Color) {

        self.uniforms.update_view_proj(camera.build_view_projection_matrix());
        self.uniforms.write(&self.queue);

        let frame = self.swap_chain
            .get_current_frame()
            .expect("Failed to aquire next swap chain texture")
            .output;
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        {

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(sky_color.into()),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniforms.bind_group, &[]);

            // weird bypass, if i dont do this it errors.
            // i know chunk_texture.bind_group is *always* valid, but the renderer doesn't.
            match &self.block_atlas.bind_group {
                Some(bg) => {
                    render_pass.set_bind_group(1, bg, &[]);
                },
                None => ()
            }

            

            for mesh in pool {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
            }

        }
        
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.swap_chain_desc);

        //self.camera.aspect = new_size.width as f32 / new_size.height as f32;
    }

}