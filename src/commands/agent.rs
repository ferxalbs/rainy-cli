use crate::{config::Config, ui, utils};
use miette::Result;
use clap::{Args, Subcommand};
use std::fs;
use std::path::Path;

const AGENTS_MD_FILENAME: &str = "AGENTS.md";

#[derive(Args)]
pub struct AgentArgs {
    #[clap(subcommand)]
    command: AgentCommand,
}

#[derive(Subcommand)]
enum AgentCommand {
    /// Initializes the AGENTS.md file in the current project.
    Init,
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
    }
    Ok(())
}
