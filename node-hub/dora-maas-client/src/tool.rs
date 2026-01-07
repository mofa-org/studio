//! MCP tool management for function calling support
//!
//! This module provides the infrastructure for integrating MCP (Model Context Protocol)
//! tools with the LLM streaming client, enabling function calling capabilities.

use std::{collections::HashMap, sync::Arc};

use eyre::Result;
use rmcp::{
    RoleClient,
    model::{CallToolRequestParam, CallToolResult, Tool as McpTool},
    service::{RunningService, ServerSink},
};
use serde_json::Value;

/// Trait for tool implementations that can be called by the LLM
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Get the name of the tool
    fn name(&self) -> String;

    /// Get the description of the tool
    fn description(&self) -> String;

    /// Get the JSON schema for the tool's parameters
    fn parameters(&self) -> Value;

    /// Execute the tool with the given arguments
    async fn call(&self, args: Value) -> Result<CallToolResult>;
}

/// Adapter that wraps an MCP tool to implement our Tool trait
pub struct McpToolAdapter {
    tool: McpTool,
    server: ServerSink,
}

impl McpToolAdapter {
    /// Create a new MCP tool adapter
    pub fn new(tool: McpTool, server: ServerSink) -> Self {
        Self { tool, server }
    }
}

#[async_trait::async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> String {
        self.tool.name.clone().to_string()
    }

    fn description(&self) -> String {
        self.tool
            .description
            .clone()
            .unwrap_or_default()
            .to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(&self.tool.input_schema).unwrap_or(serde_json::json!({}))
    }

    async fn call(&self, args: Value) -> Result<CallToolResult> {
        let arguments = match args {
            Value::Object(map) => Some(map),
            _ => None,
        };

        let call_result = self
            .server
            .call_tool(CallToolRequestParam {
                name: self.tool.name.clone(),
                arguments,
            })
            .await?;

        Ok(call_result)
    }
}

/// Container for managing multiple tools
#[derive(Default)]
pub struct ToolSet {
    tools: HashMap<String, Arc<dyn Tool>>,
    clients: HashMap<String, RunningService<RoleClient, ()>>,
}

impl ToolSet {
    /// Set the MCP clients for this tool set
    pub fn set_clients(&mut self, clients: HashMap<String, RunningService<RoleClient, ()>>) {
        self.clients = clients;
    }

    /// Add a tool to the set
    pub fn add_tool<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name(), Arc::new(tool));
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Get all tools
    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }

    /// Check if any tools are available
    pub fn has_tools(&self) -> bool {
        !self.tools.is_empty()
    }
}

/// Discover and load all tools from an MCP server
pub async fn get_mcp_tools(server: ServerSink) -> Result<Vec<McpToolAdapter>> {
    let tools = server.list_all_tools().await?;
    Ok(tools
        .into_iter()
        .map(|tool| McpToolAdapter::new(tool, server.clone()))
        .collect())
}
