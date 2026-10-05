## Context

See proposal.md for why. Each of the four YouTube adapters builds a blocking
`reqwest` request with `.query(&[..., ("key", api_key)])`. A `reqwest::Error`
for a failed send formats as `error sending request for url (<full URL>)`, and
`error_report::cause_chain` (logs and the task executor's `last_error`) prints
it verbatim. Non-success responses are reported as
`YouTube API request failed with status {status}: {body}`, which has no URL.
Their tests use mockito, matching on query parameters.

## Goals / Non-Goals

**Goals:**
- No request URL carries the key, so no error built from one can leak it.

**Non-Goals:**
- Scrubbing secrets generically from every error or log line (a redaction
  layer). There is one secret in a URL; removing it from the URL is enough.
- Rewriting already-stored `last_error` values or rotating the key.
- Paging Plex section listings (see decision 2).

## Decisions

### 1. Key in the `X-Goog-Api-Key` header, not the query
Google APIs accept the API key in the `X-Goog-Api-Key` header as an
alternative to `key=`. Each adapter drops the `key` query pair and adds the
header. The URL in any `reqwest::Error` is then key-free.

Alternatives: `reqwest::Error::without_url()` on every error path. Rejected
because every current and future call site must remember it, while the header
removes the key from the URL once. A log-redaction layer was also rejected as
far more machinery for one secret.

Tests: each adapter's existing mockito tests also match the
`x-goog-api-key` header. One test per adapter points it at an unroutable base
URL and asserts the error's full cause chain doesn't contain the key, which
is the spec scenario and fails before the change.

### 2. Plex timeout 30 s → 60 s
`REQUEST_TIMEOUT` stays one client-wide value, raised to 60 s. The pass runs
every 30 min on the Light lane, so 60 s still bounds a hung server without
starving it.

Alternative: paging `list_items` with `X-Plex-Container-Start`/`Size`.
Deferred: the slow responses all fell inside the maintenance window while the
payload size didn't change, which points at a busy server rather than size,
so smaller pages might each still be slow.

## Risks / Trade-offs

- [The header isn't honored by some endpoint, giving 403 "API key missing"] →
  `X-Goog-Api-Key` is the documented header for Google APIs. The adapters'
  first production call after deploy confirms it, and the failure would be
  loud (every reconcile errors).
- [A 60 s request holds the Light lane longer when Plex hangs] → bounded to
  60 s a few times per pass at most, every 30 min.

## Migration Plan

Deploy. No config change. Optionally rotate the YouTube API key afterwards,
since it is in earlier logs and stored task errors.
