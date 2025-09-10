use miette::Result;
use crate::{config::Config, error::CliError, ui, utils};
use std::fs;
use std::path::Path;

pub async fn handle_codebase_command(update: bool, config: &Config) -> Result<()> {
    ui::print_command_start("CODEBASE", &format!("{} Managing codebase context (rainy.md)", ui::BOOK));

    let rainy_md_path = Path::new("rainy.md");

    if update && rainy_md_path.exists() {
        ui::print_info(&format!("--update flag detected. Removing existing `{}` to regenerate.", rainy_md_path.display()));
        fs::remove_file(rainy_md_path)
            .map_err(|e| crate::error::CliError::file_error("Failed to remove rainy.md", e))?;
    }

    if !rainy_md_path.exists() {
        utils::rainy_md::ensure_rainy_md_exists(config)
            .await
            .map_err(|e| CliError::context_error("Failed to ensure rainy.md exists", e))?;
        ui::print_success("Codebase context (rainy.md) has been successfully generated/updated.");
    } else {
        ui::print_info(&format!("`{}` already exists. Use the --update flag to regenerate it.", rainy_md_path.display()));
    }

    Ok(())
}
