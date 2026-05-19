# AGENTS.md
## Overview
- **obscura-plugin**: A Rust binary (`obscura-mcp`) that wraps the Obscura headless browser CLI, exposes it as a Model Context Protocol (MCP) server, and supports automatic installation into 6 major AI coding tools.
## Commands
- Build: `cargo build --release` | Test: `cargo test` | Test single: `cargo test --test <test_file>`
- Lint: `cargo clippy -- -D warnings` | Format: `cargo fmt -- --check` | Run: `cargo run -- serve`
- Gen/Sync: N/A

## Project Structure & Architecture
- `src/` — Rust source: 
  - `main.rs` (CLI entry - serve, install, uninstall, list)
  - `mcp.rs` (JSON-RPC MCP server + 5 tools dispatch)
  - `install.rs` (Config/injection & per-tool transforms e.g., Cursor globs, Gemini remapping)
  - `wizard.rs` (crossterm TUI picker)
- `tests/` — Integration tests: `mcp.rs` (protocol handling, validation), `install.rs` (registry)
- `hooks/install.js` — Node.js installer (Runs on SessionStart via Claude Code plugin lifecycle)
- `registry/` — Static content bundled via `include_str!`: `skills/`, `agents/`
- `.claude-plugin/` — Claude Code plugin manifest (`plugin.json`, `marketplace.json`)
- `.github/workflows/` — CI: `release.yml` (cross-platform build on tag push)

## MCP Tools (src/mcp.rs)
- `obscura_fetch` → `obscura fetch <url>` (Timeout: 30s)
- `obscura_scrape` → `obscura scrape <urls...>` (Timeout: 60s)
- `obscura_serve` → `obscura serve` (Background daemon)
- `obscura_screenshot` → `obscura fetch <url> --eval` (Timeout: 30s)
- `obscura_extract_markdown` → `obscura fetch <url> --dump md` (Timeout: 30s)

## Code Style
- Rust 2021 edition | MSRV: stable (no pinned toolchain)
- Naming: `snake_case` (fn/var/mod), `PascalCase` (type/enum/struct), `SCREAMING_SNAKE` (const)
- Error handling: return `Result<T, String>` with descriptive messages; use `.map_err(|e| format!(...))` for context; never panic in library code
- Imports: grouped as `std → external crates → crate modules`; prefer `use` at module top
- Async: synchronous (`std::process::Command`) — no async runtime in the MCP stdio server; tokio is a dependency but not actively used for I/O
- State: no global mutable state; all data flows through function arguments and return values
- Conventions: `serde_json::json!` macro for building JSON values; `include_str!` for bundling registry content; atomic file writes via tmp+rename pattern; `#[cfg(unix)]` / `#[cfg(not(unix))]` for platform-conditional code

### Golden Path
```rust
use serde_json::{json, Value};

pub fn call_tool(name: &str, args: &Value) -> Value {
    let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    if url.is_empty() {
        return error_result("Missing required parameter: url");
    }
    match run_obscura(&["fetch", url, "--quiet"], 30_000) {
        Ok(out) => text_result(&out),
        Err(e) => error_result(&e),
    }
}
```

## Testing & Workflow
- Framework: built-in `#[test]` + `serde_json` for assertions | Run all: `cargo test` | Coverage: N/A
- File naming: `tests/<module>.rs` matching `src/<module>.rs` (e.g. `tests/mcp.rs` tests `src/mcp.rs`)
- Mocking: N/A — tests call library functions directly; `#[ignore]` integration test (`stdio_roundtrip`) requires `cargo build --release` first, run with `cargo test -- --ignored`
- Git Workflow: Branch strategy not observed; CI `release.yml` triggers on `v*` tags → cross-compile → attach to GitHub Release

## Boundaries
- Always: run `cargo test` after changes to `src/` or `tests/`
- Always: validate new MCP tools have `name`, `description`, `inputSchema` with `type`, `properties`, and `required` fields (see `tools_have_required_fields` test)
- Always: use `obscura_bin_path()` / `OBSCURA_BIN` env for resolving the obscura binary — never hardcode paths
- Always: use atomic write pattern (tmp+rename) for JSON config files
- Always: include platform guards (`#[cfg(unix)]`) for OS-specific process management code
- Ask first: before adding a new tool to the `ALL_TOOLS` registry or changing `ToolConfig` structs
- Ask first: before modifying registry content (`registry/skills/`, `registry/agents/`)
- Never: modify `hooks/install.js` without understanding the Claude Code plugin lifecycle
- Never: introduce async I/O — the MCP server is intentionally synchronous stdio
- Never: suppress linter warnings inline — fix the root cause instead
- Never: Use `#[allow(...)]` to suppress warnings — fix the root cause; use `#[expect(...)]` only with a tracking issue comment
