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

Skill-driven plugin for [Obscura](https://github.com/h4ckf0r0day/obscura) headless browser — gives AI coding agents direct access to web scraping and browser automation via slash commands.

> **Two approaches in Obscura:**
> - `obscura-plugin` (this plugin) — skill-driven with read-only commands (`obscura_fetch`, `obscura_scrape`, etc.), designed for autonomous agent pipelines

## Install

### Grok Build (xAI)

```bash
grok plugin install epicsagas/obscura-plugin --trust
```

Grok reads skills from `skills/` and agents from `agents/` at the plugin root. No extra configuration needed.

### Claude Code (zero-touch)

`obscura-plugin` ships a `.claude-plugin/` manifest. Register the standalone marketplace once, then install:

```bash
claude plugin marketplace add epicsagas/obscura-plugin
claude plugin install obscura-plugin@obscura-plugin
```

The `SessionStart` hook downloads `obscura` and `obscura-worker` automatically on first load. No manual steps needed.

### Codex CLI

```bash
codex plugin marketplace add epicsagas/obscura-plugin
codex plugin add obscura-plugin@obscura-plugin
```

Skills and agents are available immediately — no further steps needed.

### Antigravity

```bash
agy plugin install https://github.com/epicsagas/obscura-plugin
```

---

> `obscura-plugin install` automatically downloads the `obscura` and `obscura-worker` binaries from [Obscura releases](https://github.com/h4ckf0r0day/obscura/releases). Linux requires glibc 2.35+ (Ubuntu 22.04+).

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
obscura-plugin install opencode     # OpenCode
obscura-plugin install cline        # Cline
obscura-plugin install all          # all at once
```

```bash
obscura-plugin uninstall cursor     # remove from a specific tool
obscura-plugin list                 # show supported tools and status
```

### What gets installed

| Tool | Skills | Method |
|------|--------|--------|
| Claude Code | plugin auto-discovers | `.claude-plugin/` manifest |
| Codex CLI | plugin auto-discovers | `.codex-plugin/` manifest |
| Antigravity | plugin auto-discovers | root `plugin.json` |
| Cursor | `~/.cursor/rules/` | `obscura-plugin install` |
| OpenCode | `~/.opencode/skills/` | `obscura-plugin install` |
| Cline | `~/.cline/skills/` | `obscura-plugin install` |

## Skills

Registered skills are available as slash commands inside your agent:

```
/obscura-fetch <url> [--dump html|text|links|markdown] [--eval <js>] [--selector <css>] [--stealth]
/obscura-scrape <url1> <url2> ... [--eval <js>] [--concurrency <N>] [--format json|text]
/obscura-pipeline <index-url>   # discover links → scrape in one pipeline
/obscura-crawl <seed-url>       # recursive crawl with depth control, pagination, sitemap discovery
```

### Skill parameters

**`obscura-fetch`**

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

**`obscura-scrape`**

| Parameter | Type | Description |
|-----------|------|-------------|
| `urls` | string[] | URLs to scrape *(required)* |
| `eval` | string | JavaScript expression applied to each page |
| `concurrency` | number | Parallel workers (default: `10`) |
| `format` | string | `json` · `text` (default: `json`) |
| `proxy` | string | HTTP/SOCKS5 proxy URL for all workers |

**`obscura-pipeline`**

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | Index page URL *(required)* |
| `selector` | string | CSS selector for link extraction |
| `concurrency` | number | Parallel workers for scraping |
| `stealth` | boolean | Anti-detection mode¹ |

> ¹ **Stealth mode** requires Obscura built with `--features stealth`. The default release binary includes stealth.

**`obscura-crawl`**

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | Seed URL to start crawling from *(required)* |
| `max_depth` | number | Maximum crawl depth (default: `3`) |
| `same_domain` | boolean | Only follow links on the same domain (default: `true`) |
| `url_filter` | string | Regex/glob pattern to filter URLs by path |

## The `obscura-browser` agent

A self-directed web data collection specialist. Invoke it for tasks like:

> "Collect all product titles and prices from these 30 URLs"
> "Fetch the docs at example.com/api and summarize the endpoints"
> "Scrape the HN front page and return structured JSON"
> "Crawl example.com/blog up to depth 2 and extract all post titles"

The agent knows Obscura's limits — it stops and escalates to Playwright when login or interaction is required. Supports agentic crawling with depth control, pagination detection, sitemap discovery, and adaptive extraction.

## Project structure

```
obscura-plugin/
  skills/                        # shared skills
    obscura-fetch/SKILL.md
    obscura-scrape/SKILL.md
    obscura-pipeline/SKILL.md
    obscura-crawl/SKILL.md
  agents/                        # shared agent (markdown)
    obscura-browser.md
  scripts/
    install.js                   # binary bootstrap (SessionStart hook)

  .claude-plugin/                # Claude Code plugin
    plugin.json, hooks.json, marketplace.json
  .codex-plugin/                 # Codex CLI plugin
    plugin.json, hooks.json
    agents/obscura-browser.toml  # Codex-specific TOML agent
  plugin.json                    # Antigravity (agy) plugin marker
  hooks.json                     # Antigravity hooks

  src/                           # Rust installer (cursor, opencode, cline)
  tests/
  Cargo.toml
```

## Build from source

```bash
cargo build --release
./target/release/obscura-plugin --help
```

## License

Apache 2.0
