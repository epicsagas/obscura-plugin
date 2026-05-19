# Contributing to obscura-plugin

We welcome contributions of all kinds — bug fixes, new features, documentation improvements, and more.

## Prerequisites

- Rust stable (2021 edition)
- [`obscura`](https://github.com/h4ckf0r0day/obscura) binary installed and on PATH

## Getting Started

1. Fork the repository on GitHub.

2. Clone your fork locally:

   ```bash
   git clone https://github.com/<your-username>/obscura-plugin.git
   cd obscura-plugin
   ```

3. Create a feature branch:

   ```bash
   git checkout -b feat/my-feature   # new feature
   git checkout -b fix/my-bug        # bug fix
   git checkout -b chore/my-change   # tooling / non-functional
   ```

4. Make your changes, then run the checks below before pushing.

## Code Style

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Testing

```bash
cargo test
```

The `stdio_roundtrip` integration test requires a release build first:

```bash
cargo build --release
cargo test -- --ignored
```

All existing tests must pass. New functionality must include tests.

## Adding a new MCP tool

1. Add the tool definition to `ALL_TOOLS` in `src/mcp.rs` — include `name`, `description`, and `inputSchema` with `type`, `properties`, and `required`.
2. Add a dispatch branch in `call_tool()`.
3. Verify with `cargo test tools_have_required_fields`.

## Pull Request Checklist

- [ ] `cargo fmt` run
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] New code has test coverage
- [ ] Related issues linked (e.g., `Closes #42`)
