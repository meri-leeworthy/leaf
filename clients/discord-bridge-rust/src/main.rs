// Discord Bridge for Roomy
//
// Standalone binary for running the Discord → Roomy bridge

use discord_bridge_rust::Result;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("discord_bridge=debug".parse().unwrap())
                .add_directive("roomy_sdk=info".parse().unwrap()),
        )
        .init();

    info!("🌉 Discord Bridge for Roomy v{}", discord_bridge_rust::VERSION);
    info!("Starting bridge...");

    // TODO: Load configuration from environment/database
    // TODO: Initialize orchestrator
    // TODO: Run until shutdown signal

    info!("Bridge initialized successfully");

    Ok(())
}
