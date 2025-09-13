use crate::commands::chat::handle_chat_command;
use miette::Result;
use crate::{config::Config, ui};
use std::path::PathBuf;

use crate::utils::context;

use crate::executor;

pub async fn handle_analyze_command(
    paths: Vec<PathBuf>,
    analysis_type: String,
    config: &Config,
    json_output: bool,
) -> Result<()> {
    ui::print_command_start(
        "ANALYZE",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let full_context = context::collect_context_from_paths(&paths)
        .map_err(|e| crate::error::CliError::context_error("Failed to collect context from paths", e))?;
    let paths_str: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();

    let initial_message = format!(
        "Please perform a {} analysis on the files: {}.\n\nHere is the combined content of the files:\n\n{}",
        analysis_type,
        paths_str.join(", "),
        full_context,
    );

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
        ui::print_info("The `analyze` command is now handled by the agentic chat.");
        ui::print_info("You will be dropped into a chat session with your request pre-filled.");

        // Call the chat handler with a pre-filled message and the file paths for context
        handle_chat_command(Some(initial_message), Some(paths), false, config).await
    }
}
