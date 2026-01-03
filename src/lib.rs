//! Adamant Terminal Emulator Library
//!
//! This crate provides the core functionality for Adamant.
//!
//! # Module Structure
//!
//! - `renderer`: GPU rendering pipeline using wgpu
//! - `grid`: Terminal state grid (Phase 3)
//! - `pty`: Pseudo-terminal handling (Phase 3)

pub mod renderer;

// TODO: Phase 3 - Uncomment when implementing terminal logic
// pub mod grid;
// pub mod pty;

mod app;

pub use app::App;
