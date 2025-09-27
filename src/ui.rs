use colored::*;
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};

pub const ROCKET: Emoji = Emoji("🚀", "");
pub const GEAR: Emoji = Emoji("⚙️", "");
pub const SPARKLES: Emoji = Emoji("✨", "");
pub const BRAIN: Emoji = Emoji("🧠", "");
pub const CODE: Emoji = Emoji("💻", "");
pub const CHECK: Emoji = Emoji("✅", "");
pub const CROSS: Emoji = Emoji("❌", "");
pub const WARNING: Emoji = Emoji("⚠️", "");
pub const INFO: Emoji = Emoji("ℹ️", "");
pub const KEY: Emoji = Emoji("🔑", "");
pub const CHAT: Emoji = Emoji("💬", "");
pub const FORWARD: Emoji = Emoji("↪️", "");
pub const TOKENS: Emoji = Emoji("</>", "TOK");
pub const ROBOT: Emoji = Emoji("🤖", "");
pub const STOPWATCH: Emoji = Emoji("⏱️", "");
pub const ADD: Emoji = Emoji("➕", "");
pub const LIST: Emoji = Emoji("📋", "");
pub const EDIT: Emoji = Emoji("✏️", "");
pub const TAG: Emoji = Emoji("🏷️", "");
pub const DELETE: Emoji = Emoji("🗑️", "");
pub const CLEAR: Emoji = Emoji("🧹", "");
pub const SEARCH: Emoji = Emoji("🔎", "");
pub const EXPORT: Emoji = Emoji("📤", "");
pub const IMPORT: Emoji = Emoji("📥", "");

pub fn print_header() {
    println!("{}", style("╔══════════════════════════════════════════════════════════════════════════════╗").cyan());
    println!("{}", style("║                                                                            ║").cyan());
    println!("{}", style(format!("║{:^74}║", format!("{} Rainy CLI - AI Code Assistant", ROCKET))).bold().white());
    println!("{}", style(format!("║{:^74}║", "Built with Rust & AI")).dim());
    println!("{}", style("║                                                                            ║").cyan());
    println!("{}", style("╚══════════════════════════════════════════════════════════════════════════════╝").cyan());
    println!();
}

pub fn print_welcome() {
    print_header();
    println!("{} {}", SPARKLES, "Welcome to your AI-powered coding companion!".bold().bright_green());
    println!("{} {}", BRAIN, "Ready to analyze, generate, review, and chat about code.".bright_blue());
    println!();
}

pub fn print_api_key_setup() {
    println!("{}", style("┌─ API Key Setup ─────────────────────────────────────────────────────────────┐").yellow());
    println!("{}", style(format!("│ {} Please provide your API key to get started.{:47}│", KEY, "")).yellow());
    println!("{}", style(format!("│    Your key will be securely stored in: ~/.rainy-cli/config.toml{:3}│", "")).yellow());
    println!("{}", style(format!("│    Default model: moonshotai/kimi-k2-instruct{:23}│", "")).yellow());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").yellow());
    println!();
}

pub fn print_success(message: &str) {
    println!("{} {}", CHECK, message.green().bold());
}

pub fn print_error(message: &str) {
    println!("{} {}", CROSS, message.red().bold());
}

pub fn print_warning(message: &str) {
    println!("{} {}", WARNING, message.yellow().bold());
}

pub fn print_info(message: &str) {
    println!("{} {}", INFO, message.blue().bold());
}

pub fn print_command_start(command: &str, description: &str) {
    println!();
    println!("{}", style(format!("┌─ {} {}", command, description)).cyan().bold());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").cyan());
}

pub fn print_progress(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

pub fn print_code_block(title: &str, content: &str) {
    println!();
    println!("{}", style(format!("┌─ {} ────────────────────────────────────────────────────────────────────────┐", title)).green());
    println!("{}", style("│").green());

    for line in content.lines() {
        println!("{}", style(format!("│ {}", line)).green());
    }

    println!("{}", style("│").green());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").green());
}

pub fn print_separator() {
    println!("{}", style("────────────────────────────────────────────────────────────────────────────────").dim());
}

pub fn prompt_api_key() -> Result<String, std::io::Error> {
    print!("{} ", style("API Key:").cyan().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn prompt_input_with_prompt(prompt: &str) -> Result<String, std::io::Error> {
    print!("{} ", style(prompt).cyan().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn print_chat_header() {
    println!("{}", style("┌─ Interactive Chat Mode ──────────────────────────────────────────────────────┐").magenta());
    println!("{}", style(format!("│ {} Type your messages and press Enter. Type 'exit' or 'quit' to leave.│", CHAT)).magenta());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").magenta());
    println!();
}

pub fn print_ai_message(message: &str) {
    println!();
    println!("{}", message);
    println!();
}


pub fn print_generation_header(description: &str) {
    println!();
    println!("{}", style("┌─ Code Generation ────────────────────────────────────────────────────────────┐".to_string()).yellow());
    println!("{}", style(format!("│ {} {}{}│", CODE, description, " ".repeat(50 - description.len().min(50)))).yellow());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").yellow());
}

pub fn prompt_input() -> Result<String, std::io::Error> {
    print!("{} ", style(">").cyan().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}



pub fn print_agent_plan_conversationally(plan: &[crate::tools::ToolCall]) {
    println!("{}", style("Okay, I will do the following:").green());
    for (i, tool_call) in plan.iter().enumerate() {
        let message = match tool_call {
            crate::tools::ToolCall::ReadFile { path } => format!("Read the file `{}`", path),
            crate::tools::ToolCall::WriteFile { path, .. } => format!("Write to the file `{}`", path),
            crate::tools::ToolCall::PatchFile { path, .. } => format!("Patch the file `{}`", path),
            crate::tools::ToolCall::DeleteFile { path } => format!("Delete the file `{}`", path),
            crate::tools::ToolCall::ListFiles { path } => format!("List the files in `{}`", path),
            crate::tools::ToolCall::Grep { pattern, path } => format!("Search for `{}` in `{}`", pattern, path.as_deref().unwrap_or(".")),
        };
        println!("  {}. {}", i + 1, message);
    }
    println!();
}

pub fn prompt_for_confirmation() -> Result<bool, std::io::Error> {
    print!("{} > ", style("Do you want to execute this plan? (y/n)").bold().yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

pub fn print_file_modification_summary(modifications: &Vec<crate::utils::diff::FileModification>) {
    println!();
    println!("{}", style("File Modification Summary:").bold().yellow());
    for modification in modifications {
        let added = format!("+{}", modification.lines_added).green();
        let removed = format!("-{}", modification.lines_removed).red();
        println!("  - {}: {} {}", modification.path, added, removed);
    }
    println!();
}

pub fn print_response_metrics(response: &rainy_sdk::ChatCompletionResponse, duration: std::time::Duration) {
    if let Some(usage) = &response.usage {
        let stats = format!(
            "{} Tokens: [Prompt: {}, Completion: {}, Total: {}]  {} Model: {}  {} Speed: {:.2?}",
            TOKENS,
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens,
            ROBOT,
            response.model,
            STOPWATCH,
            duration
        );
        println!("{}", style(stats).dim());
    }
}