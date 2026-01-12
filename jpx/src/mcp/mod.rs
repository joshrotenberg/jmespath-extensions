//! MCP server for JMESPath evaluation and introspection
//!
//! Provides tools for evaluating JMESPath expressions and discovering
//! available functions.

mod tools;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub use tools::JpxMcp;

/// Run the MCP server on stdio
///
/// # Arguments
/// * `strict` - If true, only standard JMESPath functions are available in evaluate tools
pub async fn run(strict: bool) -> Result<()> {
    // Initialize tracing to stderr (stdout is used for MCP protocol)
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive("jpx_mcp=info".parse()?))
        .init();

    info!(
        "Starting jpx MCP server{}",
        if strict { " (strict mode)" } else { "" }
    );

    let service = JpxMcp::new(strict);
    let server = service.serve(stdio()).await?;

    info!("jpx MCP server running on stdio");

    server.waiting().await?;

    Ok(())
}
