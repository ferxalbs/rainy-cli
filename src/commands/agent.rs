use crate::{config::Config, ui, utils};
use miette::Result;
use clap::{Args, Subcommand};
use std::fs;
use std::path::Path;

const AGENTS_MD_FILENAME: &str = "AGENTS.md";

use crate::tools;

#[derive(Args)]
pub struct AgentArgs {
    #[clap(subcommand)]
    command: AgentCommand,
}

#[derive(Subcommand)]
enum AgentCommand {
    /// Initializes the AGENTS.md file in the current project.
    Init,
    /// Reads the content of a file.
    ReadFile { path: String },
    /// Writes content to a file.
    WriteFile { path: String, content: String },
    /// Patches a file with instructions.
    PatchFile { path: String, instructions: String },
    /// Deletes a file.
    DeleteFile { path: String },
    /// Lists files in a directory.
    ListFiles { path: String },
    /// Greps for a pattern in files.
    Grep { pattern: String, path: Option<String> },
}

pub async fn handle_agent_command(args: AgentArgs, config: &Config) -> Result<()> {
    match args.command {
        AgentCommand::Init => {
            let path = Path::new(AGENTS_MD_FILENAME);
            if path.exists() {
                ui::print_info(&format!("`{}` already exists.", AGENTS_MD_FILENAME));
                return Ok(());
            }

            ui::print_info(&format!("{} Analyzing project to generate `{}`...", ui::INFO, AGENTS_MD_FILENAME));

            let context = match utils::context::analyze_project(config).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    ui::print_warning(&format!("Could not fully analyze project for {}: {}. A default file will be created.", AGENTS_MD_FILENAME, e));
                    // Fallback to a default context if analysis fails
                    utils::context::ProjectContext::default()
                }
            };

            let content = utils::agents_md::generate_agents_md_content(&context);

            fs::write(path, content)
                .map_err(|e| crate::error::CliError::file_error(&format!("Failed to create {}", AGENTS_MD_FILENAME), e))?;

            ui::print_success(&format!(
                "{} Created `{}` with auto-detected project context. You can edit this file to provide project-specific instructions.",
                ui::CHECK,
                AGENTS_MD_FILENAME
            ));
        }
        AgentCommand::ReadFile { path } => {
            let result = tools::execute_tool(tools::ToolCall::ReadFile { path }).await
                .map_err(|e| crate::error::CliError::command_error(&format!("Failed to read file: {}", e)))?;
            println!("{}", result.output);
        }
        AgentCommand::WriteFile { path, content } => {
            let result = tools::execute_tool(tools::ToolCall::WriteFile { path, content }).await
                .map_err(|e| crate::error::CliError::command_error(&format!("Failed to write file: {}", e)))?;
            println!("{}", result.output);
        }
        AgentCommand::PatchFile { path, instructions } => {
            let result = tools::execute_tool(tools::ToolCall::PatchFile { path, instructions }).await
                .map_err(|e| crate::error::CliError::command_error(&format!("Failed to patch file: {}", e)))?;
            println!("{}", result.output);
        }
        AgentCommand::DeleteFile { path } => {
            let result = tools::execute_tool(tools::ToolCall::DeleteFile { path }).await
                .map_err(|e| crate::error::CliError::command_error(&format!("Failed to delete file: {}", e)))?;
            println!("{}", result.output);
        }
        AgentCommand::ListFiles { path } => {
            let result = tools::execute_tool(tools::ToolCall::ListFiles { path }).await
                .map_err(|e| crate::error::CliError::command_error(&format!("Failed to list files: {}", e)))?;
            println!("{}", result.output);
        }
        AgentCommand::Grep { pattern, path } => {
            let result = tools::execute_tool(tools::ToolCall::Grep { pattern, path }).await
                .map_err(|e| crate::error::CliError::command_error(&format!("Failed to grep files: {}", e)))?;
            println!("{}", result.output);
        }
    }
    Ok(())
}
