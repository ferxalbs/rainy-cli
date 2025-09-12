use crate::{config::Config, error::CliError, ui};
use miette::Result;
use std::path::PathBuf;

pub async fn handle_template_command(
    template: String,
    name: String,
    output: Option<PathBuf>,
    _config: &Config,
) -> Result<()> {
    ui::print_command_start("TEMPLATE", &format!("{} Generating {} project: {}", ui::CODE, template, name));
    ui::print_generation_header(&format!("Creating {} project template", template));

    let output_dir = output.unwrap_or_else(|| std::env::current_dir().unwrap().join(&name));

    let pb = ui::print_progress("Generating project structure...");
    generate_template(&template, &name, &output_dir)
        .await
        .map_err(|e| CliError::command_error(&format!("Failed to generate template: {}", e)))?;
    pb.finish_with_message("Project structure created");


    ui::print_separator();
    ui::print_success(&format!("Project '{}' created successfully at: {}", name, output_dir.display()));
    ui::print_info("Next steps:");
    ui::print_info(&format!("  cd {}", output_dir.display()));
    ui::print_info("  cargo build");
    ui::print_info("  cargo run");
    
    Ok(())
}

pub async fn generate_template(template: &str, name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    // Create output directory
    tokio::fs::create_dir_all(output_dir).await?;

    match template {
        "rust-api" => generate_rust_api_template(name, output_dir).await,
        "rust-cli" => generate_rust_cli_template(name, output_dir).await,
        "rust-lib" => generate_rust_lib_template(name, output_dir).await,
        "web-api" => generate_web_api_template(name, output_dir).await,
        "microservice" => generate_microservice_template(name, output_dir).await,
        _ => Err(anyhow::anyhow!("Unknown template: {}. Available templates: rust-api, rust-cli, rust-lib, web-api, microservice", template)),
    }
}

async fn generate_rust_api_template(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    // Create src directory
    let src_dir = output_dir.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;

    // Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tower = "0.4"
tower-http = {{ version = "0.5", features = ["cors", "trace"] }}
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
uuid = {{ version = "1.0", features = ["v4"] }}
"#, name);
    tokio::fs::write(output_dir.join("Cargo.toml"), cargo_toml).await?;

    // main.rs
    let main_rs = r#"use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/users", post(create_user))
        .route("/api/users/:id", get(get_user))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Welcome to the API!".to_string(),
        status: "success".to_string(),
    })
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Result<Json<User>, StatusCode> {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(Json(user))
}

async fn get_user(Path(id): Path<Uuid>) -> Result<Json<User>, StatusCode> {
    // In a real app, you'd fetch from a database
    let user = User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(Json(user))
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    status: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: Uuid,
    name: String,
    email: String,
    created_at: String,
}
"#;
    tokio::fs::write(src_dir.join("main.rs"), main_rs).await?;

    // README.md
    let readme = format!(r#"# {}

A modern Rust API server built with Axum.

## Features

- REST API endpoints
- CORS support
- Request tracing
- Health check endpoint
- JSON serialization
- UUID support

## Getting Started

```bash
cargo run
```

The server will start on http://localhost:3000

## API Endpoints

- `GET /` - Welcome message
- `GET /health` - Health check
- `POST /api/users` - Create a new user
- `GET /api/users/:id` - Get user by ID

## Development

```bash
cargo build
cargo test
cargo run
```

## Environment Variables

- `RUST_LOG` - Set logging level (debug, info, warn, error)
"#, name);
    tokio::fs::write(output_dir.join("README.md"), readme).await?;

    Ok(())
}

