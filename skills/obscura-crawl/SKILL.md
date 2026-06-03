---
name: obscura-crawl
description: Recursive web crawling with depth control, pagination detection, and sitemap discovery. Use for comprehensive site coverage when you need to follow links across multiple levels.
---

Recursive web crawling with Obscura. Combines `obscura fetch` (discovery) and `obscura scrape` (collection) into a multi-phase loop with state tracking, pagination, and adaptive extraction.

## Usage
```
/obscura-crawl <seed-url> [--max-depth N] [--same-domain] [--url-filter <pattern>]
```

## Strategy overview

```
Phase 1: Seed discovery   (robots.txt, sitemap.xml, seed page links)
Phase 2: BFS crawl loop   (breadth-first with depth tracking)
Phase 3: Pagination       (detect & follow within each depth level)
Phase 4: Adaptive extract (fallback strategies per URL)
Phase 5: Smart batching   (split large queues, rate-limit aware)
```

---

## Phase 1 — Seed discovery

### Sitemap strategy (preferred)

```bash
# Check robots.txt for sitemap directives
obscura fetch <seed-url>/robots.txt --quiet --dump text

# Fetch each Sitemap: URL and extract <loc> entries
obscura fetch <sitemap-url> --quiet --dump text
```

Sitemap URLs are high-quality seeds with known structure. Prefer this over link-following.

### Seed page strategy (fallback)

```bash
obscura fetch <seed-url> --quiet --dump links
```

Filter to same-domain URLs. Classify: index pages → crawl queue at depth 0, detail pages → extraction queue.

---

## Phase 2 — BFS crawl loop

### State tracking

Maintain this state across iterations:

| State | Type | Purpose |
|-------|------|---------|
| VISITED | Set | Canonicalized URLs already fetched (prevent re-visits) |
| QUEUE | List of {url, depth} | URLs to process |
| RESULTS | List | Accumulated extracted data |
| ERRORS | Map of url → reason | Failed URLs for retry decisions |

### Crawl step (repeat until QUEUE empty or depth limit reached)

1. Dequeue next {url, depth}
2. If url in VISITED → skip
3. Add url to VISITED
4. Fetch with `--dump links` to discover child URLs
5. For each child URL:
   - Canonicalize (see rules below)
   - If same-domain, not in VISITED, depth < max-depth → enqueue {child_url, depth + 1}
6. Extract data from current page
7. Add to RESULTS

### URL canonicalization rules

Before adding any URL to VISITED:

- Strip fragment: `page#section` → `page`
- Normalize trailing slash: pick one convention consistently
- Sort query parameters alphabetically
- Remove tracking params: `utm_*`, `fbclid`, `gclid`, `ref`, `source`
- Lowercase hostname
- Resolve relative URLs against base URL

### URL filtering

- `--same-domain`: only follow links matching seed hostname
- `--url-filter`: regex/glob pattern for URL path inclusion
- **Always exclude**: `.pdf`, `.zip`, `.png`, `.jpg`, `.css`, `.js`, `.ico`, `.xml` (feeds)
- **Always exclude paths**: `/feed`, `/rss`, `/atom`, `/wp-json`, `/api/`

---

## Phase 3 — Pagination detection

### Pattern detection

After fetching an index page, check for pagination:

```bash
obscura fetch <url> --quiet --eval "JSON.stringify({
  next_page: document.querySelector('a[rel=next]')?.href ||
    document.querySelector('.pagination .next a')?.href,
  page_links: Array.from(document.querySelectorAll('.pagination a, .pager a, nav.pagination a'))
    .map(a => ({text: a.textContent.trim(), href: a.href}))
    .filter(a => />\d+|next|›|→/i.test(a.text))
})"
```

### Pagination loop

1. Detect pagination on current page
2. If "next" link found and not in VISITED → enqueue next page at **same depth** (not depth+1)
3. This ensures full exploration of one level before going deeper
4. Stop pagination when: next link returns 404, no new items found, or max pages reached (default: 20)

---

## Phase 4 — Adaptive extraction

When extraction from a page fails or returns empty, try in order:

| Attempt | Strategy | Command flags |
|---------|----------|--------------|
| 1 | Plain text | `--dump text` |
| 2 | Stealth retry | add `--stealth` |
| 3 | Selector wait | `--selector main --wait-until networkidle0` |
| 4 | Fallback selectors | `--eval "document.querySelector('article')?.innerText \|\| document.querySelector('main')?.innerText \|\| document.body.innerText"` |
| 5 | Give up | Mark as ERROR, continue with queue |

**Quality checks per extraction:**
- Empty string or null → try next strategy
- < 50 chars for a content page → flag as "thin", try next strategy
- Contains "access denied" / "cloudflare" / "captcha" → stop, escalate
- **Max 3 attempts per URL**, then mark as ERROR and continue

---

## Phase 5 — Smart batching

| URL count | Strategy |
|-----------|----------|
| 2–5 | Single `obscura scrape` call |
| 6–30 | Single `obscura scrape` with `--concurrency 10` |
| 31–100 | Split into batches of 25, run sequentially |
| 100–500 | Batches of 25, warn user about scale |
| 500+ | **Ask user for confirmation** before proceeding |

Rate-limited site (429 observed) → `--concurrency 2`, batches of 10.

---

## Complete crawl example

Goal: Crawl a blog, collect all post titles + content, depth 2.

```bash
# Phase 1: Discover seeds from sitemap
obscura fetch https://example.com/robots.txt --quiet --dump text
# → Found Sitemap: https://example.com/sitemap.xml

obscura fetch https://example.com/sitemap.xml --quiet --dump text
# → Extract post URLs matching /blog/ → add to QUEUE at depth 0

# Phase 2: Scrape discovered URLs in batches
obscura scrape https://example.com/blog/post-1 \
  https://example.com/blog/post-2 \
  ... \
  --eval "JSON.stringify({title: document.querySelector('h1')?.textContent, content: document.querySelector('article')?.innerText?.substring(0,500)})" \
  --concurrency 5 --format json

# Phase 3: If sitemap had >25 URLs, continue with next batch
obscura scrape https://example.com/blog/post-26 \
  ... \
  --eval "..." --concurrency 5 --format json

# Phase 4: Aggregate, report, present to user
```

---

## Limitations

| Situation | Action |
|-----------|--------|
| Login required | Stop — use Playwright instead |
| CAPTCHA encountered | Stop — obscura cannot solve CAPTCHAs |
| Infinite scroll / load-more | Use `--selector` to wait, but click not possible |
| Rate limiting / 429 | Drop concurrency to 2, wait between batches |
| Depth > 3 | Ask user before going deeper |
| > 500 URLs discovered | Warn user, suggest narrowing scope |
