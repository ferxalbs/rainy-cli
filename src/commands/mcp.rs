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
        McpCommand::CallTool { server_name, tool_name, args } => {
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

            println!(
                "Calling tool '{}' on MCP server '{}' with args: {:?}",
                tool_name, server_name, args
            );
        }
    }
    Ok(())
}
