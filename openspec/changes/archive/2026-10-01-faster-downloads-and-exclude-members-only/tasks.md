## 1. Concurrent fragment download

- [x] 1.1 Add `--concurrent-fragments 4` to the download args built in `ytdlp::download_video` (`src/infrastructure/shared/ytdlp.rs`), leaving the quality/merge/remux args untouched. Verify by updating the existing fake-`yt-dlp` captured-args assertion (or adding one) to show `--concurrent-fragments 4` is passed, and `cargo test ytdlp` passes.

## 2. `Excluded` status

- [x] 2.1 Add `VideoStatus::Excluded` to `src/domain/video/video_status.rs`, mapping to/from `"EXCLUDED"` in `as_str`/`parse`. Verify with a round-trip unit test mirroring the other variants and that `parse` still rejects unknown strings.
- [x] 2.2 Add `Video::mark_excluded(now)` to `src/domain/video/video.rs` (consuming-self: sets `status = Excluded`, `updated_at = now`, leaves `last_errored_at` unchanged). Verify with a unit test asserting the whole transitioned `Video`.
- [x] 2.3 Add a unit test in `video.rs` asserting `is_due_for_recovery` returns `false` for an `Excluded` video no matter how old `updated_at`/`last_errored_at` are (confirming recovery never picks it up).

## 3. Diagnostic probe on failure

- [x] 3.1 Add a `diagnose(&self, video_url: &str) -> anyhow::Result<Option<String>>` method to the `VideoDownloaderRepository` port and implement it in the `yt-dlp` adapter (`src/infrastructure/shared/ytdlp.rs` + `youtube_video_downloader_repository.rs`): a simulate-only invocation with `--simulate --no-warnings --extractor-args "youtube:player_client=android,tv,ios,web_safari"`, returning the precise reason line `yt-dlp` prints, or `None` when none can be determined (probe errors, or only a bare "Video unavailable"). Verify with a `FakeYtDlp`/fake-repo test that a scripted reason is returned and a probe that finds nothing yields `None`.
- [x] 3.2 Extend `FakeVideoDownloaderRepository` with a configurable `diagnose` response so downloader tests can drive it without running `yt-dlp`.

## 4. Classify the reason and apply the non-retrying outcome

- [x] 4.1 Add a pure helper `is_permanently_unavailable_reason(reason: &str) -> bool` near `VideoDownloader` in `src/domain/services/video_downloader.rs`, case-insensitive substring match against the curated permanent token set (see design.md): `members-only`, `claimed content`, `copyright`, `in your country`, `private video`, `has been removed`, `no longer available`, `account associated with this video has been terminated`. Verify with unit tests covering one real reason per token, case variations, and non-matching reasons (a bare "Video unavailable", network/timeout, HTTP 429, "Sign in to confirm you're not a bot", a fragment error) returning `false`.
- [x] 4.2 In `VideoDownloader::record_failed`, on every clean failure call `diagnose`, combine its reason with the download's own stderr, and log the precise reason. If `is_permanently_unavailable_reason` matches, `mark_excluded` and return `Ok(())` (no retry/dead-letter); otherwise keep the current errored/retrying behavior and `Err`. A `diagnose` error must be swallowed (logged) and treated as undetermined, never fail the handling. Verify with downloader tests: a copyright/claimed-content reason and a members-only reason each settle as `Excluded` + `Ok(())`; a bare "Video unavailable" with no diagnosed cause and a transient failure still end `Errored`/`ErroredRetrying` + `Err`.
- [x] 4.3 Add a `DownloadVideoTask` test asserting that a diagnosed permanent failure (e.g. a claimed-content reason) makes the task return `Ok(())` (so the queue schedules no retry) and leaves the video `Excluded`, while a bare "Video unavailable" leaves it errored and the task returns `Err`.

## 5. Reconcilers never recover excluded videos

- [x] 5.1 Add a `PlaylistVideoReconciler` test: a reconcile pass finding an `Excluded` video (even with an ancient timestamp) does not reset it to `Pending` and schedules no download. (No production change expected since recovery keys off `Errored`; this locks the behavior in.)
- [x] 5.2 Add the equivalent `ChannelVideoReconciler` test for an `Excluded` video.

## 6. Hide excluded videos from the video lists

- [x] 6.1 Filter `Excluded` videos out of the playlist video-list and channel video-list read paths (prefer a `status <> 'EXCLUDED'` guard in the backing query — see design.md). Verify with HTTP-layer tests: a playlist and a channel each containing one excluded video plus others return only the others, with HTTP 200.
- [x] 6.2 Confirm the home-sections endpoint is unaffected (it already returns only `Downloaded` videos) — verify the existing home tests still pass, adding a case with an excluded video present if one is cheap to express.

## 7. Verification

- [x] 7.1 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, and `cargo test --locked`; all pass.
- [x] 7.2 Check `web/` for a video `status` union type in `src/api/types.ts`; since excluded videos are filtered server-side the frontend should need no change, but confirm `npm run check` still passes and add `EXCLUDED` to the type only if the type is exhaustive and the suite requires it.
- [x] 7.3 Run the relevant `smoke-tests/` for the download path to confirm a normal download still succeeds end-to-end with concurrent fragments enabled, and (if feasible) that a known-blocked video is diagnosed and excluded. (Normal download verified: real downloads completed with `--concurrent-fragments 4` enabled. The known-blocked-video case is not expressible in the current smoke suite — no blocked fixture — so it stays covered by the Rust acceptance tests in task 4.2.)
