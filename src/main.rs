//! Adamant - The Unbreakable Terminal
//!
//! Entry point for the terminal emulator.
//! See docs/01_architecture.md for the overall design.

use adamant::App;

fn main() {
    // Initialize logging (set RUST_LOG=debug for verbose output)
    env_logger::init();

    log::info!("Starting Adamant...");

    // Run the application
    // TODO: Handle errors gracefully instead of unwrap
    pollster::block_on(App::run()).unwrap();
}
