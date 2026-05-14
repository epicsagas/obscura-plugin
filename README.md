# obscura-plugin

MCP server for [Obscura](https://github.com/h4ckf0r0day/obscura) headless browser — CLI wrapper that gives AI coding agents direct access to web scraping and browser automation.

Unlike the built-in `obscura mcp` subcommand (in-process browser control with `browser_*` tools), this plugin uses a **CLI wrapper** approach: it shells out to the `obscura` binary, providing high-level tools like `obscura_fetch`, `obscura_scrape`, and `obscura_extract_markdown`.

## Install

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/obscura-plugin/releases/latest/download/obscura-mcp-installer.sh | sh
```

Or via Cargo:

```bash
cargo install obscura-plugin
```

> **Prerequisite**: The `obscura` binary must be installed separately. See [Obscura releases](https://github.com/h4ckf0r0day/obscura/releases).

### Register with your AI tools

After installing the binary, run `obscura-mcp install` to connect it to your tools:

```bash
obscura-mcp install              # interactive — pick which tools
obscura-mcp install claude       # Claude Code
obscura-mcp install cursor       # Cursor
obscura-mcp install gemini       # Gemini CLI
obscura-mcp install codex        # Codex CLI
obscura-mcp install opencode     # OpenCode
obscura-mcp install cline        # Cline
obscura-mcp install all          # all at once
```

```bash
obscura-mcp uninstall claude     # remove from a specific tool
obscura-mcp list                 # show supported tools and status
```

### What gets installed

| Tool | MCP | Skills | Agent |
|------|:---:|:------:|:-----:|
| Claude Code | yes | `~/.claude/skills/` | `~/.claude/agents/` |
| Cursor | yes | `~/.cursor/rules/` | `~/.cursor/agents/` |
| Gemini CLI | yes | `~/.gemini/skills/` | `~/.gemini/agents/` |
| Codex CLI | yes | `~/.codex/skills/` | `~/.codex/agents/` |
| OpenCode | yes | `~/.opencode/skills/` | `~/.config/opencode/agents/` |
| Cline | yes | `~/.cline/skills/` | — |

## MCP tools

| Tool | Description |
|------|-------------|
| `obscura_fetch` | Fetch a URL — returns HTML, text, links, or JS eval result |
| `obscura_scrape` | Parallel scrape multiple URLs with configurable concurrency |
| `obscura_serve` | Start a CDP server for Puppeteer / Playwright |
| `obscura_screenshot` | Fetch a page and evaluate a JS expression |
| `obscura_extract_markdown` | Convert a URL to clean markdown |

## Skills

Registered skills are available as slash commands inside your agent:

```
/obscura-fetch <url> [--dump text|html|links] [--eval <js>] [--stealth]
/obscura-scrape <url1> <url2> ... [--concurrency <N>] [--format json]
/obscura-pipeline <index-url>   # discover links → scrape in one pipeline
```

## The `obscura-browser` agent

A self-directed web data collection specialist. Invoke it for tasks like:

> "Collect all product titles and prices from these 30 URLs"
> "Fetch the docs at example.com/api and summarize the endpoints"
> "Scrape the HN front page and return structured JSON"

The agent knows Obscura's limits — it stops and escalates to Playwright when login or interaction is required.

## Build from source

```bash
cargo build --release
./target/release/obscura-mcp --help
```

## License

Apache 2.0
