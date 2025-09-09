use miette::Result;
use rainy_cli::{config::Config, error::CliError, executor, tools, ui, utils::{context, history, rainy_md}};
use std::path::PathBuf;

pub async fn handle_chat_command(
    message: Option<String>,
    context_files: Option<Vec<PathBuf>>,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("CHAT", &format!("{} Agentic Chat Mode", ui::CHAT));
    ui::print_chat_header();

    let api_key = config
        .get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;

    let agent = executor::AgenticExecutor::new(
        api_key.to_string(),
        Some(config.get_model().to_string()),
    )
    .await
    .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

    let mut messages = Vec::new();

    // Load hierarchical rainy.md content and add it as a system message
    match rainy_md::load_hierarchical_rainy_md() {
        Ok(rainy_md_content) => {
            if !rainy_md_content.is_empty() {
                messages.push(executor::ChatMessage {
                    role: "system".to_string(),
                    content: format!("Here is the content from the rainy.md files:\n\n{}", rainy_md_content),
                });
            }
        }
        Err(e) => {
            ui::print_warning(&format!("Failed to load rainy.md content: {}", e));
        }
    }

    if let Ok(history) = history::load_conversation_history() {
        messages.extend(history);
        if !messages.is_empty() {
            ui::print_info("Previous conversation history loaded");
        }
    }

    // If context files are provided, load their content and add it to the conversation
    if let Some(files) = context_files {
        if !files.is_empty() {
            let context_str = context::collect_context_from_paths(&files)
                .map_err(|e| CliError::context_error("Failed to collect context from paths", e))?;
            let paths_str: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
            messages.push(executor::ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "The user has provided the following files as context: {}.\n\n{}",
                    paths_str.join(", "),
                    context_str
                ),
            });
            messages.push(executor::ChatMessage {
                role: "assistant".to_string(),
                content: "Thank you for providing the file context. I will use this information in our conversation. How can I help you?".to_string(),
            });
        }
    }

    if let Some(initial_msg) = message {
        messages.push(executor::ChatMessage {
            role: "user".to_string(),
            content: initial_msg,
        });
    } else {
        // Start with an empty user message to kick off the loop
        let input =
            ui::prompt_input().map_err(|e| CliError::file_error("Failed to read input", e))?;
        messages.push(executor::ChatMessage {
            role: "user".to_string(),
            content: input,
        });
    }

    run_agentic_loop(&mut messages, &agent).await?;

    Ok(())
}

use walkdir::WalkDir;

async fn run_agentic_loop(
    messages: &mut Vec<executor::ChatMessage>,
    agent: &executor::AgenticExecutor,
) -> Result<()> {
    loop {
        // If the last message was from the assistant, get user input
        if messages.last().map_or(true, |m| m.role == "assistant") {
            let mut input = ui::prompt_input()
                .map_err(|e| CliError::file_error("Failed to read user input", e))?;

            if input.starts_with('@') {
                input = handle_at_command(&input).await?;
            }

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                ui::print_success("Goodbye! 👋");
                break;
            }
            messages.push(executor::ChatMessage {
                role: "user".to_string(),
                content: input,
            });
        }

        let pb = ui::print_progress("AI is thinking...");
        let (response, duration) = agent.chat(messages.clone()).await.map_err(|e| {
            CliError::api_error(&format!("Failed to get AI response: {}", e))
        })?;
        pb.finish_with_message("Response received");

        let response_content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // The agent's response should be a JSON plan.
        // Attempt to parse it.
        match serde_json::from_str::<Vec<tools::ToolCall>>(&response_content) {
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
                    messages.push(executor::ChatMessage {
                        role: "assistant".to_string(),
                        content: response_content,
                    });
                    messages.push(executor::ChatMessage {
                        role: "user".to_string(),
                        content: format!("Here are the results of the execution:\n{}", results_str),
                    });
                } else {
                    ui::print_warning("Plan rejected by user.");
                    messages.push(executor::ChatMessage {
                        role: "assistant".to_string(),
                        content: response_content,
                    });
                    messages.push(executor::ChatMessage {
                        role: "user".to_string(),
                        content: "I have rejected this plan. Please propose a new one.".to_string(),
                    });
                }
            }
            Err(_) => {
                // Failed to parse a plan, treat as a regular chat message
                ui::print_ai_message(&response_content);
                ui::print_response_metrics(&response, duration);
                messages.push(executor::ChatMessage {
                    role: "assistant".to_string(),
                    content: response_content,
                });
            }
        }

        let _ = history::save_conversation_history(messages);
    }
    Ok(())
}

async fn handle_at_command(input: &str) -> Result<String> {
    let search_term = &input[1..];
    ui::print_info(&format!("Searching for files matching '{}'...", search_term));

    let mut found_files = Vec::new();
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if file_name.contains(search_term) {
                    found_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    if found_files.is_empty() {
        ui::print_warning("No matching files found.");
        return Ok(input.to_string());
    }

    ui::print_info("Matching files found:");
    for (i, path) in found_files.iter().enumerate() {
        println!("{}: {}", i + 1, path.display());
    }

    let selection = ui::prompt_input_with_prompt("Select a file to add to the context (or 0 to cancel):")
        .map_err(|e| CliError::file_error("Failed to read input", e))?;

    let selection: usize = match selection.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            ui::print_warning("Invalid selection. Sending original message.");
            return Ok(input.to_string());
        }
    };

    if selection > 0 && selection <= found_files.len() {
        let selected_path = &found_files[selection - 1];
        let content = std::fs::read_to_string(selected_path)
            .map_err(|e| CliError::file_error("Failed to read file", e))?;
        let new_input = format!(
            "Using file `{}` as context.\n\n---\n\n{}\n\n---\n\n{}",
            selected_path.display(),
            content,
            input
        );
        Ok(new_input)
    } else {
        ui::print_warning("Invalid selection or cancelled. Sending original message.");
        Ok(input.to_string())
    }
}
