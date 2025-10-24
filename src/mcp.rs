use log::info;
use rmcp::{
    handler::server::router::tool::ToolRouter, model::*, service::RequestContext, tool,
    tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use tokio::io::{stdin, stdout};

use crate::wheel::Wheel;

/// Handle MCP (Model Context Protocol) related functionality
pub fn run() {
    info!("MCP mode activated");

    // Create a tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // Run the async MCP server
    if let Err(e) = runtime.block_on(run_mcp_server()) {
        eprintln!("MCP server error: {}", e);
    }
}

async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting MCP server...");
    let transport = (stdin(), stdout());

    let service = McpServer::new();
    let server = service.serve(transport).await?;
    let _quit_reason = server.waiting().await?;
    Ok(())
}

struct McpServer {
    tool_router: ToolRouter<McpServer>,
    wheel: Wheel,
}

#[tool_router]
impl McpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            wheel: Wheel::new(),
        }
    }

    #[tool(description = "Spin the PowerToys Roulette wheel and get a random executable to launch for the user")]
    async fn launch_roulette(&self, context: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        info!("MCP tool: launch_roulette called");

        // Simulate wheel spinning with progress updates
        let total_duration = std::time::Duration::from_secs(8);
        let steps = 16;
        let step_duration = total_duration / steps;

        for i in 0..=steps {
            let progress = (i as f64 / steps as f64) * 100.0;
            
            // Send progress notification if we have a progress token
            if let Some(progress_token) = context.meta.get_progress_token() {
                // Classic spinning wheel animation
                let spinner = match i % 4 {
                    0 => "|",
                    1 => "/",
                    2 => "─",
                    _ => "\\",
                };
                
                let progress_param = ProgressNotificationParam {
                    progress_token,
                    progress,
                    total: Some(100.0),
                    message: Some(format!("{} Spinning the PowerToys wheel...", spinner)),
                };
                
                if let Err(e) = context.peer.notify_progress(progress_param).await {
                    log::warn!("Failed to send progress notification: {}", e);
                }
            }
            
            // Wait before next update (but not after the last one)
            if i < steps {
                tokio::time::sleep(step_duration).await;
            }
        }

        // Use the shared wheel logic to randomly select an item
        let winner = self.wheel.spin();
        
        info!("Wheel selected: {} ({})", winner.name, winner.executable);

        // Return the executable name as the result
        Ok(CallToolResult::success(vec![Content::text(
            format!("Wheel has finished and has landed on: {}\nPlease run this executable for the user: {}", winner.name, winner.executable)
        )]))
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("A server to play PowerToys roulette".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
