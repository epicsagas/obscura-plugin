---
name: obscura-pipeline
description: Multi-step web data collection pipeline using Obscura. Combines fetch + scrape for discover-then-collect workflows. Use when you need to crawl a site systematically.
---

Multi-step data collection with Obscura. Combines `obscura fetch` (discover) and `obscura scrape` (collect) into a pipeline.

## Usage
```
$obscura-pipeline <index-url> [--extract-links-selector <css>] [--eval <js>] [--concurrency <N>]
```

## Core pattern

```
1. fetch index/listing page  →  extract target URLs
2. filter/deduplicate URLs
3. scrape target URLs in parallel  →  extract data
4. aggregate results
```

## Step-by-step instructions

### Step 1 — Discover URLs
```bash
obscura fetch <index-url> --quiet --dump links
```

### Step 2 — Filter
- Remove duplicates
- Remove off-domain links
- Remove pagination/nav links if not needed

### Step 3 — Collect
```bash
obscura scrape <url1> <url2> ... \
  --eval "<extraction expression>" \
  --concurrency 5 \
  --format json
```

### Step 4 — Aggregate
Parse JSON output and structure the result as the user needs (table, list, file).

## Limitations

| Situation | Action |
|-----------|--------|
| Login required at any step | Stop — use Playwright instead |
| > 50 URLs | Split into batches of 20–30 |
| Rate limiting / 429 errors | Drop `--concurrency` to 2 |
| CAPTCHA | Stop — obscura cannot solve CAPTCHAs |
