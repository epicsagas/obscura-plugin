# obscura-plugin


<p align="center">
  <a href="https://github.com/epicsagas/obscura-plugin/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/epicsagas/obscura-plugin?style=for-the-badge&labelColor=0d1117&color=ffd700&logo=github&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/obscura-plugin/network/members"><img alt="Forks" src="https://img.shields.io/github/forks/epicsagas/obscura-plugin?style=for-the-badge&labelColor=0d1117&color=2ecc71&logo=github&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/obscura-plugin/issues"><img alt="Issues" src="https://img.shields.io/github/issues/epicsagas/obscura-plugin?style=for-the-badge&labelColor=0d1117&color=ff6b6b&logo=github&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/obscura-plugin/commits/main"><img alt="Last commit" src="https://img.shields.io/github/last-commit/epicsagas/obscura-plugin?style=for-the-badge&labelColor=0d1117&color=58a6ff&logo=git&logoColor=white" /></a>
</p>
<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-3fb950?style=for-the-badge&labelColor=0d1117" /></a>
  <img alt="Rust" src="https://img.shields.io/badge/rust-d73a49?style=for-the-badge&labelColor=0d1117&logo=rust&logoColor=white" />
  <a href="https://buymeacoffee.com/epicsaga"><img alt="Buy Me a Coffee" src="https://img.shields.io/badge/buy_me_a_coffee-FFDD00?style=for-the-badge&labelColor=0d1117&logo=buymeacoffee&logoColor=black" /></a>
</p>

