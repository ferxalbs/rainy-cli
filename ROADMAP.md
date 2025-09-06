# 🗺️ Rainy CLI Roadmap

This document outlines the development roadmap for the Rainy CLI, an AI-powered code assistant.

## Phase 1: Core Agentic Capabilities (In Progress)

This is the initial implementation phase focused on building a robust, interactive agent that can understand and modify a codebase with user supervision.

- [x] **Core Agentic Loop:** Implement the main `chat -> plan -> confirm -> execute` workflow.
- [x] **Chat as Primary Interface:** Enhance the `chat` command to be the main entry point for agentic work.
- [x] **`rainy.md` Integration:** Allow users to define agent behavior and project context via a `rainy.md` file. The CLI will auto-generate this file if it's missing.
- [x] **Agent Safety:** Ensure all file system modifications require explicit user approval.
- [x] **File System Tools:** Equip the agent with essential tools to read, write, patch, and delete files.
- [ ] **Large Context Strategy:** Implement a strategy to load relevant files into the model's context window.
- [ ] **Patch-based Edits:** The agent will generate `diff`-like patches for modifications to handle large files and provide clear, reviewable changes.

## Phase 2: Enhanced Tooling & Intelligence

This phase will focus on expanding the agent's capabilities and making it more autonomous.

- [ ] **Test Execution:** Grant the agent the ability to run project tests (e.g., `cargo test`) to verify its changes.
- [ ] **Git Integration:** Allow the agent to stage changes and write commit messages.
- [ ] **Web Search:** Give the agent the ability to search the web for information, such as documentation for a library it's unfamiliar with.
- [ ] **Advanced Code Analysis:** Improve the agent's ability to understand complex codebases by building an internal representation of the code (e.g., an AST).
- [ ] **Multi-file Context:** Enhance the context strategy to handle changes that span multiple files more effectively.

## Phase 3: Usability & Polish

This phase will focus on improving the user experience and making the CLI more robust.

- [ ] **Configuration Enhancements:** Allow for more granular configuration of the agent's behavior.
- [ ] **Improved UI:** Enhance the terminal UI to make the agent's plans and actions even clearer.
- [ ] **Performance Optimization:** Profile and optimize the CLI for speed and resource usage.
- [ ] **Extensibility:** Introduce a plugin system for adding custom tools to the agent.
- [ ] **Shell Integration:** Provide shell completion scripts and other integrations to make the CLI easier to use.
