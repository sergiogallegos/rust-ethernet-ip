// stream_injection_example.rs
// =========================================================================
// Example demonstrating custom stream injection for EipClient
//
// This example shows how to use connect_with_stream() to:
// - Apply custom socket options (keepalive, timeouts, etc.)
// - Wrap streams for metrics/observability
// - Use in-memory streams for testing
//
// To run: cargo run --example stream_injection_example
// =========================================================================

use rust_ethernet_ip::{EipClient, RoutePath};
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!(
        "╔══════════════════════════════════════════════════════════════════════════════╗"
    );
    tracing::info!(
        "║  Stream Injection Example                                                  ║"
    );
    tracing::info!(
        "║  Demonstrates custom stream injection for EipClient                        ║"
    );
    tracing::info!(
        "╚══════════════════════════════════════════════════════════════════════════════╝"
    );
    tracing::info!("");

    // Get PLC connection details
    print!("Enter PLC IP address (default: 192.168.1.100): ");
    io::stdout().flush()?;
    let mut ip = String::new();
    io::stdin().read_line(&mut ip)?;
    let ip = ip.trim();
    let ip = if ip.is_empty() { "192.168.1.100" } else { ip };

    print!("Enter CPU slot (default: 0 for CompactLogix): ");
    io::stdout().flush()?;
    let mut slot = String::new();
    io::stdin().read_line(&mut slot)?;
    let slot: u8 = slot.trim().parse().unwrap_or(0);

    tracing::info!("🔌 Connecting to PLC at {}:44818 (slot {})...", ip, slot);

    // Example 1: Standard connection (unchanged API)
    tracing::info!(
        "\n═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("Example 1: Standard connection (existing API)");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let route_path = RoutePath::new().add_slot(slot);
    match EipClient::with_route_path(&format!("{}:44818", ip), route_path.clone()).await {
        Ok(client) => {
            tracing::info!("✅ Standard connection successful!");
            drop(client);
        }
        Err(e) => {
            tracing::warn!(
                "⚠️  Standard connection failed: {} (this is expected if PLC is not available)",
                e
            );
        }
    }

    // Example 2: Custom stream with socket options
    tracing::info!(
        "\n═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("Example 2: Custom stream with socket options");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let addr: SocketAddr = format!("{}:44818", ip).parse()?;
    match TcpStream::connect(addr).await {
        Ok(stream) => {
            // Apply custom socket options
            if let Err(e) = stream.set_nodelay(true) {
                tracing::warn!("⚠️  Failed to set nodelay: {}", e);
            }

            // Connect using custom stream
            match EipClient::connect_with_stream(stream, Some(route_path.clone())).await {
                Ok(client) => {
                    tracing::info!("✅ Custom stream connection successful!");
                    drop(client);
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️  Custom stream connection failed: {} (this is expected if PLC is not available)",
                        e
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "⚠️  Failed to create TCP stream: {} (this is expected if PLC is not available)",
                e
            );
        }
    }

    tracing::info!(
        "\n═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("Example completed!");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("\n💡 Benefits of stream injection:");
    tracing::info!("   • Apply custom socket options (keepalive, timeouts, bind local addr)");
    tracing::info!("   • Wrap streams for metrics/observability (bytes in/out)");
    tracing::info!("   • Reuse pre-established tunnels/connections");
    tracing::info!("   • Use in-memory streams for deterministic testing");

    Ok(())
}
