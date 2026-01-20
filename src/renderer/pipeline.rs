//! Render Pipeline
//!
//! This module handles the wgpu render pipeline setup.
//! A pipeline defines how vertices are processed and pixels are colored.
//!
//! See docs/04_shaders.md for shader development guide.

use wgpu::{util::DeviceExt, RenderPass};

/// Vertex data for a unit quad.
/// Position in [0, 1] range, will be scaled by instance size.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Unit quad vertices (two triangles).
/// Coordinates in [0, 1] range for easy scaling.
const QUAD_VERTICES: &[Vertex] = &[
    // Triangle 1
    Vertex { position: [0.0, 0.0] }, // bottom-left
    Vertex { position: [1.0, 0.0] }, // bottom-right
    Vertex { position: [0.0, 1.0] }, // top-left
    // Triangle 2
    Vertex { position: [1.0, 0.0] }, // bottom-right
    Vertex { position: [1.0, 1.0] }, // top-right
    Vertex { position: [0.0, 1.0] }, // top-left
];

/// Per-instance data for each quad.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub pos: [f32; 2],   // Position in clip space
    pub size: [f32; 2],  // Size in clip space
    pub color: [f32; 4], // RGBA color
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        1 => Float32x2,  // pos
        2 => Float32x2,  // size
        3 => Float32x4,  // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// The render pipeline wrapper.
pub struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl Pipeline {
    /// Create a new render pipeline.
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/quad.wgsl").into()),
        });

        // Create pipeline layout
        // TODO: Phase 2 - Add bind groups for textures (glyph atlas)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        // Create render pipeline with vertex and instance buffer layouts
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling for 2D quads
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
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create test instances (grid of colored quads)
        let instances = Self::create_test_instances();
        let instance_count = instances.len() as u32;

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            instance_count,
        }
    }

    /// Create test instances for visual debugging.
    fn create_test_instances() -> Vec<Instance> {
        let cols = 16;
        let rows = 8;
        let quad_size = 0.1;
        let spacing = 0.12;

        let mut instances = Vec::with_capacity(cols * rows);

        for row in 0..rows {
            for col in 0..cols {
                // Calculate position (centered grid)
                let x = (col as f32 - cols as f32 / 2.0) * spacing;
                let y = (row as f32 - rows as f32 / 2.0) * spacing;

                // Rainbow color based on position
                let r = col as f32 / cols as f32;
                let g = row as f32 / rows as f32;
                let b = 1.0 - (r + g) / 2.0;

                instances.push(Instance {
                    pos: [x, y],
                    size: [quad_size, quad_size],
                    color: [r, g, b, 1.0],
                });
            }
        }

        instances
    }

    /// Draw all instances.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        // Draw 6 vertices (quad) for each instance
        render_pass.draw(0..6, 0..self.instance_count);
    }
}
