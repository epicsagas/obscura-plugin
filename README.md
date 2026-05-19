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

Unlike the built-in `obscura mcp` subcommand (in-process browser control with `browser_*` tools), this plugin uses a **CLI wrapper** approach: it shells out to the `obscura` binary, providing high-level tools like `obscura_fetch`, `obscura_scrape`, and `obscura_extract_markdown`.

## Install

> **Prerequisite**: The `obscura` binary must be installed separately. See [Obscura releases](https://github.com/h4ckf0r0day/obscura/releases).

### macOS / Linux

```bash
# pre-built binary, no Rust required
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/obscura-plugin/releases/latest/download/obscura-mcp-installer.sh | sh
```

### Windows

```powershell
# pre-built binary, no Rust required
irm https://github.com/epicsagas/obscura-plugin/releases/latest/download/obscura-mcp-installer.ps1 | iex
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

> `obscura-mcp --version` to verify. Update with `brew upgrade obscura-mcp` or re-run the installer script.

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

## Claude Code plugin seeding

`obscura-plugin`은 `.claude-plugin/` 매니페스트를 포함하고 있어 `obscura-mcp install`을 수동으로 실행하지 않아도 Claude Code에 자동으로 시딩됩니다.

Claude Code가 이 플러그인을 로드하면 `hooks/install.js`(`SessionStart` 훅)가 자동 실행되어:

1. `obscura-mcp` 바이너리를 GitHub Releases에서 `~/.local/bin/`으로 다운로드
2. `obscura` 헤드리스 브라우저 바이너리를 함께 다운로드
3. MCP 서버, 스킬, `obscura-browser` 에이전트를 자동 등록

### 로컬 플러그인 로드

Claude Code 설정에 플러그인 경로를 추가합니다:

```jsonc
// ~/.claude/settings.json
{
  "plugins": [
    { "source": "/path/to/obscura-plugin" }
  ]
}
```

또는 CLI로:

```bash
claude plugin add /path/to/obscura-plugin   # 로컬 경로
claude plugin add epicsagas/obscura-plugin  # GitHub (마켓플레이스 등록 후)
```

첫 세션 시작 후 설치 확인:

```bash
obscura-mcp list   # Claude Code: installed 표시
```

### 플러그인이 시딩하는 자산

| 자산 | 설치 위치 |
|------|-----------|
| MCP 서버 | `~/.claude/claude_desktop_config.json` (또는 `.mcp.json`) |
| 스킬 | `~/.claude/skills/` — `obscura-fetch`, `obscura-scrape`, `obscura-pipeline` |
| 에이전트 | `~/.claude/agents/obscura-browser.md` |

> Codex CLI는 Claude Code 플러그인 라이프사이클을 지원하지 않습니다. Codex에는 `obscura-mcp install codex`를 사용하세요.

## Build from source

```bash
cargo build --release
./target/release/obscura-mcp --help
```

## License

Apache 2.0
