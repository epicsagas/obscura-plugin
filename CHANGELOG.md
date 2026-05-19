# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-19

### Fixed

- Replace `is_none_or` (stable since 1.82) with `map_or` to satisfy MSRV 1.80

## [0.1.0] - 2026-05-19

### Added

- `obscura_fetch` MCP tool — fetch a URL and return HTML, text, links, or JS eval result
- `obscura_scrape` MCP tool — parallel scrape multiple URLs with configurable concurrency
- `obscura_serve` MCP tool — start a CDP server for Puppeteer / Playwright
- `obscura_screenshot` MCP tool — fetch a page and evaluate a JS expression
- `obscura_extract_markdown` MCP tool — convert a URL to clean markdown
- `obscura-mcp install` command — register MCP server and seed skills/agents into 6 AI tools (Claude Code, Cursor, Gemini CLI, Codex CLI, OpenCode, Cline)
- `obscura-mcp uninstall` command — remove from a specific tool
- `obscura-mcp list` command — show supported tools and installation status
- Claude Code plugin manifest (`.claude-plugin/`) with `SessionStart` hook for zero-touch binary install and seeding
- `obscura-browser` agent — self-directed web data collection specialist
- Skills: `obscura-fetch`, `obscura-scrape`, `obscura-pipeline`
- cargo-dist release pipeline with crates.io publish and Homebrew tap auto-update
