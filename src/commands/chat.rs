use miette::Result;
use crate::{config::Config, executor, tools, ui, utils::{self, context, history, agents_md, sessions::{SessionManager, ChatMessage}}};
use std::path::PathBuf;
use regex::Regex;

fn tool_call_to_running_string(tool_call: &tools::ToolCall) -> String {
    match tool_call {
        tools::ToolCall::ReadFile { path } => format!("Reading file '{}'...", path),
        tools::ToolCall::WriteFile { path, .. } => format!("Writing to file '{}'...", path),
        tools::ToolCall::PatchFile { path, .. } => format!("Patching file '{}'...", path),
        tools::ToolCall::DeleteFile { path } => format!("Deleting file '{}'...", path),
        tools::ToolCall::ListFiles { path } => format!("Listing files in '{}'...", path),
        tools::ToolCall::Grep { pattern, path } => format!("Searching for '{}' in '{}'...", pattern, path.as_deref().unwrap_or(".")),
    }
}

fn parse_agent_response(response: &str) -> (String, String) {
    // Try to parse Qwen-style ReAct format first
    let qwen_react_regex = Regex::new(r"Thought:([\s\S]*?)Action:([\s\S]*)").unwrap();
    if let Some(caps) = qwen_react_regex.captures(response) {
        let thought = caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
        let action = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
        return (thought, action);
    }

    // Fallback to regex for <thinking> tags
    let thinking_regex = Regex::new(r"<thinking>([\s\S]*?)</thinking>").unwrap();
    if let Some(caps) = thinking_regex.captures(response) {
        let thinking_content = caps[1].trim().to_string();
        let plan_str = thinking_regex.replace(response, "").trim().to_string();
        return (thinking_content, plan_str);
    }

    // If no specific format is found, return the original response as plan
    (String::new(), response.to_string())
}

async fn generate_session_title_and_description(
    initial_message: &str,
    api_key: &str,
    model: &str,
) -> Result<(String, String)> {
    let client = rainy_sdk::RainyClient::with_api_key(api_key)
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to create API client: {}. This might be due to an invalid API key format.", e)))?;

    let prompt = format!(
        r#"Analyze the following user message and generate:
1. A concise title (max 50 chars) that captures the essence of the topic.
2. A brief description (max 100 chars) explaining what will be discussed.

User message: "{}"

Respond ONLY with a valid JSON in this format:
{{
    "title": "Generated Title",
    "description": "Generated Description"
}}"#,
        initial_message
    );

    let request = rainy_sdk::ChatCompletionRequest {
        messages: vec![rainy_sdk::ChatMessage {
            role: rainy_sdk::MessageRole::User,
            content: prompt,
        }],
        model: model.to_string(),
        provider: None,
        temperature: Some(0.3),
        max_tokens: Some(150),
        stream: Some(false),
        stop: None,
        top_p: None,
        presence_penalty: None,
        frequency_penalty: None,
        user: None,
    };

    let response = client.create_chat_completion(request).await
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to generate session info from AI: {}. Please check your network connection and API provider status.", e)))?;

    if let Some(choice) = response.choices.first() {
        let content = choice.message.content.trim();

        // Intentar parsear como JSON
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
            if let (Some(title), Some(description)) = (
                json_value.get("title").and_then(|v| v.as_str()),
                json_value.get("description").and_then(|v| v.as_str())
            ) {
                return Ok((title.to_string(), description.to_string()));
            }
        }

        // Si no es JSON válido, extraer manualmente
        if content.contains("title") && content.contains("description") {
            // Extraer entre comillas después de "title":
            if let Some(title_start) = content.find("\"title\": \"") {
                let title_rest = &content[title_start + 10..];
                if let Some(title_end) = title_rest.find("\"") {
                    let title = title_rest[..title_end].to_string();

                    // Extraer descripción
                    if let Some(desc_start) = content.find("\"description\": \"") {
                        let desc_rest = &content[desc_start + 15..];
                        if let Some(desc_end) = desc_rest.find("\"") {
                            let description = desc_rest[..desc_end].to_string();
                            return Ok((title, description));
                        }
                    }
                }
            }
        }
    }

    // Fallback: usar el mensaje inicial como base
    let title = if initial_message.len() > 47 {
        format!("{}...", &initial_message[..47])
    } else {
        initial_message.to_string()
    };

    let description = format!("Conversation about: {}", initial_message);

    Ok((title, description))
}