MCP server for [Obscura](https://github.com/h4ckf0r0day/obscura) headless browser — CLI wrapper that gives AI coding agents direct access to web scraping and browser automation.

> **Two MCP approaches in Obscura:**
> - `obscura mcp` (built-in) — in-process browser control with interactive `browser_*` tools (click, fill, navigate)
> - `obscura-plugin` (this plugin) — CLI wrapper with high-level read-only tools (`obscura_fetch`, `obscura_scrape`, etc.), designed for autonomous agent pipelines
>
> Use the built-in `obscura mcp` when you need to click, fill forms, or maintain browser state. Use this plugin for read-only scraping and batch data collection.

## Install

### Claude Code (zero-touch)

`obscura-plugin` ships a `.claude-plugin/` manifest. Register the marketplace source once, then install:

```bash
claude plugin marketplace add epicsagas/plugins
claude plugin install obscura
```

The `SessionStart` hook downloads `obscura-plugin`, `obscura`, and `obscura-worker` automatically on first load. No manual steps needed.

### Codex CLI

```bash
codex plugin marketplace add epicsagas/obscura-plugin
```

Skills and agents are available immediately — no further steps needed.

---

> **Prerequisites** (for manual install only — plugin seeding handles these automatically)
> - `obscura` binary — see [Obscura releases](https://github.com/h4ckf0r0day/obscura/releases)
> - `obscura-worker` binary — **required for `obscura_scrape` parallel mode**. Included in the same release archive as `obscura`. Keep both binaries in the same directory.
> - Linux: glibc 2.35+ (Ubuntu 22.04+)

### macOS / Linux

```bash
# pre-built binary, no Rust required
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/obscura-plugin/releases/latest/download/obscura-plugin-installer.sh | sh
```

### Windows

```powershell
# pre-built binary, no Rust required
irm https://github.com/epicsagas/obscura-plugin/releases/latest/download/obscura-plugin-installer.ps1 | iex
```

### Homebrew (macOS / Linux)

```bash
brew install epicsagas/tap/obscura-plugin
```

### Via Rust toolchain

```bash
# pre-built binary via Rust toolchain (fast)
cargo binstall obscura-plugin

# build from source (requires Rust toolchain)
cargo install obscura-plugin
```

> `obscura-plugin --version` to verify. Update with `brew upgrade obscura-plugin` or re-run the installer script.

### Register with your AI tools

After installing the binary, run `obscura-plugin install` to connect it to your tools:

```bash
obscura-plugin install              # interactive — pick which tools
obscura-plugin install cursor       # Cursor
obscura-plugin install gemini       # Gemini CLI
obscura-plugin install opencode     # OpenCode
obscura-plugin install cline        # Cline
obscura-plugin install all          # all at once
```

```bash
obscura-plugin uninstall cursor     # remove from a specific tool
obscura-plugin list                 # show supported tools and status
```

### What gets installed

| Tool | MCP | Skills | Agent | Method |
|------|:---:|:------:|:-----:|--------|
| Claude Code | yes | `~/.claude/skills/` | `~/.claude/agents/` | plugin (auto) |
| Codex CLI | yes | `~/.codex/skills/` | `~/.codex/agents/` | plugin (auto) |
| Cursor | yes | `~/.cursor/rules/` | `~/.cursor/agents/` | `obscura-plugin install` |
| Gemini CLI | yes | `~/.gemini/skills/` | `~/.gemini/agents/` | `obscura-plugin install` |
| OpenCode | yes | `~/.opencode/skills/` | `~/.config/opencode/agents/` | `obscura-plugin install` |
| Cline | yes | `~/.cline/skills/` | — | `obscura-plugin install` |

## MCP tools

| Tool | Description |
|------|-------------|
| `obscura_fetch` | Fetch a URL — returns HTML, text, links, markdown, or JS eval result |
| `obscura_scrape` | Parallel scrape multiple URLs with configurable concurrency |
| `obscura_serve` | Start a CDP WebSocket server for Puppeteer / Playwright |
| `obscura_screenshot` | Fetch a page and evaluate a JS expression (alias for fetch + eval) |
| `obscura_extract_markdown` | Fetch a URL and return clean plain text (via `document.body.innerText`) |

### Tool parameters

**`obscura_fetch`**

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | URL to fetch *(required)* |
| `dump` | string | `html` · `text` · `links` · `markdown` (default: `html`) |
| `eval` | string | JavaScript expression to evaluate |
| `wait_until` | string | `load` · `domcontentloaded` · `networkidle0` (default: `load`) |
| `selector` | string | CSS selector to wait for before returning |
| `stealth` | boolean | Enable anti-detection + tracker blocking¹ |
| `user_agent` | string | Custom User-Agent string |
| `quiet` | boolean | Suppress banner (default: `true`) |

**`obscura_scrape`**

| Parameter | Type | Description |
|-----------|------|-------------|
| `urls` | string[] | URLs to scrape *(required)* |
| `eval` | string | JavaScript expression applied to each page |
| `concurrency` | number | Parallel workers (default: `10`) |
| `format` | string | `json` · `text` (default: `json`) |
| `proxy` | string | HTTP/SOCKS5 proxy URL for all workers |

**`obscura_serve`**

| Parameter | Type | Description |
|-----------|------|-------------|
| `port` | number | WebSocket port (default: `9222`) |
| `stealth` | boolean | Anti-detection + tracker blocking¹ |
| `proxy` | string | HTTP/SOCKS5 proxy URL |
| `workers` | number | Parallel worker processes (default: `1`) |

Returns `wsEndpoint` (`ws://127.0.0.1:{port}/devtools/browser`) for Puppeteer/Playwright connection.

**`obscura_screenshot`** — convenience wrapper around `fetch --eval`

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | URL to fetch *(required)* |
| `expression` | string | JavaScript expression *(required)* |
| `wait_until` | string | `load` · `domcontentloaded` · `networkidle0` (default: `networkidle0`) |
| `stealth` | boolean | Anti-detection mode¹ |

**`obscura_extract_markdown`** — fetches page and returns `document.body.innerText`

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | URL to fetch *(required)* |
| `stealth` | boolean | Anti-detection mode¹ |
| `selector` | string | Scope extraction to a CSS selector |

> ¹ **Stealth mode** requires Obscura built with `--features stealth`. The default release binary includes stealth. If you built from source without the flag, `--stealth` has no effect — rebuild with `cargo build --release --features stealth`.

## Skills

Registered skills are available as slash commands inside your agent:

```
/obscura-fetch <url> [--dump html|text|links|markdown] [--eval <js>] [--selector <css>] [--stealth]
/obscura-scrape <url1> <url2> ... [--eval <js>] [--concurrency <N>] [--format json|text]
/obscura-pipeline <index-url>   # discover links → scrape in one pipeline
```

> Skills are seeded into `~/.claude/skills/` by `obscura-plugin install claude` and are available as `/obscura-fetch`, `/obscura-scrape`, `/obscura-pipeline` slash commands in Claude Code.

## The `obscura-browser` agent

A self-directed web data collection specialist. Invoke it for tasks like:

> "Collect all product titles and prices from these 30 URLs"
> "Fetch the docs at example.com/api and summarize the endpoints"
> "Scrape the HN front page and return structured JSON"

The agent knows Obscura's limits — it stops and escalates to Playwright when login or interaction is required.

## Plugin seeding details

When loaded, the `SessionStart` hook (`hooks/install.js`) runs automatically and:

1. Downloads `obscura-plugin` from GitHub Releases into `~/.local/bin/`
2. Downloads `obscura` + `obscura-worker` from the Obscura upstream release
3. Registers the MCP server, skills, and `obscura-browser` agent

Verify after the first session start:

```bash
obscura-plugin list   # shows Claude Code: installed
```

### What gets seeded

| Asset | Destination |
|-------|-------------|
| MCP server | `~/.claude/claude_desktop_config.json` (or `.mcp.json`) |
| Skills | `~/.claude/skills/` — `obscura-fetch`, `obscura-scrape`, `obscura-pipeline` |
| Agent | `~/.claude/agents/obscura-browser.md` |

## Build from source

```bash
cargo build --release
./target/release/obscura-plugin --help
```

## License

Apache 2.0
