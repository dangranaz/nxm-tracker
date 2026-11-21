//! Entry point for `nxm-tracker` MCP server.
//!
//! Startup contract (in order):
//! 1. `nxm_mcp_server::init_logging()` — JSON structured logging
//! 2. `TrackerConfig::load()` — configuration from ~/.nxmconfig
//! 3. `Database::open()` — SQLite datasource
//! 4. `default_migration_set()` + `db.migrate()` — apply schema migrations
//! 5. Register MCP tools on `ServerContext`
//! 6. `serve()` — reactive stdio loop

use std::sync::Arc;

use nxm_mcp_server::mcp::tool::McpTool;
use nxm_mcp_server::{init_logging, serve, ServerContext};
use nxm_tracker::{
    default_migration_set, CloseSessionTool, CreateFeatureSessionTool, CreateSessionTool, Database,
    GetSessionTool, ListFeatureSessionsTool, ListSessionsTool, SetDesignPhaseCompleteTool,
    SqliteSessionRepository, TrackerConfig, UpdateSessionTool,
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Structured logging (JSON, env NXM_MCP_LOG > RUST_LOG > info).
    init_logging().map_err(|e| format!("logging init failed: {e}"))?;

    // 2. Load configuration.
    let _config = TrackerConfig::load().map_err(|e| format!("config load failed: {e}"))?;
    tracing::info!(target: "nexum::tracker::main", "configuration loaded");

    // 3. Open database and apply migrations.
    let db = Database::open().map_err(|e| format!("database open failed: {e}"))?;
    tracing::info!(target: "nexum::tracker::main", "database opened");

    let migration_set =
        default_migration_set().map_err(|e| format!("migration set failed: {e}"))?;
    db.migrate(&migration_set)
        .map_err(|e| format!("migration failed: {e}"))?;
    tracing::info!(target: "nexum::tracker::main", "migrations applied");

    // 4. Build session repository (shared across all MCP tools).
    let repo = Arc::new(SqliteSessionRepository::from_database(db));

    // 5. Build MCP context and register tools.
    let mut ctx = ServerContext::new("nxm-tracker", env!("CARGO_PKG_VERSION"));

    ctx.register_tool(Arc::new(CreateSessionTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>);
    ctx.register_tool(Arc::new(GetSessionTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>);
    ctx.register_tool(Arc::new(ListSessionsTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>);
    ctx.register_tool(Arc::new(UpdateSessionTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>);
    ctx.register_tool(Arc::new(CloseSessionTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>);
    ctx.register_tool(
        Arc::new(SetDesignPhaseCompleteTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>
    );
    ctx.register_tool(
        Arc::new(CreateFeatureSessionTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>
    );
    ctx.register_tool(
        Arc::new(ListFeatureSessionsTool::new(Arc::clone(&repo))) as Arc<dyn McpTool>,
    );

    tracing::info!(
        target: "nexum::tracker::main",
        tool_count = ctx.tool_count(),
        "MCP tools registered"
    );

    // 6. Serve on stdio.
    serve(std::io::stdin().lock(), std::io::stdout().lock(), &ctx)
        .map_err(|e| format!("serve failed: {e}"))?;

    Ok(())
}
