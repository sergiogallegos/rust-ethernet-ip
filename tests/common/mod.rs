// Common test utilities and helpers

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize tracing for tests
/// This ensures tracing is set up once for all tests
pub fn init_test_tracing() {
    INIT.call_once(|| {
        use tracing_subscriber::fmt;
        use tracing_subscriber::EnvFilter;

        let filter = EnvFilter::try_from_env("RUST_LOG")
            .unwrap_or_else(|_| EnvFilter::new("warn"));

        let subscriber = fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_test_writer()
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber for tests");
    });
}
