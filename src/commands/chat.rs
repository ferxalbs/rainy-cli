use crate::{agent, config::Config, error::CliError, ui, utils::history, utils::context};
use miette::Result;

pub async fn handle_chat_command(
    message: Option<String>,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("CHAT", &format!("{} Interactive AI Chat", ui::CHAT));
    ui::print_chat_header();

    let api_key = config.get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;
    let agent = agent::AIAgent::new(api_key.to_string())
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

    let mut messages = Vec::new();

    if let Some(initial_msg) = message {
        handle_initial_message(initial_msg, &mut messages, &agent).await?;
    } else {
        initialize_chat_context(&mut messages);
    }

    ui::print_info(&format!("{} Type your messages or commands. Use 'help' for assistance.", ui::BOOK));
    ui::print_separator();

    // Load conversation history
    if let Ok(history) = history::load_conversation_history() {
        messages.extend(history);
        if !messages.is_empty() {
            ui::print_info("Previous conversation history loaded");
        }
    }

    // Main chat loop
    run_chat_loop(&mut messages, &agent).await?;
    
    Ok(())
}

async fn handle_initial_message(
    initial_msg: String,
    messages: &mut Vec<agent::ChatMessage>,
    agent: &agent::AIAgent,
) -> Result<()> {
    ui::print_user_message(&initial_msg);

    // Load project context
    let project_context = context::load_project_context()
        .unwrap_or_else(|_| "Could not load project context.".to_string());

    messages.push(agent::ChatMessage {
        role: "system".to_string(),
        content: format!("You are Rainy Coder 1, an AI assistant specialized in code assistance. Here is the current project context:\n\n{}", project_context),
    });

    messages.push(agent::ChatMessage {
        role: "user".to_string(),
        content: initial_msg.clone(),
    });

    let pb = ui::print_progress("AI is thinking...");
    ui::update_progress(&pb, "Processing your message...");
    let response = agent.chat(messages.clone())
        .await
        .map_err(|e| CliError::api_error(&format!("Failed to get AI response: {}", e)))?;
    pb.finish_with_message("Response received");

    ui::print_ai_message(&response);

    messages.push(agent::ChatMessage {
        role: "assistant".to_string(),
        content: response.clone(),
    });

    // Save conversation history
    let _ = history::save_conversation_history(messages);
    
    Ok(())
}

fn initialize_chat_context(messages: &mut Vec<agent::ChatMessage>) {
    // Load project context for ongoing conversation
    let project_context = context::load_project_context()
        .unwrap_or_else(|_| "Could not load project context.".to_string());

    messages.push(agent::ChatMessage {
        role: "system".to_string(),
        content: format!("You are Rainy Coder 1, an AI assistant specialized in code assistance. Here is the current project context:\n\n{}", project_context),
    });
}

async fn run_chat_loop(
    messages: &mut Vec<agent::ChatMessage>,
    agent: &agent::AIAgent,
) -> Result<()> {
    loop {
        let input = ui::prompt_input()
            .map_err(|e| CliError::file_error("Failed to read input", e))?;

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            ui::print_success("Goodbye! 👋");
            break;
        }

        if input.eq_ignore_ascii_case("help") {
            show_chat_help();
            continue;
        }

        if input.eq_ignore_ascii_case("clear") {
            messages.clear();
            let _ = std::fs::remove_file(history::get_history_file_path());
            ui::print_success("Conversation history cleared!");
            continue;
        }

        if input.eq_ignore_ascii_case("save") {
            let _ = history::save_conversation_history(messages);
            ui::print_success("Conversation saved!");
            continue;
        }

        if input.eq_ignore_ascii_case("context") {
            let context = context::load_project_context()
                .unwrap_or_else(|_| "Could not load project context.".to_string());
            ui::print_code_block("Project Context", &context);
            continue;
        }

        if input.trim().is_empty() {
            ui::print_warning("Please enter a message");
            continue;
        }

        // Handle special commands
        if input.starts_with("/analyze ") {
            let file_path = input.strip_prefix("/analyze ").unwrap().trim();
            handle_inline_analyze(file_path, messages, agent).await?;
            continue;
        }

        if input.starts_with("/review ") {
            let file_path = input.strip_prefix("/review ").unwrap().trim();
            handle_inline_review(file_path, messages, agent).await?;
            continue;
        }

        // Regular chat message
        ui::print_user_message(&input);

        messages.push(agent::ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
        });

        let pb = ui::print_progress("AI is thinking...");
        ui::update_progress(&pb, "Processing your message...");
        let response = agent.chat(messages.clone())
            .await
            .map_err(|e| CliError::api_error(&format!("Failed to get AI response: {}", e)))?;
        pb.finish_with_message("Response received");

        ui::print_ai_message(&response);

        messages.push(agent::ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        // Auto-save conversation history
        let _ = history::save_conversation_history(messages);
    }
    
    Ok(())
}

fn show_chat_help() {
    ui::print_info("Available commands:");
    ui::print_info("• 'exit' or 'quit' - End the conversation");
    ui::print_info("• 'help' - Show this help message");
    ui::print_info("• 'clear' - Clear conversation history");
    ui::print_info("• 'save' - Save conversation history");
    ui::print_info("• 'context' - Show current project context");
    ui::print_info("• '/analyze <file>' - Analyze a file inline");
    ui::print_info("• '/review <file>' - Review a file inline");
}

async fn handle_inline_analyze(
    file_path: &str,
    messages: &mut Vec<agent::ChatMessage>,
    agent: &agent::AIAgent,
) -> Result<()> {
    let path = std::path::PathBuf::from(file_path);
    
    let pb = ui::print_progress(&format!("Analyzing {}", file_path));
    let code = agent::read_file_content(&path)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to read file: {}", path.display()), std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let analysis = agent.analyze_code(&code, "general")
        .await
        .map_err(|e| CliError::analysis_error(&format!("Failed to analyze code: {}", e)))?;
    pb.finish_with_message("Analysis complete");

    ui::print_code_block(&format!("Analysis: {}", file_path), &analysis);
    
    // Add to conversation context
    messages.push(agent::ChatMessage {
        role: "assistant".to_string(),
        content: format!("I've analyzed the file '{}'. Here's what I found:\n\n{}", file_path, analysis),
    });
    
    Ok(())
}

async fn handle_inline_review(
    file_path: &str,
    messages: &mut Vec<agent::ChatMessage>,
    agent: &agent::AIAgent,
) -> Result<()> {
    let path = std::path::PathBuf::from(file_path);
    
    let pb = ui::print_progress(&format!("Reviewing {}", file_path));
    let code = agent::read_file_content(&path)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to read file: {}", path.display()), std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let review = agent.review_code(&code, None)
        .await
        .map_err(|e| CliError::analysis_error(&format!("Failed to review code: {}", e)))?;
    pb.finish_with_message("Review complete");

    ui::print_code_block(&format!("Review: {}", file_path), &review);
    
    // Add to conversation context
    messages.push(agent::ChatMessage {
        role: "assistant".to_string(),
        content: format!("I've reviewed the file '{}'. Here are my findings:\n\n{}", file_path, review),
    });
    
    Ok(())
}
