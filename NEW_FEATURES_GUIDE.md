# 🚀 New Features Guide - Rainy AI CLI

Welcome to the new and improved Rainy AI CLI! This guide will walk you through all the exciting new features we've added to make your development workflow even more seamless and powerful.

## 💬 A More Conversational Experience

We've completely overhauled the user interface to be more interactive and intuitive.

### Cleaner, Dynamic Output

Gone are the days of raw JSON plans cluttering your terminal. The agent now communicates its intentions and results in a clear, conversational manner.

**Example:**

Instead of seeing a raw JSON plan, you'll now see a clear, numbered list of actions:
`Okay, I will do the following:
  1. Read the file \`src/main.rs\``

### See the Agent Think (Without the Mess)

The agent's thought process is now more transparent, without being messy. When the agent is processing your request, you'll see a status indicator:

`Rainy AI is working...`

When it's executing a specific tool, you'll see dynamic messages like:

`Reading file 'src/main.rs'...`

The underlying "thought" process from the AI model (like content inside `<thinking>` tags) is now hidden from your view, keeping the output clean and focused on the agent's actions and results.

## 📊 File Modification Summary

After the agent performs operations that modify your files (like `write_file` or `patch_file`), you'll now see a concise summary of the changes:

`File Modification Summary:
  - src/main.rs: +158 -85
  - src/lib.rs: +20 -5`

This summary shows the number of lines added (in green) and removed (in red) for each modified file, giving you a quick overview of the agent's work.

## 🔬 For the Power Users: The `--json` Flag

We understand that sometimes you need the raw, structured output. For the `analyze`, `generate`, and `review` commands, you can use the global `--json` flag to get the original JSON plan output.

**Example:**

`rainy-cli analyze --paths src/ --json`

This will output the raw JSON plan that the agent would execute, which can be useful for debugging or scripting.

## 🔌 Integrating with the World: Model Context Protocol (MCP)

This is the biggest new feature! Rainy CLI now supports the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), allowing it to connect to external tools and data sources.

### What is MCP?

MCP allows you to extend the agent's capabilities by connecting it to other services. For example, you could connect it to:
- A local file system server to give it access to your entire project.
- A database server to allow it to query your data.
- A weather API to get real-time weather information.
- And much more!

### Configuring MCP Servers

Rainy CLI can load MCP server configurations from two sources, making it incredibly flexible:

1.  **Claude for Desktop (`claude_desktop_config.json`):** If you already use Claude for Desktop, Rainy CLI will automatically detect and load your existing MCP server configurations. This makes it incredibly easy to get started if you're already set up with Claude.

2.  **Rainy CLI's own configuration (`rainy-mcp.json`):** You can create a `rainy-mcp.json` file to define your own MCP servers. This file can be placed in two locations:
    - **Project-specific:** In the root of your project, or in a `.rainy/` directory (`.rainy/rainy-mcp.json`).
    - **Global:** In your user's global configuration directory (`~/.config/rainy-cli/rainy-mcp.json` on Linux/macOS, `%APPDATA%\\rainy-cli\\rainy-mcp.json` on Windows).

The format of `rainy-mcp.json` is the same as Claude's:
```json
{
  "mcpServers": {
    "my-custom-server": {
      "command": "node",
      "args": ["/path/to/my/server.js"],
      "env": {
        "API_KEY": "my-secret-key"
      }
    }
  }
}
```

**Note:** Configurations in `rainy-mcp.json` will override configurations from `claude_desktop_config.json` if there are servers with the same name.

### New `mcp` Commands

We've added a new `mcp` subcommand to manage your MCP servers:

-   `rainy-cli mcp list`: Lists all available MCP servers from both Claude's and Rainy's configuration files.
-   `rainy-cli mcp add <server_name> <command> [args...]`: Adds a new MCP server to your global `rainy-mcp.json` file.
-   `rainy-cli mcp remove <server_name>`: Removes an MCP server from your global `rainy-mcp.json` file.
-   `rainy-cli mcp call <server_name> <tool_name> [--args '{"json": "args"}']`: Calls a specific tool from a connected MCP server.

### Permission System

For your security, Rainy CLI will ask for your permission before using a tool from a new MCP server for the first time.

`First time using MCP server 'my-custom-server'. Please grant permission.
Do you want to allow Rainy CLI to use tools from the MCP server 'my-custom-server'? [y/n] >`

If you grant permission, it will be saved in your project's `AGENTS.md` file, and you won't be asked again for that server in that project.
---
I need to escape the backticks in the markdown so they are rendered correctly.
I will use `\` to escape them.
For example: `\`src/main.rs\``. And for code blocks, I will use four backticks.

Let's try again.
```
