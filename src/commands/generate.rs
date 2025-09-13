use crate::commands::chat::handle_chat_command;
use miette::Result;
use crate::{config::Config, ui};
use std::path::PathBuf;

use crate::executor;

pub async fn handle_generate_command(
    description: String,
    output: Option<PathBuf>,
    config: &Config,
    json_output: bool,
) -> Result<()> {
    ui::print_command_start(
        "GENERATE",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let mut initial_message = format!("Please generate code for the following description: '{}'", description);

    if let Some(o) = output {
        initial_message = format!("{} The output should be placed in `{}`.", initial_message, o.display());
    }

    if json_output {
        let api_key = config.get_api_key()
            .map_err(|e| crate::error::CliError::config_error(format!("Failed to get API key: {}", e)))?
            .to_string();

        let agent = executor::AgenticExecutor::new(
            api_key,
            Some(config.get_model().to_string()),
        )
        .await
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

        let messages = vec![executor::ChatMessage {
            role: "user".to_string(),
            content: initial_message,
        }];

        let (response, _duration) = agent.chat(messages).await.map_err(|e| {
            crate::error::CliError::api_error(&format!("Failed to get AI response: {}", e))
        })?;

        let response_content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        println!("{}", response_content);
        Ok(())
    } else {
        ui::print_info("The `generate` command is now handled by the agentic chat.");
        ui::print_info("You will be dropped into a chat session with your request pre-filled.");

        handle_chat_command(Some(initial_message), None, false, config).await
    }
}
