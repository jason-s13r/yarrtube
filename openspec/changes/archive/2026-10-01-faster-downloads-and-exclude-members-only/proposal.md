## Why

A night of production logs surfaced two issues. Per-video downloads are slow
(median ~100s, p90 ~5min, some ~9min) because `yt-dlp` fetches each video's
DASH fragments serially. Separately, permanently-unavailable videos fail every
download attempt — members-only content, and videos YouTube reports as
unavailable / removed / private / rights-blocked (the production failures were
copyright rights-blocks surfaced as "Video unavailable") — yet they burn the
full retry budget and are then revived by the 24h reconcile recovery loop
forever, generating recurring noise and wasted work.

## What Changes

- Download each video's fragments concurrently (pass `--concurrent-fragments`
  to `yt-dlp`), cutting per-video wall-clock time for fragmented formats.
- Diagnose every failed download: `yt-dlp`'s default clients report many
  permanent blocks as a bare "Video unavailable", so on every clean download
  failure the system runs a second, diagnostic `yt-dlp` probe (forcing the
  android/tv clients, simulate-only) to obtain and log the *precise* reason —
  copyright/claimed-content, region block, private, removed, terminated,
  members-only, etc. Operator visibility into why downloads fail is a primary
  goal.
- Introduce a new terminal video status, **excluded**, for videos that can
  never be downloaded.
  - When the *diagnosed* reason is one that can never succeed later —
    members-only, copyright/claimed-content block, region block, private,
    removed, or terminated account — the video becomes **excluded** instead of
    errored-but-retrying / permanently errored.
  - A generic/undetermined reason (a bare "Video unavailable" with no specific
    cause) and every transient failure (network/timeout/HTTP 5xx/429, "Sign in
    to confirm you're not a bot", fragment/merge errors) keep the current
    errored + retry + 24h recovery behavior — so a just-uploaded video still
    processing is never wrongly excluded.
  - An excluded video's download task completes without error: no further
    attempts in that sequence, and no retry is scheduled.
  - Excluded videos are **never** revived by the playlist/channel reconcile
    recovery loop (unlike permanently errored videos, which recover after 24h).
  - Excluded videos are filtered out of the playlist and channel video lists.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `video-download`: diagnoses every failed download for a precise reason; adds
  the **excluded** terminal status; a diagnosed permanent reason excludes the
  video and is not retried, while generic/undetermined and transient failures
  stay errored/retryable. (The concurrent-fragments speedup is an
  implementation-only change to the `yt-dlp` invocation — the downloaded file
  is byte-for-byte unchanged — so it carries no spec delta; it is covered in
  design.md and tasks.md.)
- `video-listing`: excluded videos are omitted from the playlist and channel
  video-list responses.
- `playlist-reconciliation`: the permanently-failed recovery loop never resets
  an excluded video.
- `channel-video-sync`: the permanently-failed recovery loop never resets an
  excluded video.

## Impact

- Code: `infrastructure/shared/ytdlp.rs` (concurrent-fragments arg; a new
  simulate-only diagnostic probe that forces the android/tv clients and
  extracts the precise reason); the `VideoDownloaderRepository` port (new
  `diagnose` method) and its `yt-dlp` adapter + fake;
  `domain/video/video_status.rs` (new `Excluded` variant + persistence string);
  `domain/video/video.rs` (`mark_excluded` transition, recovery gating);
  `domain/services/video_downloader.rs` (diagnose-on-failure, reason
  classification, logging, non-retrying outcome);
  `domain/services/playlist_video_reconciler.rs` and
  `channel_video_reconciler.rs` (skip excluded in recovery); the video-listing
  HTTP query/filtering.
- API: the videos list endpoints stop returning excluded videos; the `status`
  DTO field gains a new possible value (`EXCLUDED`) for any endpoint that still
  surfaces it (e.g. tasks/debug views).
- Storage: a new persisted status string `EXCLUDED`; no migration needed
  (existing rows keep their current statuses).
- No new environment variables; the fragment concurrency is a fixed internal
  default.
