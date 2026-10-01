## Context

See proposal.md — Why. Two independent problems in the download path, fixed
together because they both live in `video_downloader.rs` / `ytdlp.rs`:

- `yt-dlp` is invoked without `--concurrent-fragments`, so DASH-fragmented
  formats download one fragment at a time.
- A `download_video` task returns `Ok(())` to mean "done, do not retry" and
  `Err(..)` to mean "failed, retry then dead-letter" (`download_video_task.rs`
  returns `VideoDownloader::download`'s `Result` straight through). Today every
  clean `yt-dlp` failure goes through `record_failed`, which marks the video
  `ErroredRetrying`/`Errored` and returns `Err`, so the task retries up to
  `MAX_ATTEMPTS = 5`. Once `Errored`, `Video::is_due_for_recovery` (24h
  cooldown) lets each reconcile pass reset it to `Pending` and re-download —
  forever. Permanently-unavailable videos can never succeed, so they loop
  through this indefinitely. In production these were copyright rights-blocks
  that `yt-dlp` surfaces (via its default clients) as the bare string "Video
  unavailable", alongside a couple of members-only videos.

Video status is a closed enum `VideoStatus` (`domain/video/video_status.rs`)
persisted as an uppercase string in the `videos.status` column, and surfaced
raw in the videos DTO (`application/http/videos/dto.rs`).

## Goals / Non-Goals

**Goals:**
- Faster per-video downloads via concurrent fragment fetching.
- A precise, logged reason for *every* failed download — operator visibility
  into why downloads fail is a primary goal.
- A terminal `Excluded` status that short-circuits retries and recovery and is
  hidden from the playlist/channel video lists.
- Videos whose *diagnosed* reason is permanently unavailable routed to
  `Excluded`; every generic/undetermined or transient failure unchanged.

**Non-Goals:**
- No new environment variable for fragment concurrency (fixed internal value).
- No reclassification of failures whose diagnosed reason is not a known
  permanent one — transient or undetermined failures (network, timeout, HTTP
  5xx, rate-limit/429, bot-verification, fragment/merge, bare "Video
  unavailable" with no specific cause) stay errored/retryable exactly as today.
- No DB migration or backfill: existing rows keep their statuses; `EXCLUDED`
  only ever appears on videos excluded after this ships.
- No automatic "un-exclude" path if a video later becomes available again — the
  user confirmed these are never-downloadable.
- The diagnostic probe is **not** a download workaround: forcing the android/tv
  clients only *reveals* the reason (it still cannot download a blocked video),
  so it is used purely to diagnose/classify, never to retry the download with a
  different client.

## Decisions

### `--concurrent-fragments 4`
Add `--concurrent-fragments 4` to the download args in
`ytdlp::download_video` (alongside the existing quality/merge/remux args).
Four parallels the typical fragment fan-out without hammering YouTube; with
`YARRTUBE_DOWNLOAD_CONCURRENCY` default 2 whole-video lanes that is ≤8
in-flight fragment connections. Hardcoded, not env-configurable — keeps scope
tight; can be promoted to an env var later if needed. The output file is
identical, so this has no spec delta. Alternative considered: `--concurrent-
fragments 8` — rejected as needlessly aggressive for a NAS on a home link.

### New terminal status `Excluded`
Add `VideoStatus::Excluded` → string `"EXCLUDED"` (parse + `as_str`, with a
round-trip test like the other variants). Add `Video::mark_excluded(now)` as a
consuming-self transition (sets `status = Excluded`, `updated_at = now`). It
deliberately does **not** set `last_errored_at`: recovery keys off
`status == Errored`, and `Excluded` is a distinct status, so an excluded video
is naturally outside the recovery predicate. `is_due_for_recovery` stays
`status == Errored && ...`; no change needed there, but both reconcilers get a
scenario proving an excluded video is never reset. Alternative considered: a
boolean `excluded` flag on top of `Errored` — rejected; a flag that silently
overrides a status is easy to miss at every read site, whereas a first-class
status is matched exhaustively by the compiler.

### Diagnose every failed download, then classify the precise reason
The download runs with `yt-dlp`'s default clients, which collapse many distinct
permanent blocks into a bare "Video unavailable". That bare string is too
ambiguous to act on: it covers copyright blocks, removed videos, region blocks
*and* the occasional transient case (e.g. a just-uploaded video still
processing). So instead of matching the download's own error, the system
**diagnoses** every clean failure and classifies the *precise* reason.

**Diagnostic probe (new port method).** Add a `diagnose(video_url) ->
Result<Option<String>>` method to the `VideoDownloaderRepository` port,
implemented in the `yt-dlp` adapter (`ytdlp.rs`) as a second, **simulate-only**
invocation forcing the alternate clients:
`--simulate --no-warnings --extractor-args "youtube:player_client=android,tv,ios,web_safari"`.
It returns the precise reason `yt-dlp` prints (the playability/error line, e.g.
"It was blocked due to the claimed content by Mediatoon.", "This video is
private.", "This video is no longer available…"), or `None` when it cannot
determine one (the probe itself errors, or still returns only a bare "Video
unavailable"). The probe never downloads — it only reveals the reason; a
blocked video stays blocked — so it is safe and cheap (one extra simulate call
per failure). Per the user's decision, it runs on **every** clean failure, so
the precise reason is always logged, even when the download's own error was
already specific.

**Flow in `VideoDownloader::record_failed`** (clean non-zero `yt-dlp` exit):
1. Call `diagnose`. Combine its reason with the download's own stderr into the
   text used for logging and classification (the probe is the richer signal;
   the original stderr is a fallback).
2. Log the precise reason at info/warn so operators can see *why* each download
   failed (the capital observability goal).
3. Classify via a pure helper `is_permanently_unavailable_reason(&str) -> bool`
   — a case-insensitive substring match against a curated allowlist of stable,
   load-bearing permanent tokens (unit-tested without running `yt-dlp`):

   | Token (case-insensitive substring) | Covers |
   | --- | --- |
   | `members-only` | members-only content |
   | `claimed content` | copyright / content-ID claim block (the production case) |
   | `copyright` | "blocked … on copyright grounds" |
   | `in your country` | region / geo block |
   | `private video` | video made private |
   | `has been removed` / `no longer available` | video removed by uploader |
   | `account associated with this video has been terminated` | channel terminated |

4. If the reason matches: `mark_excluded`, return `Ok(())` (task settles, no
   retry, no dead-letter). Otherwise — including an undetermined reason or a
   bare "Video unavailable" with no specific cause — behave exactly as today
   (`mark_errored`/`mark_errored_retrying`, return `Err`), so the video stays
   on the normal retry + 24h reconcile-recovery path. This is the deliberate
   fallback: a video whose permanent cause we cannot name keeps looping (no
   worse than today) rather than risking a wrong permanent exclusion.

The allowlist is of *permanent* reasons, never an inverted "everything except
transient" rule, so an unseen `yt-dlp`/YouTube wording is never silently
excluded forever — the conservative direction. Alternatives considered:
matching the bare download error with no probe (what the previous draft did —
rejected because bare "Video unavailable" can't separate copyright from a
still-processing upload); treating every clean failure as excluded with a
transient allowlist (rejected — risks permanently excluding on a new, unseen
transient message).

Only `record_failed` (clean non-zero `yt-dlp` exit) diagnoses. `record_errored`
(systemic errors: missing binary, folder creation failure) stays untouched —
those are environment problems, not a YouTube unavailability reason, and there
is nothing to probe.

### Hiding excluded videos from the lists
The playlist and channel video-list read paths filter out `Excluded` videos.
Prefer filtering at the SQL query that backs those endpoints (a
`status <> 'EXCLUDED'` guard) so paging/counts stay correct, mirroring however
the current list query is shaped. The home-sections endpoint already shows only
`Downloaded` videos, so excluded videos never reach it — no change there.

## Risks / Trade-offs

- [The android/tv clients break (YouTube/`yt-dlp` churn), so the probe stops
  returning a reason] → `diagnose` returns `None`, the failure is treated as
  undetermined and stays on the normal errored/retry path (no worse than
  today); we lose the richer log line until `yt-dlp` catches up, but never
  mis-exclude. The probe failing must never fail the download handling itself.
- [`yt-dlp` rewords one of the matched reasons] → That reason falls back to the
  retry path; the chosen tokens are the stable parts of each message, and a
  later `yt-dlp` update can be re-checked against the smoke tests.
- [A recoverable failure happens to contain a matched token and is wrongly
  excluded] → Tokens are load-bearing and specific, and classification runs on
  the *diagnosed* reason, not the bare download error — so a still-processing
  upload (bare "Video unavailable", no specific cause) is undetermined and
  stays retryable. Accepted residual: a genuinely transient failure whose text
  happens to contain a permanent token would be excluded (very unlikely given
  the token choice).
- [The extra probe adds a `yt-dlp` call per failure] → Bounded to failures
  (rare in steady state), simulate-only (no media transfer). During a large
  backlog of unavailable videos it is one extra light call each — accepted,
  since the user prioritises knowing why downloads fail.
- [An excluded video is later made available again] → It stays `Excluded` and
  is not retried; acceptable per the Non-Goals (user confirmed). Manual reset
  remains possible by editing the row if ever needed.
- [Concurrent fragments increase request rate to YouTube] → Capped at 4 per
  video and gated by the existing 2 whole-video lanes; well within normal
  `yt-dlp` behavior.
- [An excluded video disappears from the UI with no explanation] → Chosen
  behavior (hidden). The exclusion is still visible in logs and in the raw
  `status` on any debug/task view that surfaces it.
