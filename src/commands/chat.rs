use miette::Result;
use crate::{config::Config, executor, tools, ui, utils::{context, history, rainy_md, sessions::{SessionManager, ChatMessage}}};
use std::path::PathBuf;

async fn generate_session_title_and_description(
    initial_message: &str,
    api_key: &str,
    model: &str,
) -> Result<(String, String)> {
    let client = rainy_sdk::RainyClient::with_api_key(api_key)
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to create client: {}", e)))?;

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
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to generate session info: {}", e)))?;

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

    let description = format!("Conversación sobre: {}", initial_message);

    Ok((title, description))
}

pub async fn handle_chat_command(
    message: Option<String>,
    context_files: Option<Vec<PathBuf>>,
    no_history: bool,
    config: &Config,
) -> Result<()> {
    // Ensure rainy.md exists before starting chat
    if let Err(e) = rainy_md::ensure_rainy_md_exists(config).await {
        ui::print_warning(&format!("Could not ensure rainy.md exists: {}", e));
    }

    ui::print_command_start("CHAT", &format!("{} Agentic Chat Mode", ui::CHAT));
    ui::print_chat_header();

    let api_key = config
        .get_api_key()
        .map_err(|e| crate::error::CliError::config_error(&format!("API key not configured: {}", e)))?;

    // Detectar si necesitamos crear una sesión automáticamente
    let session_manager = SessionManager::new()
        .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize session manager: {}", e)))?;

    let (use_session, session_id) = if let Some(initial_msg) = &message {
        // Si hay mensaje inicial, crear sesión automáticamente
        ui::print_info("🎯 Creando sesión automática para tu consulta...");

        // Generar título y descripción usando Llama-3.1-8b-instant
        let (title, description) = generate_session_title_and_description(initial_msg, &api_key, &config.title_model)
            .await
            .unwrap_or_else(|_| {
                // Fallback si falla la generación automática
                let title = if initial_msg.len() > 47 {
                    format!("{}...", &initial_msg[..47])
                } else {
                    initial_msg.clone()
                };
                let description = format!("Conversación sobre: {}", initial_msg);
                (title, description)
            });

        // Crear la sesión
        let session = session_manager.create_session(title.clone(), Some(description.clone()))
            .map_err(|e| crate::error::CliError::api_error(&format!("Failed to create session: {}", e)))?;

        // Mostrar información de la sesión creada
        ui::print_success(&format!("✅ Sesión creada: \"{}\"", title));
        ui::print_info(&format!("📝 Descripción: {}", description));
        ui::print_info(&format!("🆔 ID de sesión: {}", session.id));
        ui::print_info(&format!("💡 Puedes usar esta sesión en el futuro con: rainy-cli session chat {} <mensaje>", session.id));
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
    .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

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

    // Si estamos usando una sesión, cargar sus mensajes
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
            ui::print_info(&format!("📚 Cargados {} mensajes previos de la sesión", message_count));
        }
    } else if !no_history {
        // Si no usamos sesión, cargar historial normal (solo si no está deshabilitado)
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
                .map_err(|e| crate::error::CliError::context_error("Failed to collect context from paths", e))?;
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
            ui::prompt_input().map_err(|e| crate::error::CliError::file_error("Failed to read input", e))?;
        messages.push(executor::ChatMessage {
            role: "user".to_string(),
            content: input,
        });
    }

    // Ejecutar el loop con o sin sesión
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
    loop {
        // If the last message was from the assistant, get user input
        if messages.last().map_or(true, |m| m.role == "assistant") {
            let mut input = ui::prompt_input()
                .map_err(|e| crate::error::CliError::file_error("Failed to read user input", e))?;

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
            crate::error::CliError::api_error(&format!("Failed to get AI response: {}", e))
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
                    .map_err(|e| crate::error::CliError::file_error("Failed to read confirmation", e))?
                {
                    ui::print_info("Executing plan...");
                    let mut results = Vec::new();
                    for tool_call in plan {
                        let result = tools::execute_tool(tool_call)
                            .await
                            .map_err(|e| crate::error::CliError::api_error(&e.to_string()))?;
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
        .map_err(|e| crate::error::CliError::file_error("Failed to read input", e))?;

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
            .map_err(|e| crate::error::CliError::file_error("Failed to read file", e))?;
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
        .map_err(|e| crate::error::CliError::config_error(&format!("API key not configured: {}", e)))?;

    let agent = executor::AgenticExecutor::new(
        api_key.to_string(),
        Some(config.get_model().to_string()),
    )
    .await
    .map_err(|e| crate::error::CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

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

    // Add session messages
    messages.extend(session_messages);

    // If context files are provided, load their content and add it to the conversation
    if let Some(files) = context_files {
        if !files.is_empty() {
            let context_str = context::collect_context_from_paths(&files)
                .map_err(|e| crate::error::CliError::context_error("Failed to collect context from paths", e))?;
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

async fn run_session_chat_loop(
    messages: &mut Vec<executor::ChatMessage>,
    agent: &executor::AgenticExecutor,
    session_id: &str,
    session_manager: &SessionManager,
) -> Result<()> {
    loop {
        // If the last message was from the assistant, get user input
        if messages.last().map_or(true, |m| m.role == "assistant") {
            let mut input = ui::prompt_input()
                .map_err(|e| crate::error::CliError::file_error("Failed to read user input", e))?;

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

        let pb = ui::print_progress("AI is thinking...");
        let (response, duration) = agent.chat(messages.clone()).await.map_err(|e| {
            crate::error::CliError::api_error(&format!("Failed to get AI response: {}", e))
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
                    .map_err(|e| crate::error::CliError::file_error("Failed to read confirmation", e))?
                {
                    ui::print_info("Executing plan...");
                    let mut results = Vec::new();
                    for tool_call in plan {
                        let result = tools::execute_tool(tool_call)
                            .await
                            .map_err(|e| crate::error::CliError::api_error(&e.to_string()))?;
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
