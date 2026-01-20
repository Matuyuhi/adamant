//! Application state and event loop.
//!
//! This module contains the main application struct and the event loop logic.
//! See docs/02_event_loop.md for details on how the game-loop style architecture works.

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::renderer::Renderer;

/// Main application state.
///
/// This struct holds all the state needed to run Adamant.
/// It follows a game-loop style architecture rather than traditional MVC.
pub struct App {
    /// The window handle (None until resumed)
    window: Option<Arc<Window>>,
    /// The GPU renderer (None until window is created)
    renderer: Option<Renderer>,
}

impl App {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
        }
    }

    /// Run the application.
    ///
    /// This is the main entry point that creates the event loop and runs the application.
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;

        let mut app = App::new();

        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Application resumed, creating window...");

        // Create window attributes
        let window_attributes = Window::default_attributes()
            .with_title("Adamant")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .with_transparent(true);

        // Create the window
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        // Initialize the renderer
        // TODO: This blocks - consider moving to a separate task for smoother startup
        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window)))
            .expect("Failed to create renderer");

        self.window = Some(window);
        self.renderer = Some(renderer);

        log::info!("Window and renderer initialized");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting...");
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                log::debug!("Window resized to {:?}", new_size);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    // TODO: Update state here (Phase 3: read PTY output, update grid)

                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            log::warn!("Surface lost, reconfiguring...");
                            if let Some(window) = &self.window {
                                renderer.resize(window.inner_size());
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Out of GPU memory!");
                            event_loop.exit();
                        }
                        Err(e) => {
                            log::warn!("Render error: {:?}", e);
                        }
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                // TODO: Phase 3 - Send key events to PTY
                log::trace!("Key event: {:?}", event);
            }

            _ => {}
        }
    }
}
