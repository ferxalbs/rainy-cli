use miette::Result;
use rainy_cli::{config::Config, error::CliError, executor, tools, ui, utils::history};

pub async fn handle_chat_command(message: Option<String>, config: &Config) -> Result<()> {
    ui::print_command_start("CHAT", &format!("{} Agentic Chat Mode", ui::CHAT));
    ui::print_chat_header();

    let api_key = config
        .get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;

    let agent = executor::AgenticExecutor::new(api_key.to_string(), Some(config.get_model().to_string()))
        .await
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

    let mut messages = Vec::new();

    if let Ok(history) = history::load_conversation_history() {
        messages.extend(history);
        if !messages.is_empty() {
            ui::print_info("Previous conversation history loaded");
        }
    }

    if let Some(initial_msg) = message {
        messages.push(executor::ChatMessage {
            role: "user".to_string(),
            content: initial_msg,
        });
    } else {
        // Start with an empty user message to kick off the loop
        let input = ui::prompt_input().map_err(|e| CliError::file_error("Failed to read input", e))?;
        messages.push(executor::ChatMessage {
            role: "user".to_string(),
            content: input,
        });
    }

    run_agentic_loop(&mut messages, &agent).await?;

    Ok(())
}

async fn run_agentic_loop(
    messages: &mut Vec<executor::ChatMessage>,
    agent: &executor::AgenticExecutor,
) -> Result<()> {
    loop {
        let pb = ui::print_progress("AI is thinking...");
        let response = agent.chat(messages.clone()).await.map_err(|e| {
            CliError::api_error(&format!("Failed to get AI response: {}", e))
        })?;
        pb.finish_with_message("Response received");

        // The agent's response should be a JSON plan.
        // Attempt to parse it.
        match serde_json::from_str::<Vec<tools::ToolCall>>(&response) {
            Ok(plan) => {
                // Successfully parsed a plan
                ui::print_agent_plan(&serde_json::to_string_pretty(&plan).unwrap());

                if ui::prompt_for_confirmation()
                    .map_err(|e| CliError::file_error("Failed to read confirmation", e))?
                {
                    ui::print_info("Executing plan...");
                    let mut results = Vec::new();
                    for tool_call in plan {
                        let result = tools::execute_tool(tool_call)
                            .await
                            .map_err(|e| CliError::api_error(&e.to_string()))?;
                        results.push(result);
                    }

                    let results_str = serde_json::to_string_pretty(&results).unwrap();
                    ui::print_code_block("Execution Results", &results_str);

                    // Add both the agent's plan and the execution results to the conversation
                    messages.push(executor::ChatMessage { role: "assistant".to_string(), content: response });
                    messages.push(executor::ChatMessage { role: "user".to_string(), content: format!("Here are the results of the execution:\n{}", results_str) });

                } else {
                    ui::print_warning("Plan rejected by user.");
                    messages.push(executor::ChatMessage { role: "assistant".to_string(), content: response });
                    messages.push(executor::ChatMessage { role: "user".to_string(), content: "I have rejected this plan. Please propose a new one.".to_string() });
                }
            }
            Err(_) => {
                // Failed to parse a plan, treat as a regular chat message
                ui::print_ai_message(&response);
                messages.push(executor::ChatMessage { role: "assistant".to_string(), content: response });

                let input = ui::prompt_input()
                    .map_err(|e| CliError::file_error("Failed to read user input", e))?;
                 if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    ui::print_success("Goodbye! 👋");
                    break;
                }
                messages.push(executor::ChatMessage { role: "user".to_string(), content: input });
            }
        }

        let _ = history::save_conversation_history(messages);
    }
    Ok(())
}
