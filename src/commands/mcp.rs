use crate::config::Config;
use miette::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct McpArgs {
    #[clap(subcommand)]
    command: McpCommand,
}

#[derive(Subcommand)]
enum McpCommand {
    /// Add a new MCP server to the configuration
    Add {
        /// The name of the server
        server_name: String,
        /// The command to execute the server
        command: String,
        /// The arguments for the command
        args: Vec<String>,
    },
    /// Remove an MCP server from the configuration
    Remove {
        /// The name of the server
        server_name: String,
    },
    /// List all available MCP servers
    List,
    /// List tools available from MCP servers
    ListTools {
        /// The name of the server (optional, lists all servers if not specified)
        server_name: Option<String>,
    },
    /// Call a tool from a connected MCP server
    CallTool {
        /// The name of the server
        server_name: String,
        /// The name of the tool to call
        tool_name: String,
        /// The arguments for the tool in JSON format
        #[arg(short, long)]
        args: Option<String>,
    },
}

use crate::ui;
use crate::utils::agents_md;
use std::fs;
use crate::utils;

use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::{ServiceExt, Service, RequestContext, NotificationContext, RoleClient},
    transport::{TokioChildProcess, ConfigureCommandExt},
};
use tokio::process::Command;
use serde_json::Value;

pub async fn handle_mcp_command(args: McpArgs, _config: &Config) -> Result<()> {
    match args.command {
        McpCommand::Add { server_name, command, args } => {
            utils::mcp::add_mcp_server(&server_name, &command, &args)
                .map_err(|e| crate::error::CliError::command_error(format!("Failed to add MCP server: {}", e)))?;
            ui::print_success(&format!("MCP server '{}' added.", server_name));
        }
        McpCommand::Remove { server_name } => {
            utils::mcp::remove_mcp_server(&server_name)
                .map_err(|e| crate::error::CliError::command_error(format!("Failed to remove MCP server: {}", e)))?;
            ui::print_success(&format!("MCP server '{}' removed.", server_name));
        }
        McpCommand::List => {
            let config = utils::mcp::load_mcp_config()
                .map_err(|e| crate::error::CliError::command_error(format!("Failed to load MCP config: {}", e)))?;
            if config.mcp_servers.is_empty() {
                ui::print_info("No MCP servers configured.");
            } else {
                ui::print_info("Available MCP servers:");
                for server_name in config.mcp_servers.keys() {
                    println!("- {}", server_name);
                }
            }
        }
        McpCommand::ListTools { server_name } => {
            let config = utils::mcp::load_mcp_config()
                .map_err(|e| crate::error::CliError::command_error(format!("Failed to load MCP config: {}", e)))?;

            if config.mcp_servers.is_empty() {
                ui::print_info("No MCP servers configured.");
                return Ok(());
            }

            if let Some(server_name) = server_name {
                // List tools for specific server
                let server_config = config.mcp_servers.get(&server_name)
                    .ok_or_else(|| crate::error::CliError::command_error(format!("MCP server '{}' not found", server_name)))?;

                match list_mcp_tools(server_config).await {
                    Ok(tools) => {
                        ui::print_info(&format!("Tools available from server '{}':", server_name));
                        if tools.is_empty() {
                            println!("  No tools available");
                        } else {
                            for tool in tools {
                                println!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
                            }
                        }
                    }
                    Err(e) => {
                        ui::print_error(&format!("Failed to list tools from server '{}': {}", server_name, e));
                        return Err(crate::error::CliError::command_error(format!("Failed to list tools: {}", e)).into());
                    }
                }
            } else {
                // List tools for all servers
                ui::print_info("Tools available from all MCP servers:");
                for (server_name, server_config) in &config.mcp_servers {
                    match list_mcp_tools(server_config).await {
                        Ok(tools) => {
                            println!("Server '{}':", server_name);
                            if tools.is_empty() {
                                println!("  No tools available");
                            } else {
                                for tool in tools {
                                    println!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
                                }
                            }
                        }
                        Err(e) => {
                            println!("Server '{}' (error: {}):", server_name, e);
                            println!("  Failed to list tools");
                        }
                    }
                }
            }
        }
        McpCommand::CallTool { server_name, tool_name, args } => {
            // Check permissions
            let agents_md_path = std::path::Path::new("AGENTS.md");
            let agents_md_content = fs::read_to_string(agents_md_path).unwrap_or_default();
            let agents_md = agents_md::parse_agents_md(&agents_md_content);

            if !agents_md.mcp_permissions.contains(&server_name) {
                ui::print_info(&format!("First time using MCP server '{}'. Please grant permission.", server_name));
                let confirmation = dialoguer::Confirm::new()
                    .with_prompt(&format!("Do you want to allow Rainy CLI to use tools from the MCP server '{}'?", server_name))
                    .interact()
                    .map_err(|e| crate::error::CliError::command_error(&format!("Failed to read confirmation: {}", e)))?;

                if confirmation {
                    let mut new_content = agents_md_content;
                    new_content.push_str(&format!("\nMCP-Permission: {}\n", server_name));
                    fs::write(agents_md_path, new_content)
                        .map_err(|e| crate::error::CliError::file_error("Failed to write to AGENTS.md. Please check file permissions.", e))?;
                    ui::print_success(&format!("Permission granted for MCP server '{}'.", server_name));
                } else {
                    ui::print_warning("Permission denied.");
                    return Ok(());
                }
            }

            // Load MCP config and get server details
            let config = utils::mcp::load_mcp_config()
                .map_err(|e| crate::error::CliError::command_error(format!("Failed to load MCP config: {}", e)))?;
            
            let server_config = config.mcp_servers.get(&server_name)
                .ok_or_else(|| crate::error::CliError::command_error(format!("MCP server '{}' not found", server_name)))?;

            // Execute the MCP tool call
            match execute_mcp_tool_call(server_config, &tool_name, args.as_deref()).await {
                Ok(result) => {
                    ui::print_success("Tool executed successfully:");
                    println!("{}", result);
                }
                Err(e) => {
                     ui::print_error(&format!("Failed to execute tool: {}", e));
                     return Err(crate::error::CliError::command_error(format!("Tool execution failed: {}", e)).into());
                 }
            }
        }
    }
    Ok(())
}

