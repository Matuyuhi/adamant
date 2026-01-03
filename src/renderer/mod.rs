//! GPU Renderer Module
//!
//! This module handles all GPU-related operations using wgpu.
//! See docs/03_wgpu_basics.md for an introduction to wgpu concepts.
//!
//! # Architecture
//!
//! The renderer follows a game-engine style approach:
//! 1. Initialize GPU resources once (device, queue, pipeline)
//! 2. Each frame: update buffers -> encode commands -> submit
//!
//! # Key Concepts
//!
//! - **Surface**: The window we render to
//! - **Device**: The GPU abstraction
//! - **Queue**: Command submission queue
//! - **Pipeline**: The shader program + render state

mod pipeline;

use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use self::pipeline::Pipeline;

/// The main renderer struct.
///
/// Holds all wgpu resources and handles rendering.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipeline: Pipeline,
}

impl Renderer {
    /// Create a new renderer for the given window.
    ///
    /// This initializes all wgpu resources.
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        // Create wgpu instance
        // Backends::all() => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance.create_surface(window)?;

        // Request adapter (GPU selection)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find a suitable GPU adapter")?;

        log::info!("Using adapter: {:?}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Adamant Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);

        // Prefer sRGB format for correct color display
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // VSync enabled
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create render pipeline
        let pipeline = Pipeline::new(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
        })
    }

    /// Handle window resize.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // TODO: Phase 2 - Update projection matrix for text rendering
        }
    }

    /// Render a frame.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Get the next frame to render to
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear to dark background color
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw using the pipeline
            self.pipeline.draw(&mut render_pass);
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
