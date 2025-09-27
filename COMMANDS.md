# 🌧️ Rainy CLI Commands Guide 📖

Welcome to the complete guide for all `rainy-cli` commands. This document will help you master every feature of your AI-powered coding assistant.

## 🌍 Global Options

These options can be used with any command.

- `-v, --verbose`: Enables detailed, verbose output for debugging purposes.
- `-m, --model <MODEL>`: Overrides the default AI model for a single command.
- `--json`: Outputs the raw JSON response from the AI model, useful for scripting and integration.

---

## 💬 `chat`

Starts an interactive, agentic chat session. The AI can use tools to answer questions, modify files, and help you with your coding tasks.

**Usage:**

```sh
rainy-cli chat [MESSAGE] [--context_files <PATHS>...] [--no-history]
```

**Arguments:**

- `MESSAGE` (Optional): A message to start the conversation with. If omitted, you'll be prompted to enter a message.
- `--context_files <PATHS>...` (Optional): A list of file paths to load as context for the conversation. The AI will be aware of the contents of these files.
- `--no-history` (Optional): Starts the chat with a clean slate, ignoring previous conversation history.

**Examples:**

- **Start a new chat session:**

  ```sh
  rainy-cli chat
  ```

- **Ask a question with file context:**

  ```sh
  rainy-cli chat "Can you explain the main function in this file?" --context_files ./src/main.rs
  ```

- **Ask the agent to perform a task:**

  ```sh
  rainy-cli chat "Please add a new function to `lib.rs` that calculates the factorial of a number."
  ```

---

## 💾 `session`

Manages persistent chat sessions, allowing you to save, load, and organize your conversations.

**Usage:**

```sh
rainy-cli session <ACTION>
```

**Actions:**

- `create <NAME> [--description <DESC>]`: 📝 Creates a new, named session.
- `list`: 📋 Lists all saved sessions.
- `show <ID>`: ℹ️ Displays detailed information about a specific session.
- `chat <ID> [MESSAGE]`: 💬 Resumes a chat within a specific session.
- `rename <ID> <NEW_NAME>`: ✏️ Renames a session.
- `delete <ID>`: 🗑️ Deletes a session permanently.

**Examples:**

- **Create a new session for a feature you're working on:**

  ```sh
  rainy-cli session create "Feature: User Authentication" --description "Working on the login and registration flow."
  ```

- **List all your sessions:**

  ```sh
  rainy-cli session list
  ```

- **Resume a previous conversation:**

  ```sh
  rainy-cli session chat session_1678886400
  ```

---

## 🔍 `analyze`

Performs a static analysis of your code, using AI to identify potential issues.

**Usage:**

```sh
rainy-cli analyze --paths <PATHS>... [--analysis_type <TYPE>]
```

**Arguments:**

- `--paths <PATHS>...`: The file(s) or director(y/ies) to analyze.
- `--analysis_type <TYPE>`: The focus of the analysis. Options:
  - `general` (Default): A broad overview.
  - `security`: Checks for potential security vulnerabilities.
  - `performance`: Looks for performance bottlenecks.
  - `style`: Checks for style inconsistencies.
  - `complexity`: Measures code complexity.

**Example:**

- **Run a security audit on your entire source directory:**

  ```sh
  rainy-cli analyze --paths ./src --analysis_type security
  ```

---

## 👀 `review`

Provides an AI-powered code review. It can review specific files, directories, or even your Git changes.

**Usage:**

```sh
rainy-cli review --paths <PATHS>... [--focus <FOCUS>] [--git] [--git_ref <REF>]
```

**Arguments:**

- `--paths <PATHS>...`: The file(s) or director(y/ies) to review.
- `--focus <FOCUS>` (Optional): A specific area to focus on (e.g., `readability`, `error-handling`).
- `--git` (Optional): Reviews only the changes in your Git working directory (staged and unstaged files).
- `--git_ref <REF>` (Optional): Reviews changes against a specific Git reference (e.g., a branch or commit hash).

**Example:**

- **Review the changes you've staged for the next commit:**

  ```sh
  rainy-cli review --git
  ```

---

## ✍️ `generate`

Generates new code from a natural language description.

**Usage:**

```sh
rainy-cli generate <DESCRIPTION> [--output <PATH>]
```

**Arguments:**

- `<DESCRIPTION>`: A detailed description of the code you want to generate.
- `--output <PATH>` (Optional): A file path to save the generated code to. If omitted, the code will be printed to the console.

**Example:**

- **Generate a Rust function and save it to a file:**

  ```sh
  rainy-cli generate "a Rust function that reads a file and returns its content as a string" --output ./src/file_utils.rs
  ```

---

## 🏗️ `template`

Scaffolds a new project from a pre-built template.

**Usage:**

```sh
rainy-cli template <TEMPLATE> <NAME> [--output <PATH>]
```

**Arguments:**

- `<TEMPLATE>`: The template to use. Available templates: `rust-api`, `rust-cli`, `rust-lib`, `web-api`, `microservice`.
- `<NAME>`: The name for your new project.
- `--output <PATH>` (Optional): The directory to create the project in. Defaults to the current directory.

**Example:**

- **Create a new Rust CLI project:**

  ```sh
  rainy-cli template rust-cli my-awesome-tool
  ```

---

## 🤖 `agent`

Provides direct access to the agent's file system tools. This is useful for scripting and direct manipulation of files.

**Usage:**

```sh
rainy-cli agent <SUBCOMMAND>
```

**Subcommands:**

- `init`: Creates a new `AGENTS.md` file to configure the agent for the current project.
- `read-file <PATH>`: Reads and prints the content of a file.
- `write-file <PATH> <CONTENT>`: Writes content to a file, overwriting it if it exists.
- `patch-file <PATH> <INSTRUCTIONS>`: Applies a patch to a file based on instructions.
- `delete-file <PATH>`: Deletes a file.
- `list-files <PATH>`: Lists all files and subdirectories in a given path.
- `grep <PATTERN> [PATH]`: Searches for a pattern within files.

**Example:**

- **List all files in the `src` directory:**

  ```sh
  rainy-cli agent list-files ./src
  ```

---

## 🔌 `mcp`

Manages connections to MCP (Multi-Agent Communication Protocol) tool servers, allowing you to extend the agent's capabilities.

**Usage:**

```sh
rainy-cli mcp <SUBCOMMAND>
```

**Subcommands:**

- `add <NAME> <COMMAND> [ARGS]...`: Registers a new MCP server.
- `remove <NAME>`: Removes a registered MCP server.
- `list`: Lists all configured MCP servers.
- `list-tools [NAME]`: Lists the tools provided by one or all servers.
- `call-tool <SERVER> <TOOL> [--args <JSON>]`: Executes a specific tool from a server.

**Example:**

- **List all available tools from all connected servers:**

  ```sh
  rainy-cli mcp list-tools
  ```

---

## ⚙️ `config`

Manages your global CLI configuration.

**Usage:**

```sh
rainy-cli config <SUBCOMMAND>
```

**Subcommands:**

- `--show`: Displays the current configuration.
- `--set-api-key <KEY>`: Sets and saves your API key.
- `--set-model <MODEL>`: Sets your preferred default AI model.
- `--reset`: Resets your configuration to the default settings.

**Example:**

- **Set your default model to a different one:**

  ```sh
  rainy-cli config --set-model gemini-2.5-pro-lite
  ```