async fn execute_mcp_tool_call(
    server_config: &utils::mcp::McpServerConfig,
    tool_name: &str,
    args: Option<&str>,
) -> anyhow::Result<String> {
    // Parse arguments
    let arguments = if let Some(args_str) = args {
        Some(serde_json::from_str::<serde_json::Map<String, Value>>(args_str)?)
    } else {
        None
    };

    // Create command with environment variables
    let mut cmd = Command::new(&server_config.command);
    cmd.args(&server_config.args);
    
    // Set environment variables
    for (key, value) in &server_config.env {
        cmd.env(key, value);
    }

    // Configure the command for MCP transport
    let transport = TokioChildProcess::new(cmd.configure(|_| {}))?;

    // Create a simple client service that implements the required traits
    struct SimpleClientService;
    
    impl Service<RoleClient> for SimpleClientService {
        fn get_info(&self) -> rmcp::model::ClientInfo {
            rmcp::model::ClientInfo {
                protocol_version: rmcp::model::ProtocolVersion::V_2025_06_18,
                capabilities: rmcp::model::ClientCapabilities::default(),
                client_info: rmcp::model::Implementation {
                    name: "rainy-cli".to_string(),
                    version: "0.2.0".to_string(),
                    title: Some("Rainy CLI".to_string()),
                    icons: None,
                    website_url: None,
                },
            }
        }

        async fn handle_request(
            &self,
            _request: rmcp::model::ServerRequest,
            _context: RequestContext<RoleClient>,
        ) -> Result<rmcp::model::ClientResult, rmcp::ErrorData> {
            // For a simple client, we don't expect to handle requests from the server
            Err(rmcp::ErrorData::method_not_found::<rmcp::model::PingRequestMethod>())
        }

        async fn handle_notification(
            &self,
            _notification: rmcp::model::ServerNotification,
            _context: NotificationContext<RoleClient>,
        ) -> Result<(), rmcp::ErrorData> {
            // For a simple client, we don't need to handle notifications
            Ok(())
        }
    }

    let service = SimpleClientService;

    // Connect to the MCP server
    let running_service = service.serve(transport).await?;

    // Get the peer for making requests
    let peer = running_service.peer();



    // Call the tool
    let tool_result = peer.call_tool(CallToolRequestParam {
        name: tool_name.to_string().into(),
        arguments,
    }).await?;

    // Format the result
    let mut result_text = String::new();
    for content in tool_result.content {
        match &content.raw {
            rmcp::model::RawContent::Text(text_content) => {
                result_text.push_str(&text_content.text);
                result_text.push('\n');
            }
            rmcp::model::RawContent::Image(_) => {
                result_text.push_str("[Image content]\n");
            }
            rmcp::model::RawContent::Resource(_) => {
                result_text.push_str("[Resource content]\n");
            }
            rmcp::model::RawContent::Audio(_) => {
                result_text.push_str("[Audio content]\n");
            }
            rmcp::model::RawContent::ResourceLink(_) => {
                result_text.push_str("[Resource link]\n");
            }
        }
    }

    // Cancel the service
    running_service.cancel().await?;
    
    Ok(result_text.trim().to_string())
}

async fn list_mcp_tools(server_config: &utils::mcp::McpServerConfig) -> anyhow::Result<Vec<Tool>> {
    // Create command with environment variables
    let mut cmd = Command::new(&server_config.command);
    cmd.args(&server_config.args);

    // Set environment variables
    for (key, value) in &server_config.env {
        cmd.env(key, value);
    }

    // Configure the command for MCP transport
    let transport = TokioChildProcess::new(cmd.configure(|_| {}))?;

    // Create a simple client service that implements the required traits
    struct SimpleClientService;

    impl Service<RoleClient> for SimpleClientService {
        fn get_info(&self) -> rmcp::model::ClientInfo {
            rmcp::model::ClientInfo {
                protocol_version: rmcp::model::ProtocolVersion::V_2025_06_18,
                capabilities: rmcp::model::ClientCapabilities::default(),
                client_info: rmcp::model::Implementation {
                    name: "rainy-cli".to_string(),
                    version: "0.2.0".to_string(),
                    title: Some("Rainy CLI".to_string()),
                    icons: None,
                    website_url: None,
                },
            }
        }

        async fn handle_request(
            &self,
            _request: rmcp::model::ServerRequest,
            _context: RequestContext<RoleClient>,
        ) -> Result<rmcp::model::ClientResult, rmcp::ErrorData> {
            // For a simple client, we don't expect to handle requests from the server
            Err(rmcp::ErrorData::method_not_found::<rmcp::model::PingRequestMethod>())
        }

        async fn handle_notification(
            &self,
            _notification: rmcp::model::ServerNotification,
            _context: NotificationContext<RoleClient>,
        ) -> Result<(), rmcp::ErrorData> {
            // For a simple client, we don't need to handle notifications
            Ok(())
        }
    }

    let service = SimpleClientService;

    // Connect to the MCP server
    let running_service = service.serve(transport).await?;

    // Get the peer for making requests
    let peer = running_service.peer();

    // List tools
    let tools_result = peer.list_tools(None).await?;

    // Cancel the service
    running_service.cancel().await?;

    Ok(tools_result.tools)
}
