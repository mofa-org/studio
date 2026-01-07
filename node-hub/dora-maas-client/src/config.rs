use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use figment::{
    Figment,
    providers::{Env, Format, Json, Toml, Yaml},
};
use rmcp::{RoleClient, ServiceExt, service::RunningService, transport::ConfigureCommandExt};
use serde::Deserialize;

use crate::client::{ChatClient, GeminiClient, OpenaiClient};
use crate::tool::{Tool, ToolSet, get_mcp_tools};

/// Main configuration structure for the MaaS client.
///
/// Loaded from TOML/YAML/JSON files and environment variables.
/// Supports multiple providers and flexible model routing.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub default_model: String,
    pub system_prompt: String,
    pub max_history_exchanges: usize,
    pub enable_streaming: Option<bool>,
    pub providers: Vec<ProviderConfig>,
    pub models: Vec<ModelConfig>,
    #[serde(default = "default_log_level")]
    pub log_level: String, // Used for configuring log verbosity
    #[serde(default)]
    pub enable_tools: bool, // Enable MCP tool support
    #[serde(default)]
    pub enable_local_mcp: bool, // Enable local MCP host (false = pass through to client)
    pub mcp: Option<McpConfig>, // MCP server configurations
    // HTTP request cancellation settings
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_stream_timeout")]
    pub stream_timeout_secs: u64,
    #[serde(default = "default_enable_cancellation")]
    pub enable_cancellation: bool,
    // Anchor context settings
    pub anchor_context: Option<String>, // Path to anchor context markdown file
}

fn default_log_level() -> String {
    "INFO".to_string()
}

fn default_request_timeout() -> u64 { 30 }

fn default_stream_timeout() -> u64 { 120 }

fn default_enable_cancellation() -> bool { true }

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderConfig {
    Openai(OpenaiConfig),
    Gemini(GeminiConfig),
    Alicloud(AlicloudConfig),
    Deepseek(DeepseekConfig),
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpenaiConfig {
    pub id: String,
    pub api_key: String,
    pub api_url: String,
    #[serde(default)]
    pub proxy: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeminiConfig {
    pub id: String,
    pub api_key: String,
    pub api_url: String,
    #[serde(default)]
    pub proxy: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AlicloudConfig {
    pub id: String,
    pub api_key: String,
    pub api_url: String,
    #[serde(default)]
    pub proxy: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeepseekConfig {
    pub id: String,
    pub api_key: String,
    #[serde(default = "default_deepseek_url")]
    pub api_url: String,
    #[serde(default)]
    pub proxy: bool,
}

fn default_deepseek_url() -> String {
    "https://api.deepseek.com/v1".to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub route: ModelRoute,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelRoute {
    pub provider: String,
    pub model: Option<String>,
}

impl Config {
    /// Load configuration from file specified by MAAS_CONFIG_PATH environment variable.
    ///
    /// Supports TOML, YAML, and JSON formats based on file extension.
    /// Falls back to `maas_config.toml` if MAAS_CONFIG_PATH is not set.
    pub fn load() -> eyre::Result<Self> {
        let config_file =
            std::env::var("MAAS_CONFIG_PATH").unwrap_or_else(|_| "maas_config.toml".to_string());
        let config_path = PathBuf::from(config_file);

        if !config_path.exists() {
            eprintln!("Config file not found at: {}", config_path.display());
            std::process::exit(1);
        }

        let figment = match config_path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => Figment::new().merge(Yaml::file(config_path)),
            Some("json") => Figment::new().merge(Json::file(config_path)),
            _ => Figment::new().merge(Toml::file(config_path)),
        };

        // Allow MAAS-prefixed env vars (e.g., MAAS_LOG_LEVEL)
        // Also allow direct env vars for common settings (e.g., MAX_HISTORY_EXCHANGES)
        let config: Config = figment
            .merge(Env::prefixed("MAAS_"))
            .merge(Env::raw().only(&["MAX_HISTORY_EXCHANGES"]))
            .extract()?;
        Ok(config)
    }

    /// Create client instances for all configured providers.
    ///
    /// Returns a map from provider ID to client implementation.
    pub fn create_clients(&self) -> HashMap<String, Arc<dyn ChatClient>> {
        let mut clients = HashMap::new();

        for provider in &self.providers {
            let client: Arc<dyn ChatClient> = match provider {
                ProviderConfig::Openai(config) => Arc::new(OpenaiClient::new(config)),
                ProviderConfig::Gemini(config) => Arc::new(GeminiClient::new(config)),
                ProviderConfig::Alicloud(config) => {
                    // Alicloud uses OpenAI-compatible API, so we can reuse OpenaiClient
                    Arc::new(OpenaiClient::new(&OpenaiConfig {
                        id: config.id.clone(),
                        api_key: config.api_key.clone(),
                        api_url: config.api_url.clone(),
                        proxy: config.proxy,
                    }))
                }
                ProviderConfig::Deepseek(config) => {
                    // DeepSeek uses OpenAI-compatible API, so we can reuse OpenaiClient
                    Arc::new(OpenaiClient::new(&OpenaiConfig {
                        id: config.id.clone(),
                        api_key: config.api_key.clone(),
                        api_url: config.api_url.clone(),
                        proxy: config.proxy,
                    }))
                }
            };

            let provider_id = match provider {
                ProviderConfig::Openai(c) => &c.id,
                ProviderConfig::Gemini(c) => &c.id,
                ProviderConfig::Alicloud(c) => &c.id,
                ProviderConfig::Deepseek(c) => &c.id,
            };

            clients.insert(provider_id.clone(), client);
        }

        clients
    }

    /// Route a model ID to its provider and actual model name.
    ///
    /// # Arguments
    /// * `model_id` - The configured model ID (e.g., "gpt-4")
    ///
    /// # Returns
    /// * `Some((provider_id, model_name))` - Provider and actual model name
    /// * `None` - No routing found for the model ID
    pub fn route_model(&self, model_id: &str) -> Option<(String, String)> {
        self.models.iter().find(|m| m.id == model_id).map(|m| {
            let provider = m.route.provider.clone();
            let model = m.route.model.clone().unwrap_or_else(|| m.id.clone());
            (provider, model)
        })
    }
}

/// MCP (Model Context Protocol) configuration
#[derive(Clone, Debug, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

/// Configuration for an individual MCP server
#[derive(Clone, Debug, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(flatten)]
    pub transport: McpServerTransportConfig,
}

/// MCP server transport configuration
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "protocol", rename_all = "snake_case")]
pub enum McpServerTransportConfig {
    /// HTTP streaming transport
    Streamable { url: String },
    /// Server-Sent Events transport
    Sse { url: String },
    /// Standard I/O transport (child process)
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        envs: HashMap<String, String>,
    },
}

