use crate::commands::chat::handle_chat_command;
use miette::Result;
use crate::{config::Config, ui};
use std::path::PathBuf;

pub async fn handle_generate_command(
    description: String,
    output: Option<PathBuf>,
    config: &Config,
) -> Result<()> {
    ui::print_command_start(
        "GENERATE",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let mut initial_message = format!("Please generate code for the following description: '{}'", description);

    if let Some(o) = output {
        initial_message = format!("{} The output should be placed in `{}`.", initial_message, o.display());
    }

    ui::print_info("The `generate` command is now handled by the agentic chat.");
    ui::print_info("You will be dropped into a chat session with your request pre-filled.");

    handle_chat_command(Some(initial_message), None, false, config).await
}