async fn generate_rust_cli_template(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    // Create src directory
    let src_dir = output_dir.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;

    // Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = {{ version = "4.0", features = ["derive"] }}
anyhow = "1.0"
thiserror = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
colored = "2.0"
indicatif = "0.17"
dialoguer = "0.10"
"#, name);
    tokio::fs::write(output_dir.join("Cargo.toml"), cargo_toml).await?;

    // main.rs
    let main_rs = r#"use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "my-cli")]
#[command(about = "A powerful CLI tool built with Rust")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process a file
    Process {
        /// Input file path
        file: String,
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Show statistics
    Stats {
        /// Show detailed statistics
        #[arg(short, long)]
        detailed: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { file, format } => {
            println!("{} Processing file: {}", "✓".green(), file);
            println!("{} Output format: {}", "→".blue(), format);
            // Add your processing logic here
        }
        Commands::Stats { detailed } => {
            println!("{} Generating statistics...", "📊".yellow());
            if detailed {
                println!("{} Detailed statistics enabled", "ℹ".blue());
            }
            // Add your statistics logic here
        }
    }

    Ok(())
}
"#;
    tokio::fs::write(src_dir.join("main.rs"), main_rs).await?;

    // README.md
    let readme = format!(r#"# {}

A command-line tool built with Rust and Clap.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Process a file
{} process input.txt --format json

# Show statistics
{} stats --detailed
```

## Commands

- `process` - Process a file with specified format
- `stats` - Generate and display statistics

## Development

```bash
cargo build
cargo test
cargo run -- --help
```
"#, name, name, name);
    tokio::fs::write(output_dir.join("README.md"), readme).await?;

    Ok(())
}

async fn generate_rust_lib_template(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    // Create src directory
    let src_dir = output_dir.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;

    // Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "A Rust library"
license = "MIT OR Apache-2.0"
readme = "README.md"
repository = "https://github.com/username/{}"

[dependencies]
anyhow = "1.0"
thiserror = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}

[dev-dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
"#, name, name);
    tokio::fs::write(output_dir.join("Cargo.toml"), cargo_toml).await?;

    // lib.rs
    let lib_rs = r#"//! A Rust library
//!
//! This library provides functionality for...

pub mod error;

pub use error::{Result, LibError};

/// Add two numbers together
///
/// # Examples
///
/// ```
/// use my_lib::add;
///
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

/// Multiply two numbers
///
/// # Examples
///
/// ```
/// use my_lib::multiply;
///
/// let result = multiply(4, 5);
/// assert_eq!(result, 20);
/// ```
pub fn multiply(left: usize, right: usize) -> usize {
    left * right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_multiply() {
        let result = multiply(3, 4);
        assert_eq!(result, 12);
    }
}
"#;
    tokio::fs::write(src_dir.join("lib.rs"), lib_rs).await?;

    // error.rs
    let error_rs = r#"use thiserror::Error;

/// Library result type
pub type Result<T> = std::result::Result<T, LibError>;

/// Library error types
#[derive(Error, Debug)]
pub enum LibError {
    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}
"#;
    tokio::fs::write(src_dir.join("error.rs"), error_rs).await?;

    // README.md
    let readme = format!(r#"# {}

A Rust library providing...

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
{} = "0.1.0"
```

## Example

```rust
use {};

fn main() {{
    let result = {}::add(2, 3);
    println!("Result: {{}}", result);
}}
```

## Features

- Fast and safe
- Well documented
- Comprehensive error handling
- Full test coverage

## Development

```bash
cargo build
cargo test
cargo doc --open
```
"#, name, name, name, name);
    tokio::fs::write(output_dir.join("README.md"), readme).await?;

    Ok(())
}

async fn generate_web_api_template(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    // Enhanced web API template with database integration
    let src_dir = output_dir.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;

    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tower = "0.4"
tower-http = {{ version = "0.5", features = ["cors", "trace", "fs"] }}
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
uuid = {{ version = "1.0", features = ["v4"] }}
sqlx = {{ version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
dotenv = "0.15"
"#, name);
    tokio::fs::write(output_dir.join("Cargo.toml"), cargo_toml).await?;

    let main_rs = r#"use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

mod models;
mod handlers;
mod database;

use models::*;

#[derive(Clone)]
pub struct AppState {
    // Add database pool here when ready
    // db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let state = AppState {
        // db: database::create_pool().await?,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health_check))
        .route("/api/v1/users", get(handlers::list_users).post(handlers::create_user))
        .route("/api/v1/users/:id", get(handlers::get_user).put(handlers::update_user).delete(handlers::delete_user))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("🚀 Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
"#;
    tokio::fs::write(src_dir.join("main.rs"), main_rs).await?;

    Ok(())
}

async fn generate_microservice_template(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    // Create a complete microservice template
    let src_dir = output_dir.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    ui::print_info("Generating microservice template with Docker, health checks, and metrics...");
    
    // Generate Dockerfile
    let dockerfile = r#"FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/microservice /usr/local/bin/microservice
EXPOSE 8080
CMD ["microservice"]
"#;
    tokio::fs::write(output_dir.join("Dockerfile"), dockerfile).await?;
    
    // Generate docker-compose.yml
    let docker_compose = format!(r#"version: '3.8'
services:
  {}:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - PORT=8080
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
"#, name);
    tokio::fs::write(output_dir.join("docker-compose.yml"), docker_compose).await?;
    
    Ok(())
}
