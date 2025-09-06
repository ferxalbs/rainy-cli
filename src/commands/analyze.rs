use crate::{agent, config::Config, error::CliError, ui};
use miette::Result;
use std::path::PathBuf;

pub async fn handle_analyze_command(
    path: PathBuf,
    analysis_type: String,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("ANALYZE", &format!("{} Analyzing {}", ui::MAGNIFYING_GLASS, path.display()));
    ui::print_analysis_header(&path.display().to_string(), &analysis_type);

    let pb = ui::print_progress("Reading file...");
    let code = agent::read_file_content(&path)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to read file: {}", path.display()), std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    ui::update_progress(&pb, "File loaded, analyzing content...");
    
    let api_key = config.get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;
    let agent = agent::AIAgent::new(api_key.to_string())
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;
        
    let analysis = agent.analyze_code(&code, &analysis_type)
        .await
        .map_err(|e| CliError::analysis_error(&format!("Failed to analyze code: {}", e)))?;
    
    pb.finish_with_message("Analysis complete");

    ui::print_separator();
    ui::print_code_block("Analysis Results", &analysis);
    
    // Enhanced feature: Ask if user wants to apply suggestions
    if analysis_type == "performance" || analysis_type == "style" {
        ui::print_info("Would you like to see suggested code improvements? (y/n)");
        if let Ok(response) = ui::prompt_input() {
            if response.trim().eq_ignore_ascii_case("y") || response.trim().eq_ignore_ascii_case("yes") {
                ui::print_warning("Interactive code application feature coming soon!");
            }
        }
    }
    
    Ok(())
}
