use crate::{agent, config::Config, error::CliError, ui};
use miette::Result;
use std::path::PathBuf;

pub async fn handle_generate_command(
    description: String,
    output: Option<PathBuf>,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("GENERATE", &format!("{} Generating code", ui::CODE));
    ui::print_generation_header(&description);

    let pb = ui::print_progress("Generating code...");
    ui::update_progress(&pb, "AI is crafting your code...");
    
    let api_key = config.get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;
    let agent = agent::AIAgent::new(api_key.to_string())
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;
        
    let code = agent.generate_code(&description)
        .await
        .map_err(|e| CliError::api_error(&format!("Failed to generate code: {}", e)))?;
    
    pb.finish_with_message("Code generation complete");

    ui::print_separator();
    if let Some(output_path) = &output {
        ui::print_info(&format!("{} Saving to: {}", ui::GEAR, output_path.display()));
        tokio::fs::write(output_path, &code)
            .await
            .map_err(|e| CliError::file_error(&format!("Failed to save file: {}", output_path.display()), e))?;
        ui::print_success("Code saved successfully!");
        
        // Enhanced feature: Offer to generate tests
        ui::print_info("Would you like to generate unit tests for this code? (y/n)");
        if let Ok(response) = ui::prompt_input() {
            if response.trim().eq_ignore_ascii_case("y") || response.trim().eq_ignore_ascii_case("yes") {
                ui::print_warning("Test generation feature coming soon!");
            }
        }
    } else {
        ui::print_code_block("Generated Code", &code);
    }
    
    Ok(())
}

pub async fn handle_generate_tests(
    file_path: &PathBuf,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("GENERATE-TESTS", &format!("{} Generating tests for {}", ui::CODE, file_path.display()));
    
    let pb = ui::print_progress("Reading source file...");
    let code = agent::read_file_content(file_path)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to read file: {}", file_path.display()), std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    ui::update_progress(&pb, "Analyzing code structure...");
    
    let api_key = config.get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;
    let agent = agent::AIAgent::new(api_key.to_string())
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;
    
    let test_prompt = format!(
        "Generate comprehensive unit tests for the following Rust code. Include:\n\
        - Test for normal cases\n\
        - Test for edge cases\n\
        - Test for error conditions\n\
        - Mock external dependencies if needed\n\
        \nSource code:\n{}",
        code
    );
    
    ui::update_progress(&pb, "Generating comprehensive tests...");
    let tests = agent.generate_code(&test_prompt)
        .await
        .map_err(|e| CliError::api_error(&format!("Failed to generate tests: {}", e)))?;
    
    pb.finish_with_message("Test generation complete");
    
    // Create test file path
    let test_file = if let Some(parent) = file_path.parent() {
        let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
        parent.join("tests").join(format!("{}_test.rs", file_stem))
    } else {
        PathBuf::from(format!("test_{}", file_path.file_name().unwrap().to_str().unwrap()))
    };
    
    ui::print_separator();
    ui::print_info(&format!("{} Saving tests to: {}", ui::GEAR, test_file.display()));
    
    // Create tests directory if it doesn't exist
    if let Some(parent) = test_file.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| CliError::file_error("Failed to create tests directory", e))?;
    }
    
    tokio::fs::write(&test_file, &tests)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to save test file: {}", test_file.display()), e))?;
        
    ui::print_success("Tests generated and saved successfully!");
    ui::print_code_block("Generated Tests", &tests);
    
    Ok(())
}

pub async fn handle_generate_docs(
    file_path: &PathBuf,
    config: &Config,
) -> Result<()> {
    ui::print_command_start("GENERATE-DOCS", &format!("{} Generating documentation for {}", ui::BOOK, file_path.display()));
    
    let pb = ui::print_progress("Reading source file...");
    let code = agent::read_file_content(file_path)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to read file: {}", file_path.display()), std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    ui::update_progress(&pb, "Analyzing code structure...");
    
    let api_key = config.get_api_key()
        .map_err(|e| CliError::config_error(&format!("API key not configured: {}", e)))?;
    let agent = agent::AIAgent::new(api_key.to_string())
        .map_err(|e| CliError::api_error(&format!("Failed to initialize AI agent: {}", e)))?;
    
    let docs_prompt = format!(
        "Generate comprehensive Rust documentation comments (///) for the following code. Include:\n\
        - Function descriptions\n\
        - Parameter descriptions\n\
        - Return value descriptions\n\
        - Example usage\n\
        - Error conditions\n\
        \nProvide the complete code with documentation comments added:\n{}",
        code
    );
    
    ui::update_progress(&pb, "Generating documentation...");
    let documented_code = agent.generate_code(&docs_prompt)
        .await
        .map_err(|e| CliError::api_error(&format!("Failed to generate documentation: {}", e)))?;
    
    pb.finish_with_message("Documentation generation complete");
    
    ui::print_separator();
    ui::print_info(&format!("{} Updating file with documentation: {}", ui::GEAR, file_path.display()));
    
    tokio::fs::write(file_path, &documented_code)
        .await
        .map_err(|e| CliError::file_error(&format!("Failed to update file: {}", file_path.display()), e))?;
        
    ui::print_success("Documentation added successfully!");
    ui::print_code_block("Documented Code", &documented_code);
    
    Ok(())
}