use std::fs;

pub async fn handle_chat_command(
    message: Option<String>,
    context_files: Option<Vec<PathBuf>>,
    no_history: bool,
    config: &Config,
) -> Result<()> {
    let agents_md_path = std::path::Path::new("AGENTS.md");
    let mut agents_md_content = if agents_md_path.exists() {
        fs::read_to_string(agents_md_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut agents_md = utils::agents_md::parse_agents_md(&agents_md_content);

    if !agents_md_content.contains("Trust-Level:") {
        ui::print_info("First time in this project! Please choose a trust level for the AI agent.");
        let levels = &["low (always ask)", "medium (ask for sensitive tasks)", "high (never ask)"];
        let selection = dialoguer::Select::new()
            .with_prompt("Select a trust level:")
            .items(levels)
            .default(0)
            .interact()
            .map_err(|e| crate::error::CliError::command_error(&format!("Failed to read selection for trust level: {}", e)))?;

        let chosen_level = match selection {
            0 => "low",
            1 => "medium",
            2 => "high",
            _ => "low",
        };

        agents_md.trust_level = chosen_level.to_string();
        let trust_level_line = format!("\nTrust-Level: {}\n", chosen_level);
        agents_md_content.push_str(&trust_level_line);
        fs::write(agents_md_path, &agents_md_content)
            .map_err(|e| crate::error::CliError::file_error("Failed to write to AGENTS.md. Please check file permissions.", e))?;
        ui::print_success(&format!("Trust level set to '{}' and saved in AGENTS.md.", chosen_level));
    }

    ui::print_command_start("CHAT", &format!("{} Agentic Chat Mode", ui::CHAT));
    ui::print_chat_header();

    let api_key = config
        .get_api_key()
        .map_err(|e| crate::error::CliError::config_error(&format!("API key not configured. Please run `rainy-cli config --set-api-key YOUR_API_KEY`. Error: {}", e)))?;

    // Detectar si necesitamos crear una sesión automáticamente
    let session_manager = SessionManager::new()
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize session manager: {}. Please check permissions of the session directory.", e)))?;

    let (use_session, session_id) = if let Some(initial_msg) = &message {
        // If there is an initial message, create a session automatically
        ui::print_info("🎯 Creating automatic session for your query...");

        // Generate title and description using Llama-3.1-8b-instant
        let (title, description) = generate_session_title_and_description(initial_msg, &api_key, &config.title_model)
            .await
            .unwrap_or_else(|_| {
                // Fallback if automatic generation fails
                let title = if initial_msg.len() > 47 {
                    format!("{}...", &initial_msg[..47])
                } else {
                    initial_msg.clone()
                };
                let description = format!("Conversation about: {}", initial_msg);
                (title, description)
            });

        // Create the session
        let session = session_manager.create_session(title.clone(), Some(description.clone()))
            .map_err(|e| crate::error::CliError::api_error(&format!("Failed to create session: {}. Please check permissions of the session directory.", e)))?;

        // Show information about the created session
        ui::print_success(&format!("✅ Session created: \"{}\"", title));
        ui::print_info(&format!("📝 Description: {}", description));
        ui::print_info(&format!("🆔 Session ID: {}", session.id));
        ui::print_info(&format!("💡 You can use this session in the future with: rainy-cli session chat {} <message>", session.id));
        println!();

        (true, Some(session.id))
    } else {
        (false, None)
    };

    let agent = executor::AgenticExecutor::new(
        api_key.to_string(),
        Some(config.get_model().to_string()),
    )
    .await
    .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize AI agent: {}. This could be due to an invalid API key or network issues.", e)))?;

    let mut messages = Vec::new();

    // Load hierarchical AGENTS.md content and add it as a system message
    match agents_md::load_hierarchical_agents_md() {
        Ok(agents_md_content) => {
            if !agents_md_content.is_empty() {
                messages.push(executor::ChatMessage {
                    role: "system".to_string(),
                    content: format!("Here is the content from the AGENTS.md files:\n\n{}", agents_md_content),
                });
            }
        }
        Err(e) => {
            ui::print_warning(&format!("Failed to load AGENTS.md content: {}", e));
        }
    }

    // If we are using a session, load its messages
    if let Some(ref session_id) = session_id {
        if let Ok(session) = session_manager.load_session(session_id) {
            let message_count = session.messages.len();
            let session_messages: Vec<executor::ChatMessage> = session.messages.into_iter()
                .map(|msg| executor::ChatMessage {
                    role: msg.role,
                    content: msg.content,
                })
                .collect();

            messages.extend(session_messages);
            ui::print_info(&format!("📚 Loaded {} previous messages from the session", message_count));
        }
    } else if !no_history {
        // If we are not using a session, load normal history (only if not disabled)
        if let Ok(history) = history::load_conversation_history_truncated(500) {
            messages.extend(history);
            if !messages.is_empty() {
                ui::print_info("Previous conversation history loaded (truncated for token efficiency)");
            }
        }
    } else {
        ui::print_info("Conversation history skipped (--no-history flag used)");
    }

    // If context files are provided, load their content and add it to the conversation
    if let Some(files) = context_files {
        if !files.is_empty() {
            let context_str = context::collect_context_from_paths(&files)
                .map_err(|e| crate::error::CliError::context_error("Failed to collect context from file paths.", e))?;
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
            ui::prompt_input().map_err(|e| crate::error::CliError::file_error("Failed to read user input. Please check terminal permissions.", e))?;
        messages.push(executor::ChatMessage {
            role: "user".to_string(),
            content: input,
        });
    }

    // Run the loop with or without a session
    if use_session {
        if let Some(session_id) = session_id {
            run_session_chat_loop(&mut messages, &agent, &session_id, &session_manager).await?;
        }
    } else {
        run_agentic_loop(&mut messages, &agent).await?;
    }

    Ok(())
}

use walkdir::WalkDir;

async fn run_agentic_loop(
    messages: &mut Vec<executor::ChatMessage>,
    agent: &executor::AgenticExecutor,
) -> Result<()> {
    let agents_md_content = agents_md::load_hierarchical_agents_md().unwrap_or_default();
    let agents_md = agents_md::parse_agents_md(&agents_md_content);

    loop {
        // If the last message was from the assistant, get user input
        if messages.last().map_or(true, |m| m.role == "assistant") {
            let mut input = ui::prompt_input()
                .map_err(|e| crate::error::CliError::file_error("Failed to read user input. Please check terminal permissions.", e))?;

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

        let pb = ui::print_progress("Rainy AI is working...");
        let (response, duration) = agent.chat(messages.clone()).await.map_err(|e| {
            crate::error::CliError::api_error(&format!("Failed to get AI response: {}. Please check your network connection and API provider status.", e))
        })?;
        pb.finish_and_clear();

        let response_content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let (thinking_content, plan_str) = parse_agent_response(&response_content);
        if !thinking_content.is_empty() {
            ui::print_thinking_message(&thinking_content);
        }


        // The agent's response should be a JSON plan.
        // Attempt to parse it.
        match serde_json::from_str::<Vec<tools::ToolCall>>(&plan_str) {
            Ok(plan) => {
                if !plan.is_empty() {
                    ui::print_agent_plan_conversationally(&plan);
                }

                let should_confirm = match agents_md.trust_level.as_str() {
                    "low" => true,
                    "medium" => plan.iter().any(|call| matches!(call, tools::ToolCall::WriteFile { .. } | tools::ToolCall::PatchFile { .. } | tools::ToolCall::DeleteFile { .. })),
                    "high" => false,
                    _ => true,
                };

                let confirmed = if should_confirm {
                    ui::prompt_for_confirmation()
                        .map_err(|e| crate::error::CliError::file_error("Failed to read confirmation. Please check terminal permissions.", e))?
                } else {
                    true
                };

                if confirmed {
                    let mut file_modifications: Vec<utils::diff::FileModification> = Vec::new();
                    let mut results: Vec<crate::tools::ToolResult> = Vec::new();
                    for tool_call in &plan {
                        ui::print_info(&tool_call_to_running_string(tool_call));
                        let result = tools::execute_tool(tool_call.clone(), &mut file_modifications)
                            .await
                            .map_err(|e| crate::error::CliError::api_error(&e.to_string()))?;
                        results.push(result);
                    }

                    let mut results_str = serde_json::to_string_pretty(&results).unwrap();
                    ui::print_code_block("Execution Results", &results_str);

                    let test_results = execute_test_commands(&plan, &agents_md).await?;
                    if !test_results.is_empty() {
                        results_str.push_str("\n\n--- Test Results ---\n");
                        results_str.push_str(&test_results);
                        ui::print_code_block("Test Results", &test_results);
                    }

                    if !file_modifications.is_empty() {
                        ui::print_file_modification_summary(&file_modifications);
                    }

                    // Add both the agent's plan and the execution results to the conversation
                    messages.push(executor::ChatMessage {
                        role: "assistant".to_string(),
                        content: response_content,
                    });
                    messages.push(executor::ChatMessage {
                        role: "user".to_string(),
                        content: format!("Here are the results of the execution:\n{}", results_str),
                    });


                    // Update AGENTS.md with the activity
                    let summary = serde_json::to_string_pretty(&plan).unwrap();
                    if let Err(e) = utils::agents_md::append_activity_to_agents_md(&summary) {
                        ui::print_warning(&format!("Failed to update AGENTS.md with activity: {}", e));
                    }
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
        .map_err(|e| crate::error::CliError::file_error("Failed to read file selection. Please check terminal permissions.", e))?;

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
            .map_err(|e| crate::error::CliError::file_error(&format!("Failed to read selected file: {}", selected_path.display()), e))?;
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

pub async fn handle_chat_with_session(
    session_messages: Vec<executor::ChatMessage>,
    context_files: Option<Vec<PathBuf>>,
    _no_history: bool,
    session_id: &str,
    session_manager: &SessionManager,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("CHAT", &format!("{} Session Chat Mode", ui::CHAT));

    let api_key = config
        .get_api_key()
        .map_err(|e| crate::error::CliError::config_error(&format!("API key not configured. Please run `rainy-cli config --set-api-key YOUR_API_KEY`. Error: {}", e)))?;

    let agent = executor::AgenticExecutor::new(
        api_key.to_string(),
        Some(config.get_model().to_string()),
    )
    .await
    .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize AI agent: {}. This could be due to an invalid API key or network issues.", e)))?;

    let mut messages = Vec::new();

    // Load hierarchical AGENTS.md content and add it as a system message
    match agents_md::load_hierarchical_agents_md() {
        Ok(agents_md_content) => {
            if !agents_md_content.is_empty() {
                messages.push(executor::ChatMessage {
                    role: "system".to_string(),
                    content: format!("Here is the content from the AGENTS.md files:\n\n{}", agents_md_content),
                });
            }
        }
        Err(e) => {
            ui::print_warning(&format!("Failed to load AGENTS.md content: {}", e));
        }
    }

    // Add session messages
    messages.extend(session_messages);

    // If context files are provided, load their content and add it to the conversation
    if let Some(files) = context_files {
        if !files.is_empty() {
            let context_str = context::collect_context_from_paths(&files)
                .map_err(|e| crate::error::CliError::context_error("Failed to collect context from file paths.", e))?;
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

    run_session_chat_loop(&mut messages, &agent, session_id, session_manager).await?;

    Ok(())
}

async fn execute_test_commands(
    plan: &Vec<tools::ToolCall>,
    agents_md: &utils::agents_md::AgentsMd,
) -> Result<String> {
    let mut results = String::new();
    let should_run_tests = plan.iter().any(|call| {
        matches!(call, tools::ToolCall::WriteFile { .. } | tools::ToolCall::DeleteFile { .. })
    });

    if should_run_tests {
        if let Some(test_section) = agents_md.commands.iter().find(|s| s.heading.eq_ignore_ascii_case("Test")) {
            ui::print_info("Found test commands in AGENTS.md.");

            let run_commands = if agents_md.execution_confirmation {
                ui::print_info("Do you want to run the test commands?");
                ui::prompt_for_confirmation().unwrap_or(false)
            } else {
                true
            };

            if run_commands {
                for command in &test_section.commands {
                    ui::print_info(&format!("Running command: {}", command));
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .output()
                        .map_err(|e| crate::error::CliError::command_error(&format!("Failed to execute test command: `{}`. Error: {}", command, e)))?;
                    results.push_str(&format!("--- Output of '{}' ---\n", command));
                    results.push_str(&String::from_utf8_lossy(&output.stdout));
                    results.push_str(&String::from_utf8_lossy(&output.stderr));
                    results.push_str("\n\n");
                }
            } else {
                ui::print_warning("Test commands skipped by user.");
            }
        }
    }

    Ok(results)
}

async fn run_session_chat_loop(
    messages: &mut Vec<executor::ChatMessage>,
    agent: &executor::AgenticExecutor,
    session_id: &str,
    session_manager: &SessionManager,
) -> Result<()> {
    let agents_md_content = agents_md::load_hierarchical_agents_md().unwrap_or_default();
    let agents_md = agents_md::parse_agents_md(&agents_md_content);

    loop {
        // If the last message was from the assistant, get user input
        if messages.last().map_or(true, |m| m.role == "assistant") {
            let mut input = ui::prompt_input()
                .map_err(|e| crate::error::CliError::file_error("Failed to read user input. Please check terminal permissions.", e))?;

            if input.starts_with('@') {
                input = handle_at_command(&input).await?;
            }

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                ui::print_success("Goodbye! 👋");
                break;
            }

            messages.push(executor::ChatMessage {
                role: "user".to_string(),
                content: input.clone(),
            });
        }

        let pb = ui::print_progress("Rainy AI is working...");
        let (response, duration) = agent.chat(messages.clone()).await.map_err(|e| {
            crate::error::CliError::api_error(&format!("Failed to get AI response: {}. Please check your network connection and API provider status.", e))
        })?;
        pb.finish_and_clear();

        let response_content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let (thinking_content, plan_str) = parse_agent_response(&response_content);
        if !thinking_content.is_empty() {
            ui::print_thinking_message(&thinking_content);
        }

        // The agent's response should be a JSON plan.
        // Attempt to parse it.
        match serde_json::from_str::<Vec<tools::ToolCall>>(&plan_str) {
            Ok(plan) => {
                if !plan.is_empty() {
                    ui::print_agent_plan_conversationally(&plan);
                }

                let should_confirm = match agents_md.trust_level.as_str() {
                    "low" => true,
                    "medium" => plan.iter().any(|call| matches!(call, tools::ToolCall::WriteFile { .. } | tools::ToolCall::PatchFile { .. } | tools::ToolCall::DeleteFile { .. })),
                    "high" => false,
                    _ => true,
                };

                let confirmed = if should_confirm {
                    ui::prompt_for_confirmation()
                        .map_err(|e| crate::error::CliError::file_error("Failed to read confirmation. Please check terminal permissions.", e))?
                } else {
                    true
                };

                if confirmed {
                    let mut file_modifications: Vec<utils::diff::FileModification> = Vec::new();
                    let mut results: Vec<crate::tools::ToolResult> = Vec::new();
                    for tool_call in &plan {
                        ui::print_info(&tool_call_to_running_string(tool_call));
                        let result = tools::execute_tool(tool_call.clone(), &mut file_modifications)
                            .await
                            .map_err(|e| crate::error::CliError::api_error(&e.to_string()))?;
                        results.push(result);
                    }

                    let mut results_str = serde_json::to_string_pretty(&results).unwrap();
                    ui::print_code_block("Execution Results", &results_str);

                    let test_results = execute_test_commands(&plan, &agents_md).await?;
                    if !test_results.is_empty() {
                        results_str.push_str("\n\n--- Test Results ---\n");
                        results_str.push_str(&test_results);
                        ui::print_code_block("Test Results", &test_results);
                    }

                    if !file_modifications.is_empty() {
                        ui::print_file_modification_summary(&file_modifications);
                    }

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

        // Save messages back to session
        let session_messages: Vec<ChatMessage> = messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        if let Err(e) = session_manager.save_session_messages(session_id, &session_messages) {
            ui::print_warning(&format!("Failed to save session messages: {}", e));
        }
    }

    Ok(())
}