impl McpServerTransportConfig {
    /// Start the MCP server with the configured transport
    pub async fn start(&self) -> eyre::Result<RunningService<RoleClient, ()>> {
        let client = match self {
            McpServerTransportConfig::Streamable { url } => {
                // Retry connection a few times for streamable transport
                for i in 0..5 {
                    let transport =
                        rmcp::transport::StreamableHttpClientTransport::from_uri(url.to_string());
                    match ().serve(transport).await {
                        Ok(client) => return Ok(client),
                        Err(e) => {
                            eprintln!(
                                "Attempt {}/5 - Failed to start streamable transport: {}",
                                i + 1,
                                e
                            );
                            if i < 4 {
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            }
                        }
                    }
                }
                eyre::bail!("Failed to start streamable transport after 5 attempts");
            }
            McpServerTransportConfig::Sse { url } => {
                let transport =
                    rmcp::transport::sse_client::SseClientTransport::start(url.to_owned()).await?;
                ().serve(transport).await?
            }
            McpServerTransportConfig::Stdio {
                command,
                args,
                envs,
            } => {
                // FIX: Changed stdio from inherit() to piped() to enable proper MCP communication
                // MCP servers communicate via JSON-RPC over stdio, so pipes are required
                let transport = rmcp::transport::TokioChildProcess::new(
                    tokio::process::Command::new(command).configure(|cmd| {
                        cmd.args(args)
                            .envs(envs)
                            .stderr(Stdio::piped())
                            .stdout(Stdio::piped());
                    }),
                )?;
                ().serve(transport).await?
            }
        };
        Ok(client)
    }
}

