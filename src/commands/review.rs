use crate::{agent, config::Config, error::CliError, ui, utils::git};
use miette::Result;
use std::path::PathBuf;

pub async fn handle_review_command(
    path: Option<PathBuf>,
    focus: Option<String>,
    git_review: bool,
    git_ref: String,
    config: &Config,
) -> Result<()> {
    let api_key = config.get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;
    let agent = agent::AIAgent::new(api_key.to_string())
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;

    if git_review {
        handle_git_review(git_ref, focus, &agent).await
    } else {
        handle_file_review(path, focus, &agent).await
    }
}

async fn handle_git_review(
    git_ref: String,
    focus: Option<String>,
    agent: &agent::AIAgent,
) -> Result<()> {
    ui::print_command_start("GIT-REVIEW", &format!("{} Reviewing Git changes", ui::EYES));
    ui::print_review_header(&format!("Git changes (vs {})", git_ref), focus.as_deref());

    let pb = ui::print_progress("Reading Git diff...");
    let changed_files = git::get_git_changes(&git_ref)
        .map_err(|e| CliError::command_error(&format!("Failed to get Git changes: {}", e)))?;
    ui::update_progress(&pb, &format!("Found {} changed files", changed_files.len()));

    if changed_files.is_empty() {
        pb.finish_with_message("No changes to review");
        ui::print_success("No Git changes found to review");
        return Ok(());
    }

    let mut all_reviews = Vec::new();
    let mut critical_issues = 0;
    let mut suggestions = 0;

    let total_files = changed_files.len();
    for (file_path, content) in &changed_files {
        ui::update_progress(&pb, &format!("Reviewing {}", file_path));
        let review = agent.review_code(&content, focus.as_deref())
            .await
            .map_err(|e| CliError::analysis_error(&format!("Failed to review {}: {}", file_path, e)))?;
        
        // Count issues (simple heuristic)
        if review.to_lowercase().contains("critical") || review.to_lowercase().contains("security") {
            critical_issues += 1;
        }
        if review.to_lowercase().contains("suggest") || review.to_lowercase().contains("recommend") {
            suggestions += 1;
        }
        
        all_reviews.push(format!("### {}\n{}", file_path, review));
    }

    pb.finish_with_message("Git review complete");
    ui::print_separator();
    
    // Enhanced summary
    ui::print_review_summary(total_files, critical_issues, suggestions);
    ui::print_code_block("Git Review Results", &all_reviews.join("\n\n"));
    
    Ok(())
}

async fn handle_file_review(
    path: Option<PathBuf>,
    focus: Option<String>,
    agent: &agent::AIAgent,
) -> Result<()> {
    let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
    ui::print_command_start("REVIEW", &format!("{} Reviewing {}", ui::EYES, path.display()));
    ui::print_review_header(&path.display().to_string(), focus.as_deref());

    let pb = ui::print_progress("Reading file...");
    let code = agent::read_file_content(&path)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to read file: {}", path.display()), std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    ui::update_progress(&pb, "File loaded, reviewing content...");
    let review = agent.review_code(&code, focus.as_deref())
        .await
        .map_err(|e| CliError::analysis_error(&format!("Failed to review code: {}", e)))?;
    pb.finish_with_message("Review complete");

    ui::print_separator();
    ui::print_code_block("Review Results", &review);
    
    // Offer interactive features
    ui::print_info("Would you like to apply any suggested improvements? (y/n)");
    if let Ok(response) = ui::prompt_input() {
        if response.trim().eq_ignore_ascii_case("y") || response.trim().eq_ignore_ascii_case("yes") {
            ui::print_warning("Interactive improvement application coming soon!");
        }
    }
    
    Ok(())
}
