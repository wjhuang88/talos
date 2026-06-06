//! MCP server handler exposing Talos tools.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorCode, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool, ToolAnnotations,
};
use talos_core::tool::ToolRegistry;
use talos_plugin::{HookContext, TurnId};

use super::permission::McpPermissionGate;

/// MCP handler that republishes Talos tools to MCP clients.
pub struct TalosMcpHandler {
    tool_registry: Arc<ToolRegistry>,
    permission_gate: Arc<McpPermissionGate>,
}

impl TalosMcpHandler {
    /// Creates a new Talos MCP handler.
    #[must_use]
    pub fn new(tool_registry: Arc<ToolRegistry>, permission_gate: Arc<McpPermissionGate>) -> Self {
        Self {
            tool_registry,
            permission_gate,
        }
    }

    fn all_tools(&self) -> Vec<Tool> {
        // TODO: I009-future OAuth/auth for MCP server requests.
        // TODO: I009-future rate limiting for multi-client/high-throughput MCP usage.
        self.tool_registry
            .list()
            .into_iter()
            .map(to_mcp_tool)
            .collect()
    }
}

impl ServerHandler for TalosMcpHandler {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some("Talos MCP server exposing local tools".to_string());
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.all_tools()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_registry.get(name).map(to_mcp_tool)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let hook_context = HookContext::new(TurnId::new(), std::path::PathBuf::from("."));
        self.permission_gate
            .evaluate_call(&hook_context, &request)
            .await?;

        let Some(tool) = self.tool_registry.get(request.name.as_ref()) else {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("tool not found: {}", request.name),
                None,
            ));
        };

        let input = request
            .arguments
            .clone()
            .map(serde_json::Value::Object)
            .unwrap_or_else(|| serde_json::json!({}));
        let result = tool.execute(input).await;

        let content = vec![Content::text(result.content)];
        if result.is_error {
            Ok(CallToolResult::error(content))
        } else {
            Ok(CallToolResult::success(content))
        }
    }
}

fn to_mcp_tool(tool: &dyn talos_core::tool::AgentTool) -> Tool {
    let input_schema = tool.parameters().as_object().cloned().unwrap_or_default();

    Tool::new(
        Cow::Owned(tool.name().to_string()),
        Cow::Owned(tool.description().to_string()),
        Arc::new(input_schema),
    )
    .with_annotations(ToolAnnotations::new().read_only(tool.is_read_only()))
}
