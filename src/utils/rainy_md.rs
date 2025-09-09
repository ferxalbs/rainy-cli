use crate::ui;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const RAINY_MD_FILENAME: &str = "rainy.md";
const RAINY_MD_TEMPLATE: &str = r#"# ☔️ Welcome to Rainy.md

This file is your personal instruction manual for the Rainy CLI agent. Use it to define project-specific guidelines, coding conventions, and important context. The agent will read this file before every task to better understand your requirements.

## 📝 Example: Web App Development

### Tech Stack
- Frontend: React with TypeScript
- UI Library: Tailwind CSS
- State Management: Redux Toolkit
- Testing: Jest and React Testing Library

### Coding Conventions
- Use functional components with hooks.
- Keep components small and focused on a single responsibility.
- All new features must have corresponding unit tests.
- Follow the existing code style for consistency.

### Important Context
- The `src/api` directory contains all the code for interacting with the backend.
- The user authentication flow is handled in `src/features/auth`.
- The main color palette is defined in `tailwind.config.js`.

## 🚀 How to Use
- **Be specific:** The more detailed your instructions, the better the agent will perform.
- **Use Markdown:** Structure your instructions with headings, lists, and code blocks for clarity.
- **Update as needed:** Keep this file up-to-date as your project evolves.
"#;

pub fn ensure_rainy_md_exists() -> Result<()> {
    let path = Path::new(RAINY_MD_FILENAME);
    if !path.exists() {
        fs::write(path, RAINY_MD_TEMPLATE)
            .context(format!("Failed to create {}", RAINY_MD_FILENAME))?;
        ui::print_info(&format!(
            "{} Created `{}` with a default template. You can edit this file to provide project-specific instructions to the agent.",
            ui::INFO,
            RAINY_MD_FILENAME
        ));
    }
    Ok(())
}

pub fn load_hierarchical_rainy_md() -> Result<String> {
    let mut combined_content = String::new();
    let current_dir = std::env::current_dir()?;
    let home_dir = dirs::home_dir().context("Could not find home directory")?;

    let mut path = current_dir.as_path();

    loop {
        let rainy_md_path = path.join(RAINY_MD_FILENAME);
        if rainy_md_path.exists() {
            let content = fs::read_to_string(&rainy_md_path)?;
            combined_content.push_str(&format!(
                "\n\n---\n\n# Context from {}\n\n{}",
                rainy_md_path.display(),
                content
            ));
        }

        if path == home_dir {
            break;
        }

        if let Some(parent) = path.parent() {
            path = parent;
        } else {
            break;
        }
    }

    Ok(combined_content)
}
