use log::info;
use rmcp::{
    handler::server::router::tool::ToolRouter, model::*, service::RequestContext, tool,
    tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use tokio::io::{stdin, stdout};

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
}

#[tool_router]
impl McpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Say hello to the client")]
    fn say_hello(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("hello")]))
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
