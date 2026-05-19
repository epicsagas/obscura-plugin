# AGENTS.md
## Commands
- Build: `cargo build --release` | Test: `cargo test` | Test single: `cargo test --test <test_file>`
- Lint: `cargo clippy -- -D warnings` | Format: `cargo fmt -- --check` | Run: `cargo run -- serve`
- Gen/Sync: N/A

## Project Structure
- `src/` — Rust source: `main.rs` (CLI entry), `lib.rs` (module root), `install.rs` (tool config/injection/skill+agent transforms), `mcp.rs` (JSON-RPC MCP server + tool dispatch), `wizard.rs` (interactive TUI picker)
- `tests/` — Integration tests: `mcp.rs` (protocol handling, tool validation, stdio roundtrip), `install.rs` (skill/agent transforms, tool registry)
- `hooks/` — Claude Code plugin hooks: `install.js` (binary installer/launcher), `hooks.json` (SessionStart hook)
- `registry/` — Static content bundled via `include_str!`: `skills/` (obscura-fetch, obscura-scrape, obscura-pipeline SKILL.md), `agents/` (obscura-browser.md)
- `.claude-plugin/` — Claude Code plugin manifest (`plugin.json`, `marketplace.json`)
- `.github/workflows/` — CI: `release.yml` (cross-platform build on tag push)

## Code Style
- Rust 2021 edition | MSRV: stable (no pinned toolchain)
- Naming: `snake_case` for functions/variables/modules, `PascalCase` for types/enums/structs, `SCREAMING_SNAKE` for statics/constants
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

## Testing
- Framework: built-in `#[test]` + `serde_json` for assertions | Run all: `cargo test` | Coverage: N/A
- File naming: `tests/<module>.rs` matching `src/<module>.rs` (e.g. `tests/mcp.rs` tests `src/mcp.rs`)
- Mocking: N/A — tests call library functions directly; `#[ignore]` integration test (`stdio_roundtrip`) requires `cargo build --release` first, run with `cargo test -- --ignored`

## Git Workflow
- Branch strategy: not observed (single main branch)
- Commit format: not observed
- CI: `release.yml` triggers on `v*` tags → cross-compile for linux/macOS/windows → attach binaries to GitHub Release

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
