//! Render Pipeline
//!
//! This module handles the wgpu render pipeline setup.
//! A pipeline defines how vertices are processed and pixels are colored.
//!
//! See docs/04_shaders.md for shader development guide.

use wgpu::RenderPass;

/// The render pipeline wrapper.
///
/// This struct encapsulates all pipeline-related resources.
pub struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    // TODO: Phase 1 - Add vertex/instance buffers for instancing
    // vertex_buffer: wgpu::Buffer,
    // instance_buffer: wgpu::Buffer,
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
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
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
                cull_mode: Some(wgpu::Face::Back),
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
            multiview: None,
            cache: None,
        });

        Self { render_pipeline }
    }

    /// Draw the current state.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);

        // Draw a single quad (2 triangles = 6 vertices)
        // TODO: Phase 1 - Replace with instanced draw for many quads
        render_pass.draw(0..6, 0..1);
    }
}
