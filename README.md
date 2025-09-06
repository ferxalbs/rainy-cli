# 🚀 Rainy CLI - Professional AI Code Assistant

A premium AI-powered code assistant built with Rust that helps developers analyze, generate, review, and chat about code. Inspired by professional tools like [Cursor CLI](https://cursor.com/cli) and designed for maximum productivity.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## ✨ Features

### 🔍 **Code Analysis**
- **Security Analysis**: Detect vulnerabilities, injection flaws, and security weaknesses
- **Performance Analysis**: Identify bottlenecks and optimization opportunities
- **Style Analysis**: Check code style and maintainability
- **Complexity Analysis**: Calculate complexity metrics and suggest refactoring
- **General Analysis**: Comprehensive code review and quality assessment

### 💻 **Code Generation**
- **Natural Language to Code**: Generate code from descriptions
- **Unit Test Generation**: Automatically create comprehensive test suites
- **Documentation Generation**: Add inline documentation and comments
- **Project Templates**: Bootstrap new projects with best practices

### 📝 **Code Review**
- **File Review**: Detailed analysis of individual files
- **Git-Aware Review**: Review only changed files in commits
- **Interactive Suggestions**: Apply improvements with guided assistance
- **Quality Scoring**: Get overall quality grades (A+ to C)

### 💬 **Interactive AI Chat**
- **Context-Aware**: Understands your current project structure
- **Inline Commands**: `/analyze`, `/review` commands within chat
- **Conversation History**: Persistent chat sessions
- **Project Context**: Automatically loads project information

### 🏗️ **Project Templates**
- **rust-api**: Modern REST API with Axum, database integration
- **rust-cli**: Feature-rich CLI with clap and colored output
- **rust-lib**: Library template with comprehensive documentation
- **web-api**: Full-stack web API with CORS and middleware
- **microservice**: Docker-ready microservice with health checks

### ⚙️ **Configuration Management**
- **API Key Storage**: Secure credential management
- **Model Selection**: Choose from multiple AI models
- **Custom Settings**: Temperature, max tokens, themes
- **Profile Support**: Multiple configuration profiles

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/your-org/rainy-cli
cd rainy-cli
cargo install --path .
```

### Setup

```bash
# First run - configure your API key
rainy-cli config --set-api-key "your-rainy-api-key"

# Show current configuration
rainy-cli config --show
```

### Basic Usage

```bash
# Analyze code for security issues
rainy-cli analyze --path src/main.rs --analysis-type security

# Generate a new Rust API project
rainy-cli template rust-api my-api-project

# Review Git changes
rainy-cli review --git --focus performance

# Start interactive chat
rainy-cli chat "How can I optimize this Rust code?"

# Generate tests for a file
rainy-cli tests --file src/utils.rs

# Add documentation to code
rainy-cli docs --file src/main.rs
```

## 📖 Command Reference

### Analysis Commands

```bash
# Security analysis
rainy-cli analyze -p src/ -a security

# Performance analysis with suggestions
rainy-cli analyze -p main.rs -a performance --apply

# Style and maintainability check
rainy-cli analyze -p . -a style
```

### Generation Commands

```bash
# Generate code from description
rainy-cli generate "Create a REST API endpoint for user authentication" -o auth.rs

# Generate with tests and docs
rainy-cli generate "Hash password function" --with-tests --with-docs

# Project templates
rainy-cli template rust-api my-service
rainy-cli template microservice payment-service
```

### Review Commands

```bash
# Review specific file
rainy-cli review -p src/main.rs -f readability

# Review Git changes
rainy-cli review --git --git-ref origin/main

# Review with focus area
rainy-cli review -p . -f security
```

### Chat Commands

```bash
# Start chat with initial message
rainy-cli chat "Explain this codebase architecture"

# Chat with specific file context
rainy-cli chat --context-file src/main.rs "How can I improve this?"

# Interactive commands within chat:
# /analyze src/main.rs
# /review src/utils.rs
# help
# clear
# exit
```

## 🏗️ Architecture

The CLI is designed with modularity and maintainability in mind:

```
src/
├── main.rs           # CLI entry point and routing
├── commands/         # Command implementations
│   ├── mod.rs
│   ├── analyze.rs    # Code analysis
│   ├── chat.rs       # Interactive chat
│   ├── generate.rs   # Code generation
│   ├── review.rs     # Code review
│   └── template.rs   # Project templates
├── utils/            # Utility modules
│   ├── mod.rs
│   ├── context.rs    # Project context loading
│   ├── git.rs        # Git integration
│   └── history.rs    # Chat history management
├── agent.rs          # AI agent and SDK integration
├── config.rs         # Configuration management
├── error.rs          # Error types and handling
└── ui.rs             # User interface and styling
```

## 🔧 Configuration

Configuration is stored in `~/.rainy-cli/config.toml`:

```toml
[config]
api_key = "ra-xxxxxxxxxxxx"
default_model = "moonshotai/kimi-k2-instruct"
theme = "dark"
max_tokens = 4096
temperature = 0.7
auto_save = true
verbose = false
```

### Available Models

- `moonshotai/kimi-k2-instruct` (default)
- `rainy-coder-1` (default v0.5.0)
- `rainy-coder-1-max`
- `anthropic/claude-sonnet-4`
- `openai/gpt-4-turbo`
- `google/gemini-2.5-pro`

### Configuration Commands

```bash
# Show current settings
rainy-cli config --show

# Set API key
rainy-cli config --set-api-key "your-key"

# Set default model
rainy-cli config --set-model "anthropic/claude-3-sonnet"

# Reset to defaults
rainy-cli config --reset
```

## 🎨 UI Features

- **Professional Design**: Clean, modern interface inspired by Cursor CLI
- **Color-Coded Output**: Syntax highlighting and semantic colors
- **Progress Indicators**: Dynamic spinners with status updates
- **Emoji Icons**: Clear visual indicators for different operations
- **Formatted Results**: Well-structured output with borders and sections
- **Error Reporting**: Detailed error messages with helpful suggestions

## 🔌 Integration

### Rainy SDK Integration

The CLI uses the official [Rainy SDK](https://docs.rs/rainy-sdk/latest/rainy_sdk/) for AI interactions:

- Automatic authentication with API keys
- Rate limiting and error handling
- Support for multiple AI models
- Streaming responses for real-time feedback

### Git Integration

- Detect changed files automatically
- Review only modified code
- Support for different Git references
- Status summaries and branch awareness

### Editor Integration

Works seamlessly with any text editor or IDE:
- Generate code and save to files
- Review existing codebases
- Apply suggestions interactively

## 🚀 Advanced Features

### Interactive Code Application

```bash
# Analyze and apply suggestions interactively
rainy-cli analyze -p src/ -a performance --apply

# Review with interactive improvements
rainy-cli review -p main.rs --interactive
```

### Batch Operations

```bash
# Analyze entire project
rainy-cli analyze -p . -a security

# Generate tests for all source files
find src -name "*.rs" -exec rainy-cli tests --file {} \;
```

### Continuous Integration

```bash
# In CI/CD pipelines
rainy-cli review --git --git-ref origin/main > review-report.md
```

## 🎯 Roadmap

- [ ] **Plugin System**: Extensible architecture for custom commands
- [ ] **IDE Extensions**: VSCode, Neovim, and other editor integrations
- [ ] **Team Features**: Shared configurations and review workflows
- [ ] **Performance Monitoring**: Track code quality over time
- [ ] **Custom Models**: Support for fine-tuned and local models
- [ ] **Web Interface**: Browser-based dashboard for project overview

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/your-org/rainy-cli
cd rainy-cli
cargo build
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Run lints
cargo clippy

# Check for issues
cargo audit
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [Cursor CLI](https://cursor.com/cli)
- Built with [Rainy SDK](https://docs.rs/rainy-sdk/latest/rainy_sdk/)
- Powered by Enosis Labs AI models

---

**Built with ❤️ for developers, by developers.**

For more information, visit [docs.rainy-cli.dev](https://docs.rainy-cli.dev)