impl Config {
    /// Initialize the tool set with MCP tools
    pub async fn init_tool_set(&self) -> eyre::Result<Option<ToolSet>> {
        // Only initialize local MCP if both enable_tools and enable_local_mcp are true
        if !self.enable_tools || !self.enable_local_mcp || self.mcp.is_none() {
            return Ok(None);
        }

        let mut tool_set = ToolSet::default();
        let mut mcp_clients = HashMap::new();

        // FIX: Made MCP server initialization graceful - if one server fails, others can still work
        // This prevents a single misconfigured server from blocking all MCP functionality
        if let Some(mcp_config) = &self.mcp {
            for server in &mcp_config.servers {
                eprintln!("Starting MCP server: {}", server.name);
                match server.transport.start().await {
                    Ok(client) => {
                        mcp_clients.insert(server.name.clone(), client);
                        eprintln!("Successfully started MCP server: {}", server.name);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to start MCP server '{}': {}",
                            server.name, e
                        );
                        // Continue with other servers instead of failing entirely
                    }
                }
            }
        }

        // If no clients were created successfully, return None
        if mcp_clients.is_empty() {
            eprintln!("Warning: No MCP servers could be started");
            return Ok(None);
        }

        // Load tools from successfully started servers
        for (name, client) in mcp_clients.iter() {
            eprintln!("Loading tools from MCP server: {}", name);
            match get_mcp_tools(client.peer().clone()).await {
                Ok(tools) => {
                    for tool in tools {
                        eprintln!("  - Registered tool: {}", tool.name());
                        tool_set.add_tool(tool);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load tools from '{}': {}", name, e);
                    // Continue with other servers
                }
            }
        }

        tool_set.set_clients(mcp_clients);

        let tool_count = tool_set.tools().len();
        if tool_count > 0 {
            eprintln!("Initialized {} MCP tools", tool_count);
            Ok(Some(tool_set))
        } else {
            eprintln!("Warning: No tools were loaded from any MCP server");
            Ok(None)
        }
    }
}

/// Helper to resolve environment variable references in configuration.
///
/// If value starts with "env:", looks up the environment variable.
/// Otherwise returns the value as-is.
///
/// # Example
/// * `"env:OPENAI_API_KEY"` -> Looks up OPENAI_API_KEY env var
/// * `"sk-..."` -> Returns the literal string
pub fn get_env_or_value(value: &str) -> String {
    if value.starts_with("env:") {
        let env_var = &value[4..];
        let result = std::env::var(env_var)
            .unwrap_or_else(|_| {
                eprintln!("❌ Environment variable {} not found", env_var);
                String::new()
            })
            .trim()
            .to_string();

        // Log masked API key for debugging
        if result.is_empty() {
            eprintln!("⚠️  {} resolved to EMPTY string", env_var);
        } else if result.len() > 8 {
            eprintln!("✓ {} = {}...{} (len={})", env_var, &result[..4], &result[result.len()-4..], result.len());
        } else {
            eprintln!("⚠️  {} = {} (too short, len={})", env_var, result, result.len());
        }
        result
    } else {
        // Literal value - also log masked
        let result = value.trim().to_string();
        if result.len() > 8 {
            eprintln!("✓ Literal API key: {}...{} (len={})", &result[..4], &result[result.len()-4..], result.len());
        }
        result
    }
}

/// Load anchor context from a markdown file.
///
/// Reads the file content and validates it contains anchor points [A0]-[A11].
/// Returns the formatted context string for inclusion in developer role.
///
/// # Arguments
/// * `file_path` - Path to the anchor context markdown file
///
/// # Returns
/// * `Ok(String)` - Formatted context content
/// * `Err(eyre::Error)` - If file cannot be read or is invalid
pub fn load_anchor_context(file_path: &str) -> eyre::Result<String> {
    use std::fs;
    use std::path::Path;

    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(eyre::eyre!("Anchor context file not found: {}", file_path));
    }

    // Read file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| eyre::eyre!("Failed to read anchor context file {}: {}", file_path, e))?;

    // Validate that content contains anchor points
    if !content.contains("[A0") {
        return Err(eyre::eyre!("Invalid anchor context file: missing anchor points [A0]-[A11] in {}", file_path));
    }

    // Trim whitespace and return
    Ok(content.trim().to_string())
}

/// Format anchor context for inclusion in developer role message.
///
/// Wraps the context content with the required format:
/// CONTEXT:\n<<<BEGIN ANCHOR CONTEXT\n[content]\nEND ANCHOR CONTEXT>>>
///
/// # Arguments
/// * `context_content` - Raw context content from file
///
/// # Returns
/// Formatted string ready for developer role
pub fn format_anchor_context(context_content: &str) -> String {
    format!(
        "CONTEXT:\n<<<BEGIN ANCHOR CONTEXT\n{}\nEND ANCHOR CONTEXT>>>",
        context_content
    )
}
