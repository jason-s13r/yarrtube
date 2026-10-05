## Why

Yarrtube runs unattended on users' NASes. The maintainer currently has no way to tell people about a new release, a breaking config change, or a known issue. An announcements file in the public repo, which the web UI reads when it loads, adds that channel with no new backend or service.

## What Changes

- New `announcements/announcements.json` at the repo root. It holds an array of `{ id, text }` entries, newest first. `text` supports inline `[label](url)` links.
- On every load, the web UI fetches that file from `raw.githubusercontent.com` on `main`.
- When an announcement hasn't been dismissed in this browser, a full-width bar above the header shows the newest one, with any links rendered as real links.
- A dismiss button records the id in localStorage. That announcement never shows again in that browser, and the next unread one (if any) takes its place.
- If the fetch fails, the JSON is malformed, or storage is unavailable, no bar appears and nothing else breaks.
- New smoke test that stubs the GitHub response and covers display, links, dismissal, persistence across reload, and fetch failure.

Non-goals: version targeting, read state shared across devices, any backend endpoint, rich formatting beyond links.

## Capabilities

### New Capabilities
- `announcements`: fetches maintainer announcements from the repo, shows unread ones in a dismissible bar, renders inline links, and remembers dismissals per browser.

### Modified Capabilities
- `smoke-tests`: adds an end-to-end requirement covering the announcements bar against the real image.

## Impact

- `web/`: new API call, query hook, `lib/` helpers (link parsing, dismissed-id storage), a bar component, and an `App.tsx` layout change.
- `smoke-tests/`: new spec file.
- Repo root: new `announcements/announcements.json`, seeded with `[]`.
- Rust backend: no change.
- Runtime: each browser that loads the UI makes one outbound request to GitHub.
