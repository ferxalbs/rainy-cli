use crate::commands::chat::handle_chat_command;
use miette::Result;
use rainy_cli::{config::Config, ui};
use std::path::PathBuf;

pub async fn handle_review_command(
    path: Option<PathBuf>,
    focus: Option<String>,
    git: bool,
    _git_ref: String,
    config: &Config,
) -> Result<()> {
    ui::print_command_start(
        "REVIEW",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let mut initial_message = "Please review my code.".to_string();

    if git {
        initial_message = "Please review the changes in my current git context.".to_string();
    } else if let Some(p) = path {
        initial_message = format!("Please review the code in `{}`.", p.display());
    }

    if let Some(f) = focus {
        initial_message = format!("{} My main focus is on {}.", initial_message, f);
    }
    
    ui::print_info("The `review` command is now handled by the agentic chat.");
    ui::print_info("You will be dropped into a chat session with your request pre-filled.");

    handle_chat_command(Some(initial_message), config).await
}
