---
name: obscura-scrape
description: Scrape multiple URLs in parallel using Obscura. Use when you have a list of URLs to process in batch. Returns JSON or text per URL.
---

Parallel-fetch multiple URLs with Obscura. Use this over sequential `obscura fetch` calls whenever you have 2+ URLs.

## Usage
```
/obscura-scrape <url1> <url2> ... [--eval <js>] [--concurrency <N>] [--format json|text]
```

## When to use

- Known list of URLs (e.g., search results, sitemaps, link extractions)
- Same extraction logic applies to all URLs
- Read-only, no login required

## Instructions

1. Parse URLs and options from user input.
2. Run:
   ```bash
   obscura scrape <url1> <url2> ... [--eval <expression>] [--concurrency <N>] [--format json|text]
   ```
3. Default concurrency is 10 — lower to 3–5 for polite scraping or rate-limited sites.
4. On partial failure (some URLs errored), report which failed and continue with successful results.
5. If results are large, summarize: total count, success/fail breakdown, key extracted values.

## Two-phase pattern: discover → scrape

When URLs aren't known upfront:
```bash
# Phase 1: extract links from index page
obscura fetch https://example.com/blog --quiet --dump links

# Phase 2: scrape the discovered URLs
obscura scrape <url1> <url2> <url3> --eval "document.title" --format json
```

## Examples

```bash
# Basic parallel fetch
obscura scrape https://example.com https://example.org --format text

# Extract titles from multiple pages
obscura scrape url1 url2 url3 --eval "document.title" --format json

# Polite scraping (rate-limited site)
obscura scrape url1 url2 url3 --concurrency 2 --format json

# Extract structured data from each page
obscura scrape url1 url2 \
  --eval "JSON.stringify({title: document.title, h1: document.querySelector('h1')?.textContent})" \
  --format json
```

## Known limitations

- All URLs processed with identical options — no per-URL customization
- No retry logic built in — re-run failed URLs manually
- No session/cookie — login-required pages not supported

## Retry and adaptive strategies

For failed URLs in a batch scrape:
1. Collect failed URLs from output (those with error/null results)
2. Retry failed URLs as a separate scrape call with `--concurrency 2`
3. If still failing: retry each individually with `obscura fetch --stealth`
4. If individual fetch also fails: mark URL as unreachable, move on

## Large URL set handling

When scraping more than 50 URLs:
- Split into batches of 20–30 URLs
- Run batches sequentially (not in parallel)
- For rate-limited sites: use `--concurrency 3` per batch
- Aggregate results across batches
- Report: total attempted, successful, failed per batch

## Partial failure pattern

```bash
# Batch 1
obscura scrape url1 url2 ... url20 --eval "..." --format json
# Parse output: collect successes, note failures

# Retry batch (failed URLs only)
obscura scrape failed1 failed2 --eval "..." --concurrency 2 --format json

# Final fallback (still-failing URLs)
obscura fetch failed1 --quiet --stealth --eval "..."
```
