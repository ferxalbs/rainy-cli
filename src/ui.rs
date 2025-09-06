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
pub const MAGNIFYING_GLASS: Emoji = Emoji("🔍", "");
pub const EYES: Emoji = Emoji("👁️", "");
pub const BOOK: Emoji = Emoji("📖", "");
pub const FORWARD: Emoji = Emoji("↪️", "");

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

pub fn update_progress(pb: &ProgressBar, message: &str) {
    pb.set_message(message.to_string());
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

pub fn print_chat_header() {
    println!("{}", style("┌─ Interactive Chat Mode ──────────────────────────────────────────────────────┐").magenta());
    println!("{}", style(format!("│ {} Type your messages and press Enter. Type 'exit' or 'quit' to leave.│", CHAT)).magenta());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").magenta());
    println!();
}

pub fn print_user_message(message: &str) {
    println!("{} {}", style("You:").cyan().bold(), message);
}

pub fn print_ai_message(message: &str) {
    println!("{} {}", style("AI:").green().bold(), message);
    println!();
}

pub fn print_analysis_header(path: &str, analysis_type: &str) {
    println!();
    println!("{}", style(format!("┌─ Code Analysis: {} ──────────────────────────────────────────────────────────┐", path)).blue());
    println!("{}", style(format!("│ {} Analysis Type: {}{:40}│", MAGNIFYING_GLASS, analysis_type, "")).bold().blue());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").blue());
}

pub fn print_review_header(path: &str, focus: Option<&str>) {
    println!();
    println!("{}", style(format!("┌─ Code Review: {} ────────────────────────────────────────────────────────────┐", path)).magenta());
    if let Some(focus_area) = focus {
        println!("{}", style(format!("│ {} Focus Area: {}{:45}│", EYES, focus_area, "")).bold().magenta());
    }
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").magenta());
}

pub fn print_generation_header(description: &str) {
    println!();
    println!("{}", style(format!("┌─ Code Generation ────────────────────────────────────────────────────────────┐")).yellow());
    println!("{}", style(format!("│ {} {}{}│", CODE, description, " ".repeat(50 - description.len().min(50)))).yellow());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").yellow());
}

pub fn prompt_input() -> Result<String, std::io::Error> {
    print!("{} ", style("You:").cyan().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn print_review_summary(files_reviewed: usize, critical_issues: usize, suggestions: usize) {
    println!();
    println!("{}", style("┌─ Review Summary ──────────────────────────────────────────────────────────────┐").green());
    println!("{}", style(format!("│ {} Files reviewed: {}{:35}│", CHECK, files_reviewed, "")).green());
    println!("{}", style(format!("│ {} Critical issues: {}{:32}│", if critical_issues > 0 { WARNING } else { CHECK }, critical_issues, "")).green());
    println!("{}", style(format!("│ {} Suggestions: {}{:36}│", INFO, suggestions, "")).green());
    
    let grade = if critical_issues == 0 && suggestions <= 2 {
        "A+"
    } else if critical_issues == 0 && suggestions <= 5 {
        "A"
    } else if critical_issues <= 1 && suggestions <= 8 {
        "B"
    } else {
        "C"
    };
    
    println!("{}", style(format!("│ {} Overall Grade: {}{:35}│", SPARKLES, grade, "")).green());
    println!("{}", style("└─────────────────────────────────────────────────────────────────────────────┘").green());
}

pub fn print_agent_plan(plan_json: &str) {
    println!("{}", style("AI Agent has proposed the following plan:").bold().yellow());
    print_code_block("Execution Plan", plan_json);
}

pub fn prompt_for_confirmation() -> Result<bool, std::io::Error> {
    print!("{} {}", style("Do you want to execute this plan? (y/n)").bold().yellow(), "> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}