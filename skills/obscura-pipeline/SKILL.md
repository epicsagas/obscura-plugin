---
name: obscura-pipeline
description: Multi-step web data collection pipeline using Obscura. Combines fetch + scrape for discover-then-collect workflows. Use when you need to crawl a site systematically.
---

Multi-step data collection with Obscura. Combines `obscura fetch` (discover) and `obscura scrape` (collect) into a pipeline.

## Usage
```
/obscura-pipeline <index-url> [--extract-links-selector <css>] [--eval <js>] [--concurrency <N>]
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
Or with selector for specific link groups:
```bash
obscura fetch <index-url> --quiet \
  --eval "JSON.stringify(Array.from(document.querySelectorAll('<selector> a')).map(a => a.href))"
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

## Concrete examples

### Blog post index → all post titles + content
```bash
# Discover
obscura fetch https://example.com/blog --quiet --dump links

# Collect (after filtering to post URLs)
obscura scrape https://example.com/blog/post-1 https://example.com/blog/post-2 \
  --eval "JSON.stringify({title: document.title, body: document.querySelector('article')?.innerText})" \
  --format json
```

### Hacker News front page → titles + scores
```bash
obscura fetch https://news.ycombinator.com --quiet \
  --eval "JSON.stringify(Array.from(document.querySelectorAll('.athing')).map(el => ({
    title: el.querySelector('.titleline > a')?.textContent,
    url: el.querySelector('.titleline > a')?.href,
    score: document.getElementById('score_' + el.id)?.textContent
  })))"
```

### Product listing → all product pages → prices
```bash
# Step 1: get product links
obscura fetch https://shop.example.com/products --quiet \
  --eval "JSON.stringify(Array.from(document.querySelectorAll('.product-link')).map(a => a.href))"

# Step 2: scrape each product
obscura scrape <product-url-1> <product-url-2> ... \
  --eval "JSON.stringify({name: document.querySelector('h1')?.textContent, price: document.querySelector('.price')?.textContent})" \
  --format json
```

## Limitations & when to stop

| Situation | Action |
|-----------|--------|
| Login required at any step | Stop — use Playwright instead |
| Infinite scroll / load-more button | Use `--selector` to wait, but click not possible |
| > 50 URLs | Split into batches of 20–30 |
| Rate limiting / 429 errors | Drop `--concurrency` to 2, add delay between batches |
| CAPTCHA | Stop — obscura cannot solve CAPTCHAs |

## Pagination handling

For index pages with pagination (e.g., blog page 1, 2, 3...):

1. Fetch first index page
2. Extract pagination links (look for "Next", ">", `rel="next"`, `/page/N` patterns)
3. Fetch each subsequent index page to discover more URLs
4. Collect ALL discovered URLs across all pagination pages
5. Then run the scrape step on the complete URL set

```bash
# Detect pagination
obscura fetch https://example.com/blog --quiet \
  --eval "JSON.stringify(Array.from(document.querySelectorAll('.pagination a, a[rel=next]')).map(a => ({text: a.textContent.trim(), href: a.href})))"
```

## Multi-level pipeline

For deeper discovery (e.g., category pages → product listing → product pages):

```
Level 0: fetch index → extract category URLs
Level 1: scrape categories → extract item URLs
Level 2: scrape items → extract data
```

At each level, apply URL deduplication and same-domain filtering before proceeding.
